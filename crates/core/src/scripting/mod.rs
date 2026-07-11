//! JavaScript/TypeScript scripting engine.
//!
//! Error handling spans three layers, each with its own error type:
//! - the public API returns the module's [`Result`]/[`ScriptError`];
//! - the TypeScript transpiler returns [`typescript::TranspileError`] (surfaced as
//!   [`ScriptError::Compile`]);
//! - the callbacks worker works at the rquickjs FFI boundary and uses `rquickjs::Result`.
//!
//! Caught JavaScript errors from every entry point (`eval*` and [`Engine::with`]) flow
//! through [`Engine::process_caught_result`] so they share one structured, source-mapped path.

use std::{
    collections::{HashMap, hash_map::Entry},
    hash::{DefaultHasher, Hash, Hasher},
    mem::take,
    sync::Arc,
};

use derive_where::derive_where;
use parking_lot::Mutex;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, CaughtResult, Coerced, Ctx, Exception,
    FromJs, Object, Promise, Value, context::EvalOptions, markers::ParallelSend,
};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{error::CommonError, runtime::WithUserData, scripting::typescript::TsToJs};

pub mod callbacks;
mod diagnostics;
mod error;
pub mod typescript;

pub(in crate::scripting) use diagnostics::new_tty_handler;
#[cfg(test)]
use diagnostics::{
    find_closest_call_identifier_range, find_closest_identifier_range,
    parse_reference_error_identifier, parse_type_error_identifier, runtime_primary_span,
};
pub use diagnostics::{
    is_js_identifier_continue, is_js_identifier_start, try_emit_script_diagnostic,
};
use error::parse_callstack;
pub use error::{CallStackFrame, Result, RuntimeScriptError, ScriptError, UnhandledException};

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

impl Engine {
    #[instrument(skip_all)]
    pub async fn new() -> Result<Self> {
        let runtime = AsyncRuntime::new().map_err(ScriptError::quickjs)?;
        let context = AsyncContext::full(&runtime)
            .await
            .map_err(ScriptError::quickjs)?;

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
                        let processed = Self::process_exception(object, sourcemaps_clone.clone());
                        let mut unhandled_exceptions_clone = unhandled_exceptions_clone.lock();
                        unhandled_exceptions_clone.push((processed.message, processed.stack));
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
        let sourcemaps = self.sourcemaps.clone();
        self.context
            .with(move |ctx| {
                let result = f(ctx.clone()).catch(&ctx);
                Self::process_caught_result(result, sourcemaps)
            })
            .await
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

    #[instrument(skip_all)]
    pub async fn eval_async<T>(&self, script: &str) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send + 'static,
    {
        self.eval_async_with_filename(script, None).await
    }

    /// Evaluates a script and processes its resulting JavaScript value inside
    /// the engine context.
    #[instrument(skip_all)]
    pub async fn eval_async_with<T>(
        &self,
        script: &str,
        f: impl for<'js> FnOnce(Ctx<'js>, Value<'js>) -> rquickjs::Result<T> + ParallelSend,
    ) -> Result<T>
    where
        T: Send + 'static,
    {
        let (hash, js_code) = self.prepare_script(script, None, false)?;
        let sourcemaps = self.sourcemaps.clone();

        self.context
            .async_with(async |ctx| {
                let mut options = EvalOptions::default();
                options.promise = true;
                options.filename = Some(format!("{hash}"));

                let result = async {
                    let func: Promise = ctx.eval_with_options(js_code, options).catch(&ctx)?;
                    let future: Object = func.into_future().await.catch(&ctx)?;
                    future.get::<_, Value>("value").catch(&ctx)
                }
                .await;
                let value = Self::process_caught_result(result, sourcemaps.clone())?;

                Self::process_caught_result(f(ctx.clone(), value).catch(&ctx), sourcemaps)
            })
            .await
    }

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

        self.context
            .async_with(async |ctx| {
                let mut options = EvalOptions::default();
                options.promise = true;
                options.filename = Some(format!("{hash}"));

                let result = async {
                    let func: Promise = ctx.eval_with_options(js_code, options).catch(&ctx)?;
                    let future: Object = func.into_future().await.catch(&ctx)?;
                    future.get::<_, T>("value").catch(&ctx)
                }
                .await;

                Self::process_caught_result(result, sourcemaps)
            })
            .await
    }

