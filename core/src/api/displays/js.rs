use itertools::Itertools;
use rquickjs::{
    Ctx, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
};
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        js::classes::{SingletonClass, ValueClass},
        name::js::JsNameLike,
        point::js::{JsPoint, JsPointLike},
        rect::js::JsRect,
    },
    runtime::{self, WithUserData},
    types::display::display_with_type,
};

/// The global displays singleton for querying connected monitors and screens.
///
/// ```ts
/// // Get a random point across all displays
/// const point = await displays.randomPoint();
///
/// // Find which display contains a point
/// const info = await displays.fromPoint(100, 200);
/// if (info) println(info.name, info.rect);
///
/// // Find a display by friendly name
/// const monitor = await displays.fromName("HDMI-1");
///
/// // Get the largest or smallest display
/// const largest = await displays.largest();
/// const smallest = await displays.smallest();
/// ```
///
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
    /// Returns a random point within the bounds of all connected displays.
    /// @readonly
    pub async fn random_point(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self
            .inner
            .random_point(ctx.user_data().rng())
            .await
            .into_js_result(&ctx)?
            .into())
    }

    /// Returns the display that contains the given point, or `undefined` if none.
    /// @readonly
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

    /// Finds a display by its friendly name (e.g. `"HDMI-1"`), or `undefined` if not found.
    /// @readonly
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
        for display_info in displays_infos.iter() {
            if name.0.matches(&ctx, &display_info.friendly_name)? {
                return Ok(Some(display_info.clone().into()));
            }
        }

        Ok(None)
    }

    /// Finds a display by its device name, or `undefined` if not found.
    /// @readonly
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
        for display_info in displays_infos.iter() {
            if name.0.matches(&ctx, &display_info.name)? {
                return Ok(Some(display_info.clone().into()));
            }
        }

        Ok(None)
    }

    /// Finds a display by its unique numeric ID, or `undefined` if not found.
    /// @readonly
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

    /// Returns the smallest display by area, or `undefined` if no displays are connected.
    /// @readonly
    pub async fn smallest<'js>(&self, ctx: Ctx<'js>) -> Result<Option<JsDisplayInfo>> {
        Ok(self
            .inner
            .smallest()
            .await
            .into_js_result(&ctx)?
            .map(|display_info| display_info.into()))
    }

    /// Returns the largest display by area, or `undefined` if no displays are connected.
    /// @readonly
    pub async fn largest<'js>(&self, ctx: Ctx<'js>) -> Result<Option<JsDisplayInfo>> {
        Ok(self
            .inner
            .largest()
            .await
            .into_js_result(&ctx)?
            .map(|display_info| display_info.into()))
    }

    /// Returns all displays.
    /// @readonly
    pub async fn all<'js>(&self, ctx: Ctx<'js>) -> Result<Vec<JsDisplayInfo>> {
        Ok(self
            .inner
            .all()
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(|display_info| display_info.into())
            .collect_vec())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Displays", &self.inner)
    }
}

/// Information about a connected display, including its name, geometry,
/// rotation, scale factor, and refresh rate.
///
/// ```ts
/// const info = await displays.fromName("HDMI-1");
/// if (info) {
///     println(info.friendlyName, info.rect, formatFrequency(info.frequency));
///     println("Primary:", info.isPrimary);
/// }
/// ```
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
    /// Unique numeric identifier for this display.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.inner.id
    }

    /// The display device name (e.g. `"DP-1"`).
    /// @get
    #[qjs(get)]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// The display friendly name (e.g. `"HDMI-1"`).
    /// @get
    #[qjs(get)]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn friendly_name(&self) -> &str {
        &self.inner.friendly_name
    }

    /// The display rectangle (position and size in pixels).
    /// @get
    /// @readonly
    #[qjs(get)]
    #[must_use]
    pub fn rect(&self) -> JsRect {
        self.inner.rect.into()
    }

    /// The physical width of the display in millimeters.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn width_mm(&self) -> i32 {
        self.inner.width_mm
    }

    /// The physical height of the display in millimeters.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn height_mm(&self) -> i32 {
        self.inner.height_mm
    }

    /// The display rotation in clock-wise degrees (0, 90, 180, or 270).
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn rotation(&self) -> f32 {
        self.inner.rotation
    }

    /// The display's pixel scale factor (e.g. `2.0` for HiDPI/Retina).
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.inner.scale_factor
    }

    /// The display refresh rate in Hz.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn frequency(&self) -> f32 {
        self.inner.frequency
    }

    /// Whether this is the primary (main) display.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.inner.is_primary
    }

    /// Returns a string representation of the display.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Display", &self.inner)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{api::point::js::JsPoint, runtime::Runtime};

    #[test]
    #[traced_test]
    fn test_random_point() {
        Runtime::test_with_script_engine(async |script_engine| {
            let point: JsPoint = script_engine
                .eval_async("await displays.randomPoint()")
                .await
                .unwrap();

            println!("point: {}", point.inner());
        })
    }
}
