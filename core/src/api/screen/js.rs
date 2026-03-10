use macros::{js_class, js_methods, platform};
use rquickjs::{
    Ctx, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
};

use crate::{
    IntoJsResult,
    api::{
        color::js::JsColor,
        displays::js::JsDisplayInfo,
        image::{find_image::SearchIn, js::JsImage},
        js::classes::{HostClass, SingletonClass, register_host_class},
        point::js::JsPointLike,
        rect::{Rect, js::JsRectLike},
        screen::Screen,
        windows::js::JsWindowHandle,
    },
    types::display::DisplayFields,
};
#[derive(Clone, Debug)]
enum JsSearchInInner {
    Desktop,
    Display(u32),
    Rect(Rect),
    Window(JsWindowHandle),
}

impl<'js> Trace<'js> for JsSearchInInner {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Self::Window(window) = self {
            window.trace(tracer);
        }
    }
}

/// Specifies the screen area to search within for find-image operations.
///
/// ```ts
/// // Search the entire desktop
/// const match = await image.findOnScreen(SearchIn.desktop());
///
/// // Search a specific display
/// const display = displays.primary();
/// const match = await image.findOnScreen(SearchIn.display(display));
///
/// // Search a specific rectangle
/// const match = await image.findOnScreen(SearchIn.rect(0, 0, 1920, 1080));
/// ```
#[derive(Clone, Debug, JsLifetime, Trace)]
#[js_class]
pub struct JsSearchIn {
    inner: JsSearchInInner,
}

impl HostClass<'_> for JsSearchIn {}

impl From<JsSearchIn> for SearchIn {
    fn from(js: JsSearchIn) -> Self {
        match js.inner {
            JsSearchInInner::Desktop => Self::Desktop,
            JsSearchInInner::Display(id) => Self::Display(id),
            JsSearchInInner::Rect(r) => Self::Rect(r),
            JsSearchInInner::Window(handle) => Self::Window(handle.window_id()),
        }
    }
}

impl JsSearchIn {
    fn to_string_inner(&self) -> String {
        let fields = match &self.inner {
            JsSearchInInner::Desktop => "(desktop)".to_string(),
            JsSearchInInner::Display(id) => DisplayFields::default()
                .display("display_id", id)
                .finish_as_string(),
            JsSearchInInner::Rect(r) => DisplayFields::default()
                .display("rect", r)
                .finish_as_string(),
            JsSearchInInner::Window(handle) => DisplayFields::default()
                .display("window_id", handle.window_id().as_u64())
                .finish_as_string(),
        };
        format!("SearchIn{fields}")
    }
}

#[js_methods]
impl JsSearchIn {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "SearchIn cannot be instantiated directly; use SearchIn.desktop(), SearchIn.display(), SearchIn.rect(), or SearchIn.window().",
        ))
    }

    /// Searches within the entire desktop (the bounding rectangle of all connected displays).
    ///
    /// ```ts
    /// const match = await image.findOnScreen(SearchIn.desktop());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn desktop() -> Self {
        Self {
            inner: JsSearchInInner::Desktop,
        }
    }

    /// Searches within a specific display.
    ///
    /// ```ts
    /// const display = displays.primary();
    /// const match = await image.findOnScreen(SearchIn.display(display));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn display(display: JsDisplayInfo) -> Self {
        Self {
            inner: JsSearchInInner::Display(display.id()),
        }
    }

    /// Searches within the given screen rectangle.
    ///
    /// ```ts
    /// const match = await image.findOnScreen(SearchIn.rect(0, 0, 1920, 1080));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn rect(rect: JsRectLike) -> Self {
        Self {
            inner: JsSearchInInner::Rect(rect.0),
        }
    }

    /// Searches within the bounding rectangle of the given window.
    ///
    /// ```ts
    /// const win = windows.activeWindow();
    /// const match = await image.findOnScreen(SearchIn.window(win));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn window(handle: JsWindowHandle) -> Self {
        Self {
            inner: JsSearchInInner::Window(handle),
        }
    }

    /// Returns a string representation of this search area.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.to_string_inner()
    }
}

