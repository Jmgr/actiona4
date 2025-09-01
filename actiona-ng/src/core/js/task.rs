//! @verbatim /**
//! @verbatim  * Task: cancellable promise.
//! @verbatim  */
//! @verbatim type Task<Result> = Promise<Result> & {
//! @verbatim     cancel(): void;
//! @verbatim };
//! @verbatim
//! @verbatim /**
//! @verbatim  * ProgressTask: task with progress.
//! @verbatim  */
//! @verbatim type ProgressTask<Result, Progress> = Task<Result> & {
//! @verbatim     [Symbol.asyncIterator](): AsyncIterator<Progress>;
//! @verbatim };
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use rquickjs::{Ctx, Function, IntoJs, Object, Promise, Result, Value, atom::PredefinedAtom};
use tokio::{select, sync::watch};
use tokio_util::sync::CancellationToken;

use crate::{
    IntoJsResult, core::js::abort_controller::IntoToken, error::CommonError, runtime::WithUserData,
};

// TODO: use task in more places

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
    let promise_object = task_with_token_impl(ctx, token, future)?;

    Ok(promise_object.as_promise().unwrap().clone())
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

    let cancel_fn = Function::new(ctx.clone(), move || token.cancel())?;

    let promise_object = promise
        .as_object()
        .expect("Promise should be an Object")
        .clone();

    promise_object.set("cancel", cancel_fn)?;

    Ok(promise_object)
}

pub trait IsDone {
    fn is_done(&self) -> bool;
}

// Small helper: make an async iterator over a watch::Receiver<P>
fn make_progress_async_iter<'js, P>(ctx: Ctx<'js>, rx: watch::Receiver<P>) -> Result<Object<'js>>
where
    P: IntoJs<'js> + Clone + IsDone + 'js,
{
    let iter = Object::new(ctx.clone())?;
    let rx = Arc::new(tokio::sync::Mutex::new(rx));
    let finished = Arc::new(AtomicBool::new(false));

    // `next(): Promise<Value>`
    let next_fn = Function::new(ctx, {
        move |ctx: Ctx<'js>| {
            let rx = rx.clone();
            let local_ctx = ctx.clone();
            let finished = finished.clone();

            let promise = Promise::wrap_future(&local_ctx, async move {
                if finished.load(Ordering::Relaxed) {
                    let result = Object::new(ctx.clone())?;
                    result.set("done", true)?;
                    return Ok::<Value<'js>, rquickjs::Error>(result.into_value());
                }

                let cancellation_token = ctx.user_data().cancellation_token();

                let mut rx = rx.lock().await;

                select! {
                    _ = cancellation_token.cancelled() => { Err(CommonError::Cancelled) },
                    _ = rx.changed() => { Ok(()) },
                }
                .into_js(&ctx)?;

                let value = rx.borrow_and_update();

                if value.is_done() {
                    finished.store(true, Ordering::Relaxed);
                }

                let result = Object::new(ctx.clone())?;
                result.set("value", value.clone().into_js(&ctx)?)?;
                result.set("done", false)?;

                Ok::<Value<'js>, rquickjs::Error>(result.into_value())
            })?;

            Ok::<Promise<'js>, rquickjs::Error>(promise)
        }
    })?;

    iter.set("next", next_fn)?;
    Ok(iter)
}

