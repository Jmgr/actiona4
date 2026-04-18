use itertools::Itertools;
use macros::{js_class, js_methods};
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
        rect::{Rect, js::JsRect},
    },
    runtime::{self, WithUserData},
    types::display::display_with_type,
};

/// The global displays singleton for querying connected monitors and screens.
///
/// ```ts
/// // Get the primary display and convert a global coordinate to display-local
/// const display = displays.primary();
/// const local = display.toLocal(globalX, globalY);
///
/// // Find which display contains a point
/// const info = displays.fromPoint(100, 200);
/// if (info) println(info.name, info.rect);
///
/// // Find a display by friendly name
/// const monitor = displays.fromName("HDMI-1");
///
/// // Get the largest or smallest display
/// const largest = displays.largest();
/// const smallest = displays.smallest();
///
/// // Get a random point across all displays
/// const point = await displays.randomPoint();
/// ```
///
/// @singleton
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
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

#[js_methods]
impl JsDisplays {
    /// Returns a random point within the bounds of all connected displays, or `undefined` if no display is found.
    /// @readonly
    pub async fn random_point(&self, ctx: Ctx<'_>) -> Result<Option<JsPoint>> {
        Ok(self
            .inner
            .random_point(ctx.user_data().rng())
            .await
            .into_js_result(&ctx)?
            .map(Into::into))
    }

    /// Returns the primary display, or `undefined` if no display is found.
    /// @readonly
    pub fn primary(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .find(|d| d.is_primary)
            .cloned()
            .map(Into::into))
    }

    /// Returns the display that contains the given point, or `undefined` if none.
    /// @readonly
    pub fn from_point(&self, ctx: Ctx<'_>, point: JsPointLike) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .find(|d| d.rect.contains(point.0))
            .cloned()
            .map(Into::into))
    }

    /// Finds a display by its friendly name (e.g. `"HDMI-1"`), or `undefined` if not found.
    /// @readonly
    pub fn from_name<'js>(
        &self,
        ctx: Ctx<'js>,
        name: JsNameLike<'js>,
    ) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        for display_info in displays_info.iter() {
            if name.0.matches(&ctx, &display_info.friendly_name)? {
                return Ok(Some(display_info.clone().into()));
            }
        }
        Ok(None)
    }

    /// Finds a display by its device name, or `undefined` if not found.
    /// @readonly
    pub fn from_device_name<'js>(
        &self,
        ctx: Ctx<'js>,
        name: JsNameLike<'js>,
    ) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        for display_info in displays_info.iter() {
            if name.0.matches(&ctx, &display_info.name)? {
                return Ok(Some(display_info.clone().into()));
            }
        }
        Ok(None)
    }

    /// Finds a display by its unique numeric ID, or `undefined` if not found.
    /// @readonly
    pub fn from_id(&self, ctx: Ctx<'_>, id: u32) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .find(|d| d.id == id)
            .cloned()
            .map(Into::into))
    }

    /// Returns the smallest display by area, or `undefined` if no displays are connected.
    /// @readonly
    pub fn smallest(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .min_by(|a, b| a.rect.surface().cmp(&b.rect.surface()))
            .cloned()
            .map(Into::into))
    }

    /// Returns the largest display by area, or `undefined` if no displays are connected.
    /// @readonly
    pub fn largest(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .max_by(|a, b| a.rect.surface().cmp(&b.rect.surface()))
            .cloned()
            .map(Into::into))
    }

    /// Returns the display furthest to the left (minimum left edge), or `undefined` if none.
    /// @readonly
    pub fn leftmost(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .min_by_key(|d| d.rect.top_left.x)
            .cloned()
            .map(Into::into))
    }

    /// Returns the display furthest to the right (maximum right edge), or `undefined` if none.
    /// @readonly
    pub fn rightmost(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .max_by_key(|d| d.rect.top_left.x + d.rect.size.width)
            .cloned()
            .map(Into::into))
    }

    /// Returns the display furthest to the top (minimum top edge), or `undefined` if none.
    /// @readonly
    pub fn topmost(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .min_by_key(|d| d.rect.top_left.y)
            .cloned()
            .map(Into::into))
    }

    /// Returns the display furthest to the bottom (maximum bottom edge), or `undefined` if none.
    /// @readonly
    pub fn bottommost(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info
            .iter()
            .max_by_key(|d| d.rect.top_left.y + d.rect.size.height)
            .cloned()
            .map(Into::into))
    }

    /// Returns the display whose center is closest to the center of the desktop, or `undefined` if none.
    /// @readonly
    pub fn center(&self, ctx: Ctx<'_>) -> Result<Option<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        let mut iter = displays_info.iter();
        let Some(first) = iter.next() else {
            return Ok(None);
        };
        let desktop: Rect = iter.fold(first.rect, |acc, d| acc.union(d.rect));
        let desktop_c2 = desktop.top_left * 2 + desktop.size;
        Ok(displays_info
            .iter()
            .min_by_key(|d| {
                let diff = (d.rect.top_left * 2 + d.rect.size) - desktop_c2;
                diff.length_squared()
            })
            .cloned()
            .map(Into::into))
    }

    /// Returns all displays.
    /// @readonly
    pub fn all(&self, ctx: Ctx<'_>) -> Result<Vec<JsDisplayInfo>> {
        let displays_info = self.inner.get_info_sync().into_js_result(&ctx)?;
        Ok(displays_info.iter().cloned().map(Into::into).collect_vec())
    }

    /// Returns a string representation of the `displays` singleton.
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
#[js_class]
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

