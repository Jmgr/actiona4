use std::{fmt::Debug, time::Duration};

use macros::{FromJsObject, js_class, js_methods, options};
use rquickjs::{
    JsLifetime, Promise, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::*,
};
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        image::js::JsImage,
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_enum, register_host_class},
            duration::JsDuration,
            task::task_with_token,
        },
    },
    newtype,
};

newtype!(
    #[derive(JsLifetime)]
    Clipboard,
    arboard::Clipboard
);

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Clipboard").finish()
    }
}

pub type JsClipboardMode = super::ClipboardMode;

/// Options for waiting until clipboard content changes.
///
/// ```ts
/// // Wait for any clipboard change
/// await clipboard.waitForChanged();
///
/// // Wait on Linux selection clipboard with a custom polling interval
/// await clipboard.waitForChanged({ mode: ClipboardMode.Selection, interval: 0.05 });
///
/// // Wait up to 1 second for a clipboard change
/// await Concurrency.race([
///   clipboard.waitForChanged(),
///   sleep("1s"),
/// ]);
/// ```
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsWaitForChangedOptions {
    /// Clipboard source to watch.
    #[default(ts = "ClipboardMode.Clipboard")]
    pub mode: Option<JsClipboardMode>,

    /// Polling interval in seconds.
    #[default(Duration::from_millis(200).into(), ts = "0.2")]
    pub interval: JsDuration,

    /// Abort signal to cancel the wait.
    pub signal: Option<JsAbortSignal>,
}

impl From<JsWaitForChangedOptions> for super::WaitForChangedOptions {
    fn from(value: JsWaitForChangedOptions) -> Self {
        Self {
            mode: value.mode,
            interval: value.interval.into(),
        }
    }
}

/// The global clipboard singleton for reading and writing clipboard content.
///
/// Supports text, images, file lists, and HTML content. Each content type
/// is accessed through a dedicated sub-object.
///
/// ```ts
/// // Copy and paste text
/// clipboard.text.set("Hello, world!");
/// const text = clipboard.text.get();
///
/// // Copy and paste an image
/// const img = screen.captureDesktop();
/// clipboard.image.set(img);
///
/// // Work with file lists
/// clipboard.fileList.set(["/path/to/file.txt"]);
///
/// // HTML content with alt text fallback
/// clipboard.html.set("<b>bold</b>", "bold");
///
/// // Clear the clipboard
/// clipboard.clear();
///
/// // On Linux, use the selection clipboard
/// clipboard.text.set("selected", ClipboardMode.Selection);
///
/// // Wait until clipboard content changes
/// await clipboard.waitForChanged();
/// ```
///
/// @singleton
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsClipboard {
    inner: super::Clipboard,
    text: JsClipboardText,
    image: JsClipboardImage,
    file_list: JsClipboardFileList,
    html: JsClipboardHtml,
}

impl<'js> Trace<'js> for JsClipboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsClipboard {
    fn register_dependencies(ctx: &Ctx<'js>) -> Result<()> {
        register_enum::<JsClipboardMode>(ctx)?;
        register_host_class::<JsClipboardText>(ctx)?;
        register_host_class::<JsClipboardImage>(ctx)?;
        register_host_class::<JsClipboardFileList>(ctx)?;
        register_host_class::<JsClipboardHtml>(ctx)?;

        Ok(())
    }
}

impl JsClipboard {
    /// @skip
    #[must_use]
    #[instrument(skip_all)]
    pub fn new(clipboard: super::Clipboard) -> Self {
        let text = JsClipboardText::new(clipboard.clone());
        let image = JsClipboardImage::new(clipboard.clone());
        let file_list = JsClipboardFileList::new(clipboard.clone());
        let html = JsClipboardHtml::new(clipboard.clone());

        Self {
            inner: clipboard,
            text,
            image,
            file_list,
            html,
        }
    }
}

#[js_methods]
impl JsClipboard {
    /// Sub-object for text clipboard operations.
    #[get]
    #[must_use]
    pub fn text(&self) -> JsClipboardText {
        self.text.clone()
    }

    /// Sub-object for image clipboard operations.
    #[get]
    #[must_use]
    pub fn image(&self) -> JsClipboardImage {
        self.image.clone()
    }

    /// Sub-object for file list clipboard operations.
    #[get]
    #[must_use]
    pub fn file_list(&self) -> JsClipboardFileList {
        self.file_list.clone()
    }

    /// Sub-object for HTML clipboard operations.
    #[get]
    #[must_use]
    pub fn html(&self) -> JsClipboardHtml {
        self.html.clone()
    }

    /// Clears the clipboard contents.
    ///
    /// ```ts
    /// clipboard.clear();
    ///
    /// // On Linux, clear the selection clipboard
    /// clipboard.clear(ClipboardMode.Selection);
    /// ```
    pub fn clear(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.clear(*mode).into_js_result(&ctx)
    }

    /// Waits until clipboard content changes.
    ///
    /// ```ts
    /// const controller = new AbortController();
    /// const task = clipboard.waitForChanged({ signal: controller.signal });
    /// // controller.abort();
    /// await task;
    /// ```
    /// @returns Task<void>
    pub fn wait_for_changed<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsWaitForChangedOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let local_clipboard = self.inner.clone();
        let options: super::WaitForChangedOptions = options.into();

        task_with_token(ctx, signal, async move |ctx, token| {
            local_clipboard
                .wait_for_changed(options, token)
                .await
                .into_js_result(&ctx)
        })
    }

    /// Returns a string representation of the `clipboard` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Clipboard".to_string()
    }
}

