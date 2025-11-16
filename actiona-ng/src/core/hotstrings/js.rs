use std::sync::Arc;

use rquickjs::{
    Coerced, Ctx, JsLifetime, Result, Value,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    core::{hotstrings::Replacement, image::js::JsImage, js::classes::SingletonClass},
    runtime::{Runtime, WithUserData},
};

pub type JsHotstringOptions = super::HotstringOptions;

/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Hotstrings")]
pub struct JsHotstrings {
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
            inner: super::Hotstrings::new(runtime, task_tracker, cancellation_token),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsHotstrings {
    /// @param source: string
    /// @param replacement: string | (() => string | Promise<string>) | Image | (() => Image | Promise<Image>)
    /// @param options?: HotstringOptions
    pub fn add<'js>(
        &self,
        ctx: Ctx<'js>,
        source: String,
        replacement: Value<'js>,
        options: Opt<JsHotstringOptions>,
    ) -> Result<()> {
        let options = options.unwrap_or_default();
        if let Some(replacement) = replacement.as_function() {
            let user_data = ctx.user_data();
            let callbacks = user_data.callbacks();
            let function_key = callbacks.register(&ctx, replacement.clone());
            self.inner.add(
                &source,
                Replacement::JsCallback((user_data.script_engine().context(), function_key)),
                options,
            );
        } else if let Ok(image) = replacement.get::<JsImage>() {
            self.inner
                .add(&source, Replacement::Image(image.into_inner()), options);
        } else {
            let text = replacement.get::<Coerced<String>>()?.0;
            self.inner.add(&source, Replacement::Text(text), options);
        }

        Ok(())
    }

    pub fn remove(&self, source: String) {
        self.inner.remove(&source);
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;
    use tracing_test::traced_test;

    use crate::runtime::Runtime;

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

                //hotstrings.add("time", async () => "" + Date.now());
                //hotstrings.add("time", "1762879038878");

                //await sleep(100000);
            "#,
                )
                .await
                .unwrap();
        });
    }
}
