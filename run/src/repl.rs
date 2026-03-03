use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use actiona_core::{
    format_js_value_for_console,
    scripting::{self, Engine, is_js_identifier_continue, is_js_identifier_start},
};
use clap::{CommandFactory, Parser};
use color_eyre::{
    Report, Result,
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
use tokio::{fs, runtime::Handle, select, signal};
use tokio_util::sync::CancellationToken;
use tracing::{instrument, warn};
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

    const fn complete_js(&self, _input: &str) -> rustyline::Result<(usize, Vec<Pair>)> {
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
            let script = fs::read_to_string(&filename).await?;
            let value = script_engine
                .eval_async_with_filename::<Option<Coerced<String>>>(&script, Some(&filename))
                .await;
            if let Some(value) = value? {
                println!("{}", value.0);
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
    let syntax_reference = syntax_set.find_syntax_by_extension("ts").map_or_else(
        || {
            warn!(
                extension = "ts",
                "typescript syntax definition not found, falling back to plain text highlighting"
            );
            syntax_set.find_syntax_plain_text().clone()
        },
        Clone::clone,
    );

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

                // Create a per-expression cancellation token so Ctrl+C cancels
                // only the current expression, not the entire runtime.
                let expr_token = CancellationToken::new();
                script_engine
                    .set_scoped_cancellation_token(Some(expr_token.clone()))
                    .await;

                let eval_future = script_engine.eval_async_fn::<Option<String>>(&line, |value| {
                    Ok(if value.is_undefined() {
                        None
                    } else if value.is_promise() {
                        let rendered = format_js_value_for_console(value.clone());
                        Some(format!("{} (hint: call `await {line}`)", rendered))
                    } else {
                        Some(format_js_value_for_console(value))
                    })
                });

                let value = select! {
                    result = eval_future => Some(result),
                    _ = signal::ctrl_c() => {
                        expr_token.cancel();
                        None
                    }
                };

                script_engine.set_scoped_cancellation_token(None).await;

                match value {
                    Some(Ok(Some(value))) => {
                        println!("{value}");
                    }
                    Some(Ok(None)) => {
                        if likely_print_without_newline(&line) {
                            println!();
                        }
                    }
                    Some(Err(err)) => {
                        if !scripting::try_emit_script_diagnostic(&err, &line) {
                            eprintln!("{err}");
                        }

                        if let Some(receiver) =
                            missing_await_promise_receiver_hint(&script_engine, &line, &err).await
                        {
                            eprintln!("(hint: call `await {receiver}`)");
                        }
                    }
                    None => {
                        // Ctrl+C interrupted the expression
                        println!();
                    }
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

async fn missing_await_promise_receiver_hint(
    script_engine: &Engine,
    line: &str,
    err: &Report,
) -> Option<String> {
    if !is_plain_not_a_function_error(err) {
        return None;
    }

    let receiver = extract_simple_member_call_receiver(line)?;
    let receiver_name = receiver.to_string();
    let lookup_name = receiver_name.clone();
    let is_promise = script_engine
        .with(move |ctx| {
            // `let` bindings in the REPL are lexical bindings and may not exist on `globalThis`.
            // Evaluate the identifier directly so we can detect both lexical and global bindings.
            let value = match ctx.eval::<Value, _>(lookup_name.as_str()) {
                Ok(value) => value,
                Err(_) => return Ok(false),
            };
            Ok(value.is_promise())
        })
        .await
        .ok()?;

    is_promise.then_some(receiver_name)
}

fn is_plain_not_a_function_error(err: &Report) -> bool {
    err.to_string()
        .lines()
        .next()
        .is_some_and(|line| line.trim() == "TypeError: not a function")
}

fn extract_simple_member_call_receiver(line: &str) -> Option<&str> {
    for (open_paren_idx, ch) in line.char_indices() {
        if ch != '(' {
            continue;
        }

        let method_end = trim_trailing_whitespace(line, open_paren_idx);
        let (method_start, _) = parse_identifier_ending_at(line, method_end)?;
        let before_method = trim_trailing_whitespace(line, method_start);
        let (dot_idx, dot) = line[..before_method].char_indices().next_back()?;
        if dot != '.' {
            continue;
        }

        let receiver_end = trim_trailing_whitespace(line, dot_idx);
        let (receiver_start, receiver_end) = parse_identifier_ending_at(line, receiver_end)?;
        return Some(&line[receiver_start..receiver_end]);
    }

    None
}

fn trim_trailing_whitespace(line: &str, mut end: usize) -> usize {
    while let Some((idx, ch)) = line[..end].char_indices().next_back() {
        if ch.is_whitespace() {
            end = idx;
        } else {
            break;
        }
    }
    end
}

fn parse_identifier_ending_at(line: &str, end: usize) -> Option<(usize, usize)> {
    let (mut start, last) = line[..end].char_indices().next_back()?;
    if !is_js_identifier_continue(last) {
        return None;
    }

    while let Some((idx, ch)) = line[..start].char_indices().next_back() {
        if is_js_identifier_continue(ch) {
            start = idx;
        } else {
            break;
        }
    }

    let first = line[start..end].chars().next()?;
    is_js_identifier_start(first).then_some((start, end))
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

#[cfg(test)]
mod tests {
    use actiona_core::scripting::Engine;

    use super::{extract_simple_member_call_receiver, missing_await_promise_receiver_hint};

    #[test]
    fn extract_simple_member_call_receiver_from_method_call() {
        let line = "image.drawCircle(50, 50, 50, new Color(0, 0, 0, 128))";
        assert_eq!(extract_simple_member_call_receiver(line), Some("image"));
    }

    #[test]
    fn extract_simple_member_call_receiver_ignores_non_member_call() {
        let line = "drawCircle(50, 50, 50)";
        assert_eq!(extract_simple_member_call_receiver(line), None);
    }

    #[test]
    fn extract_simple_member_call_receiver_handles_spaces() {
        let line = "image . drawCircle (50)";
        assert_eq!(extract_simple_member_call_receiver(line), Some("image"));
    }

    #[tokio::test]
    async fn missing_await_promise_receiver_hint_detects_lexical_binding() {
        let engine = Engine::new().await.expect("engine should initialize");
        engine
            .eval::<()>("let image = Promise.resolve({});")
            .await
            .expect("setup script should succeed");

        let err = engine
            .eval::<()>("image.drawCircle(50);")
            .await
            .expect_err("calling missing method on Promise should fail");

        let hint = missing_await_promise_receiver_hint(&engine, "image.drawCircle(50)", &err).await;
        assert_eq!(hint.as_deref(), Some("image"));
    }
}
