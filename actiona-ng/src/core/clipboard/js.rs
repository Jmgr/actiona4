use std::{borrow::Cow, fmt::Debug};

#[cfg(linux)]
use arboard::{ClearExtLinux, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use arboard::{Get, ImageData, Set};
use convert_case::{Case, Casing};
use eyre::eyre;
use image::{DynamicImage, RgbaImage};
use itertools::Itertools;
use macros::ExposeEnum;
use rquickjs::{
    Exception, JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};
use strum::Display;

use crate::{
    IntoJS,
    core::{SingletonClass, image::js::JsImage},
    newtype,
};

impl<T> IntoJS<T> for super::Result<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> rquickjs::Result<T> {
        // TODO
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

#[derive(Clone, Copy, Debug, Default, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "ClipboardMode")]
pub enum JsClipboardMode {
    #[default]
    Clipboard,

    /// @platforms =linux
    Selection,
}

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
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        JsClipboardMode::register(ctx)?;

        Ok(())
    }
}

impl JsClipboard {
    /// @skip
    pub fn new(ctx: &Ctx<'_>) -> rquickjs::Result<Self> {
        Ok(Self {
            inner: super::Clipboard::new().into_js(ctx)?,
        })
    }

    /// @skip
    fn set(&'_ mut self, mode: Opt<JsClipboardMode>) -> Set<'_> {
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner.inner;

        #[cfg(linux)]
        if mode == JsClipboardMode::Selection {
            inner.set().clipboard(LinuxClipboardKind::Primary)
        } else {
            inner.set()
        }

        #[cfg(not(linux))]
        inner.set()
    }

    /// @skip
    fn get(&'_ mut self, mode: Opt<JsClipboardMode>) -> Get<'_> {
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner.inner;

        #[cfg(linux)]
        if mode == JsClipboardMode::Selection {
            inner.get().clipboard(LinuxClipboardKind::Primary)
        } else {
            inner.get()
        }

        #[cfg(not(linux))]
        inner.get()
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
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner.inner;

        let clipboard = {
            #[cfg(linux)]
            if mode == JsClipboardMode::Selection {
                inner.set().clipboard(LinuxClipboardKind::Primary)
            } else {
                inner.set()
            }

            #[cfg(not(linux))]
            inner.set()
        };

        clipboard
            .text(text)
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)
    }

    pub async fn get_text(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.get(mode)
            .text()
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)
        // TODO: fails if the clipboard is empty?
        // TODO: errors?
    }

    pub async fn set_image(
        &mut self,
        ctx: Ctx<'_>,
        image: JsImage,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        let image = image.into_inner().to_rgba8().into_owned();
        let (width, height) = image.dimensions();
        let bytes = Cow::Owned(image.into_raw());

        self.set(mode)
            .image(ImageData {
                width: width as usize,
                height: height as usize,
                bytes,
            })
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)?;

        Ok(())
    }

    pub async fn get_image(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<JsImage> {
        let image = self
            .get(mode)
            .image()
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)?;

        let img = RgbaImage::from_vec(
            image.width as u32,
            image.height as u32,
            image.bytes.to_vec(),
        )
        .unwrap();

        Ok(JsImage::new(DynamicImage::ImageRgba8(img).into()))
    }

    /// @returns string[]
    pub async fn get_file_list(
        &mut self,
        ctx: Ctx<'_>,
        mode: Opt<JsClipboardMode>,
    ) -> Result<Vec<String>> {
        let result = self
            .get(mode)
            .file_list()
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)?;

        let result = result
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect_vec();

        Ok(result)
    }

    pub async fn set_html(
        &mut self,
        ctx: Ctx<'_>,
        html: String,
        alt_text: Opt<String>,
        mode: Opt<JsClipboardMode>,
    ) -> Result<()> {
        self.set(mode)
            .html(html, alt_text.0)
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)
    }

    pub async fn get_html(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<String> {
        self.get(mode)
            .html()
            .map_err(|err| eyre!("{err}"))
            .into_js(&ctx)
    }

    pub async fn clear(&mut self, ctx: Ctx<'_>, mode: Opt<JsClipboardMode>) -> Result<()> {
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner.inner;

        #[cfg(linux)]
        if mode == JsClipboardMode::Selection {
            inner
                .clear_with()
                .clipboard(LinuxClipboardKind::Primary)
                .map_err(|err| eyre!("{err}"))
                .into_js(&ctx)?;
        } else {
            inner.clear().map_err(|err| eyre!("{err}")).into_js(&ctx)?;
        }

        #[cfg(not(linux))]
        inner.clear().map_err(|err| eyre!("{err}")).into_js(&ctx)?;

        Ok(())
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
                await clipboard.setHtml("<b>test</b>", "test")                "#,
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
