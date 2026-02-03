use std::sync::Arc;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
};
use tracing::instrument;

use crate::{
    IntoJsResult,
    core::{
        color::js::JsColor, displays::Displays, image::js::JsImage, js::classes::SingletonClass,
        point::js::JsPointLike, rect::js::JsRectLike,
    },
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
    #[instrument(skip_all)]
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> super::Result<Self> {
        Ok(Self {
            inner: super::Screenshot::new(runtime, displays).await?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsScreenshot {
    pub async fn capture_rect(&self, ctx: Ctx<'_>, rect: JsRectLike) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner.capture_rect(rect.0).await.into_js_result(&ctx)?,
        ))
    }

    pub async fn capture_display(&self, ctx: Ctx<'_>, display_id: u32) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner
                .capture_display(display_id)
                .await
                .into_js_result(&ctx)?,
        ))
    }

    pub async fn capture_pixel(&self, ctx: Ctx<'_>, position: JsPointLike) -> Result<JsColor> {
        Ok(self
            .inner
            .capture_pixel(position.0)
            .await
            .into_js_result(&ctx)?
            .into())
    }
}
