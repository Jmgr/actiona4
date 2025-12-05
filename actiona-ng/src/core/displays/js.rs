use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
};
use tracing::instrument;

use crate::{
    IntoJsResult,
    core::{
        js::classes::{SingletonClass, ValueClass},
        name::js::JsNameLike,
        point::js::{JsPoint, JsPointLike},
        rect::js::JsRect,
    },
    runtime::{self, WithUserData},
};

impl<T> IntoJsResult<T> for super::Result<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

/// @singleton
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "Displays")]
pub struct JsDisplays {
    inner: super::Displays,
}

impl SingletonClass<'_> for JsDisplays {}

impl<'js> Trace<'js> for JsDisplays {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsDisplays {
    /// @skip
    #[instrument(skip_all)]
    pub fn new(displays: super::Displays) -> Result<Self> {
        Ok(Self { inner: displays })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDisplays {
    pub async fn random_point(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self
            .inner
            .random_point(ctx.user_data().rng())
            .await
            .into_js_result(&ctx)?
            .into())
    }

    pub async fn from_point(
        &self,
        ctx: Ctx<'_>,
        point: JsPointLike,
    ) -> Result<Option<JsDisplayInfo>> {
        Ok(self
            .inner
            .from_point(point.0)
            .await
            .into_js_result(&ctx)?
            .map(|display_info| display_info.into()))
    }

    pub async fn from_name<'js>(
        &self,
        ctx: Ctx<'js>,
        name: JsNameLike<'js>,
    ) -> Result<Option<JsDisplayInfo>> {
        let displays_infos = self
            .inner
            .displays_info
            .wait_get()
            .await
            .into_js_result(&ctx)?;
        Ok(displays_infos
            .iter()
            .find(|display_info| name.0.matches(&ctx, &display_info.friendly_name))
            .cloned()
            .map(|display_info| display_info.into()))
    }

    pub async fn from_device_name<'js>(
        &self,
        ctx: Ctx<'js>,
        name: JsNameLike<'js>,
    ) -> Result<Option<JsDisplayInfo>> {
        let displays_infos = self
            .inner
            .displays_info
            .wait_get()
            .await
            .into_js_result(&ctx)?;
        Ok(displays_infos
            .iter()
            .find(|display_info| name.0.matches(&ctx, &display_info.name))
            .cloned()
            .map(|display_info| display_info.into()))
    }

    pub async fn from_id<'js>(&self, ctx: Ctx<'js>, id: u32) -> Result<Option<JsDisplayInfo>> {
        let displays_infos = self
            .inner
            .displays_info
            .wait_get()
            .await
            .into_js_result(&ctx)?;
        Ok(displays_infos
            .iter()
            .find(|display_info| display_info.id == id)
            .cloned()
            .map(|display_info| display_info.into()))
    }

    pub async fn smallest<'js>(&self, ctx: Ctx<'js>) -> Result<Option<JsDisplayInfo>> {
        Ok(self
            .inner
            .smallest()
            .await
            .into_js_result(&ctx)?
            .map(|display_info| display_info.into()))
    }

    pub async fn largest<'js>(&self, ctx: Ctx<'js>) -> Result<Option<JsDisplayInfo>> {
        Ok(self
            .inner
            .largest()
            .await
            .into_js_result(&ctx)?
            .map(|display_info| display_info.into()))
    }
}

/// Display info
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "DisplayInfo")]
pub struct JsDisplayInfo {
    inner: runtime::events::DisplayInfo,
}

impl ValueClass<'_> for JsDisplayInfo {}

impl<'js> Trace<'js> for JsDisplayInfo {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl From<runtime::events::DisplayInfo> for JsDisplayInfo {
    fn from(value: runtime::events::DisplayInfo) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDisplayInfo {
    /// Unique identifier associated with the display
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.inner.id
    }

    /// The display name
    /// @get
    #[qjs(get)]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// The display friendly name
    /// @get
    #[qjs(get)]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn friendly_name(&self) -> &str {
        &self.inner.friendly_name
    }

    /// The display rectangle
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn rect(&self) -> JsRect {
        self.inner.rect.into()
    }

    /// The display pixel width
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn width_mm(&self) -> i32 {
        self.inner.width_mm
    }

    /// The display pixel height
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn height_mm(&self) -> i32 {
        self.inner.height_mm
    }

    /// The display rotation: can be 0, 90, 180, 270 and represents the screen rotation in clock-wise degrees
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn rotation(&self) -> f32 {
        self.inner.rotation
    }

    /// Output device's pixel scale factor
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.inner.scale_factor
    }

    /// The display refresh rate
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn frequency(&self) -> f32 {
        self.inner.frequency
    }

    /// Whether the screen is the main screen
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.inner.is_primary
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{core::point::js::JsPoint, runtime::Runtime};

    #[test]
    #[traced_test]
    fn test_random_point() {
        Runtime::test_with_script_engine(async |script_engine| {
            let point: JsPoint = script_engine.eval("displays.randomPoint()").await.unwrap();

            println!("point: {}", point.inner());
        })
    }
}
