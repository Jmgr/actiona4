use std::{fmt::Debug, sync::Arc};

use rquickjs::{
    JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};
use tracing::instrument;

use crate::{
    IntoJSError, IntoJsResult,
    core::{
        image::js::JsImage,
        js::classes::{HostClass, SingletonClass, register_enum, register_host_class},
    },
    newtype,
};

impl IntoJSError for super::Error {}

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

// TODO: add waitForChanged
/// The global clipboard singleton for reading and writing clipboard content.
///
/// Supports text, images, file lists, and HTML content. Each content type
/// is accessed through a dedicated sub-object.
///
/// ```ts
/// // Copy and paste text
/// await clipboard.text.set("Hello, world!");
/// const text = await clipboard.text.get();
///
/// // Copy and paste an image
/// const img = display.screenshot();
/// await clipboard.image.set(img);
///
/// // Work with file lists
/// await clipboard.fileList.set(["/path/to/file.txt"]);
///
/// // HTML content with alt text fallback
/// await clipboard.html.set("<b>bold</b>", "bold");
///
/// // Clear the clipboard
/// await clipboard.clear();
///
/// // On Linux, use the selection clipboard
/// await clipboard.text.set("selected", ClipboardMode.Selection);
/// ```
///
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Clipboard")]
pub struct JsClipboard {
    inner: Arc<super::Clipboard>,
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
    pub fn new(clipboard: Arc<super::Clipboard>) -> Self {
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

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboard {
    /// Sub-object for text clipboard operations.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn text(&self) -> JsClipboardText {
        self.text.clone()
    }

    /// Sub-object for image clipboard operations.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn image(&self) -> JsClipboardImage {
        self.image.clone()
    }

    /// Sub-object for file list clipboard operations.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn file_list(&self) -> JsClipboardFileList {
        self.file_list.clone()
    }

    /// Sub-object for HTML clipboard operations.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn html(&self) -> JsClipboardHtml {
        self.html.clone()
    }

    /// Clears the clipboard contents.
    ///
    /// ```ts
    /// await clipboard.clear();
    ///
    /// // On Linux, clear the selection clipboard
    /// await clipboard.clear(ClipboardMode.Selection);
    /// ```
    pub async fn clear(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.clear(*mode).into_js_result(&ctx)
    }
}

/// Provides text clipboard operations.
///
/// ```ts
/// await clipboard.text.set("Hello!");
/// const text = await clipboard.text.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "ClipboardText")]
pub struct JsClipboardText {
    inner: Arc<super::Clipboard>,
}

impl<'js> HostClass<'js> for JsClipboardText {}

impl<'js> Trace<'js> for JsClipboardText {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardText {
    /// @skip
    #[must_use]
    pub fn new(clipboard: Arc<super::Clipboard>) -> Self {
        Self { inner: clipboard }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboardText {
    /// Sets the clipboard text content.
    pub async fn set(&self, ctx: Ctx<'_>, text: String, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.set_text(text, *mode).into_js_result(&ctx)
    }

    /// Gets the clipboard text content.
    pub async fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_text(*mode).into_js_result(&ctx)
    }
}

/// Provides image clipboard operations.
///
/// ```ts
/// const img = display.screenshot();
/// await clipboard.image.set(img);
/// const clipped = await clipboard.image.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "ClipboardImage")]
pub struct JsClipboardImage {
    inner: Arc<super::Clipboard>,
}

impl<'js> HostClass<'js> for JsClipboardImage {}

impl<'js> Trace<'js> for JsClipboardImage {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardImage {
    /// @skip
    #[must_use]
    pub fn new(clipboard: Arc<super::Clipboard>) -> Self {
        Self { inner: clipboard }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboardImage {
    /// Sets the clipboard image content.
    pub async fn set(
        &self,
        ctx: Ctx<'_>,
        image: JsImage,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.inner
            .set_image(image.into_inner(), *mode)
            .into_js_result(&ctx)
    }

    /// Gets the clipboard image content.
    pub async fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<JsImage> {
        let image = self.inner.get_image(*mode).into_js_result(&ctx)?;

        Ok(image.into())
    }
}

/// Provides file list clipboard operations.
///
/// ```ts
/// await clipboard.fileList.set(["/home/user/doc.pdf", "/home/user/img.png"]);
/// const files = await clipboard.fileList.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "ClipboardFileList")]
pub struct JsClipboardFileList {
    inner: Arc<super::Clipboard>,
}

impl<'js> HostClass<'js> for JsClipboardFileList {}

impl<'js> Trace<'js> for JsClipboardFileList {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardFileList {
    /// @skip
    #[must_use]
    pub fn new(clipboard: Arc<super::Clipboard>) -> Self {
        Self { inner: clipboard }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboardFileList {
    /// Sets the clipboard file list content.
    pub async fn set(
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
    pub async fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<Vec<String>> {
        self.inner.get_file_list(*mode).into_js_result(&ctx)
    }
}

/// Provides HTML clipboard operations.
///
/// ```ts
/// // Set HTML with a plain-text fallback
/// await clipboard.html.set("<b>bold</b>", "bold");
///
/// // Set HTML without a fallback
/// await clipboard.html.set("<em>italic</em>");
///
/// const html = await clipboard.html.get();
/// ```
#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "ClipboardHtml")]
pub struct JsClipboardHtml {
    inner: Arc<super::Clipboard>,
}

impl<'js> HostClass<'js> for JsClipboardHtml {}

impl<'js> Trace<'js> for JsClipboardHtml {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsClipboardHtml {
    /// @skip
    #[must_use]
    pub fn new(clipboard: Arc<super::Clipboard>) -> Self {
        Self { inner: clipboard }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboardHtml {
    /// Sets the clipboard HTML content, with an optional plain-text alternative.
    pub async fn set(
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
    pub async fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_html(*mode).into_js_result(&ctx)
    }
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};
    use imageproc::drawing::Canvas;
    use tracing_test::traced_test;

    use crate::{core::image::js::JsImage, runtime::Runtime};

    #[test]
    #[traced_test]
    #[ignore]
    fn test_set_text() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<String>(
                    r#"
                await clipboard.text.set("test");
                await clipboard.text.get()
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
                        .set("image", JsImage::new(local_image.into()))
                        .unwrap();
                })
                .await;

            let result = script_engine
                .eval_async::<JsImage>(
                    r#"
                await clipboard.image.set(image);
                await clipboard.image.get()
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
                await clipboard.html.set("<b>test</b>", "test")
                "#,
                )
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<String>(
                    r#"
                await clipboard.html.get()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result, "<b>test</b>");

            let result = script_engine
                .eval_async::<String>(
                    r#"
                await clipboard.text.get()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result, "test");
        });
    }
}
