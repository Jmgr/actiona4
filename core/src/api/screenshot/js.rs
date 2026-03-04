use std::sync::Arc;

use color_eyre::eyre::eyre;
use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::sync::watch;

use crate::{
    IntoJsResult,
    api::{
        color::js::JsColor,
        image::{
            find_image::{FindImageProgress, FindImageStage, FindImageTemplateOptions, Template},
            js::{JsFindImageOptions, JsFindImageProgress, JsImage, JsMatch},
        },
        js::{
            classes::{HostClass, SingletonClass, register_host_class},
            task::progress_task_with_token,
        },
        name::{Name, js::JsNameLike},
        point::js::JsPointLike,
        rect::{Rect, js::JsRectLike},
        screenshot::{Screenshot, display_selector::DisplaySelector, search_in::SearchIn},
        windows::js::JsWindowHandle,
    },
    runtime::WithUserData,
    types::display::DisplayFields,
};

/// Inner storage for `JsDisplay`.
///
/// Either a fully-resolved `DisplaySelector` (for all non-name cases), or a
/// `Name<'js>` that will be matched against the display list at resolution time.
#[derive(Clone, Debug, JsLifetime)]
enum JsDisplayInner<'js> {
    Selector(DisplaySelector),
    ByName(Name<'js>),
}

impl<'js> Trace<'js> for JsDisplayInner<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Self::ByName(name) = self {
            name.trace(tracer);
        }
    }
}

/// A display selector resolved at capture or search time.
///
/// Use the static factory methods to create a `Display`:
///
/// ```ts
/// // Capture a specific display
/// const img = await screenshot.captureDisplay(Display.primary());
/// const img = await screenshot.captureDisplay(Display.largest());
/// const img = await screenshot.captureDisplay(Display.fromId(474));
/// const img = await screenshot.captureDisplay(Display.fromName("HDMI-1"));
/// const img = await screenshot.captureDisplay(Display.fromName(new Wildcard("HDMI-*")));
/// const img = await screenshot.captureDisplay(Display.fromName(/HDMI-.*/));
/// const img = await screenshot.captureDisplay(Display.fromPoint(100, 200));
/// ```
#[derive(Clone, Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "Display")]
pub struct JsDisplay<'js> {
    inner: JsDisplayInner<'js>,
}

impl<'js> HostClass<'js> for JsDisplay<'js> {}

impl<'js> JsDisplay<'js> {
    /// Resolve `self` to a `DisplaySelector`.
    async fn resolve(self, ctx: Ctx<'js>) -> color_eyre::Result<DisplaySelector> {
        match self.inner {
            JsDisplayInner::Selector(sel) => Ok(sel),
            JsDisplayInner::ByName(name) => {
                let displays = ctx.user_data().displays();
                let displays_info = displays.wait_get_info().await?;
                let mut matching: Vec<u32> = displays_info
                    .iter()
                    .filter_map(|d| {
                        name.matches(&ctx, &d.friendly_name)
                            .ok()
                            .and_then(|m| if m { Some(d.id) } else { None })
                    })
                    .collect();
                match matching.len() {
                    0 => Err(match &name {
                        Name::String(s) => eyre!("display not found: {s}"),
                        Name::Wildcard(w) => {
                            eyre!("display not found matching: {}", w.pattern())
                        }
                        Name::Regex(_) => eyre!("no display found matching the given RegExp"),
                    }),
                    1 => Ok(DisplaySelector::ById(matching.remove(0))),
                    n => Err(match &name {
                        Name::String(s) => eyre!(
                            "{n} displays match the name \"{s}\"; use Display.fromId() to select by ID"
                        ),
                        Name::Wildcard(w) => eyre!(
                            "{n} displays match the pattern \"{}\"; use a more specific pattern",
                            w.pattern()
                        ),
                        Name::Regex(_) => eyre!(
                            "{n} displays match the given RegExp; use a more specific pattern"
                        ),
                    }),
                }
            }
        }
    }

