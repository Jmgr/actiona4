use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use actiona_core::{
    JsValueToString,
    scripting::{self, Engine},
};
use clap::{CommandFactory, Parser};
use color_eyre::{
    Result,
    owo_colors::{self, OwoColorize},
};
use directories::BaseDirs;
use rquickjs::{CatchResultExt, Coerced, Ctx, Module, Value};
use rustyline::{
    Editor, Helper, Hinter,
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::{CmdKind, Highlighter, MatchingBracketHighlighter},
    validate::{ValidationContext, ValidationResult, Validator},
};
use tokio::{fs, runtime::Handle};
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use two_face::re_exports::syntect::{
    easy::HighlightLines,
    highlighting::{Style, Theme},
    parsing::{SyntaxReference, SyntaxSet},
    util::as_24_bit_terminal_escaped,
};

#[derive(Parser)]
#[command(
    version,
    no_binary_name = true,
    disable_help_subcommand = true,
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = "{before-help}{all-args}{after-help}"
)]
pub enum ReplArgs {
    /// Load and execute a script file
    Load { filename: String },

    /// Clear context
    Clear,

    /// Exit REPL
    Exit,

    /// Show help
    Help,
}

#[derive(Helper, Hinter)]
struct ReplHelper {
    script_engine: Engine,
    file_completer: FilenameCompleter,
    cmd_names: Vec<String>,
    bracket: MatchingBracketHighlighter,
    syntax_set: SyntaxSet,
    syntax_reference: SyntaxReference,
    theme: Theme,
}

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        // If line begins with dot-command, skip JS highlighting
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            // we don't apply JS highlighting for commands
            return std::borrow::Cow::Borrowed(line);
        }

        // Otherwise apply JS highlighting
        let mut h = HighlightLines::new(&self.syntax_reference, &self.theme);
        // For simplicity we just highlight entire line as one chunk
        let ranges: Vec<(Style, &str)> = match h.highlight_line(line, &self.syntax_set) {
            Ok(r) => r,
            Err(_) => {
                // fallback: no highlight
                return std::borrow::Cow::Borrowed(line);
            }
        };
        let mut escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        escaped.push_str(&format!("{}", owo_colors::Style::new().suffix_formatter()));
        // Now apply bracket highlighting on the result
        let bracket_processed = self.bracket.highlight(&escaped, pos);
        std::borrow::Cow::Owned(bracket_processed.to_string())
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        if default {
            Cow::Owned(prompt.to_string().bold().cyan().to_string())
        } else {
            Cow::Owned(prompt.to_string().bold().bright_blue().to_string())
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }
}

impl ReplHelper {
    fn complete_commands(
        &self,
        input: &str,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        if input.starts_with("load ") {
            return self.file_completer.complete(line, pos, ctx);
        }

        let matches = self
            .cmd_names
            .iter()
            .filter(|cmd| cmd.starts_with(input))
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: cmd.clone(),
            })
            .collect();

        Ok((1, matches))
    }

    fn complete_js(&self, _input: &str) -> rustyline::Result<(usize, Vec<Pair>)> {
        Ok((0, Vec::new())) // TODO
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let input = &line[..pos];

        if let Some(rest) = input.strip_prefix('.') {
            return self.complete_commands(rest, line, pos, ctx);
        };

        self.complete_js(input)
    }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();

        if input.starts_with(".") {
            return Ok(ValidationResult::Valid(None));
        }

        if input.trim().is_empty() {
            return Ok(ValidationResult::Valid(None));
        }

        let result = tokio::task::block_in_place(|| {
            let script_engine = self.script_engine.clone();
            Handle::current().block_on(async move {
                self.script_engine
                    .with(move |ctx| Ok(validate_repl_input(&ctx, input, &script_engine)))
                    .await
            })
        });

        result.map_err(|e| ReadlineError::Io(std::io::Error::other(e)))
    }
}

const HISTORY_FILENAME: &str = "repl-history.txt";