impl<'js> Trace<'js> for super::Screen {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// Provides methods to capture the entire desktop, a specific display, a screen
/// region, or a single pixel.
///
/// ```ts
/// const image = await screen.captureDesktop();
/// println(image.size());
/// ```
///
/// ```ts
/// const display = displays.primary();
/// const image = await screen.captureDisplay(display);
/// println(image.size());
/// ```
///
/// ```ts
/// const pixel = await screen.capturePixel(100, 100);
/// println(pixel);
/// ```
///
/// @singleton
#[derive(Debug, JsLifetime, Trace)]
#[js_class]
pub struct JsScreen {
    inner: Screen,
}

impl SingletonClass<'_> for JsScreen {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_host_class::<JsSearchIn>(ctx)?;
        Ok(())
    }
}

impl JsScreen {
    /// @skip
    #[must_use]
    pub const fn new(inner: super::Screen) -> Self {
        Self { inner }
    }
}

#[js_methods]
impl JsScreen {
    /// Captures a screenshot of the entire desktop.
    ///
    /// ```ts
    /// const image = await screen.captureDesktop();
    /// ```
    #[platform(not = "wayland")]
    pub async fn capture_desktop(&self, ctx: Ctx<'_>) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner.capture_desktop().await.into_js_result(&ctx)?,
        ))
    }

    /// Captures a screenshot of the given display.
    ///
    /// ```ts
    /// const image = await screen.captureDisplay(displays.primary());
    /// const image = await screen.captureDisplay(displays.fromId(474));
    /// const image = await screen.captureDisplay(displays.largest());
    /// ```
    #[platform(not = "wayland")]
    pub async fn capture_display(&self, ctx: Ctx<'_>, display: JsDisplayInfo) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner
                .capture_display(display.id())
                .await
                .into_js_result(&ctx)?,
        ))
    }

    /// Captures a screenshot of a screen rectangle.
    ///
    /// ```ts
    /// const image = await screen.captureRect(0, 0, 1920, 1080);
    /// ```
    #[platform(not = "wayland")]
    pub async fn capture_rect(&self, ctx: Ctx<'_>, rect: JsRectLike) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner.capture_rect(rect.0).await.into_js_result(&ctx)?,
        ))
    }

    /// Captures a screenshot of the bounding rectangle of the given window.
    ///
    /// ```ts
    /// const win = windows.activeWindow();
    /// const image = await screen.captureWindow(win);
    /// ```
    #[platform(not = "wayland")]
    pub async fn capture_window(&self, ctx: Ctx<'_>, handle: JsWindowHandle) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner
                .capture_window(handle.window_id())
                .await
                .into_js_result(&ctx)?,
        ))
    }

    /// Captures the color of a single pixel on screen.
    ///
    /// ```ts
    /// const color = await screen.capturePixel(100, 200);
    /// println(color);
    /// ```
    #[platform(not = "wayland")]
    pub async fn capture_pixel(&self, ctx: Ctx<'_>, position: JsPointLike) -> Result<JsColor> {
        Ok(self
            .inner
            .capture_pixel(position.0)
            .await
            .into_js_result(&ctx)?
            .into())
    }

    /// Returns a string representation of the `screen` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Screen".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use tracing_test::traced_test;

    use crate::{
        api::test_helpers::{js_path, random_name},
        runtime::Runtime,
    };

    #[test]
    #[ignore]
    #[traced_test]
    fn test_capture_desktop() {
        Runtime::test_with_script_engine(async |script_engine| {
            let width: u32 = script_engine
                .eval_async("(await screen.captureDesktop()).width")
                .await
                .unwrap();
            let height: u32 = script_engine
                .eval_async("(await screen.captureDesktop()).height")
                .await
                .unwrap();
            println!("desktop: {width}x{height}");
            assert!(width > 0 && height > 0);
        });
    }

    #[test]
    #[ignore]
    #[traced_test]
    fn test_capture_display_primary() {
        Runtime::test_with_script_engine(async |script_engine| {
            let width: u32 = script_engine
                .eval_async("(await screen.captureDisplay(displays.primary())).width")
                .await
                .unwrap();
            let height: u32 = script_engine
                .eval_async("(await screen.captureDisplay(displays.primary())).height")
                .await
                .unwrap();
            println!("primary: {width}x{height}");
            assert!(width > 0 && height > 0);
        });
    }

    #[test]
    #[ignore]
    #[traced_test]
    fn test_capture_display_largest() {
        Runtime::test_with_script_engine(async |script_engine| {
            let width: u32 = script_engine
                .eval_async("(await screen.captureDisplay(displays.largest())).width")
                .await
                .unwrap();
            let height: u32 = script_engine
                .eval_async("(await screen.captureDisplay(displays.largest())).height")
                .await
                .unwrap();
            println!("largest: {width}x{height}");
            assert!(width > 0 && height > 0);
        });
    }

    #[test]
    //#[ignore]
    #[traced_test]
    fn test_capture_display_from_id_to_file() {
        Runtime::test_with_script_engine(async |script_engine| {
            let output_path = temp_dir().join(format!(
                "actiona4_capture_display_from_id_{}.png",
                random_name()
            ));
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                    const image = await screen.captureDesktop();
                    await image.save({});
                    "#,
                    js_path(&output_path)
                ))
                .await
                .unwrap();
            println!("saved capture to {}", output_path.display());
        });
    }

    #[test]
    #[ignore]
    #[traced_test]
    fn test_capture_display_from_name_to_file() {
        Runtime::test_with_script_engine(async |script_engine| {
            let output_path = temp_dir().join(format!(
                "actiona4_capture_display_from_name_{}.png",
                random_name()
            ));
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                    const displayInfo = displays.all()[0];
                    if (!displayInfo) throw new Error("No display available");
                    const image = await screen.captureDisplay(displayInfo);
                    await image.save({});
                    "#,
                    js_path(&output_path)
                ))
                .await
                .unwrap();
            println!("saved capture to {}", output_path.display());
        });
    }

    #[test]
    #[ignore]
    #[traced_test]
    fn test_capture_display_from_point_to_file() {
        Runtime::test_with_script_engine(async |script_engine| {
            let output_path = temp_dir().join(format!(
                "actiona4_capture_display_from_point_{}.png",
                random_name()
            ));
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                    const displayInfo = displays.fromPoint(0, 0);
                    if (!displayInfo) throw new Error("No display at origin");
                    const image = await screen.captureDisplay(displayInfo);
                    await image.save({});
                    "#,
                    js_path(&output_path)
                ))
                .await
                .unwrap();
            println!("saved capture to {}", output_path.display());
        });
    }

    #[test]
    #[ignore]
    #[traced_test]
    fn test_capture_rect_to_file() {
        Runtime::test_with_script_engine(async |script_engine| {
            let output_path =
                temp_dir().join(format!("actiona4_capture_rect_{}.png", random_name()));
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                    const displayInfo = displays.all()[0];
                    if (!displayInfo) throw new Error("No display available");
                    const rect = displayInfo.rect;
                    const captureWidth = Math.max(1, Math.min(rect.width, 300));
                    const captureHeight = Math.max(1, Math.min(rect.height, 200));
                    const image = await screen.captureRect(rect.x, rect.y, captureWidth, captureHeight);
                    await image.save({});
                    "#,
                    js_path(&output_path)
                ))
                .await
                .unwrap();
            println!("saved capture to {}", output_path.display());
        });
    }

    #[test]
    #[ignore]
    #[traced_test]
    fn test_capture_window_to_file() {
        Runtime::test_with_script_engine(async |script_engine| {
            let output_path =
                temp_dir().join(format!("actiona4_capture_window_{}.png", random_name()));
            script_engine
                .eval_async::<()>(&format!(
                    r#"
                    const win = windows.foreground();
                    const image = await screen.captureWindow(win);
                    await image.save({});
                    "#,
                    js_path(&output_path)
                ))
                .await
                .unwrap();
            println!("saved capture to {}", output_path.display());
        });
    }

    #[test]
    #[traced_test]
    fn test_search_in_to_string() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cases: &[(&str, &str)] = &[
                ("SearchIn.desktop().toString()", "SearchIn(desktop)"),
                (
                    "SearchIn.display(displays.primary()).toString()",
                    "SearchIn(display_id: <id>)",
                ),
                (
                    "SearchIn.rect(0, 0, 1920, 1080).toString()",
                    "SearchIn(rect: (x: 0, y: 0, width: 1920, height: 1080))",
                ),
            ];
            for (expr, expected) in cases {
                let s: String = script_engine.eval_async(expr).await.unwrap();
                // For the display case the ID is dynamic, just check the prefix
                if expected.contains("<id>") {
                    assert!(
                        s.starts_with("SearchIn(display_id:"),
                        "failed for: {expr}, got: {s}"
                    );
                } else {
                    assert_eq!(s, *expected, "failed for: {expr}");
                }
            }
        });
    }
}
