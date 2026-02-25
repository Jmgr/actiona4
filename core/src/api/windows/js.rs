use std::sync::Arc;

use rquickjs::{
    Ctx, JsLifetime, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
};
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        ResultExt,
        js::classes::{HostClass, SingletonClass, register_host_class},
        name::js::JsName,
        point::js::{JsPoint, JsPointLike},
        rect::js::JsRect,
        size::js::{JsSize, JsSizeLike},
    },
    runtime::Runtime,
    types::display::{DisplayFields, display_with_type},
};

/// Window search options.
///
/// @options
#[derive(Debug, Default)]
pub struct JsWindowsFindOptions<'js> {
    /// Match by internal window ID.
    /// When undefined, any window ID is accepted.
    /// @default `undefined`
    pub id: Option<u64>,

    /// Match by window process ID.
    /// When undefined, any process ID is accepted.
    /// @default `undefined`
    pub process_id: Option<u32>,

    /// Match by window visibility.
    /// When undefined, visibility is not filtered.
    /// @default `undefined`
    pub visible: Option<bool>,

    /// Match by window title.
    /// When undefined, title is not filtered.
    /// @default `undefined`
    pub title: Option<JsName<'js>>,

    /// Match by window class name.
    /// When undefined, class name is not filtered.
    /// @default `undefined`
    pub class_name: Option<JsName<'js>>,
}

impl<'js> rquickjs::FromJs<'js> for JsWindowsFindOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.is_undefined() || value.is_null() {
            return Ok(Self::default());
        }

        let object = value
            .into_object()
            .or_throw_message(ctx, "Expected an object")?;

        Ok(Self {
            id: object.get("id")?,
            process_id: object.get("processId")?,
            visible: object.get("visible")?,
            title: object.get("title")?,
            class_name: object.get("className")?,
        })
    }
}

/// Manages desktop windows: enumerate, focus, move, resize, and close windows.
///
/// ```ts
/// // Get all windows
/// const allWindows = windows.all();
/// for (const win of allWindows) {
///     println(win.title());
/// }
/// ```
///
/// ```ts
/// // Get the active window and move it
/// const win = windows.activeWindow();
/// win.setPosition(100, 100);
/// win.setSize(800, 600);
/// ```
///
/// ```ts
/// // Find and close a window by title
/// const matches = windows.find({ title: new Wildcard("*Notepad*") });
/// for (const win of matches) {
///     win.close();
/// }
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Windows")]
pub struct JsWindows {
    inner: super::Windows,
}

impl<'js> SingletonClass<'js> for JsWindows {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsWindowHandle>(ctx)?;

        Ok(())
    }
}

impl<'js> Trace<'js> for JsWindows {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsWindows {
    /// @skip
    #[must_use]
    #[instrument(skip_all)]
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            inner: super::Windows::new(runtime),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWindows {
    /// Returns all currently open windows.
    ///
    /// ```ts
    /// const allWindows = windows.all();
    /// println(`Found ${allWindows.length} windows`);
    /// ```
    /// @readonly
    pub fn all(&self, ctx: Ctx<'_>) -> Result<Vec<JsWindowHandle>> {
        let ids = self.inner.all().into_js_result(&ctx)?;

        Ok(ids
            .into_iter()
            .map(|id| JsWindowHandle {
                inner: self.inner.clone(),
                id,
            })
            .collect())
    }

    /// Returns the currently active (focused) window.
    ///
    /// ```ts
    /// const win = windows.activeWindow();
    /// println(win.title());
    /// ```
    /// @readonly
    pub fn active_window(&self, ctx: Ctx<'_>) -> Result<JsWindowHandle> {
        let id = self.inner.active_window().into_js_result(&ctx)?;

        Ok(JsWindowHandle {
            inner: self.inner.clone(),
            id,
        })
    }

    /// Finds windows matching the provided criteria.
    ///
    /// `title` and `className` support NameLike matching (`string | Wildcard | RegExp`).
    ///
    /// ```ts
    /// const byId = windows.find({ id: 1 });
    /// const visibleCode = windows.find({ visible: true, title: new Wildcard("*Code*") });
    /// const byPid = windows.find({ processId: 12345 });
    /// const byTitle = windows.find({ title: new Wildcard("*Code*") });
    /// const byClass = windows.find({ className: /^gnome-terminal/i });
    /// const exact = windows.find({ title: "Calculator", className: "ApplicationFrameWindow" });
    /// ```
    /// @readonly
    pub fn find<'js>(
        &self,
        ctx: Ctx<'js>,
        options: JsWindowsFindOptions<'js>,
    ) -> Result<Vec<JsWindowHandle>> {
        let ids = self.inner.all().into_js_result(&ctx)?;
        let mut windows = Vec::new();

        for id in ids {
            if let Some(filter_id) = options.id
                && id.as_u64() != filter_id
            {
                continue;
            }

            if let Some(filter_visible) = options.visible {
                let visible = self.inner.is_visible(id).into_js_result(&ctx)?;
                if visible != filter_visible {
                    continue;
                }
            }

            if let Some(filter_process_id) = options.process_id {
                let process_id = self.inner.process_id(id).into_js_result(&ctx)?;
                if process_id != filter_process_id {
                    continue;
                }
            }

            if let Some(title) = options.title.as_ref() {
                let window_title = self.inner.title(id).into_js_result(&ctx)?;
                if !title.inner().matches(&ctx, &window_title)? {
                    continue;
                }
            }

            if let Some(class_name) = options.class_name.as_ref() {
                let window_class_name = self.inner.classname(id).into_js_result(&ctx)?;
                if !class_name.inner().matches(&ctx, &window_class_name)? {
                    continue;
                }
            }

            windows.push(JsWindowHandle {
                inner: self.inner.clone(),
                id,
            });
        }

        Ok(windows)
    }

