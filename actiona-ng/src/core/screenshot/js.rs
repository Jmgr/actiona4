use std::sync::Arc;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
};

use crate::{
    IntoJsResult,
    core::{displays::Displays, js::classes::SingletonClass},
    runtime::Runtime,
};

impl<T> IntoJsResult<T> for super::Result<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<'js> Trace<'js> for super::Screenshot {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// @singleton
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "Screenshot")]
pub struct JsScreenshot {
    inner: super::Screenshot,
}

impl SingletonClass<'_> for JsScreenshot {}

impl JsScreenshot {
    /// @skip
    pub async fn new(runtime: Arc<Runtime>, displays: Arc<Displays>) -> super::Result<Self> {
        Ok(Self {
            inner: super::Screenshot::new(runtime, displays).await?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsScreenshot {
    // TODO
}
