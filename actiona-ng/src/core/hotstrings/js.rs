use std::sync::Arc;

use rquickjs::{
    Coerced, Ctx, Function, JsLifetime, Result, Value,
    class::{Trace, Tracer},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    core::{hotstrings::Replacement, js::classes::SingletonClass},
    runtime::{Runtime, WithUserData},
};

/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Hotstrings")]
pub struct JsHotstrings {
    runtime: Arc<Runtime>,
    inner: super::Hotstrings,
}

impl<'js> Trace<'js> for JsHotstrings {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsHotstrings {}

impl JsHotstrings {
    /// @skip
    pub fn new(
        runtime: Arc<Runtime>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            runtime: runtime.clone(),
            inner: super::Hotstrings::new(runtime, task_tracker, cancellation_token),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsHotstrings {
    /// @overload
    /// @param source: string
    /// @param replacement: string | (() => string | Promise<string>)
    pub fn add<'js>(&self, ctx: Ctx<'js>, source: String, replacement: Value<'js>) -> Result<()> {
        if let Some(replacement) = replacement.as_function() {
            let user_data = ctx.user_data();
            let callbacks = user_data.callbacks();
            let function_key = callbacks.register(&ctx, replacement.clone());
            self.inner.add(
                &source,
                Replacement::JsCallback((user_data.script_engine().context(), function_key)),
            );
        } else {
            let text = replacement.get::<Coerced<String>>()?.0;
            self.inner.add(&source, Replacement::Text(text));
        }

        Ok(())
    }

    pub fn remove(&self, source: String) {
        self.inner.remove(&source);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;
    use tracing::info;
    use tracing_test::traced_test;

    use crate::{core::color::js::JsColor, runtime::Runtime};

    #[test]
    #[traced_test]
    #[ignore]
    fn test_hotstrings() {
        Runtime::test_with_script_engine(async |script_engine| {
            info!("start");
            script_engine
                .eval_async::<()>(
                    r#"
                    console.printLn("time: " + Date.now());

                hotstrings.add("time", async () => "" + Date.now());
                //hotstrings.add("time", "1762879038878");

                await sleep(100000);
            "#,
                )
                .await
                .unwrap();
            sleep(Duration::from_secs(60)).await;
        });
    }
}
