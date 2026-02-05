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
    /// Text operations
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn text(&self) -> JsClipboardText {
        self.text.clone()
    }

    /// Image operations
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn image(&self) -> JsClipboardImage {
        self.image.clone()
    }

    /// File list operations
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn file_list(&self) -> JsClipboardFileList {
        self.file_list.clone()
    }

    /// Html operations
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn html(&self) -> JsClipboardHtml {
        self.html.clone()
    }

    pub async fn clear(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.clear(*mode).into_js_result(&ctx)
    }
}

/// ClipboardText
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
    pub async fn set(&self, ctx: Ctx<'_>, text: String, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.set_text(text, *mode).into_js_result(&ctx)
    }

    pub async fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_text(*mode).into_js_result(&ctx)
    }
}

/// ClipboardImage
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

    pub async fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<JsImage> {
        let image = self.inner.get_image(*mode).into_js_result(&ctx)?;

        Ok(image.into())
    }
}

/// ClipboardFileList
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

    pub async fn get(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<Vec<String>> {
        self.inner.get_file_list(*mode).into_js_result(&ctx)
    }
}

/// ClipboardHtml
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