#[js_methods]
impl JsDisplayInfo {
    /// Unique numeric identifier for this display.
    #[get]
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.inner.id
    }

    /// The display device name (e.g. `"DP-1"`).
    #[get]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// The display friendly name (e.g. `"HDMI-1"`).
    #[get]
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn friendly_name(&self) -> &str {
        &self.inner.friendly_name
    }

    /// The display rectangle (position and size in pixels).
    /// @readonly
    #[get]
    #[must_use]
    pub fn rect(&self) -> JsRect {
        self.inner.rect.into()
    }

    /// The physical width of the display in millimeters.
    #[get]
    #[must_use]
    pub const fn width_mm(&self) -> i32 {
        self.inner.width_mm
    }

    /// The physical height of the display in millimeters.
    #[get]
    #[must_use]
    pub const fn height_mm(&self) -> i32 {
        self.inner.height_mm
    }

    /// The display rotation in clock-wise degrees (0, 90, 180, or 270).
    #[get]
    #[must_use]
    pub const fn rotation(&self) -> f32 {
        self.inner.rotation
    }

    /// The display's pixel scale factor (e.g. `2.0` for HiDPI/Retina).
    #[get]
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.inner.scale_factor
    }

    /// The display refresh rate in Hz.
    #[get]
    #[must_use]
    pub const fn frequency(&self) -> f32 {
        self.inner.frequency
    }

    /// Whether this is the primary (main) display.
    #[get]
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.inner.is_primary
    }

    /// Converts a global desktop point to display-local coordinates.
    ///
    /// The result is the position relative to this display's top-left corner,
    /// in the same logical-pixel unit used for mouse coordinates and `rect`.
    ///
    /// ```ts
    /// const display = displays.primary();
    /// // After finding something at global coordinate (1980, 50):
    /// const local = display.toLocal(1980, 50);
    /// println(local.x, local.y); // position within the display
    /// ```
    #[must_use]
    pub fn to_local(&self, point: JsPointLike) -> JsPoint {
        (point.0 - self.inner.rect.top_left).into()
    }

    /// Converts a display-local point to global desktop coordinates.
    ///
    /// The inverse of `toLocal`: adds this display's top-left offset so the
    /// point can be used with mouse, keyboard, or capture APIs that expect
    /// global coordinates.
    ///
    /// ```ts
    /// const display = displays.primary();
    /// // A point at (100, 50) within the display image:
    /// const global = display.toGlobal(100, 50);
    /// ```
    #[must_use]
    pub fn to_global(&self, point: JsPointLike) -> JsPoint {
        (point.0 + self.inner.rect.top_left).into()
    }

    /// Returns a string representation of this display.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Display", &self.inner)
    }
}