    fn to_string_inner(&self) -> String {
        let fields = match &self.inner {
            JsDisplayInner::Selector(sel) => format!("{sel}"),
            JsDisplayInner::ByName(Name::String(s)) => DisplayFields::default()
                .display("name", s)
                .finish_as_string(),
            JsDisplayInner::ByName(Name::Wildcard(w)) => DisplayFields::default()
                .display("pattern", w.pattern())
                .finish_as_string(),
            JsDisplayInner::ByName(Name::Regex(_)) => "(name: <regex>)".to_string(),
        };
        format!("Display{fields}")
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> JsDisplay<'js> {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "Display cannot be instantiated directly; use Display.primary(), Display.fromName(), etc.",
        ))
    }

    /// Selects the entire desktop (the bounding rectangle of all connected displays).
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.desktop());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn desktop() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Desktop),
        }
    }

    /// Selects the primary (main) display.
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.primary());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn primary() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Primary),
        }
    }

    /// Selects the display with the largest area.
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.largest());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn largest() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Largest),
        }
    }

    /// Selects the display with the smallest area.
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.smallest());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn smallest() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Smallest),
        }
    }

    /// Selects the display furthest to the left (minimum left edge).
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.leftmost());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn leftmost() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Leftmost),
        }
    }

    /// Selects the display furthest to the right (maximum right edge).
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.rightmost());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn rightmost() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Rightmost),
        }
    }

    /// Selects the display furthest to the top (minimum top edge).
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.topmost());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn topmost() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Topmost),
        }
    }

    /// Selects the display furthest to the bottom (maximum bottom edge).
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.bottommost());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn bottommost() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Bottommost),
        }
    }

    /// Selects the display whose center is closest to the center of the desktop.
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.center());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn center() -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::Center),
        }
    }

    /// Selects a display by its unique numeric ID.
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.fromId(474));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn from_id(id: u32) -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::ById(id)),
        }
    }

    /// Selects a display by its friendly name.
    ///
    /// Accepts a plain string (exact match), a `Wildcard` pattern, or a `RegExp`.
    /// String and wildcard names are resolved at capture time (no cache required at
    /// construction); regex names require the display cache to be available when used
    /// with `findImage`, or will wait for it with `captureDisplay`.
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.fromName("HDMI-1"));
    /// const img = await screenshot.captureDisplay(Display.fromName(new Wildcard("HDMI-*")));
    /// const img = await screenshot.captureDisplay(Display.fromName(/HDMI-.*/));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn from_name(name: JsNameLike<'js>) -> Self {
        Self {
            inner: JsDisplayInner::ByName(name.0),
        }
    }

    /// Selects the display that contains the given point.
    ///
    /// ```ts
    /// const img = await screenshot.captureDisplay(Display.fromPoint(100, 200));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn from_point(point: JsPointLike) -> Self {
        Self {
            inner: JsDisplayInner::Selector(DisplaySelector::FromPoint(point.0)),
        }
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.to_string_inner()
    }
}

#[derive(Clone, Debug, JsLifetime)]
enum JsSearchInInner<'js> {
    Desktop,
    Display(JsDisplay<'js>),
    Rect(Rect),
    Window(JsWindowHandle),
}

impl<'js> Trace<'js> for JsSearchInInner<'js> {
    fn trace<'a>(&self, tracer: Tracer<'a, 'js>) {
        if let Self::Display(display) = self {
            display.trace(tracer);
        }
        if let Self::Window(window) = self {
            window.trace(tracer);
        }
    }
}

/// Specifies the screen area to search within for find-image operations.
///
/// ```ts
/// // Search the entire desktop
/// const match = await screenshot.findImage(image, SearchIn.desktop());
///
/// // Search a specific display
/// const match = await screenshot.findImage(image, SearchIn.display(Display.primary()));
///
/// // Search a specific rectangle
/// const match = await screenshot.findImage(image, SearchIn.rect(0, 0, 1920, 1080));
/// ```
#[derive(Clone, Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "SearchIn")]
pub struct JsSearchIn<'js> {
    inner: JsSearchInInner<'js>,
}

impl<'js> HostClass<'js> for JsSearchIn<'js> {}

