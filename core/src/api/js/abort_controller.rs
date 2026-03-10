use macros::{js_class, js_methods};
use rquickjs::{Ctx, JsLifetime, atom::PredefinedAtom, class::Trace};
use tokio_util::sync::CancellationToken;

use crate::{
    api::js::{
        classes::{HostClass, ValueClass},
        task::IntoToken,
    },
    runtime::WithUserData,
};
/// A signal that can be used to abort asynchronous operations.
///
/// Obtained from an `AbortController` via the `signal` property. Pass it
/// to cancellable operations (e.g., `findImage`) in their options.
///
/// ```ts
/// const controller = new AbortController();
/// const task = source.findImage(template, { signal: controller.signal });
/// // Cancel from elsewhere:
/// controller.abort();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsAbortSignal {
    token: CancellationToken,
}

impl<'js> HostClass<'js> for JsAbortSignal {}

impl<'js> Trace<'js> for JsAbortSignal {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsAbortSignal {
    /// @skip
    #[must_use]
    pub fn into_token(self) -> CancellationToken {
        self.token
    }
}

#[js_methods]
impl JsAbortSignal {
    /// Returns a string representation of this abort signal.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "AbortSignal".to_string()
    }
}

impl IntoToken for Option<JsAbortSignal> {
    fn into_token(self) -> Option<CancellationToken> {
        self.map(|token| token.into_token())
    }
}

/// Controls cancellation of asynchronous operations.
///
/// Create an `AbortController`, pass its `signal` to a cancellable operation,
/// and call `abort()` to cancel it.
///
/// ```ts
/// const controller = new AbortController();
///
/// // Start a long-running operation
/// const task = source.findImage(template, { signal: controller.signal });
///
/// // Cancel after 5 seconds
/// await sleep("5s");
/// controller.abort();
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsAbortController {
    token: CancellationToken,
}

impl<'js> ValueClass<'js> for JsAbortController {}

impl<'js> Trace<'js> for JsAbortController {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

#[js_methods]
impl JsAbortController {
    /// @constructor
    #[qjs(constructor)]
    #[must_use]
    pub fn new(ctx: Ctx<'_>) -> Self {
        Self {
            token: ctx.user_data().cancellation_token().child_token(),
        }
    }

    /// Signals cancellation to all operations using this controller's signal.
    pub fn abort(&self) {
        self.token.cancel();
    }

    #[get]
    #[must_use]
    pub fn signal(&self) -> JsAbortSignal {
        JsAbortSignal {
            token: self.token.child_token(),
        }
    }

    /// Returns a string representation of this abort controller.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "AbortController".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use macros::{js_class, js_methods};
    use rquickjs::{
        JsLifetime,
        class::{Trace, Tracer},
    };

    use crate::{
        api::js::{
            abort_controller::JsAbortSignal,
            classes::{SingletonClass, register_singleton_class},
        },
        runtime::Runtime,
    };

    #[derive(Clone, Debug, Default, JsLifetime)]
    #[js_class]
    pub struct JsTestStruct {
        has_run: Arc<AtomicBool>,
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

        pub async fn run(&self, signal: JsAbortSignal) {
            signal.into_token().cancelled().await;
            self.has_run.store(true, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_abort_controller() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let test = JsTestStruct::default();

            script_engine
                .with(|ctx| {
                    register_singleton_class::<JsTestStruct>(&ctx, test)?;
                    Ok(())
                })
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<JsTestStruct>(
                    r#"
                let controller = new AbortController();

                let promise = testStruct.run(controller.signal);

                await sleep("50ms");

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