fn history_file_path() -> Option<PathBuf> {
    let base = BaseDirs::new()?;
    #[cfg(target_os = "linux")]
    let dir = base.state_dir()?; // usually ~/.local/state
    #[cfg(not(target_os = "linux"))]
    let dir = base.data_local_dir(); // fallback ~/.local/share or similar

    Some(dir.join("core").join(HISTORY_FILENAME))
}

fn try_parse_dot_command(input: &str) -> Result<ReplArgs> {
    let parts = shell_words::split(input)?;
    let args = ReplArgs::try_parse_from(parts)?;
    Ok(args)
}

enum ProcessResult {
    Continue,
    Break,
}

async fn process_dot_command(command: ReplArgs, script_engine: &Engine) -> Result<ProcessResult> {
    match command {
        ReplArgs::Load { filename } => {
            // TODO: read file and call
            // script_engine.eval_async(script)
            let value: Result<Option<String>> = script_engine
                .with(|ctx| {
                    let value = ctx.eval_file::<Value<'_>, _>(&filename); // TODO: promise
                    match value {
                        Ok(value) => {
                            if value.is_undefined() {
                                Ok(None)
                            } else {
                                Ok(Some(value.get::<Coerced<String>>()?.0))
                            }
                        }
                        Err(_) => {
                            let caught = ctx.catch();
                            Ok(Some(caught.get::<Coerced<String>>()?.0))
                        }
                    }
                })
                .await;
            if let Some(value) = value? {
                println!("{value}");
            }
        }
        ReplArgs::Clear => {
            // Do nothing for now, since we can't reliably "clear" the context.
        }
        ReplArgs::Exit => return Ok(ProcessResult::Break),
        ReplArgs::Help => {
            let mut out = Vec::new();
            ReplArgs::command().write_long_help(&mut out)?;
            print!("{}", String::from_utf8(out)?);
            println!();
            println!("Keyboard:");
            println!("  Ctrl+C  Cancel current input");
            println!("  Ctrl+D  Exit the REPL");
        }
    }

    Ok(ProcessResult::Continue)
}

async fn parse_and_process_dot_command(
    line: &str,
    script_engine: &Engine,
) -> Result<ProcessResult> {
    let command = try_parse_dot_command(line)?;

    process_dot_command(command, script_engine).await
}

#[instrument(skip_all)]
fn setup_highlighting() -> (SyntaxSet, Theme, SyntaxReference) {
    let syntax_set = two_face::syntax::extra_no_newlines();
    let theme_set = two_face::theme::extra();
    let theme = theme_set.get(two_face::theme::EmbeddedThemeName::Nord);
    let syntax_reference = syntax_set.find_syntax_by_extension("ts").unwrap().clone();

    (syntax_set, theme.clone(), syntax_reference)
}