impl<'js> JsSearchIn<'js> {
    async fn resolve(self, ctx: Ctx<'js>) -> color_eyre::Result<SearchIn> {
        match self.inner {
            JsSearchInInner::Desktop => Ok(SearchIn::Desktop),
            JsSearchInInner::Rect(r) => Ok(SearchIn::Rect(r)),
            JsSearchInInner::Display(display) => Ok(SearchIn::Display(display.resolve(ctx).await?)),
            JsSearchInInner::Window(handle) => Ok(SearchIn::Window(handle.window_id())),
        }
    }

    fn to_string_inner(&self) -> String {
        let fields = match &self.inner {
            JsSearchInInner::Desktop => "(desktop)".to_string(),
            JsSearchInInner::Display(display) => DisplayFields::default()
                .display("display", display.to_string_inner())
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

#[rquickjs::methods(rename_all = "camelCase")]
impl<'js> JsSearchIn<'js> {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'js>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "SearchIn cannot be instantiated directly; use SearchIn.desktop(), SearchIn.display(), SearchIn.rect(), or SearchIn.window().",
        ))
    }

    /// Searches within the entire desktop (the bounding rectangle of all connected displays).
    ///
    /// ```ts
    /// const match = await screenshot.findImage(image, SearchIn.desktop());
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn desktop() -> Self {
        Self {
            inner: JsSearchInInner::Desktop,
        }
    }

    /// Searches within a specific display identified by a `Display` selector.
    ///
    /// ```ts
    /// const match = await screenshot.findImage(image, SearchIn.display(Display.primary()));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn display(display: JsDisplay<'js>) -> Self {
        Self {
            inner: JsSearchInInner::Display(display),
        }
    }

    /// Searches within the given screen rectangle.
    ///
    /// ```ts
    /// const match = await screenshot.findImage(image, SearchIn.rect(0, 0, 1920, 1080));
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
    /// const match = await screenshot.findImage(image, SearchIn.window(win));
    /// ```
    #[qjs(static)]
    #[must_use]
    pub const fn window(handle: JsWindowHandle) -> Self {
        Self {
            inner: JsSearchInInner::Window(handle),
        }
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.to_string_inner()
    }
}

impl<'js> Trace<'js> for super::Screenshot {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// Screenshot capture and image search.
///
/// Provides methods to capture the entire desktop, a specific display, a screen
/// region, or a single pixel, as well as finding images on screen.
///
/// ```ts
/// const image = await screenshot.captureDesktop();
/// println(image.size());
/// ```
///
/// ```ts
/// const image = await screenshot.captureDisplay(Display.primary());
/// println(image.size());
/// ```
///
/// ```ts
/// const pixel = await screenshot.capturePixel(100, 100);
/// println(pixel);
/// ```
///
/// @singleton
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "Screenshot")]
pub struct JsScreenshot {
    inner: super::Screenshot,
}

impl SingletonClass<'_> for JsScreenshot {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_host_class::<JsDisplay<'_>>(ctx)?;
        register_host_class::<JsSearchIn<'_>>(ctx)?;
        Ok(())
    }
}

impl JsScreenshot {
    /// @skip
    #[must_use]
    pub const fn new(inner: super::Screenshot) -> Self {
        Self { inner }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsScreenshot {
    /// Captures a screenshot of the entire desktop.
    ///
    /// ```ts
    /// const image = await screenshot.captureDesktop();
    /// ```
    pub async fn capture_desktop(&self, ctx: Ctx<'_>) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner.capture_desktop().await.into_js_result(&ctx)?,
        ))
    }

