use std::sync::Arc;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
};

use crate::{
    IntoJS,
    core::{
        SingletonClass, ValueClass,
        name::js::JsNameParam,
        point::js::{JsPoint, JsPointParam},
        rect::js::JsRect,
    },
    runtime,
};

impl<T> IntoJS<T> for super::Result<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

/// @singleton
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "Displays")]
pub struct JsDisplays {
    inner: Arc<super::Displays>,
}

impl SingletonClass<'_> for JsDisplays {}

impl<'js> Trace<'js> for JsDisplays {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsDisplays {
    /// @skip
    pub const fn new(displays: Arc<super::Displays>) -> Result<Self> {
        Ok(Self { inner: displays })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDisplays {
    pub fn random_point(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self.inner.random_point().into_js(&ctx)?.into())
    }

    pub fn from_point(&self, point: JsPointParam) -> Option<JsDisplayInfo> {
        self.inner
            .from_point(point.0)
            .map(|display_info| display_info.into())
    }

    pub fn from_name<'js>(&self, ctx: Ctx<'js>, name: JsNameParam<'js>) -> Option<JsDisplayInfo> {
        let displays_infos = self.inner.displays_info.lock().unwrap();
        displays_infos
            .iter()
            .find(|display_info| name.0.matches(&ctx, &display_info.friendly_name))
            .cloned()
            .map(|display_info| display_info.into())
    }

    pub fn from_device_name<'js>(
        &self,
        ctx: Ctx<'js>,
        name: JsNameParam<'js>,
    ) -> Option<JsDisplayInfo> {
        let displays_infos = self.inner.displays_info.lock().unwrap();
        displays_infos
            .iter()
            .find(|display_info| name.0.matches(&ctx, &display_info.name))
            .cloned()
            .map(|display_info| display_info.into())
    }

    pub fn from_id(&self, id: u32) -> Option<JsDisplayInfo> {
        let displays_infos = self.inner.displays_info.lock().unwrap();
        displays_infos
            .iter()
            .find(|display_info| display_info.id == id)
            .cloned()
            .map(|display_info| display_info.into())
    }

    pub fn smallest(&self) -> Option<JsDisplayInfo> {
        self.inner
            .smallest()
            .map(|display_info| display_info.into())
    }

    pub fn largest(&self) -> Option<JsDisplayInfo> {
        self.inner.largest().map(|display_info| display_info.into())
    }
}

/// Display info
///
/// @prop readonly id: number // Unique identifier associated with the display
/// @prop readonly name: string // The display name
/// @prop readonly friendlyName: string // The display friendly name
/// @prop readonly x: number // The display x coordinate
/// @prop readonly y: number // The display x coordinate
/// @prop readonly width: number // The display pixel width
/// @prop readonly height: number // The display pixel height
/// @prop readonly widthMm: number // The width of a display in millimeters. This value may be 0
/// @prop readonly heightMm: number // The height of a display in millimeters. This value may be 0
/// @prop readonly rotation: number // Can be 0, 90, 180, 270, represents screen rotation in clock-wise degrees
/// @prop readonly scaleFactor: number // Output device's pixel scale factor
/// @prop readonly frequency: number // The display refresh rate
/// @prop readonly isPrimary: boolean // Whether the screen is the main screen
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "DisplayInfo")]
pub struct JsDisplayInfo {
    inner: runtime::DisplayInfo,
}

impl ValueClass<'_> for JsDisplayInfo {}

impl<'js> Trace<'js> for JsDisplayInfo {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl From<runtime::DisplayInfo> for JsDisplayInfo {
    fn from(value: runtime::DisplayInfo) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDisplayInfo {
    /// @skip
    #[qjs(get)]
    pub const fn id(&self) -> u32 {
        self.inner.id
    }

    /// @skip
    #[qjs(get)]
    #[allow(clippy::missing_const_for_fn)]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// @skip
    #[qjs(get)]
    #[allow(clippy::missing_const_for_fn)]
    pub fn friendly_name(&self) -> &str {
        &self.inner.friendly_name
    }

    /// @skip
    #[qjs(get)]
    pub fn rect(&self) -> JsRect {
        self.inner.rect.into()
    }

    /// @skip
    #[qjs(get)]
    pub const fn width_mm(&self) -> i32 {
        self.inner.width_mm
    }

    /// @skip
    #[qjs(get)]
    pub const fn height_mm(&self) -> i32 {
        self.inner.height_mm
    }

    /// @skip
    #[qjs(get)]
    pub const fn rotation(&self) -> f32 {
        self.inner.rotation
    }

    /// @skip
    #[qjs(get)]
    pub const fn scale_factor(&self) -> f32 {
        self.inner.scale_factor
    }

    /// @skip
    #[qjs(get)]
    pub const fn frequency(&self) -> f32 {
        self.inner.frequency
    }

    /// @skip
    #[qjs(get)]
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
