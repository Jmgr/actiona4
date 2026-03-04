use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    mem::take,
    sync::Arc,
};

use color_eyre::{Report, Result, eyre::eyre};
use derive_where::derive_where;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Exception, FromJs, Object,
    Promise, Value, async_with, context::EvalOptions, markers::ParallelSend,
};
use swc_common::{
    BytePos, FileName, FilePathMapping, SourceMap, Span,
    errors::{ColorConfig, Handler},
    sync::Lrc,
};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{runtime::WithUserData, scripting::typescript::TsToJs};

pub mod callbacks;
pub mod pragma;
pub mod typescript;

pub type UnhandledException = (String, Vec<CallStackFrame>);

const MAX_STACK_NOTES: usize = 8;

/// Attempts to emit a rich SWC-style diagnostic for a script error.
///
/// Returns `true` if the error was handled and emitted, `false` if the caller
/// should fall back to its own error display.
#[must_use]
pub fn try_emit_script_diagnostic(err: &Report, source_code: &str) -> bool {
    if err
        .downcast_ref::<typescript::EmittedDiagnosticError>()
        .is_some()
    {
        return true;
    }

    let Some(runtime_error) = err.downcast_ref::<RuntimeScriptError>() else {
        return false;
    };

    let (cm, handler) = new_tty_handler();

    let primary_span = runtime_primary_span(runtime_error, source_code, &cm);
    let mut diagnostic = primary_span.map_or_else(
        || handler.struct_err(&runtime_error.message),
        |span| handler.struct_span_err(span, &runtime_error.message),
    );

    let first_note_index = if primary_span.is_some() { 1 } else { 0 };
    for frame in runtime_error
        .stack
        .iter()
        .skip(first_note_index)
        .take(MAX_STACK_NOTES)
    {
        diagnostic.note(&frame.to_string());
    }
    if runtime_error.stack.len() > (first_note_index + MAX_STACK_NOTES) {
        diagnostic.note("...");
    }

    diagnostic.emit();
    true
}

pub(in crate::scripting) fn new_tty_handler() -> (Lrc<SourceMap>, Handler) {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    (cm, handler)
}

#[derive(Clone)]
#[derive_where(Debug)]
pub struct Engine {
    #[derive_where(skip)]
    runtime: AsyncRuntime,

    #[derive_where(skip)]
    context: AsyncContext,

    #[derive_where(skip)]
    sourcemaps: Arc<Mutex<HashMap<u64, TsToJs>>>,

    #[derive_where(skip)]
    unhandled_exceptions: Arc<Mutex<Vec<UnhandledException>>>,
}

static CALLSTACK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*at(?: (?P<func>.+?) \()?(?P<file>.+?):(?P<line>\d+):(?P<col>\d+)\)?$")
        .expect("Failed to compile regex")
});

#[derive(Debug)]
pub struct CallStackFrame {
    function: String,
    file: String,
    line: u32,
    col: u32,
}

impl fmt::Display for CallStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.function.is_empty() {
            write!(f, "    at {}:{}:{}", self.file, self.line, self.col)
        } else {
            write!(
                f,
                "    at {} ({}:{}:{})",
                self.function, self.file, self.line, self.col
            )
        }
    }
}

#[derive(Debug)]
struct RuntimeScriptError {
    message: String,
    stack: Vec<CallStackFrame>,
}

impl RuntimeScriptError {
    const fn new(message: String, stack: Vec<CallStackFrame>) -> Self {
        Self { message, stack }
    }
}