    /// Captures a screenshot of the display identified by the given selector.
    ///
    /// ```ts
    /// const image = await screenshot.captureDisplay(Display.primary());
    /// const image = await screenshot.captureDisplay(Display.fromId(474));
    /// const image = await screenshot.captureDisplay(Display.fromName(/HDMI-.*/));
    /// ```
    pub async fn capture_display<'js>(
        &self,
        ctx: Ctx<'js>,
        display: JsDisplay<'js>,
    ) -> Result<JsImage> {
        let selector = display.resolve(ctx.clone()).await.into_js_result(&ctx)?;
        Ok(JsImage::new(
            self.inner
                .capture_display(&selector)
                .await
                .into_js_result(&ctx)?,
        ))
    }

    /// Captures a screenshot of a screen rectangle.
    ///
    /// ```ts
    /// const image = await screenshot.captureRect(0, 0, 1920, 1080);
    /// ```
    pub async fn capture_rect(&self, ctx: Ctx<'_>, rect: JsRectLike) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner.capture_rect(rect.0).await.into_js_result(&ctx)?,
        ))
    }

    /// Captures a screenshot of the bounding rectangle of the given window.
    ///
    /// ```ts
    /// const win = windows.activeWindow();
    /// const image = await screenshot.captureWindow(win);
    /// ```
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
    /// const color = await screenshot.capturePixel(100, 200);
    /// println(color);
    /// ```
    pub async fn capture_pixel(&self, ctx: Ctx<'_>, position: JsPointLike) -> Result<JsColor> {
        Ok(self
            .inner
            .capture_pixel(position.0)
            .await
            .into_js_result(&ctx)?
            .into())
    }

    /// Finds the best match of an image within the given search area.
    ///
    /// ```ts
    /// const match = await screenshot.findImage(image, SearchIn.desktop());
    /// ```
    ///
    /// ```ts
    /// const task = screenshot.findImage(image, SearchIn.display(Display.primary()));
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const match = await task;
    /// ```
    /// @returns ProgressTask<Match | undefined, FindImageProgress>
    pub fn find_image<'js>(
        &self,
        ctx: Ctx<'js>,
        image: JsImage,
        search_in: JsSearchIn<'js>,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        find_image_task(
            ctx,
            &self.inner,
            search_in,
            image,
            options,
            |inner, search_in, template, opts, token, progress| async move {
                let result = inner
                    .find_image(&template, &search_in, opts, token, progress)
                    .await?;
                Ok(result.map(JsMatch::from))
            },
        )
    }

    /// Finds all matches of an image within the given search area.
    ///
    /// ```ts
    /// const matches = await screenshot.findImageAll(image, SearchIn.desktop());
    /// ```
    ///
    /// ```ts
    /// const task = screenshot.findImageAll(image, SearchIn.rect(0, 0, 1920, 1080));
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const matches = await task;
    /// ```
    /// @returns ProgressTask<Match[], FindImageProgress>
    pub fn find_image_all<'js>(
        &self,
        ctx: Ctx<'js>,
        image: JsImage,
        search_in: JsSearchIn<'js>,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        find_image_task(
            ctx,
            &self.inner,
            search_in,
            image,
            options,
            |inner, search_in, template, opts, token, progress| async move {
                let results = inner
                    .find_image_all(&template, &search_in, opts, token, progress)
                    .await?;
                Ok(results.into_iter().map(JsMatch::from).collect::<Vec<_>>())
            },
        )
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Screenshot".to_string()
    }
}

