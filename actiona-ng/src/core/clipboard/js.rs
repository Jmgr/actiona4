use std::fmt::Debug;

#[cfg(linux)]
use rquickjs::{
    JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};

use crate::{
    IntoJS, IntoJSError,
    core::{image::js::JsImage, js::classes::SingletonClass},
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
    inner: super::Clipboard,
}

impl<'js> Trace<'js> for JsClipboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsClipboard {
    fn register_dependencies(ctx: &Ctx<'js>) -> Result<()> {
        JsClipboardMode::register(ctx)?;

        Ok(())
    }
}

impl JsClipboard {
    /// @skip
    pub fn new(ctx: &Ctx<'_>) -> Result<Self> {
        Ok(Self {
            inner: super::Clipboard::new().into_js(ctx)?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsClipboard {
    pub async fn set_text(
        &mut self,
        ctx: Ctx<'_>,
        text: String,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.inner.set_text(text, *mode).await.into_js(&ctx)
    }

    pub async fn get_text(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_text(*mode).await.into_js(&ctx)
    }

    pub async fn set_image(
        &mut self,
        ctx: Ctx<'_>,
        image: JsImage,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.inner
            .set_image(image.into_inner(), *mode)
            .await
            .into_js(&ctx)
    }

    pub async fn get_image(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<JsImage> {
        let image = self.inner.get_image(*mode).await.into_js(&ctx)?;

        Ok(image.into())
    }

    /// @returns string[]
    pub async fn get_file_list(
        &mut self,
        ctx: Ctx<'_>,
        mode: Opt<JsClipboardMode>,
    ) -> Result<Vec<String>> {
        self.inner.get_file_list(*mode).await.into_js(&ctx)
    }

    pub async fn set_html(
        &mut self,
        ctx: Ctx<'_>,
        html: String,
        alt_text: Opt<String>,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.inner
            .set_html(html, alt_text.0, *mode)
            .await
            .into_js(&ctx)
    }

    pub async fn get_html(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.inner.get_html(*mode).await.into_js(&ctx)
    }

    pub async fn clear(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<()> {
        self.inner.clear(*mode).await.into_js(&ctx)
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
    fn test_set_text() {
        Runtime::test_with_script_engine(async move |script_engine| {
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
    fn test_set_image() {
        Runtime::test_with_script_engine(async move |script_engine| {
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
    fn test_set_html() {
        Runtime::test_with_script_engine(async move |script_engine| {
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
