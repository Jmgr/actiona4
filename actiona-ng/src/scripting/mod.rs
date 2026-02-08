use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    hash::{DefaultHasher, Hash, Hasher},
    mem::take,
    sync::Arc,
};

use color_eyre::{Result, eyre::eyre};
use derive_where::derive_where;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Exception, FromJs, Object,
    Promise, Value, async_with, context::EvalOptions, markers::ParallelSend,
};
use tracing::instrument;

use crate::scripting::typescript::TsToJs;

pub mod callbacks;
pub mod typescript;

pub type UnhandledException = (String, Vec<CallStackFrame>);

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
                        let (message, stack) =
                            Self::process_exception(object, sourcemaps_clone.clone()).unwrap();

                        let mut unhandled_exceptions_clone = unhandled_exceptions_clone.lock();
                        unhandled_exceptions_clone.push((message, stack));
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
    pub fn ts_to_js(&self, script: &str) -> Result<(u64, String)> {
        self.prepare_script(script, None)
    }

    #[allow(clippy::significant_drop_tightening)]
    pub fn prepare_script(&self, script: &str, filename: Option<&str>) -> Result<(u64, String)> {
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
                        entry
                            .insert(TsToJs::new(script, display_name)?)
                            .code()
                            .to_string()
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
        let (hash, js_code) = self.prepare_script(script, filename)?;
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
        let (hash, js_code) = self.prepare_script(script, filename)?;
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
        let (hash, js_code) = self.ts_to_js(script)?;
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
        let message = exception.message().unwrap();
        let message = format!("{name}: {message}");
        let stack = exception.stack().unwrap();
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

                    Err(if stack.is_empty() {
                        eyre!("{message}")
                    } else {
                        let stack_str = stack
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join("\n");
                        eyre!("{message}\n{stack_str}")
                    })
                }
                CaughtError::Value(value) => Err(eyre!("script value: {value:?}")),
            },
        }
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
        async fn sleep(secs: f64) -> rquickjs::Result<()> {
            tokio::time::sleep(Duration::from_secs_f64(secs)).await;
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
}