    #[instrument(skip_all)]
    pub async fn eval_async_fn_result<T, E>(
        &self,
        script: &str,
        f: impl FnOnce(Value) -> std::result::Result<T, E> + Send,
        map_script_error: impl Fn(ScriptError) -> E + Send + Sync,
    ) -> std::result::Result<T, E>
    where
        T: Send + 'static,
        E: Send + 'static,
    {
        self.eval_async_values_fn_result(
            &[script],
            |mut values| f(values.remove(0)),
            |_, error| map_script_error(error),
        )
        .await
    }

    #[instrument(skip_all)]
    pub async fn eval_async_values_fn_result<T, E>(
        &self,
        scripts: &[&str],
        f: impl FnOnce(Vec<Value>) -> std::result::Result<T, E> + Send,
        map_script_error: impl Fn(usize, ScriptError) -> E + Send + Sync,
    ) -> std::result::Result<T, E>
    where
        T: Send + 'static,
        E: Send + 'static,
    {
        let js_codes = scripts
            .iter()
            .enumerate()
            .map(|(index, script)| {
                self.prepare_script(script, None, false)
                    .map_err(|error| map_script_error(index, error))
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let sourcemaps = self.sourcemaps.clone();

        self.context
            .async_with(async |ctx| {
                let mut values = Vec::with_capacity(js_codes.len());

                for (index, (hash, js_code)) in js_codes.into_iter().enumerate() {
                    let mut options = EvalOptions::default();
                    options.promise = true;
                    options.filename = Some(format!("{hash}"));

                    let result = async {
                        let func: Promise = ctx.eval_with_options(js_code, options).catch(&ctx)?;
                        let future: Object = func.into_future().await.catch(&ctx)?;
                        future.get::<_, Value>("value").catch(&ctx)
                    }
                    .await;

                    values.push(
                        Self::process_caught_result(result, sourcemaps.clone())
                            .map_err(|error| map_script_error(index, error))?,
                    );
                }

                f(values)
            })
            .await
    }

    #[allow(clippy::significant_drop_tightening)]
    fn process_exception(
        exception: Exception,
        sourcemaps: Arc<Mutex<HashMap<u64, TsToJs>>>,
    ) -> ProcessedException {
        let name: String = exception
            .as_object()
            .get("name")
            .unwrap_or_else(|_| "Error".to_string());
        let raw_message = exception.message().unwrap_or_default();
        let cancelled = raw_message == CommonError::Cancelled.to_string();
        let message = format!("{name}: {raw_message}");
        let stack = exception.stack().unwrap_or_default();

        let stack = parse_callstack(&stack)
            .into_iter()
            .map(|mut frame| {
                let Ok(source_hash) = frame.file().parse::<u64>() else {
                    // File field is not a hash (e.g. already a real filename) — keep as-is
                    return frame;
                };
                let sourcemaps = sourcemaps.lock();
                let Some(ts_to_js) = sourcemaps.get(&source_hash) else {
                    return frame;
                };

                // Replace the hash with the real filename
                frame.set_file(ts_to_js.filename());

                // Translate line/col via sourcemap if available
                if let Some((_, ts_line, ts_col)) =
                    ts_to_js.lookup_source_location(frame.line(), frame.column())
                {
                    frame.set_line(ts_line);
                    frame.set_column(ts_col);
                }

                frame
            })
            .collect();

        ProcessedException {
            message,
            stack,
            cancelled,
        }
    }

    fn process_caught_result<T>(
        result: CaughtResult<T>,
        sourcemaps: Arc<Mutex<HashMap<u64, TsToJs>>>,
    ) -> Result<T> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => match err {
                CaughtError::Error(err) => Err(ScriptError::quickjs(err)),
                CaughtError::Exception(exception) => {
                    let processed = Self::process_exception(exception, sourcemaps);
                    Err(RuntimeScriptError::new(
                        processed.message,
                        processed.stack,
                        processed.cancelled,
                    )
                    .into())
                }
                CaughtError::Value(value) => Err(ScriptError::Value(value_to_string(&value))),
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

/// A processed JavaScript exception: a formatted message, a source-mapped call stack,
/// and whether it represents a cancellation.
struct ProcessedException {
    message: String,
    stack: Vec<CallStackFrame>,
    cancelled: bool,
}

/// Converts a thrown JavaScript value into a human-readable string using JS `String(value)`
/// coercion, falling back to the value's type name if coercion fails.
fn value_to_string(value: &Value<'_>) -> String {
    Coerced::<String>::from_js(value.ctx(), value.clone())
        .map(|coerced| coerced.0)
        .unwrap_or_else(|_| value.type_name().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use macros::{FromJsObject, PlatformValidate, js_class, js_methods, options, platform};
    use rquickjs::Function;
    use swc_common::{FileName, SourceMap, SourceMapper, sync::Lrc};
    use swc_ecma_ast::EsVersion;
    use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
    use tokio::time::Duration;

    use super::*;
    use crate::{IntoJsResult, platform_info::Platform};

    // ──────────────────────────────────────────────────────────────────────────
    // Helper JS class we sometimes expose to JS for sleeps / debug printing.
    // ──────────────────────────────────────────────────────────────────────────
    #[js_class]
    #[derive(Debug, rquickjs::JsLifetime, rquickjs::class::Trace)]
    struct JsHelper {}

    #[js_methods]
    impl JsHelper {
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

    #[derive(Debug, Default, rquickjs::JsLifetime, rquickjs::class::Trace)]
    #[js_class]
    struct JsMacroCounter {
        value: i32,
    }

    #[js_methods]
    impl JsMacroCounter {
        #[qjs(constructor)]
        fn new() -> Self {
            Self::default()
        }

        #[get("value")]
        fn value(&self) -> i32 {
            self.value
        }

        #[set("value")]
        fn set_value(&mut self, value: i32) {
            self.value = value;
        }
    }

    #[options]
    #[derive(Clone, Debug, FromJsObject, PlatformValidate)]
    struct JsMacroOptions {
        #[default(true)]
        enabled: bool,
        #[platform(only = "linux")]
        linux_label: Option<String>,
        #[platform(only = "windows")]
        windows_label: Option<String>,
    }

    fn validate_macro_options(ctx: Ctx<'_>, options: JsMacroOptions) -> rquickjs::Result<bool> {
        options
            .validate_for_platform(Platform::detect())
            .into_js_result(&ctx)?;
        Ok(options.enabled)
    }

    #[platform(only = "linux")]
    fn linux_only_marker() -> color_eyre::Result<&'static str> {
        Ok("linux")
    }

    #[platform(only = "windows")]
    fn windows_only_marker() -> color_eyre::Result<&'static str> {
        Ok("windows")
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
                let helper = rquickjs::Class::instance(ctx.clone(), JsHelper {})?;
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

    #[tokio::test]
    async fn macro_helpers_work_with_engine() {
        let engine = Engine::new().await.unwrap();

        engine
            .with(|ctx| {
                rquickjs::Class::<JsMacroCounter>::define(&ctx.globals())?;
                ctx.globals().prop(
                    "validateMacroOptions",
                    Function::new(ctx.clone(), validate_macro_options),
                )?;
                Ok(())
            })
            .await
            .unwrap();

        let incremented_value: i32 = engine
            .eval(
                r#"
                const counter = new MacroCounter();
                counter.value = 41;
                counter.value + 1;
                "#,
            )
            .await
            .unwrap();
        assert_eq!(incremented_value, 42);

        let valid_options_script = if Platform::detect().is_windows() {
            r#"validateMacroOptions({ enabled: false, windowsLabel: "ok" });"#
        } else {
            r#"validateMacroOptions({ enabled: false, linuxLabel: "ok" });"#
        };
        let enabled: bool = engine.eval(valid_options_script).await.unwrap();
        assert!(!enabled);

        let invalid_options_script = if Platform::detect().is_windows() {
            r#"validateMacroOptions({ linuxLabel: "nope" });"#
        } else {
            r#"validateMacroOptions({ windowsLabel: "nope" });"#
        };
        let invalid_options_error = engine
            .eval::<bool>(invalid_options_script)
            .await
            .unwrap_err();
        assert!(
            invalid_options_error
                .to_string()
                .contains("only available on"),
            "unexpected validation error: {invalid_options_error}"
        );
    }

    #[test]
    fn platform_macro_respects_current_platform() {
        if Platform::detect().is_windows() {
            assert_eq!(windows_only_marker().unwrap(), "windows");

            let error = linux_only_marker().unwrap_err();
            assert!(error.to_string().contains("only available on Linux"));
        } else {
            assert_eq!(linux_only_marker().unwrap(), "linux");

            let error = windows_only_marker().unwrap_err();
            assert!(error.to_string().contains("only available on Windows"));
        }
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

        assert_eq!(frame.line(), 2, "idle boom should be on line 2");

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

    #[tokio::test]
    async fn runtime_script_error_exposes_primary_location() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .eval_async_with_filename::<()>(
                r#"
function fail() {
    throw new Error("structured location");
}
await fail();
"#,
                Some("location.ts"),
            )
            .await
            .unwrap_err();

        let runtime_error = err
            .runtime_error()
            .expect("error should preserve RuntimeScriptError");
        let frame = runtime_error
            .primary_frame()
            .expect("runtime error should expose a primary frame");

        assert!(runtime_error.message().contains("structured location"));
        assert_eq!(frame.function(), "fail");
        assert_eq!(frame.file(), "location.ts");
        assert_eq!(frame.line(), 3);
        assert!(frame.column() > 0);
        assert_eq!(runtime_error.stack().first().unwrap().line(), frame.line());
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
            vec![CallStackFrame::new("", "test2.ts", 1, 21)],
            false,
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
            vec![CallStackFrame::new("", "script", 1, 41)],
            false,
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
            vec![CallStackFrame::new("", "script", 1, 34)],
            false,
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

    #[test]
    fn parse_callstack_keeps_parseable_frames() {
        let stack = "    at good1 (script:2:3)\n\
             this line is not a stack frame\n\
                 at good2 (script:5:6)";

        let frames = parse_callstack(stack);
        assert_eq!(
            frames.len(),
            2,
            "malformed line should be skipped, not fatal"
        );
        assert_eq!(frames[0].function(), "good1");
        assert_eq!(frames[0].line(), 2);
        assert_eq!(frames[1].function(), "good2");
        assert_eq!(frames[1].line(), 5);
    }

    #[test]
    fn cancelled_runtime_error_is_suppressed() {
        let runtime_error =
            RuntimeScriptError::new("Error: Cancelled".to_string(), Vec::new(), true);
        assert!(runtime_error.is_cancelled());

        let err: ScriptError = runtime_error.into();
        assert!(err.is_cancelled());
        // A cancelled error is treated as "handled" so callers stay silent.
        assert!(try_emit_script_diagnostic(&err, ""));
    }

    #[tokio::test]
    async fn with_closure_exception_is_structured() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .with(|ctx| ctx.eval::<(), _>("throw new Error('with boom');"))
            .await
            .unwrap_err();

        let runtime_error = err
            .runtime_error()
            .expect("with() should surface a structured RuntimeScriptError, like eval()");
        assert!(
            runtime_error.message().contains("with boom"),
            "got: {}",
            runtime_error.message()
        );
    }

    #[tokio::test]
    async fn thrown_value_is_coerced_not_debug_formatted() {
        let engine = Engine::new().await.unwrap();

        let string_err = engine.eval::<()>("throw 'oops';").await.unwrap_err();
        assert!(matches!(string_err, ScriptError::Value(_)));
        assert!(
            string_err.to_string().contains("oops"),
            "thrown string should be coerced, got: {string_err}"
        );

        let number_err = engine.eval::<()>("throw 42;").await.unwrap_err();
        assert!(
            number_err.to_string().contains("42"),
            "thrown number should be coerced, got: {number_err}"
        );
    }

    #[tokio::test]
    async fn prepare_script_surfaces_compile_error() {
        let engine = Engine::new().await.unwrap();

        let err = engine
            .prepare_script("function {{{ invalid", Some("bad.ts"), true)
            .unwrap_err();

        assert!(
            matches!(err, ScriptError::Compile(_)),
            "TS compile failure should be a ScriptError::Compile, got: {err:?}"
        );
    }
}
