use std::sync::Arc;

use rquickjs::{
    Ctx, JsLifetime, Result,
    class::{Trace, Tracer},
};
use tracing::instrument;

use crate::{
    IntoJsResult,
    core::{
        js::classes::{HostClass, SingletonClass, register_host_class},
        point::js::{JsPoint, JsPointLike},
        rect::js::JsRect,
        size::js::{JsSize, JsSizeLike},
    },
    runtime::Runtime,
};

impl<T> IntoJsResult<T> for super::Result<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| rquickjs::Exception::throw_message(ctx, &err.to_string()))
    }
}

/// Manages desktop windows: enumerate, focus, move, resize, and close windows.
///
/// ```ts
/// // Get all windows
/// const allWindows = await windows.all();
/// for (const win of allWindows) {
///     console.log(await win.title());
/// }
/// ```
///
/// ```ts
/// // Get the active window and move it
/// const win = await windows.activeWindow();
/// await win.setPosition(new Point(100, 100));
/// await win.setSize(800, 600);
/// ```
///
/// ```ts
/// // Find and close a window by title
/// const allWindows = await windows.all();
/// for (const win of allWindows) {
///     if ((await win.title()).includes("Notepad")) {
///         await win.close();
///     }
/// }
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Windows")]
pub struct JsWindows {
    inner: Arc<super::Windows>,
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
            inner: Arc::new(super::Windows::new(runtime)),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWindows {
    /// Returns all currently open windows.
    ///
    /// ```ts
    /// const allWindows = await windows.all();
    /// console.log(`Found ${allWindows.length} windows`);
    /// ```
    pub async fn all(&self, ctx: Ctx<'_>) -> Result<Vec<JsWindowHandle>> {
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
    /// const win = await windows.activeWindow();
    /// console.log(await win.title());
    /// ```
    pub async fn active_window(&self, ctx: Ctx<'_>) -> Result<JsWindowHandle> {
        let id = self.inner.active_window().into_js_result(&ctx)?;

        Ok(JsWindowHandle {
            inner: self.inner.clone(),
            id,
        })
    }
}

/// A handle to a specific desktop window.
///
/// Obtained from `windows.all()` or `windows.activeWindow()`.
/// Provides methods to query and manipulate the window.
///
/// ```ts
/// const win = await windows.activeWindow();
/// console.log(await win.title());
/// console.log(await win.isVisible());
/// console.log(await win.rect());
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "WindowHandle")]
pub struct JsWindowHandle {
    inner: Arc<super::Windows>,
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
    /// const visible = await win.isVisible();
    /// ```
    pub async fn is_visible(&self, ctx: Ctx<'_>) -> Result<bool> {
        self.inner.is_visible(self.id).into_js_result(&ctx)
    }

    /// Returns the window title.
    ///
    /// ```ts
    /// const title = await win.title();
    /// ```
    pub async fn title(&self, ctx: Ctx<'_>) -> Result<String> {
        self.inner.title(self.id).into_js_result(&ctx)
    }

    /// Returns the window class name.
    ///
    /// ```ts
    /// const className = await win.className();
    /// ```
    pub async fn class_name(&self, ctx: Ctx<'_>) -> Result<String> {
        self.inner.classname(self.id).into_js_result(&ctx)
    }

    /// Closes this window.
    ///
    /// ```ts
    /// await win.close();
    /// ```
    pub async fn close(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.close(self.id).into_js_result(&ctx)
    }

    /// Returns the process ID of the window's owning process.
    ///
    /// ```ts
    /// const pid = await win.processId();
    /// ```
    pub async fn process_id(&self, ctx: Ctx<'_>) -> Result<u32> {
        self.inner.process_id(self.id).into_js_result(&ctx)
    }

    /// Returns the window's bounding rectangle.
    ///
    /// ```ts
    /// const r = await win.rect();
    /// console.log(`${r.x}, ${r.y}, ${r.width}x${r.height}`);
    /// ```
    pub async fn rect(&self, ctx: Ctx<'_>) -> Result<JsRect> {
        Ok(self.inner.rect(self.id).into_js_result(&ctx)?.into())
    }

    /// Makes this window the active (focused) window.
    ///
    /// ```ts
    /// await win.setActive();
    /// ```
    pub async fn set_active(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.set_active(self.id).into_js_result(&ctx)
    }

    /// Minimizes this window.
    ///
    /// ```ts
    /// await win.minimize();
    /// ```
    pub async fn minimize(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.minimize(self.id).into_js_result(&ctx)
    }

    /// Maximizes this window.
    ///
    /// ```ts
    /// await win.maximize();
    /// ```
    pub async fn maximize(&self, ctx: Ctx<'_>) -> Result<()> {
        self.inner.maximize(self.id).into_js_result(&ctx)
    }

    /// Sets the window position.
    ///
    /// ```ts
    /// await win.setPosition(new Point(100, 200));
    /// await win.setPosition(100, 200);
    /// ```
    pub async fn set_position(&self, ctx: Ctx<'_>, position: JsPointLike) -> Result<()> {
        self.inner
            .set_position(self.id, position.0)
            .into_js_result(&ctx)
    }

    /// Returns the window position.
    ///
    /// ```ts
    /// const pos = await win.position();
    /// console.log(`${pos.x}, ${pos.y}`);
    /// ```
    pub async fn position(&self, ctx: Ctx<'_>) -> Result<JsPoint> {
        Ok(self.inner.position(self.id).into_js_result(&ctx)?.into())
    }

    /// Sets the window size.
    ///
    /// ```ts
    /// await win.setSize(new Size(800, 600));
    /// await win.setSize(800, 600);
    /// ```
    pub async fn set_size(&self, ctx: Ctx<'_>, size: JsSizeLike) -> Result<()> {
        self.inner.set_size(self.id, size.0).into_js_result(&ctx)
    }

    /// Returns the window size.
    ///
    /// ```ts
    /// const s = await win.size();
    /// console.log(`${s.width}x${s.height}`);
    /// ```
    pub async fn size(&self, ctx: Ctx<'_>) -> Result<JsSize> {
        Ok(self.inner.size(self.id).into_js_result(&ctx)?.into())
    }

    /// Returns whether this window is the active (focused) window.
    ///
    /// ```ts
    /// const active = await win.isActive();
    /// ```
    pub async fn is_active(&self, ctx: Ctx<'_>) -> Result<bool> {
        self.inner.is_active(self.id).into_js_result(&ctx)
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
                .eval_async("(await windows.all()).length")
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
                .eval_async("await (await windows.activeWindow()).title()")
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
                    const win = await windows.activeWindow();
                    const title = await win.title();
                    const visible = await win.isVisible();
                    const active = await win.isActive();
                    const pos = await win.position();
                    const s = await win.size();
                    const r = await win.rect();
                    const pid = await win.processId();
                    const cls = await win.className();
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
}