impl fmt::Display for RuntimeScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        for frame in &self.stack {
            write!(f, "\n{frame}")?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeScriptError {}

fn runtime_primary_span(
    runtime_error: &RuntimeScriptError,
    source_code: &str,
    cm: &Lrc<SourceMap>,
) -> Option<Span> {
    let frame = runtime_error.stack.first()?;
    let line_index = usize::try_from(frame.line.checked_sub(1)?).ok()?;
    let column_index = usize::try_from(frame.col.checked_sub(1)?).ok()?;

    let source_file = cm.new_source_file(
        Lrc::new(FileName::Custom(frame.file.clone())),
        source_code.to_string(),
    );

    let line_start = *source_file.analyze().lines.get(line_index)?;
    let line = source_file.get_line(line_index)?;

    if let Some((start_byte, end_byte)) =
        reference_error_identifier_range(&runtime_error.message, &line, frame.col)
    {
        let lo = line_start + BytePos(u32::try_from(start_byte).ok()?);
        let hi = line_start + BytePos(u32::try_from(end_byte).ok()?);
        return Some(Span::new(lo, hi));
    }

    if let Some((start_byte, end_byte)) =
        not_a_function_identifier_range(&runtime_error.message, &line, frame.col)
    {
        let lo = line_start + BytePos(u32::try_from(start_byte).ok()?);
        let hi = line_start + BytePos(u32::try_from(end_byte).ok()?);
        return Some(Span::new(lo, hi));
    }

    let column_byte = line
        .char_indices()
        .nth(column_index)
        .map_or_else(|| line.len(), |(index, _)| index);
    let column_byte = u32::try_from(column_byte).ok()?;
    let lo = line_start + BytePos(column_byte);

    Some(Span::new(lo, lo))
}

fn reference_error_identifier_range(
    message: &str,
    line: &str,
    reported_col: u32,
) -> Option<(usize, usize)> {
    let identifier = parse_reference_error_identifier(message)?;
    let reported_col = usize::try_from(reported_col.checked_sub(1)?).ok()?;
    find_closest_identifier_range(line, identifier, reported_col)
}

fn parse_reference_error_identifier(message: &str) -> Option<&str> {
    let identifier = message
        .strip_prefix("ReferenceError: ")?
        .strip_suffix(" is not defined")?
        .trim();
    let identifier = strip_matching_quotes(identifier);
    is_valid_js_identifier(identifier).then_some(identifier)
}

fn not_a_function_identifier_range(
    message: &str,
    line: &str,
    reported_col: u32,
) -> Option<(usize, usize)> {
    let reported_col = usize::try_from(reported_col.checked_sub(1)?).ok()?;

    if let Some(identifier) = parse_type_error_identifier(message) {
        return find_closest_identifier_range(line, identifier, reported_col);
    }

    is_plain_not_a_function_type_error(message)
        .then(|| find_closest_call_identifier_range(line, reported_col))
        .flatten()
}

fn parse_type_error_identifier(message: &str) -> Option<&str> {
    let identifier = message
        .strip_prefix("TypeError: ")?
        .strip_suffix(" is not a function")?
        .trim();
    let identifier = strip_matching_quotes(identifier);
    is_valid_js_identifier(identifier).then_some(identifier)
}

fn is_plain_not_a_function_type_error(message: &str) -> bool {
    message.trim() == "TypeError: not a function"
}

fn strip_matching_quotes(value: &str) -> &str {
    match value.as_bytes() {
        [b'\'' | b'"', .., last] if *last == value.as_bytes()[0] => &value[1..value.len() - 1],
        _ => value,
    }
}

fn find_closest_identifier_range(
    line: &str,
    identifier: &str,
    reported_col: usize,
) -> Option<(usize, usize)> {
    let ident_char_len = identifier.chars().count();
    let mut best_match: Option<(usize, usize, usize)> = None;
    for (start_byte, _) in line.match_indices(identifier) {
        let end_byte = start_byte + identifier.len();
        if !is_identifier_boundary(line, start_byte, end_byte) {
            continue;
        }

        let start_col = line[..start_byte].chars().count();
        let end_col_exclusive = start_col + ident_char_len;
        let distance = column_distance_to_identifier(reported_col, start_col, end_col_exclusive);
        let candidate = (distance, start_byte, end_byte);

        if best_match.is_none_or(|current| candidate < current) {
            best_match = Some(candidate);
        }
    }

    best_match.map(|(_, start_byte, end_byte)| (start_byte, end_byte))
}

fn find_closest_call_identifier_range(line: &str, reported_col: usize) -> Option<(usize, usize)> {
    let mut best_non_constructor: Option<(usize, usize, usize)> = None;
    let mut best_match: Option<(usize, usize, usize)> = None;
    for (start_byte, ch) in line.char_indices() {
        if !is_js_identifier_start(ch) {
            continue;
        }

        if line[..start_byte]
            .chars()
            .next_back()
            .is_some_and(is_js_identifier_continue)
        {
            continue;
        }

        let mut ident_end_byte = start_byte + ch.len_utf8();
        for next in line[ident_end_byte..].chars() {
            if is_js_identifier_continue(next) {
                ident_end_byte += next.len_utf8();
            } else {
                break;
            }
        }

        let after_ident_byte = ident_end_byte
            + line[ident_end_byte..]
                .chars()
                .take_while(|c| c.is_whitespace())
                .map(char::len_utf8)
                .sum::<usize>();

        let Some(next) = line[after_ident_byte..].chars().next() else {
            continue;
        };
        if next != '(' {
            continue;
        }

        let start_col = line[..start_byte].chars().count();
        let end_col_exclusive = start_col + line[start_byte..ident_end_byte].chars().count();
        let distance = column_distance_to_identifier(reported_col, start_col, end_col_exclusive);
        let candidate = (distance, start_byte, ident_end_byte);
        let is_constructor = is_constructor_call(line, start_byte);

        if best_match.is_none_or(|current| candidate < current) {
            best_match = Some(candidate);
        }
        if !is_constructor && best_non_constructor.is_none_or(|current| candidate < current) {
            best_non_constructor = Some(candidate);
        }
    }

    best_non_constructor
        .or(best_match)
        .map(|(_, start_byte, end_byte)| (start_byte, end_byte))
}

fn is_constructor_call(line: &str, start_byte: usize) -> bool {
    let prefix = line[..start_byte].trim_end_matches(char::is_whitespace);
    let Some(before_new) = prefix.strip_suffix("new") else {
        return false;
    };
    before_new
        .chars()
        .next_back()
        .is_none_or(|ch| !is_js_identifier_continue(ch))
}

const fn column_distance_to_identifier(
    reported_col: usize,
    start_col: usize,
    end_col_exclusive: usize,
) -> usize {
    if reported_col < start_col {
        start_col - reported_col
    } else if reported_col >= end_col_exclusive {
        reported_col - (end_col_exclusive.saturating_sub(1))
    } else {
        0
    }
}

fn is_identifier_boundary(line: &str, start_byte: usize, end_byte: usize) -> bool {
    let prev = line[..start_byte].chars().next_back();
    let next = line[end_byte..].chars().next();
    prev.is_none_or(|ch| !is_js_identifier_continue(ch))
        && next.is_none_or(|ch| !is_js_identifier_continue(ch))
}

fn is_valid_js_identifier(identifier: &str) -> bool {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    is_js_identifier_start(first) && chars.all(is_js_identifier_continue)
}

#[must_use]
pub fn is_js_identifier_start(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_alphabetic()
}

#[must_use]
pub fn is_js_identifier_continue(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_alphanumeric()
}

fn parse_callstack_line(line: &str) -> Result<CallStackFrame> {
    CALLSTACK_REGEX
        .captures(line)
        .and_then(|caps| {
            // Use and_then to chain Option results, returning None if any part fails
            let function = caps.name("func").map_or("", |cap| cap.as_str());
            let file = caps.name("file").map_or("", |cap| cap.as_str());
            // Parse line and col, converting parse errors into None
            let line = caps.name("line")?.as_str().parse::<u32>().ok()?;
            let col = caps.name("col")?.as_str().parse::<u32>().ok()?;

            Some(CallStackFrame {
                function: function.to_string(),
                file: file.to_string(),
                line,
                col,
            })
        })
        .ok_or_else(|| eyre!("failed parsing callstack line: {line}"))
}

impl Engine {
    #[instrument(skip_all)]
    pub async fn new() -> Result<Self> {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;

        let sourcemaps = Arc::new(Mutex::new(Default::default()));
        let unhandled_exceptions = Arc::new(Mutex::new(Vec::default()));

        let sourcemaps_clone = sourcemaps.clone();
        let unhandled_exceptions_clone = unhandled_exceptions.clone();
        runtime
            .set_host_promise_rejection_tracker(Some(Box::new(
                move |_ctx: Ctx, _promise: Value, reason: Value, is_handled: bool| {
                    if is_handled {
                        return;
                    }

                    if let Ok(object) = reason.try_into_exception() {
                        let mut unhandled_exceptions_clone = unhandled_exceptions_clone.lock();
                        if let Ok((message, stack)) =
                            Self::process_exception(object, sourcemaps_clone.clone())
                        {
                            unhandled_exceptions_clone.push((message, stack));
                        } else {
                            unhandled_exceptions_clone.push((
                                "Error: failed to process unhandled promise rejection".to_string(),
                                Vec::new(),
                            ));
                        }
                    }
                },
            )))
            .await;

        Ok(Self {
            runtime,
            context,
            sourcemaps,
            unhandled_exceptions,
        })
    }

