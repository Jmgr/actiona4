use std::{
    collections::{HashMap, hash_map::Entry},
    hash::{DefaultHasher, Hash, Hasher},
    mem::take,
    sync::{Arc, Mutex},
};

use eyre::{Result, eyre};
use once_cell::sync::Lazy;
use regex::Regex;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Exception, FromJs, Object,
    Promise, Value, async_with, context::EvalOptions, markers::ParallelSend,
};
use swc_common::{
    FileName, FilePathMapping, GLOBALS, Globals, Mark, SourceMap,
    errors::{ColorConfig, Handler},
    source_map::DefaultSourceMapGenConfig,
    sync::Lrc,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecma_transforms_base::{fixer::fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::strip;

struct TsToJs {
    js_code: String,
    sourcemap: swc_sourcemap::SourceMap,
}

impl TsToJs {
    pub fn new(code: &str) -> Result<Self> {
        let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

        let fm = cm.new_source_file(
            Lrc::new(FileName::Custom("file.ts".to_string())),
            code.to_string(),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(Default::default()),
            EsVersion::Es2020,
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        let module = parser
            .parse_program()
            .map_err(|e| e.into_diagnostic(&handler).emit())
            .map_err(|_| eyre!("Module parsing failed"))?;

        let globals = Globals::default();
        let (code, srcmap) = GLOBALS.set(
            &globals,
            || -> Result<(std::string::String, swc_sourcemap::SourceMap)> {
                let unresolved_mark = Mark::new();
                let top_level_mark = Mark::new();

                // Optionally transforms decorators here before the resolver pass
                // as it might produce runtime declarations.

                // Conduct identifier scope analysis
                let module = module.apply(resolver(unresolved_mark, top_level_mark, true));

                // Remove typescript types
                let module = module.apply(strip(unresolved_mark, top_level_mark));

                // Fix up any identifiers with the same name, but different contexts
                let module = module.apply(hygiene());

                // Ensure that we have enough parenthesis.
                let program = module.apply(fixer(None));

                let mut code = Vec::new();
                let mut srcmap = Vec::new();

                {
                    let mut emitter = Emitter {
                        cfg: Default::default(),
                        cm: cm.clone(),
                        comments: None,
                        wr: JsWriter::new(cm.clone(), "\n", &mut code, Some(&mut srcmap)),
                    };

                    emitter.emit_program(&program)?;
                }

                let srcmap = cm.build_source_map(&srcmap, None, DefaultSourceMapGenConfig);
                let code = String::from_utf8(code)?;

                Ok((code, srcmap))
            },
        )?;

        Ok(Self {
            js_code: code,
            sourcemap: srcmap,
        })
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn code(&self) -> &str {
        &self.js_code
    }

    /*
    /// Returns the generated sourcemap as a string.
    pub fn sourcemap_string(&self) -> Result<String> {
        let mut writer = Vec::new();
        self.sourcemap.to_writer(&mut writer)?;
        Ok(String::from_utf8(writer)?)
    }
    */

    /// Looks up the original TypeScript location for a given JavaScript line and column.
    ///
    /// # Arguments
    ///
    /// * `js_line` - The 1-based line number in the generated JavaScript code.
    /// * `js_col` - The 1-based column number in the generated JavaScript code.
    ///
    /// # Returns
    ///
    /// An `Option` containing a tuple `(filename, ts_line, ts_col)` if a mapping exists:
    /// * `filename`: The original source file name (e.g., "file.ts").
    /// * `ts_line`: The 1-based line number in the original TypeScript file.
    /// * `ts_col`: The 1-based column number in the original TypeScript file.
    ///
    /// Returns `None` if no mapping is found for the given JavaScript location.
    fn lookup_source_location(&self, js_line: u32, js_col: u32) -> Option<(String, u32, u32)> {
        // Input validation: lines and columns are typically 1-based for users,
        // but the sourcemap crate expects 0-based.
        if js_line == 0 || js_col == 0 {
            return None; // Or handle as an error, 1-based indexing is conventional
        }

        // Convert 1-based user input to 0-based for the lookup
        let zero_based_line = js_line - 1;
        let zero_based_col = js_col - 1;

        // Perform the lookup using the sourcemap crate
        self.sourcemap
            .lookup_token(zero_based_line, zero_based_col)
            .and_then(|token| {
                // Check if the token has source information
                match (
                    token.get_source(),
                    token.get_src_line(),
                    token.get_src_col(),
                ) {
                    (Some(filename), src_line, src_col) => {
                        // Convert 0-based source location back to 1-based for the user
                        let one_based_ts_line = src_line + 1;
                        let one_based_ts_col = src_col + 1;
                        Some((filename.to_string(), one_based_ts_line, one_based_ts_col))
                    }
                    _ => None, // No mapping found for this specific token
                }
            })
    }
}

pub type UnhandledException = (String, Vec<CallStackFrame>);

pub struct Engine {
    runtime: AsyncRuntime,
    context: AsyncContext,
    sourcemaps: Arc<Mutex<HashMap<u64, TsToJs>>>,
    unhandled_exceptions: Arc<Mutex<Vec<UnhandledException>>>,
}

static CALLSTACK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*at(?: (?P<func>.+?) \()?(?P<file>.+?):(?P<line>\d+):(?P<col>\d+)\)?$")
        .expect("Failed to compile regex")
});

#[derive(Debug)]
pub struct CallStackFrame {
    _function: String,
    file: String,
    line: usize,
    col: usize,
}

fn parse_callstack_line(line: &str) -> Result<CallStackFrame> {
    CALLSTACK_REGEX
        .captures(line)
        .and_then(|caps| {
            // Use and_then to chain Option results, returning None if any part fails
            let function = caps.name("func").map_or("", |cap| cap.as_str());
            let file = caps.name("file").map_or("", |cap| cap.as_str());
            // Parse line and col, converting parse errors into None
            let line = caps.name("line")?.as_str().parse::<usize>().ok()?;
            let col = caps.name("col")?.as_str().parse::<usize>().ok()?;

            Some(CallStackFrame {
                _function: function.to_string(),
                file: file.to_string(),
                line,
                col,
            })
        })
        .ok_or_else(|| eyre!("failed parsing callstack line: {line}"))
}

impl Engine {
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

                        let mut unhandled_exceptions_clone =
                            unhandled_exceptions_clone.lock().unwrap();
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

    pub async fn with<F, R>(&self, f: F) -> R
    where
        F: for<'js> FnOnce(Ctx<'js>) -> R + ParallelSend,
        R: ParallelSend,
    {
        self.context.with(f).await
    }

    #[allow(clippy::significant_drop_tightening)]
    fn ts_to_js(&self, script: &str) -> Result<(u64, String)> {
        let mut hasher = DefaultHasher::new();
        script.hash(&mut hasher);
        let hash = hasher.finish();

        let mut sourcemaps = self.sourcemaps.lock().unwrap();
        let sourcemap = sourcemaps.entry(hash);

        Ok((
            hash,
            match sourcemap {
                Entry::Occupied(entry) => entry.get().code().to_string(),
                Entry::Vacant(entry) => entry.insert(TsToJs::new(script)?).code().to_string(),
            },
        ))
    }

    pub async fn eval<T>(&mut self, script: &str) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send,
    {
        let (hash, js_code) = self.ts_to_js(script)?;
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
    pub async fn eval_async<T>(&mut self, script: &str) -> Result<T>
    where
        for<'any_js> T: FromJs<'any_js> + Send + 'static + std::fmt::Debug,
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
                future.get::<_, T>("value").catch(&ctx)
            }.await;

            Self::process_caught_result(result, sourcemaps)
        })
        .await
    }

    #[allow(clippy::significant_drop_tightening)]
    fn process_exception(
        exception: Exception,
        sourcemaps: Arc<Mutex<HashMap<u64, TsToJs>>>,
    ) -> Result<UnhandledException> {
        let message = exception.message().unwrap();
        let stack = exception.stack().unwrap();
        let lines = stack.lines().map(|line| parse_callstack_line(line.trim()));
        let stack = lines.collect::<Result<Vec<_>>>()?;

        let stack = stack.into_iter().map(|mut frame| {
            let Ok(source_hash) = frame.file.parse() else {
                return Ok(CallStackFrame {
                    _function: "unknown".to_string(),
                    file: "unknown".to_string(),
                    line: 0,
                    col: 0,
                });
            };
            let sourcemaps = sourcemaps.lock().unwrap();
            let ts_to_js = sourcemaps.get(&source_hash).ok_or_else(|| {
                eyre!("failed to find sourcemap for code with hash {source_hash}")
            })?;
            let ts_line_col = ts_to_js
                .lookup_source_location(frame.line as u32, frame.col as u32)
                .ok_or_else(|| eyre!("failed finding line and col, frame: {frame:?}"))?;
            frame.line = ts_line_col.1 as usize;
            frame.col = ts_line_col.2 as usize;
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
                        eyre!("{message}: {stack:?}")
                    })
                }
                CaughtError::Value(value) => Err(eyre!("script value: {value:?}")),
            },
        }
    }

    pub async fn idle(&self) -> Vec<UnhandledException> {
        self.runtime.idle().await;

        let mut unhandled_exceptions = self.unhandled_exceptions.lock().unwrap();

        take(&mut *unhandled_exceptions)
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;
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
        let mut engine = Engine::new().await.unwrap();

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
        let mut engine = Engine::new().await.unwrap();

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
        let mut engine = Engine::new().await.unwrap();

        let ts_script = r#"
function outer() {
    const a: number = 1;
    throw new Error("ts fail");   // <-- line 4
}
outer();
"#;

        let err = engine.eval::<()>(ts_script).await.unwrap_err();
        let err_msg = err.to_string();

        // quick sanity       …
        assert!(err_msg.contains("ts fail"));

        // pull the first "line: <n>" out of the Debug‑printed CallStackFrame
        let line_rx = Regex::new(r"line:\s*(\d+)").unwrap();
        let cap = line_rx
            .captures(&err_msg)
            .expect("error string should contain a stack frame");
        let mapped_line: u32 = cap[1].parse().unwrap();

        assert_eq!(
            mapped_line, 4,
            "sourcemap should translate JS positions back to TS line 4"
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // 4. Extra – verify script‑hash caching really keeps the map count at 1.
    // ──────────────────────────────────────────────────────────────────────────
    #[tokio::test]
    async fn script_caching() {
        let mut engine = Engine::new().await.unwrap();
        let script = "(() => 6 * 7)();";

        // first compile + run
        let first: i32 = engine.eval(script).await.unwrap();
        assert_eq!(first, 42);
        let maps_after_first = engine.sourcemaps.lock().unwrap().len();

        // second run should *not* add a new TsToJs
        let second: i32 = engine.eval(script).await.unwrap();
        assert_eq!(second, 42);
        assert_eq!(
            engine.sourcemaps.lock().unwrap().len(),
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
        let mut engine = Engine::new().await.unwrap();

        // expose our Helper so JS can await a real delay
        engine
            .with(|ctx| {
                let helper = rquickjs::Class::instance(ctx.clone(), Helper {}).unwrap();
                ctx.globals().set("helper", helper).unwrap();
            })
            .await;

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
        let mut engine = Engine::new().await.unwrap();

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

        assert_eq!(frame.line, 2);
        assert_eq!(frame.col, 78);

        assert!(
            message.contains("idle boom"),
            "unhandled rejection must be reported through the tracker"
        );
    }
}