pub(crate) fn progress_task_with_token<'js, R, Fut, F, T, P>(
    ctx: Ctx<'js>,
    token: T,
    progress: watch::Receiver<P>,
    future: F,
) -> Result<Promise<'js>>
where
    F: FnOnce(Ctx<'js>, CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
    T: IntoToken,
    P: IntoJs<'js> + Clone + IsDone + 'js,
{
    let promise_obj = task_with_token_impl(ctx.clone(), token, future)?;

    // [Symbol.asyncIterator]() { return { next() { return Promise<progress> } } }
    let async_iterator = Function::new(ctx, move |ctx: Ctx<'js>| {
        make_progress_async_iter(ctx, progress.clone())
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

    use indicatif::ProgressBar;
    use rquickjs::{
        Ctx, IntoJs, JsLifetime, Promise, Result, Value,
        class::{Trace, Tracer},
    };
    use tokio::{sync::watch, time::sleep};
    use tokio_util::sync::CancellationToken;

    use crate::{
        IntoJSError,
        core::{
            js::{
                classes::{SingletonClass, ValueClass},
                task::{IsDone, progress_task_with_token, task, task_with_token},
            },
            test_helpers::JsCounter,
        },
        error::CommonError,
        runtime::Runtime,
    };

    // TODO: skip everything in "tests" mod
    /// @skip
    #[derive(Debug, Default, Clone, Copy)]
    pub struct ProgressValue(pub i32);

    impl From<i32> for ProgressValue {
        fn from(value: i32) -> Self {
            Self(value)
        }
    }

    impl IsDone for ProgressValue {
        fn is_done(&self) -> bool {
            self.0 >= 100
        }
    }

    impl<'js> IntoJs<'js> for ProgressValue {
        fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
            Ok(Value::new_int(ctx.clone(), self.0))
        }
    }

    #[derive(Default, JsLifetime)]
    #[rquickjs::class]
    pub struct TestStruct {
        pub has_started: Arc<AtomicBool>,
        pub was_canceled: Arc<AtomicBool>,
        pub token: CancellationToken,
        pub sender: watch::Sender<ProgressValue>,
    }

    impl<'js> Trace<'js> for TestStruct {
        fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
    }

    impl<'js> SingletonClass<'js> for TestStruct {}

    #[rquickjs::methods(rename_all = "camelCase")]
    impl TestStruct {
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
            let sender = self.sender.clone();
            let receiver = sender.subscribe();

            let bar = ProgressBar::new(100);

            progress_task_with_token(ctx, token, receiver, async move |_ctx, _token| {
                has_started.store(true, Ordering::Relaxed);

                for i in 1..=100 {
                    sender.send_replace(i.into());
                    bar.inc(1);
                    sleep(Duration::from_millis(5)).await;
                }

                bar.finish();

                Result::<()>::Ok(())
            })
        }
    }

    #[test]
    fn test_task() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let test = TestStruct::default();
            let has_started = test.has_started.clone();
            let was_canceled = test.was_canceled.clone();

            script_engine
                .with(|ctx| {
                    TestStruct::register(&ctx, test).unwrap();
                    Result::<()>::Ok(())
                })
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<()>(
                    r#"
                const task = testStruct.testTask();
                task.cancel();
                await task;
                "#,
                )
                .await;

            assert_eq!(result.err().unwrap().to_string(), "Cancelled");
            assert!(has_started.load(Ordering::Relaxed));
            assert!(was_canceled.load(Ordering::Relaxed));
        });
    }

    #[test]
    fn test_task_with_token() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let test = TestStruct::default();
            let has_started = test.has_started.clone();
            let was_canceled = test.was_canceled.clone();
            let token = test.token.clone();

            script_engine
                .with(|ctx| {
                    TestStruct::register(&ctx, test).unwrap();
                    Result::<()>::Ok(())
                })
                .await
                .unwrap();

            token.cancel();

            let result = script_engine
                .eval_async::<()>(
                    r#"
                await testStruct.testTaskWithToken();
                "#,
                )
                .await;

            assert_eq!(result.err().unwrap().to_string(), "Cancelled");
            assert!(has_started.load(Ordering::Relaxed));
            assert!(was_canceled.load(Ordering::Relaxed));
        });
    }

    #[test]
    fn test_task_with_progress() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let test = TestStruct::default();
            let has_started = test.has_started.clone();
            let was_canceled = test.was_canceled.clone();

            // TODO: factorize

            script_engine
                .with(|ctx| {
                    TestStruct::register(&ctx, test).unwrap();
                    JsCounter::register(&ctx).unwrap();
                    Result::<()>::Ok(())
                })
                .await
                .unwrap();

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
