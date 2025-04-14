//! @verbatim /**
//! @verbatim  * A cancellable promise.
//! @verbatim  *
//! @verbatim  * ```ts
//! @verbatim  * const task = sleep("5s");
//! @verbatim  * task.cancel(); // cancel early
//! @verbatim  * await task;    // or await it
//! @verbatim  * ```
//! @verbatim  */
//! @verbatim type Task<Result> = Promise<Result> & {
//! @verbatim     cancel(): void;
//! @verbatim };
//! @verbatim
//! @verbatim /**
//! @verbatim  * A cancellable promise that also emits progress updates.
//! @verbatim  *
//! @verbatim  * ```ts
//! @verbatim  * const task = source.find(template);
//! @verbatim  * for await (const progress of task) {
//! @verbatim  *   console.println(`Stage: ${progress.stage}`);
//! @verbatim  * }
//! @verbatim  * const match = await task;
//! @verbatim  * ```
//! @verbatim  */
//! @verbatim type ProgressTask<Result, Progress> = Task<Result> & {
//! @verbatim     [Symbol.asyncIterator](): AsyncIterator<Progress>;
//! @verbatim };
use std::{cell::Cell, sync::Arc};

use rquickjs::{
    Ctx, Exception, Function, IntoJs, Object, Promise, Result, Value,
    atom::PredefinedAtom,
    prelude::{Opt, This},
};
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;

use crate::{IntoJsResult, error::CommonError, runtime::WithUserData};

pub trait IntoToken {
    fn into_token(self) -> Option<CancellationToken>;
}

impl IntoToken for CancellationToken {
    fn into_token(self) -> Option<CancellationToken> {
        Some(self)
    }
}

/// Cancelable future wrapper.
pub(crate) fn task<'js, R, Fut, F>(ctx: Ctx<'js>, future: F) -> Result<Promise<'js>>
where
    F: FnOnce(Ctx<'js>, CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
{
    task_with_token(ctx, None, future)
}

/// Cancelable future wrapper with a user-provided token (allows cancellation via an AbortController).
pub(crate) fn task_with_token<'js, R, Fut, F, T>(
    ctx: Ctx<'js>,
    token: T,
    future: F,
) -> Result<Promise<'js>>
where
    F: FnOnce(Ctx<'js>, CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
    T: IntoToken,
{
    let promise_object = task_with_token_impl(ctx.clone(), token, future)?;

    promise_object.as_promise().cloned().ok_or_else(|| {
        Exception::throw_message(&ctx, "Task implementation did not return a Promise")
    })
}

fn task_with_token_impl<'js, R, Fut, F, T>(
    ctx: Ctx<'js>,
    token: T,
    future: F,
) -> Result<Object<'js>>
where
    F: FnOnce(Ctx<'js>, CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
    T: IntoToken,
{
    let token = token.into_token().unwrap_or_else(|| {
        let user_data = ctx.user_data();
        user_data.child_cancellation_token()
    });

    let fut = future(ctx.clone(), token.clone());

    let promise = Promise::wrap_future(&ctx, fut)?;

    let cancel_fn = Function::new(ctx.clone(), {
        let token = token.clone();
        move || token.cancel()
    })?;

    let promise_object = promise
        .as_object()
        .expect("Promise should be an Object")
        .clone();

    promise_object.set("cancel", cancel_fn)?;

    // Override .then() so chained promises inherit cancel.
    // We capture only the CancellationToken (a pure-Rust value) to avoid creating
    // a GC-invisible cycle between JS Function objects stored in Rust closures.
    let then_fn = Function::new(ctx.clone(), {
        move |ctx: Ctx<'js>,
              this: This<Object<'js>>,
              on_fulfilled: Opt<Value<'js>>,
              on_rejected: Opt<Value<'js>>| {
            // Get the original Promise.prototype.then from the object's prototype.
            let proto: Object = this
                .0
                .get_prototype()
                .ok_or_else(|| rquickjs::Error::new_from_js("object", "prototype"))?;
            let original_then: Function = proto.get(PredefinedAtom::Then)?;
            let chained: Object =
                original_then.call((This(this.0), on_fulfilled.0, on_rejected.0))?;
            // Attach a fresh cancel function backed by the same token.
            let cancel = Function::new(ctx, {
                let token = token.clone();
                move || token.cancel()
            })?;
            chained.set("cancel", cancel)?;
            Ok::<Object<'js>, rquickjs::Error>(chained)
        }
    })?;
    promise_object.set(PredefinedAtom::Then, then_fn)?;

    Ok(promise_object)
}

