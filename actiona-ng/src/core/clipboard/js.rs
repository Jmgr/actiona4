use std::{fmt::Debug, sync::Arc};

use rquickjs::{
    JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};

use crate::{
    IntoJSError, IntoJsResult,
    core::{
        image::js::JsImage,
        js::classes::{SingletonClass, register_enum},
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

/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Clipboard")]
pub struct JsClipboard {
    inner: Arc<super::Clipboard>,
}

impl<'js> Trace<'js> for JsClipboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsClipboard {
    fn register_dependencies(ctx: &Ctx<'js>) -> Result<()> {
        register_enum::<JsClipboardMode>(ctx)?;

        Ok(())
    }
}

impl JsClipboard {
    /// @skip
    pub fn new(clipboard: Arc<super::Clipboard>) -> Self {
        Self { inner: clipboard }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboard {
    pub fn set_text(&self, ctx: Ctx<'_>, text: String, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.set_text(text, *mode).into_js_result(&ctx)
    }

    pub fn get_text(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_text(*mode).into_js_result(&ctx)
    }

    pub fn set_image(
        &self,
        ctx: Ctx<'_>,
        image: JsImage,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.inner
            .set_image(image.into_inner(), *mode)
            .into_js_result(&ctx)
    }

    pub fn get_image(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<JsImage> {
        let image = self.inner.get_image(*mode).into_js_result(&ctx)?;

        Ok(image.into())
    }

    pub fn get_file_list(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<Vec<String>> {
        self.inner.get_file_list(*mode).into_js_result(&ctx)
    }

    pub fn set_html(
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

    pub fn get_html(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_html(*mode).into_js_result(&ctx)
    }

    pub fn clear(&self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.clear(*mode).into_js_result(&ctx)
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, Rgba, RgbaImage};
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
                await clipboard.setText("test");
                await clipboard.getText()
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
                        .set(
                            "image",
                            JsImage::new(DynamicImage::ImageRgba8(local_image).into()),
                        )
                        .unwrap();
                })
                .await;

            let result = script_engine
                .eval_async::<JsImage>(
                    r#"
                await clipboard.setImage(image);
                await clipboard.getImage()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result.into_inner().to_rgba8().into_owned(), image);
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
                await clipboard.setHtml("<b>test</b>", "test")
                "#,
                )
                .await
                .unwrap();

            let result = script_engine
                .eval_async::<String>(
                    r#"
                await clipboard.getHtml()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result, "<b>test</b>");

            let result = script_engine
                .eval_async::<String>(
                    r#"
                await clipboard.getText()
                "#,
                )
                .await
                .unwrap();

            assert_eq!(result, "test");
        });
    }
}