/// Shared helper for all `find_image*` JS bindings.
fn find_image_task<'js, R, F, Fut>(
    ctx: Ctx<'js>,
    inner: &Screenshot,
    search_in: JsSearchIn<'js>,
    image: JsImage,
    options: Opt<JsFindImageOptions>,
    search: F,
) -> Result<Promise<'js>>
where
    R: rquickjs::IntoJs<'js> + 'js,
    F: FnOnce(
            Screenshot,
            SearchIn,
            Arc<Template>,
            FindImageTemplateOptions,
            tokio_util::sync::CancellationToken,
            watch::Sender<FindImageProgress>,
        ) -> Fut
        + 'js,
    Fut: Future<Output = color_eyre::Result<R>> + 'js,
{
    let options = options.0.unwrap_or_default();
    let signal = options.signal.clone();
    let template = Arc::<Template>::try_from(image.to_inner()).into_js_result(&ctx)?;
    let inner = inner.clone();
    let (progress_sender, progress_receiver) =
        watch::channel(FindImageProgress::new(FindImageStage::Capturing, 0));

    progress_task_with_token::<_, _, _, _, _, JsFindImageProgress>(
        ctx,
        signal,
        progress_receiver,
        async move |ctx, token| {
            let search_in = search_in.resolve(ctx.clone()).await.into_js_result(&ctx)?;
            let result = search(
                inner,
                search_in,
                template,
                options.into_inner(),
                token,
                progress_sender,
            )
            .await
            .into_js_result(&ctx)?;

            Ok(result)
        },
    )
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
                .eval_async("(await screenshot.captureDesktop()).width")
                .await
                .unwrap();
            let height: u32 = script_engine
                .eval_async("(await screenshot.captureDesktop()).height")
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
                .eval_async("(await screenshot.captureDisplay(Display.primary())).width")
                .await
                .unwrap();
            let height: u32 = script_engine
                .eval_async("(await screenshot.captureDisplay(Display.primary())).height")
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
                .eval_async("(await screenshot.captureDisplay(Display.largest())).width")
                .await
                .unwrap();
            let height: u32 = script_engine
                .eval_async("(await screenshot.captureDisplay(Display.largest())).height")
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
                    const image = await screenshot.captureDesktop();
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
                    const displayInfo = (await displays.all())[0];
                    if (!displayInfo) throw new Error("No display available");
                    const image = await screenshot.captureDisplay(Display.fromName(displayInfo.friendlyName));
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
                    const displayInfo = (await displays.all())[0];
                    if (!displayInfo) throw new Error("No display available");
                    const rect = displayInfo.rect;
                    const centerX = Math.floor(rect.x + rect.width / 2);
                    const centerY = Math.floor(rect.y + rect.height / 2);
                    const image = await screenshot.captureDisplay(Display.fromPoint(centerX, centerY));
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
                    const displayInfo = (await displays.all())[0];
                    if (!displayInfo) throw new Error("No display available");
                    const rect = displayInfo.rect;
                    const captureWidth = Math.max(1, Math.min(rect.width, 300));
                    const captureHeight = Math.max(1, Math.min(rect.height, 200));
                    const image = await screenshot.captureRect(rect.x, rect.y, captureWidth, captureHeight);
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
                    const image = await screenshot.captureWindow(win);
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
    fn test_display_to_string() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cases: &[(&str, &str)] = &[
                ("Display.desktop().toString()", "Display(desktop)"),
                ("Display.primary().toString()", "Display(primary)"),
                ("Display.largest().toString()", "Display(largest)"),
                ("Display.smallest().toString()", "Display(smallest)"),
                ("Display.fromId(42).toString()", "Display(id: 42)"),
                (
                    r#"Display.fromName("HDMI-1").toString()"#,
                    "Display(name: HDMI-1)",
                ),
                (
                    r#"Display.fromName(new Wildcard("HDMI-*")).toString()"#,
                    "Display(pattern: HDMI-*)",
                ),
                (
                    r#"Display.fromName(/HDMI-.*/).toString()"#,
                    "Display(name: <regex>)",
                ),
            ];
            for (expr, expected) in cases {
                let s: String = script_engine.eval_async(expr).await.unwrap();
                assert_eq!(s, *expected, "failed for: {expr}");
            }
        });
    }

    #[test]
    #[traced_test]
    fn test_search_in_to_string() {
        Runtime::test_with_script_engine(async |script_engine| {
            let cases: &[(&str, &str)] = &[
                ("SearchIn.desktop().toString()", "SearchIn(desktop)"),
                (
                    "SearchIn.display(Display.primary()).toString()",
                    "SearchIn(display: Display(primary))",
                ),
                (
                    "SearchIn.rect(0, 0, 1920, 1080).toString()",
                    "SearchIn(rect: (x: 0, y: 0, width: 1920, height: 1080))",
                ),
            ];
            for (expr, expected) in cases {
                let s: String = script_engine.eval_async(expr).await.unwrap();
                assert_eq!(s, *expected, "failed for: {expr}");
            }
        });
    }
}