    /// Finds windows whose rectangle contains the given screen point.
    ///
    /// ```ts
    /// const underMouse = windows.findAt(mouse.position());
    /// const atOrigin = windows.findAt(0, 0);
    /// ```
    /// @readonly
    pub fn find_at(&self, ctx: Ctx<'_>, point: JsPointLike) -> Result<Vec<JsWindowHandle>> {
        let ids = self.inner.all().into_js_result(&ctx)?;
        let mut windows = Vec::new();

        for id in ids {
            let rect = self.inner.rect(id).into_js_result(&ctx)?;
            if !rect.contains(point.0) {
                continue;
            }

            windows.push(JsWindowHandle {
                inner: self.inner.clone(),
                id,
            });
        }

        Ok(windows)
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Windows", &self.inner)
    }
}

/// A handle to a specific desktop window.
///
/// Obtained from `windows.all()` or `windows.activeWindow()`.
/// Provides methods to query and manipulate the window.
///
/// ```ts
/// const win = windows.activeWindow();
/// println(win.title());
/// println(win.isVisible());
/// println(win.rect());
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "WindowHandle")]
pub struct JsWindowHandle {
    inner: super::Windows,
    id: super::WindowId,
}

impl<'js> HostClass<'js> for JsWindowHandle {}

impl<'js> Trace<'js> for JsWindowHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWindowHandle {
    /// Returns whether this window is visible.
    ///
    /// ```ts
    /// const visible = win.isVisible();
    /// ```
    pub fn is_visible(&self, ctx: Ctx<'_>) -> Result<bool> {
        self.inner.is_visible(self.id).into_js_result(&ctx)
    }

    /// Returns the window title.
    ///
    /// ```ts
    /// const title = win.title();
    /// ```
    pub fn title(&self, ctx: Ctx<'_>) -> Result<String> {
        self.inner.title(self.id).into_js_result(&ctx)
    }

    /// Returns the window class name.
    ///
    /// ```ts
    /// const className = win.className();
    /// ```
    pub fn class_name(&self, ctx: Ctx<'_>) -> Result<String> {
        self.inner.classname(self.id).into_js_result(&ctx)
    }

    /// Closes this window.
    ///
    /// ```ts
    /// win.close();
    /// ```
    pub fn close(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.close(self.id).into_js_result(&ctx)
    }

    /// Returns the process ID of the window's owning process.
    ///
    /// ```ts
    /// const pid = win.processId();
    /// ```
    pub fn process_id(&self, ctx: Ctx<'_>) -> Result<u32> {
        self.inner.process_id(self.id).into_js_result(&ctx)
    }

    /// Returns the window's bounding rectangle.
    ///
    /// ```ts
    /// const r = win.rect();
    /// println(`${r.x}, ${r.y}, ${r.width}x${r.height}`);
    /// ```
    /// @readonly
    pub fn rect(&self, ctx: Ctx<'_>) -> Result<JsRect> {
        Ok(self.inner.rect(self.id).into_js_result(&ctx)?.into())
    }

    /// Makes this window the active (focused) window.
    ///
    /// ```ts
    /// win.setActive();
    /// ```
    pub fn set_active(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.set_active(self.id).into_js_result(&ctx)
    }

    /// Minimizes this window.
    ///
    /// ```ts
    /// win.minimize();
    /// ```
    pub fn minimize(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.minimize(self.id).into_js_result(&ctx)
    }

    /// Maximizes this window.
    ///
    /// ```ts
    /// win.maximize();
    /// ```
    pub fn maximize(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.maximize(self.id).into_js_result(&ctx)
    }

    /// Sets the window position.
    ///
    /// ```ts
    /// win.setPosition(100, 200);
    /// win.setPosition(new Point(100, 200));
    /// win.setPosition({x: 100, y: 200});
    /// ```
    pub fn set_position(&self, ctx: Ctx<'_>, position: JsPointLike) -> Result<()> {
        self.inner
            .set_position(self.id, position.0)
            .into_js_result(&ctx)
    }