    pub async fn with<F, R>(&self, f: F) -> Result<R>
    where
        F: for<'js> FnOnce(Ctx<'js>) -> rquickjs::Result<R> + ParallelSend,
        R: Send,
    {
        let result = self
            .context
            .with(|ctx| f(ctx.clone()).catch(&ctx).map_err(|err| err.to_string()))
            .await
            .map_err(|err| eyre!("{err}"))?;
        Ok(result)
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn prepare_script(
        &self,
        script: &str,
        filename: Option<&str>,
        silent: bool,
    ) -> Result<(u64, String)> {
        let mut hasher = DefaultHasher::new();
        script.hash(&mut hasher);
        let hash = hasher.finish();

        let is_js = filename
            .map(|f| f.ends_with(".js") || f.ends_with(".mjs"))
            .unwrap_or(false);

        let mut sourcemaps = self.sourcemaps.lock();
        let sourcemap = sourcemaps.entry(hash);

        let display_name = filename.unwrap_or("script");

        Ok((
            hash,
            match sourcemap {
                Entry::Occupied(entry) => entry.get().code().to_string(),
                Entry::Vacant(entry) => {
                    if is_js {
                        entry
                            .insert(TsToJs::passthrough(script, display_name))
                            .code()
                            .to_string()
                    } else {
                        let ts_to_js = if silent {
                            TsToJs::new_silent(script, display_name)?
                        } else {
                            TsToJs::new(script, display_name)?
                        };

                        entry.insert(ts_to_js).code().to_string()
                    }
                }
            },
        ))
    }

    pub async fn eval<T>(&self, script: &str) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send,
    {
        self.eval_with_filename(script, None).await
    }

    pub async fn eval_with_filename<T>(&self, script: &str, filename: Option<&str>) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send,
    {
        let (hash, js_code) = self.prepare_script(script, filename, false)?;
        let sourcemaps = self.sourcemaps.clone();

        let mut options = EvalOptions::default();
        options.filename = Some(format!("{hash}"));

        self.context
            .with(|ctx| {
                let result = ctx.eval_with_options::<T, _>(js_code, options).catch(&ctx);
                Self::process_caught_result(result, sourcemaps)
            })
            .await
    }

    // SAFETY: Required due to unsafe operations within rquickjs::async_with! macro
    #[allow(unsafe_op_in_unsafe_fn)]
    #[instrument(skip_all)]
    pub async fn eval_async<T>(&self, script: &str) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send + 'static,
    {
        self.eval_async_with_filename(script, None).await
    }

    // SAFETY: Required due to unsafe operations within rquickjs::async_with! macro
    #[allow(unsafe_op_in_unsafe_fn)]
    #[instrument(skip_all)]
    pub async fn eval_async_with_filename<T>(
        &self,
        script: &str,
        filename: Option<&str>,
    ) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send + 'static,
    {
        let (hash, js_code) = self.prepare_script(script, filename, false)?;
        let sourcemaps = self.sourcemaps.clone();

        async_with!(self.context => |ctx| {
            let mut options = EvalOptions::default();
            options.promise = true;
            options.filename = Some(format!("{hash}"));

            let result = async {
                let func: Promise = ctx.eval_with_options(js_code, options).catch(&ctx)?;
                let future: Object = func.into_future().await.catch(&ctx)?;
                future.get::<_, T>("value").catch(&ctx)
            }.await;

            Self::process_caught_result(result, sourcemaps)
        })
        .await
    }

    // SAFETY: Required due to unsafe operations within rquickjs::async_with! macro
    #[allow(unsafe_op_in_unsafe_fn)]
    #[instrument(skip_all)]
    pub async fn eval_async_fn<T>(
        &self,
        script: &str,
        f: impl FnOnce(Value) -> Result<T> + Send,
    ) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send + 'static,
    {
        let (hash, js_code) = self.prepare_script(script, None, false)?;
        let sourcemaps = self.sourcemaps.clone();

        async_with!(self.context => |ctx| {
            let mut options = EvalOptions::default();
            options.promise = true;
            options.filename = Some(format!("{hash}"));

            let result = async {
                let func: Promise = ctx.eval_with_options(js_code, options).catch(&ctx)?;
                let future: Object = func.into_future().await.catch(&ctx)?;
                future.get::<_, Value>("value").catch(&ctx)
            }.await;

            let result = Self::process_caught_result(result, sourcemaps)?;

            f(result)
        })
        .await
    }

    #[allow(clippy::significant_drop_tightening)]
    fn process_exception(
        exception: Exception,
        sourcemaps: Arc<Mutex<HashMap<u64, TsToJs>>>,
    ) -> Result<UnhandledException> {
        let name: String = exception
            .as_object()
            .get("name")
            .unwrap_or_else(|_| "Error".to_string());
        let message = exception.message().unwrap_or_default();
        let message = format!("{name}: {message}");
        let stack = exception.stack().unwrap_or_default();
        let lines = stack.lines().map(|line| parse_callstack_line(line.trim()));
        let stack = match lines.collect::<Result<Vec<_>>>() {
            Ok(res) => res,
            Err(_) => return Ok((message, Default::default())),
        };

        let stack = stack.into_iter().map(|mut frame| {
            let Ok(source_hash) = frame.file.parse::<u64>() else {
                // File field is not a hash (e.g. already a real filename) — keep as-is
                return Ok(frame);
            };
            let sourcemaps = sourcemaps.lock();
            let Some(ts_to_js) = sourcemaps.get(&source_hash) else {
                return Ok(frame);
            };

            // Replace the hash with the real filename
            frame.file = ts_to_js.filename().to_string();

            // Translate line/col via sourcemap if available
            if let Some((_, ts_line, ts_col)) =
                ts_to_js.lookup_source_location(frame.line, frame.col)
            {
                frame.line = ts_line;
                frame.col = ts_col;
            }

            Ok(frame)
        });
        let stack = stack.collect::<Result<Vec<_>>>()?;

        Ok((message, stack))
    }

    fn process_caught_result<T>(
        result: rquickjs::CaughtResult<T>,
        sourcemaps: Arc<Mutex<HashMap<u64, TsToJs>>>,
    ) -> Result<T> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => match err {
                CaughtError::Error(err) => Err(eyre!("script error: {err}")),
                CaughtError::Exception(exception) => {
                    let (message, stack) = Self::process_exception(exception, sourcemaps)?;
                    Err(eyre!(RuntimeScriptError::new(message, stack)))
                }
                CaughtError::Value(value) => Err(eyre!("script value: {value:?}")),
            },
        }
    }

    /// Sets (or clears) a scoped cancellation token on the JS context.
    /// When set, JS tasks spawned via `task_with_token` will use children of this
    /// token instead of the root runtime token, allowing per-expression cancellation.
    pub async fn set_scoped_cancellation_token(&self, token: Option<CancellationToken>) {
        self.context
            .with(move |ctx| {
                ctx.user_data().set_scoped_cancellation_token(token);
            })
            .await;
    }

    pub async fn idle(&self) -> Vec<UnhandledException> {
        self.runtime.idle().await;

        let mut unhandled_exceptions = self.unhandled_exceptions.lock();

        take(&mut *unhandled_exceptions)
    }

    #[must_use]
    pub fn context(&self) -> AsyncContext {
        self.context.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use swc_common::{FileName, SourceMap, SourceMapper, sync::Lrc};
    use swc_ecma_ast::EsVersion;
    use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
    use tokio::time::Duration;

    use super::*;

    // ──────────────────────────────────────────────────────────────────────────
    // Helper JS class we sometimes expose to JS for sleeps / debug printing.
    // ──────────────────────────────────────────────────────────────────────────
    #[rquickjs::class]
    #[derive(Debug, rquickjs::JsLifetime, rquickjs::class::Trace)]
    struct Helper {}

    #[rquickjs::methods]
    impl Helper {
        async fn sleep(ctx: Ctx<'_>, secs: f64) -> rquickjs::Result<()> {
            let duration = Duration::try_from_secs_f64(secs).map_err(|_| {
                Exception::throw_message(
                    &ctx,
                    "Invalid duration: expected a finite number of seconds greater or equal to 0",
                )
            })?;

            tokio::time::sleep(duration).await;
            Ok(())
        }

        fn log(s: String) {
            println!("{s}");
        }
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 1. Plain JavaScript – simple happy‑path value round‑trip.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn js_basic_add() {
        let engine = Engine::new().await.unwrap();

        let result: i32 = engine
            .eval(
                r#"
                function add(a, b) { return a + b; }
                add(40, 2);
                "#,
            )
            .await
            .unwrap();

        assert_eq!(result, 42);
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 2. Async JavaScript – failure after an await.
    //    We make sure the error bubbles back and keeps its message.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn js_async_error() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .eval_async::<()>(
                r#"
                async function later() {
                    await Promise.resolve("tick");
                    throw new Error("boom");
                }
                await later();
                "#,
            )
            .await
            .unwrap_err();

        assert!(
            err.to_string().contains("boom"),
            "original JS error message must propagate"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 3. TypeScript – verify sourcemap line/column translation.
    //    The thrown error is on TS line 3; we make sure the mapped
    //    frame in Rust’s error string reports line 3 as well.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn ts_error_line_col() {
        let engine = Engine::new().await.unwrap();

        let ts_script = r#"
function outer() {
    const a: number = 1;
    throw new Error("ts fail");   // <-- line 4
}
outer();
"#;

        let err = engine.eval::<()>(ts_script).await.unwrap_err();
        let err_msg = err.to_string();

        assert!(err_msg.contains("ts fail"));

        // Error now uses Display format: "at outer (script:4:11)"
        assert!(
            err_msg.contains("script:4:"),
            "sourcemap should translate JS positions back to TS line 4, got: {err_msg}"
        );
    }

    #[test]
    fn generated_index_d_ts_is_parseable() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../run/assets/index.d.ts");
        let code = std::fs::read_to_string(&path).unwrap();

        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Custom("index.d.ts".into()).into(), code);
        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                dts: true,
                ..Default::default()
            }),
            EsVersion::Es2020,
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);

        let parse_result = parser.parse_program();
        let parser_errors = parser.take_errors();

        assert!(
            parse_result.is_ok() && parser_errors.is_empty(),
            "d.ts parse errors: result={parse_result:?}, errors={parser_errors:?}"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 4. Extra – verify script‑hash caching really keeps the map count at 1.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn script_caching() {
        let engine = Engine::new().await.unwrap();
        let script = "(() => 6 * 7)();";

        // first compile + run
        let first: i32 = engine.eval(script).await.unwrap();
        assert_eq!(first, 42);
        let maps_after_first = engine.sourcemaps.lock().len();

        // second run should *not* add a new TsToJs
        let second: i32 = engine.eval(script).await.unwrap();
        assert_eq!(second, 42);
        assert_eq!(
            engine.sourcemaps.lock().len(),
            maps_after_first,
            "running the identical script twice should hit the cache"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 5. Post‑evaluation async – code spawns work that finishes *after*
    //    Rust's future resolves.  We run `engine.idle()` to flush the queue
    //    and then assert on the side‑effect.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn js_async_after_eval_completes() {
        let engine = Engine::new().await.unwrap();

        // expose our Helper so JS can await a real delay
        engine
            .with(|ctx| {
                let helper = rquickjs::Class::instance(ctx.clone(), Helper {})?;
                ctx.globals().set("helper", helper)?;
                Ok(())
            })
            .await
            .unwrap();

        // JS launches an async function but *doesn't* await it;
        // eval_async therefore resolves immediately with ().
        engine
            .eval_async::<()>(
                r#"
                (async () => {
                    await helper.sleep(0.05);
                    // leave a breadcrumb so we can observe completion from Rust
                    globalThis.__done__ = 99;
                })();
                "#,
            )
            .await
            .unwrap();

        // At this point the async JS body is still pending.
        // Running `idle` processes the job queue in QuickJS.
        engine.idle().await;

        // Pull the breadcrumb back out of JS to prove it ran.
        let done: i32 = engine.eval("globalThis.__done__ ?? 0;").await.unwrap();
        assert_eq!(done, 99, "async body should have executed after idle()");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 6. Exception during `idle()` – tracked via host‑promise‑rejection handler
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn js_error_during_idle() {
        let engine = Engine::new().await.unwrap();

        // Enqueue a promise that rejects *after* the top‑level script returns.
        engine
            .eval::<()>(
                r#"
                let x: number = 42; Promise.resolve().then(() => { throw new Error("idle boom"); });
                "#,
            )
            .await
            .unwrap();

        // Drain the QuickJS job queue; the rejection occurs here.
        let exceptions = engine.idle().await;
        let (message, stack) = exceptions
            .first()
            .expect("no promise rejection captured by the tracker");

        let frame = stack.first().unwrap();

        assert_eq!(frame.line, 2, "idle boom should be on line 2");

        assert!(
            message.contains("idle boom"),
            "unhandled rejection must be reported through the tracker"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 7. Error with custom filename – verify filename appears in error output.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn ts_error_displays_filename() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .eval_with_filename::<()>(
                r#"
function fail() {
    throw new Error("named file error");
}
fail();
"#,
                Some("my_script.ts"),
            )
            .await
            .unwrap_err();

        let err_msg = err.to_string();
        assert!(
            err_msg.contains("my_script.ts"),
            "error should contain the original filename, got: {err_msg}"
        );
        assert!(
            err_msg.contains("named file error"),
            "error should contain the message, got: {err_msg}"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 8. JS file – no transpilation, error shows correct line/col and filename.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn js_error_no_sourcemap() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .eval_with_filename::<()>(
                r#"
function boom() {
    throw new Error("js boom");
}
boom();
"#,
                Some("test.js"),
            )
            .await
            .unwrap_err();

        let err_msg = err.to_string();
        assert!(
            err_msg.contains("test.js"),
            "JS error should contain the filename, got: {err_msg}"
        );
        assert!(
            err_msg.contains("js boom"),
            "JS error should contain the message, got: {err_msg}"
        );
        // Line 3 is where `throw` is
        assert!(
            err_msg.contains("test.js:3:"),
            "JS error should show correct line, got: {err_msg}"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 9. Error formatting – verify human-readable format, not Debug.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn error_backtrace_formatting() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .eval::<()>(
                r#"
function inner() { throw new Error("fmt test"); }
function outer() { inner(); }
outer();
"#,
            )
            .await
            .unwrap_err();

        let err_msg = err.to_string();
        // Should use "at func (file:line:col)" format, not Debug struct format
        assert!(
            err_msg.contains("at inner ("),
            "stack should use human-readable format, got: {err_msg}"
        );
        assert!(
            err_msg.contains("at outer ("),
            "stack should include outer frame, got: {err_msg}"
        );
        // Should NOT contain Debug format artifacts
        assert!(
            !err_msg.contains("CallStackFrame"),
            "error should not use Debug format, got: {err_msg}"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 10. Nested TS call stack – verify all frames are translated.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn ts_nested_call_stack() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .eval_with_filename::<()>(
                r#"
function a() {
    const x: number = 1;
    throw new Error("nested");
}
function b() {
    const y: string = "hello";
    a();
}
b();
"#,
                Some("nested.ts"),
            )
            .await
            .unwrap_err();

        let err_msg = err.to_string();
        assert!(
            err_msg.contains("at a (nested.ts:4:"),
            "should show function a at line 4, got: {err_msg}"
        );
        assert!(
            err_msg.contains("at b (nested.ts:8:"),
            "should show function b at line 8, got: {err_msg}"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 11. Syntax error – should return an error, not panic.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn syntax_error_does_not_crash() {
        let engine = Engine::new().await.unwrap();

        let result = engine.eval::<()>("function {{{ invalid").await;
        assert!(result.is_err(), "syntax error should return Err, not panic");
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 12. Async error with filename – verify filename propagates through async.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn async_error_with_filename() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .eval_async_with_filename::<()>(
                r#"
async function doWork() {
    const val: number = 42;
    throw new Error("async named");
}
await doWork();
"#,
                Some("worker.ts"),
            )
            .await
            .unwrap_err();

        let err_msg = err.to_string();
        assert!(
            err_msg.contains("worker.ts"),
            "async error should contain filename, got: {err_msg}"
        );
        assert!(
            err_msg.contains("async named"),
            "async error should contain message, got: {err_msg}"
        );
    }

    #[test]
    fn reference_error_identifier_parsing() {
        assert_eq!(
            parse_reference_error_identifier("ReferenceError: mouse2 is not defined"),
            Some("mouse2")
        );
        assert_eq!(
            parse_reference_error_identifier("ReferenceError: 'mouse2' is not defined"),
            Some("mouse2")
        );
        assert_eq!(
            parse_reference_error_identifier("TypeError: mouse2 is not defined"),
            None
        );
    }

    #[test]
    fn type_error_identifier_parsing() {
        assert_eq!(
            parse_type_error_identifier("TypeError: mouse2 is not a function"),
            Some("mouse2")
        );
        assert_eq!(
            parse_type_error_identifier("TypeError: 'mouse2' is not a function"),
            Some("mouse2")
        );
        assert_eq!(
            parse_type_error_identifier("TypeError: not a function"),
            None
        );
    }

    #[test]
    fn closest_identifier_range_uses_nearest_match() {
        let line = "mouse2 + x + mouse2";
        let second_start = line
            .rfind("mouse2")
            .expect("line should contain a second identifier");

        assert_eq!(
            find_closest_identifier_range(line, "mouse2", 0),
            Some((0, 6))
        );
        assert_eq!(
            find_closest_identifier_range(line, "mouse2", second_start),
            Some((second_start, second_start + 6))
        );
    }

    #[test]
    fn closest_call_identifier_range_uses_nearest_match() {
        let line = "foo(1) + bar(2)";
        let bar_start = line.rfind("bar").expect("line should contain bar");

        assert_eq!(find_closest_call_identifier_range(line, 0), Some((0, 3)));
        assert_eq!(
            find_closest_call_identifier_range(line, bar_start),
            Some((bar_start, bar_start + 3))
        );
    }

    #[test]
    fn closest_call_identifier_range_prefers_non_constructor() {
        let line = "image.drawCircle(50, 50, 50, new Color(0, 0, 0, 128))";
        let color_start = line.find("Color").expect("line should contain Color");
        let draw_start = line
            .find("drawCircle")
            .expect("line should contain drawCircle");

        assert_eq!(
            find_closest_call_identifier_range(line, color_start),
            Some((draw_start, draw_start + "drawCircle".len()))
        );
    }

    #[test]
    fn runtime_primary_span_highlights_reference_identifier() {
        let runtime_error = RuntimeScriptError::new(
            "ReferenceError: mouse2 is not defined".to_string(),
            vec![CallStackFrame {
                function: String::new(),
                file: "test2.ts".to_string(),
                line: 1,
                col: 21,
            }],
        );
        let source = "await windows.findAt(await mouse2.position());";

        let (cm, _) = new_tty_handler();
        let span = runtime_primary_span(&runtime_error, source, &cm)
            .expect("runtime span should be produced");
        let snippet = cm
            .span_to_snippet(span)
            .expect("span should be mappable to source snippet");

        assert_eq!(snippet, "mouse2");
    }

    #[test]
    fn runtime_primary_span_highlights_not_a_function_callsite() {
        let runtime_error = RuntimeScriptError::new(
            "TypeError: not a function".to_string(),
            vec![CallStackFrame {
                function: String::new(),
                file: "script".to_string(),
                line: 1,
                col: 41,
            }],
        );
        let source = r#"await screen.captureRect(0, 0, 100, 100).save("out.png")"#;

        let (cm, _) = new_tty_handler();
        let span = runtime_primary_span(&runtime_error, source, &cm)
            .expect("runtime span should be produced");
        let snippet = cm
            .span_to_snippet(span)
            .expect("span should be mappable to source snippet");

        assert_eq!(snippet, "save");
    }

    #[test]
    fn runtime_primary_span_highlights_not_a_function_method_not_constructor_argument() {
        let runtime_error = RuntimeScriptError::new(
            "TypeError: not a function".to_string(),
            vec![CallStackFrame {
                function: String::new(),
                file: "script".to_string(),
                line: 1,
                col: 34,
            }],
        );
        let source = "image.drawCircle(50, 50, 50, new Color(0, 0, 0, 128))";

        let (cm, _) = new_tty_handler();
        let span = runtime_primary_span(&runtime_error, source, &cm)
            .expect("runtime span should be produced");
        let snippet = cm
            .span_to_snippet(span)
            .expect("span should be mappable to source snippet");

        assert_eq!(snippet, "drawCircle");
    }
}