// Small helper: make an async iterator over an mpsc::UnboundedReceiver<FromP>,
// converting each value to P via From before exposing it to JS.
fn make_progress_async_iter<'js, FromP, P>(
    ctx: Ctx<'js>,
    rx: mpsc::UnboundedReceiver<FromP>,
) -> Result<Object<'js>>
where
    P: IntoJs<'js> + From<FromP> + 'js,
    FromP: Send + 'static,
{
    let iter = Object::new(ctx.clone())?;
    let rx = Arc::new(tokio::sync::Mutex::new(rx));

    // `next(): Promise<Value>`
    let next_fn = Function::new(ctx, {
        move |ctx: Ctx<'js>| {
            let rx = rx.clone();
            let local_ctx = ctx.clone();

            let promise = Promise::wrap_future(&local_ctx, async move {
                let cancellation_token = ctx.user_data().cancellation_token();

                let value = {
                    let mut rx = rx.lock().await;

                    select! {
                        _ = cancellation_token.cancelled() => { Err(CommonError::Cancelled) },
                        value = rx.recv() => { Ok(value) },
                    }
                    .into_js_result(&ctx)?
                };

                let result = Object::new(ctx.clone())?;

                match value {
                    None => {
                        result.set("done", true)?;
                    }
                    Some(value) => {
                        result.set("value", P::from(value).into_js(&ctx)?)?;
                        result.set("done", false)?;
                    }
                }

                Ok::<Value<'js>, rquickjs::Error>(result.into_value())
            })?;

            Ok::<Promise<'js>, rquickjs::Error>(promise)
        }
    })?;

    iter.set("next", next_fn)?;
    Ok(iter)
}

