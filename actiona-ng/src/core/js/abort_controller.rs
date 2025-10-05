use rquickjs::{Ctx, JsLifetime, class::Trace};
use tokio_util::sync::CancellationToken;

use crate::{
    core::js::{classes::ValueClass, task::IntoToken},
    runtime::WithUserData,
};

#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "AbortSignal")]
pub struct JsAbortSignal {
    token: CancellationToken,
}

impl<'js> ValueClass<'js> for JsAbortSignal {}

impl<'js> Trace<'js> for JsAbortSignal {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl JsAbortSignal {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Self {
        Self {
            token: ctx.user_data().cancellation_token().child_token(),
        }
    }
}

impl JsAbortSignal {
    /// @skip
    pub fn into_token(self) -> CancellationToken {
        self.token
    }
}

impl IntoToken for Option<JsAbortSignal> {
    fn into_token(self) -> Option<CancellationToken> {
        self.map(|token| token.into_token())
    }
}

/// @prop readonly signal: AbortSignal
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "AbortController")]
pub struct JsAbortController {
    token: CancellationToken,
}

impl<'js> ValueClass<'js> for JsAbortController {}

impl<'js> Trace<'js> for JsAbortController {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[rquickjs::methods]
impl JsAbortController {
    /// @constructor
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Self {
        Self {
            token: ctx.user_data().cancellation_token().child_token(),
        }
    }

    pub fn abort(&self) {
        self.token.cancel();
    }

    /// @skip
    #[qjs(get)]
    pub fn signal(&self) -> JsAbortSignal {
        JsAbortSignal {
            token: self.token.child_token(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use rquickjs::{
        JsLifetime, Result,
        class::{Trace, Tracer},
    };

    use crate::{
        core::js::{abort_controller::JsAbortSignal, classes::SingletonClass},
        runtime::Runtime,
    };

    #[derive(Clone, Debug, Default, JsLifetime)]
    #[rquickjs::class]
    pub struct TestStruct {
        has_run: Arc<AtomicBool>,
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

        pub async fn run(&self, signal: JsAbortSignal) {
            signal.into_token().cancelled().await;
            self.has_run.store(true, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_abort_controller() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let test = TestStruct::default();

            script_engine
                .with(|ctx| {
                    TestStruct::register(&ctx, test).unwrap();
                    Result::<()>::Ok(())
                })
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<TestStruct>(
                    r#"
                let controller = new AbortController();

                let promise = testStruct.run(controller.signal);

                await sleep(50);

                controller.abort();

                await promise;

                testStruct
                "#,
                )
                .await
                .unwrap();

            assert!(result.has_run.load(Ordering::Relaxed));
        });
    }
}