pub async fn repl(script_engine: Engine, cancellation_token: CancellationToken) -> Result<()> {
    let (syntax_set, theme, syntax_reference) = setup_highlighting();

    let validator = ReplHelper {
        script_engine: script_engine.clone(),
        file_completer: FilenameCompleter::new(),
        cmd_names: ReplArgs::command()
            .get_subcommands()
            .map(|command| command.get_name().to_string())
            .collect(),
        bracket: MatchingBracketHighlighter::new(),
        syntax_set,
        syntax_reference,
        theme: theme.clone(),
    };

    let mut repl = Editor::new()?;

    repl.set_helper(Some(validator));

    let history_filepath =
        history_file_path().unwrap_or_else(|| Path::new(HISTORY_FILENAME).to_path_buf());
    _ = repl.load_history(&history_filepath);

    println!("Actiona 4 💻 REPL (Read-Eval-Print-Loop)\n");
    println!("Use Ctrl+D or enter \".exit\" to exit.");
    println!("Enter \".help\" to display help.");

    loop {
        if cancellation_token.is_cancelled() {
            break;
        }

        let readline = repl.readline("≫ ");
        match readline {
            Ok(line) => {
                repl.add_history_entry(&line)?;

                if let Some(line) = line.strip_prefix(".") {
                    let result = parse_and_process_dot_command(line, &script_engine).await;
                    match result {
                        Ok(result) => match result {
                            ProcessResult::Continue => {}
                            ProcessResult::Break => break,
                        },
                        Err(err) => {
                            let message = err.to_string();
                            // Remove the "Usage:"" part
                            if let Some((before, _)) = message.split_once("\n\nUsage:") {
                                eprintln!("{before}");
                            } else {
                                eprintln!("{message}");
                            }
                        }
                    }

                    if cancellation_token.is_cancelled() {
                        break;
                    }

                    continue;
                }

                let value = script_engine
                    .eval_async_fn::<Option<String>>(&line, |value| {
                        Ok(if value.is_undefined() {
                            None // TODO: print objects
                        } else if value.is_promise() {
                            Some(format!(
                                "{} (hint: call `await {line}`)",
                                value.to_string_coerced()?
                            ))
                        } else {
                            Some(value.to_string_coerced()?)
                        })
                    })
                    .await;

                match value {
                    Ok(Some(value)) => {
                        println!("{value}");
                    }
                    Ok(None) => {
                        if likely_print_without_newline(&line) {
                            println!();
                        }
                    }
                    Err(err) => {
                        if !scripting::try_emit_script_diagnostic(&err, &line) {
                            eprintln!("{err}");
                        }
                    }
                }

                if cancellation_token.is_cancelled() {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl + C
                if cancellation_token.is_cancelled() {
                    break;
                }
                println!("(hint: press Ctrl+D to exit)");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl + D
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    if let Some(parent) = history_filepath.parent() {
        fs::create_dir_all(parent).await?;
    }
    repl.save_history(&history_filepath)?;

    Ok(())
}

fn likely_print_without_newline(line: &str) -> bool {
    // Best-effort heuristic on source text to emit a trailing newline after `print(...)` calls
    // so the next REPL prompt starts on its own line.
    // Known limitations:
    //  - False positives for identifiers ending in "print", e.g. `myprint(...)`.
    //  - False negatives for indirect calls, e.g. `let p = print; p(...)`.
    //  - Does not parse strings/comments, so `"print("` would match.
    if line.contains("console.print(") {
        return true;
    }

    line.match_indices("print(").any(|(idx, _)| {
        line[..idx]
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
    })
}

fn validate_repl_input<'js>(
    ctx: &Ctx<'js>,
    code: &str,
    script_engine: &Engine,
) -> ValidationResult {
    let (_, js) = match script_engine.prepare_script(code, None, true) {
        Ok(compiled) => compiled,
        Err(err) => {
            return if is_likely_incomplete(code, &err.to_string()) {
                ValidationResult::Incomplete
            } else {
                // Let eval path report syntax/runtime errors to avoid blocking Enter.
                ValidationResult::Valid(None)
            };
        }
    };

    match Module::declare(ctx.clone(), "repl_temp", js).catch(ctx) {
        Ok(_) => ValidationResult::Valid(None),
        Err(err) => {
            let message = err.to_string();
            if is_likely_incomplete(code, &message) {
                ValidationResult::Incomplete
            } else {
                // Let eval path report syntax/runtime errors to avoid blocking Enter.
                ValidationResult::Valid(None)
            }
        }
    }
}

fn is_likely_incomplete(code: &str, message: &str) -> bool {
    let message_lower = message.to_ascii_lowercase();
    if message_lower.contains("<eof>")
        || message_lower.contains("unexpected end")
        || message_lower.contains("unterminated")
        || message_lower.contains("unclosed")
    {
        return true;
    }

    // Count unmatched brackets, skipping those inside string literals and comments.
    let mut balance = 0_i32;
    let mut chars = code.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            // Skip string literals
            '\'' | '"' | '`' => {
                let quote = ch;
                while let Some(c) = chars.next() {
                    if c == '\\' {
                        chars.next(); // skip escaped character
                    } else if c == quote {
                        break;
                    }
                }
            }
            // Skip single-line comments
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            // Skip block comments
            '/' if chars.peek() == Some(&'*') => {
                chars.next(); // consume '*'
                while let Some(c) = chars.next() {
                    if c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        break;
                    }
                }
            }
            '(' | '[' | '{' => balance += 1,
            ')' | ']' | '}' => balance -= 1,
            _ => {}
        }
    }

    balance > 0
}