pub(crate) fn progress_task_with_token<'js, R, Fut, F, T, FromP, P>(
    ctx: Ctx<'js>,
    token: T,
    progress: mpsc::UnboundedReceiver<FromP>,
    future: F,
) -> Result<Promise<'js>>
where
    F: FnOnce(Ctx<'js>, CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
    T: IntoToken,
    P: IntoJs<'js> + From<FromP> + 'js,
    FromP: Send + 'static,
{
    let promise_obj = task_with_token_impl(ctx.clone(), token, future)?;

    // [Symbol.asyncIterator]() { return { next() { return Promise<progress> } } }
    // The receiver is wrapped in an Option so it can be moved out on first call.
    let progress = Cell::new(Some(progress));
    let async_iterator = Function::new(ctx, move |ctx: Ctx<'js>| {
        let rx = progress.take().ok_or_else(|| {
            rquickjs::Error::new_from_js("ProgressTask", "async iterator already consumed")
        })?;
        make_progress_async_iter::<FromP, P>(ctx, rx)
    })?;

    promise_obj.set(PredefinedAtom::SymbolAsyncIterator, async_iterator)?;

    Ok(promise_obj
        .as_promise()
        .expect("Task impl must be a Promise")
        .clone())
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };

    use derive_more::From;
    use indicatif::ProgressBar;
    use macros::{js_class, js_methods};
    use rquickjs::{
        Ctx, IntoJs, JsLifetime, Promise, Result, Value,
        class::{Trace, Tracer},
    };
    use tokio::{sync::mpsc, time::sleep};
    use tokio_util::sync::CancellationToken;

    use crate::{
        IntoJSError,
        api::{
            js::{
                classes::{SingletonClass, register_singleton_class, register_value_class},
                task::{progress_task_with_token, task, task_with_token},
            },
            test_helpers::JsCounter,
        },
        error::CommonError,
        runtime::Runtime,
        scripting,
    };

    #[derive(Clone, Copy, Debug, Default, From)]
    pub struct ProgressValue(pub i32);

    impl<'js> IntoJs<'js> for ProgressValue {
        fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
            Ok(Value::new_int(ctx.clone(), self.0))
        }
    }

    #[derive(Default, JsLifetime)]
    #[js_class]
    pub struct JsTestStruct {
        pub has_started: Arc<AtomicBool>,
        pub was_canceled: Arc<AtomicBool>,
        pub token: CancellationToken,
    }

    impl<'js> Trace<'js> for JsTestStruct {
        fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
    }

    impl<'js> SingletonClass<'js> for JsTestStruct {}

    #[js_methods]
    impl JsTestStruct {
        #[qjs(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        pub fn test_task<'js>(&self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
            let has_started = self.has_started.clone();
            let was_canceled = self.was_canceled.clone();
            task(ctx, async move |ctx, token| {
                has_started.store(true, Ordering::Relaxed);
                token.cancelled().await;
                was_canceled.store(true, Ordering::Relaxed);
                Result::<()>::Err(CommonError::Cancelled.into_js(&ctx))
            })
        }

        pub fn test_task_with_token<'js>(&self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
            let has_started = self.has_started.clone();
            let was_canceled = self.was_canceled.clone();
            let token = self.token.clone();
            task_with_token(ctx, token, async move |ctx, token| {
                has_started.store(true, Ordering::Relaxed);
                token.cancelled().await;
                was_canceled.store(true, Ordering::Relaxed);
                Result::<()>::Err(CommonError::Cancelled.into_js(&ctx))
            })
        }

        pub fn test_task_with_progress<'js>(&self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
            let has_started = self.has_started.clone();
            let token = self.token.clone();
            let (sender, receiver) = mpsc::unbounded_channel::<ProgressValue>();

            let bar = ProgressBar::new(100);

            progress_task_with_token::<_, _, _, _, _, ProgressValue>(
                ctx,
                token,
                receiver,
                async move |_ctx, _token| {
                    has_started.store(true, Ordering::Relaxed);

                    for i in 1..=100 {
                        let _ = sender.send(i.into());
                        bar.inc(1);
                        sleep(Duration::from_millis(5)).await;
                    }

                    bar.finish();

                    Result::<()>::Ok(())
                },
            )
        }
    }

    async fn setup(script_engine: scripting::Engine, test: JsTestStruct) {
        script_engine
            .with(|ctx| {
                register_singleton_class::<JsTestStruct>(&ctx, test)?;
                register_value_class::<JsCounter>(&ctx)?;
                Ok(())
            })
            .await
            .unwrap();
    }

    #[test]
    fn test_task() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let test = JsTestStruct::default();
            let has_started = test.has_started.clone();
            let was_canceled = test.was_canceled.clone();

            setup(script_engine.clone(), test).await;

            let result = script_engine
                .eval_async::<()>(
                    r#"
                const task = testStruct.testTask();
                task.cancel();
                await task;
                "#,
                )
                .await;

            assert_eq!(result.err().unwrap().to_string(), "Error: Cancelled");
            assert!(has_started.load(Ordering::Relaxed));
            assert!(was_canceled.load(Ordering::Relaxed));
        });
    }

    #[test]
    fn test_task_with_token() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let test = JsTestStruct::default();
            let has_started = test.has_started.clone();
            let was_canceled = test.was_canceled.clone();
            let token = test.token.clone();

            setup(script_engine.clone(), test).await;

            token.cancel();

            let result = script_engine
                .eval_async::<()>(
                    r#"
                await testStruct.testTaskWithToken();
                "#,
                )
                .await;

            assert_eq!(result.err().unwrap().to_string(), "Error: Cancelled");
            assert!(has_started.load(Ordering::Relaxed));
            assert!(was_canceled.load(Ordering::Relaxed));
        });
    }

    #[test]
    fn test_task_with_progress() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let test = JsTestStruct::default();
            let has_started = test.has_started.clone();
            let was_canceled = test.was_canceled.clone();

            setup(script_engine.clone(), test).await;

            let counter = script_engine
                .eval_async::<u64>(
                    r#"
                const task = testStruct.testTaskWithProgress();
                let counter = 0;
                for await (const p of task) {
                    counter += 1;
                }
                await task;
                counter
                "#,
                )
                .await
                .unwrap();

            assert!(has_started.load(Ordering::Relaxed));
            assert!(!was_canceled.load(Ordering::Relaxed));
            assert_eq!(counter, 100);
        });
    }
}