/// Provides text clipboard operations.
///
/// ```ts
/// clipboard.text.set("Hello!");
/// const text = clipboard.text.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsClipboardText {
    inner: super::Clipboard,
}

impl<'js> HostClass<'js> for JsClipboardText {}

impl<'js> Trace<'js> for JsClipboardText {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardText {
    /// @skip
    #[must_use]
    pub const fn new(clipboard: super::Clipboard) -> Self {
        Self { inner: clipboard }
    }
}

#[js_methods]
impl JsClipboardText {
    /// Sets the clipboard text content.
    pub fn set(&self, ctx: Ctx<'_>, text: String, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.set_text(text, *mode).into_js_result(&ctx)
    }

    /// Gets the clipboard text content.
    pub fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_text(*mode).into_js_result(&ctx)
    }

    /// Returns a string representation of this clipboard text.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "ClipboardText".to_string()
    }
}

/// Provides image clipboard operations.
///
/// ```ts
/// const img = screen.captureDesktop();
/// clipboard.image.set(img);
/// const clipped = clipboard.image.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsClipboardImage {
    inner: super::Clipboard,
}

impl<'js> HostClass<'js> for JsClipboardImage {}

impl<'js> Trace<'js> for JsClipboardImage {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardImage {
    /// @skip
    #[must_use]
    pub const fn new(clipboard: super::Clipboard) -> Self {
        Self { inner: clipboard }
    }
}

#[js_methods]
impl JsClipboardImage {
    /// Sets the clipboard image content.
    pub fn set(&self, ctx: Ctx<'_>, image: JsImage, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner
            .set_image(image.into_inner(), *mode)
            .into_js_result(&ctx)
    }

    /// Gets the clipboard image content.
    pub fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<JsImage> {
        let image = self.inner.get_image(*mode).into_js_result(&ctx)?;

        Ok(image.into())
    }

    /// Returns a string representation of this clipboard image.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "ClipboardImage".to_string()
    }
}

/// Provides file list clipboard operations.
///
/// ```ts
/// clipboard.fileList.set(["/home/user/doc.pdf", "/home/user/img.png"]);
/// const files = clipboard.fileList.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsClipboardFileList {
    inner: super::Clipboard,
}

impl<'js> HostClass<'js> for JsClipboardFileList {}

impl<'js> Trace<'js> for JsClipboardFileList {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardFileList {
    /// @skip
    #[must_use]
    pub const fn new(clipboard: super::Clipboard) -> Self {
        Self { inner: clipboard }
    }
}

#[js_methods]
impl JsClipboardFileList {
    /// Sets the clipboard file list content.
    pub fn set(
        &self,
        ctx: Ctx<'_>,
        file_list: Vec<String>,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.inner
            .set_file_list(&file_list, *mode)
            .into_js_result(&ctx)
    }

    /// Gets the clipboard file list content.
    /// @readonly
    pub fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<Vec<String>> {
        self.inner.get_file_list(*mode).into_js_result(&ctx)
    }

    /// Returns a string representation of this clipboard file list.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "ClipboardFileList".to_string()
    }
}

/// Provides HTML clipboard operations.
///
/// ```ts
/// // Set HTML with a plain-text fallback
/// clipboard.html.set("<b>bold</b>", "bold");
///
/// // Set HTML without a fallback
/// clipboard.html.set("<em>italic</em>");
///
/// const html = clipboard.html.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[js_class]
pub struct JsClipboardHtml {
    inner: super::Clipboard,
}

impl<'js> HostClass<'js> for JsClipboardHtml {}

impl<'js> Trace<'js> for JsClipboardHtml {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardHtml {
    /// @skip
    #[must_use]
    pub const fn new(clipboard: super::Clipboard) -> Self {
        Self { inner: clipboard }
    }
}

#[js_methods]
impl JsClipboardHtml {
    /// Sets the clipboard HTML content, with an optional plain-text alternative.
    pub fn set(
        &self,
        ctx: Ctx<'_>,
        html: String,
        alt_text: Opt<String>,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.inner
            .set_html(html, alt_text.0, *mode)
            .into_js_result(&ctx)
    }

    /// Gets the clipboard HTML content.
    pub fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_html(*mode).into_js_result(&ctx)
    }

    /// Returns a string representation of this clipboard html.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "ClipboardHtml".to_string()
    }
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};
    use imageproc::drawing::Canvas;
    use tracing_test::traced_test;

    use crate::{api::image::js::JsImage, runtime::Runtime};

    #[test]
    #[traced_test]
    #[ignore]
    fn test_set_text() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<String>(
                    r#"
                clipboard.text.set("test");
                clipboard.text.get()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result, "test");
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_set_image() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let mut image = RgbaImage::new(128, 128);

            image.draw_pixel(32, 32, Rgba([16, 32, 64, 128]));

            let local_image = image.clone();
            script_engine
                .with(|ctx| {
                    ctx.globals()
                        .set("image", JsImage::new(local_image.into()))?;
                    Ok(())
                })
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<JsImage>(
                    r#"
                clipboard.image.set(image);
                clipboard.image.get()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result.into_inner().into_rgba8(), image);
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_set_html() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                clipboard.html.set("<b>test</b>", "test")
                "#,
                )
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<String>(
                    r#"
                clipboard.html.get()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result, "<b>test</b>");

            let result = script_engine
                .eval_async::<String>(
                    r#"
                clipboard.text.get()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result, "test");
        });
    }
}