    /// Returns the window position.
    ///
    /// ```ts
    /// const pos = win.position();
    /// println(`${pos.x}, ${pos.y}`);
    /// ```
    /// @readonly
    pub fn position(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self.inner.position(self.id).into_js_result(&ctx)?.into())
    }

    /// Sets the window size.
    ///
    /// ```ts
    /// win.setSize(800, 600);
    /// win.setSize(new Size(800, 600));
    /// win.setSize({width: 800, height: 600});
    /// ```
    pub fn set_size(&self, ctx: Ctx<'_>, size: JsSizeLike) -> Result<()> {
        self.inner.set_size(self.id, size.0).into_js_result(&ctx)
    }

    /// Returns the window size.
    ///
    /// ```ts
    /// const s = win.size();
    /// println(`${s.width}x${s.height}`);
    /// ```
    /// @readonly
    pub fn size(&self, ctx: Ctx<'_>) -> Result<JsSize> {
        Ok(self.inner.size(self.id).into_js_result(&ctx)?.into())
    }

    /// Returns whether this window is the active (focused) window.
    ///
    /// ```ts
    /// const active = win.isActive();
    /// ```
    pub fn is_active(&self, ctx: Ctx<'_>) -> Result<bool> {
        self.inner.is_active(self.id).into_js_result(&ctx)
    }

    /// Returns a string representation of this window handle.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        let id = format!("{:?}", self.id);
        let title = self
            .inner
            .title(self.id)
            .unwrap_or_else(|_| "<unavailable>".to_string());
        let class_name = self
            .inner
            .classname(self.id)
            .unwrap_or_else(|_| "<unavailable>".to_string());
        let process_id = self
            .inner
            .process_id(self.id)
            .map_or_else(|_| "?".to_string(), |pid| pid.to_string());
        let visible = self
            .inner
            .is_visible(self.id)
            .map_or_else(|_| "?".to_string(), |value| value.to_string());

        display_with_type(
            "WindowHandle",
            DisplayFields::default()
                .display("id", id)
                .display("title", title)
                .display("className", class_name)
                .display("processId", process_id)
                .display("visible", visible)
                .finish_as_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::runtime::Runtime;

    #[test]
    #[traced_test]
    #[ignore]
    fn test_all() {
        Runtime::test_with_script_engine(async |script_engine| {
            let count: i32 = script_engine
                .eval_async("windows.all().length")
                .await
                .unwrap();
            assert!(count > 0, "Expected at least one window");
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_active_window() {
        Runtime::test_with_script_engine(async |script_engine| {
            let title: String = script_engine
                .eval_async("windows.activeWindow().title()")
                .await
                .unwrap();
            println!("Active window title: {title}");
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_window_properties() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    const win = windows.activeWindow();
                    const title = win.title();
                    const visible = win.isVisible();
                    const active = win.isActive();
                    const pos = win.position();
                    const s = win.size();
                    const r = win.rect();
                    const pid = win.processId();
                    const cls = win.className();
                    console.log(`title: ${title}, visible: ${visible}, active: ${active}`);
                    console.log(`position: ${pos.x},${pos.y}, size: ${s.width}x${s.height}`);
                    console.log(`rect: ${r.x},${r.y} ${r.width}x${r.height}`);
                    console.log(`pid: ${pid}, class: ${cls}`);
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_find() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    const byAnyTitle = windows.find({ title: /.*/ });
                    for (const win of byAnyTitle) {
                        const title = win.title();
                        if (!/.*/.test(title)) {
                            throw new Error("title filter mismatch");
                        }
                    }

                    const byAnyClass = windows.find({ className: /.*/ });
                    for (const win of byAnyClass) {
                        const className = win.className();
                        if (!/.*/.test(className)) {
                            throw new Error("className filter mismatch");
                        }
                    }

                    const active = windows.activeWindow();
                    const pid = active.processId();
                    const byPid = windows.find({ processId: pid });
                    if (byPid.length === 0) {
                        throw new Error("processId filter mismatch");
                    }

                    const byVisible = windows.find({ visible: true });
                    for (const win of byVisible) {
                        if (!win.isVisible()) {
                            throw new Error("visible filter mismatch");
                        }
                    }

                    const activeRect = active.rect();
                    const center = {
                        x: Math.floor(activeRect.x + activeRect.width / 2),
                        y: Math.floor(activeRect.y + activeRect.height / 2),
                    };

                    const byPoint = windows.findAt(center);
                    if (byPoint.length === 0) {
                        throw new Error("findAt filter mismatch");
                    }

                    for (const win of byPoint) {
                        const rect = win.rect();
                        if (
                            center.x < rect.x
                            || center.x >= rect.x + rect.width
                            || center.y < rect.y
                            || center.y >= rect.y + rect.height
                        ) {
                            throw new Error("findAt containment mismatch");
                        }
                    }
                    "#,
                )
                .await
                .unwrap();
        });
    }
}
