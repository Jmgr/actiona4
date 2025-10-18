use std::sync::Arc;

use macros::FromJsObject;
use rquickjs::{
    Ctx, JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};

use crate::{
    core::{
        displays::{Displays, js::JsDisplayInfo},
        image::js,
        js::classes::SingletonClass,
        point::js::JsPoint,
    },
    runtime::Runtime,
};

/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Ui")]
pub struct JsUi {
    _runtime: Arc<Runtime>,
    displays: Arc<Displays>,
}

impl SingletonClass<'_> for JsUi {}

impl<'js> Trace<'js> for JsUi {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> Trace<'js> for super::Ui {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// Window options
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsWindowOptions {
    display: Option<JsDisplayInfo>, // TODO: add position
    position: Option<JsPoint>,
}

/*
#[allow(unsafe_code)]
mod ui {
    #[cfg(not(doc))]
    slint::slint! {
        import { Button, StandardButton } from "std-widgets.slint";

        export component ImageWindow inherits Window {
            callback closed;

            in property <image> image;
            in property <length> window_width;
            in property <length> window_height;

            width: self.window_width;
            height: self.window_height;

            Image {
                source: image;
                image-fit: contain;
                image-rendering: pixelated;
            }

            forward-focus: my-key-handler;
            my-key-handler := FocusScope {
                key-pressed(event) => {
                    if (event.text == Key.Escape || event.text == Key.Return) {
                        root.closed();
                    }
                    accept
                }
            }
        }
    }
}
*/

#[rquickjs::methods(rename_all = "camelCase")]
impl JsUi {
    /// @skip
    #[qjs(skip)]
    pub const fn new(runtime: Arc<Runtime>, displays: Arc<Displays>) -> eyre::Result<Self> {
        Ok(Self {
            _runtime: runtime,
            displays,
        })
    }

    pub fn display_image(
        &self,
        ctx: Ctx<'_>,
        image: &js::JsImage,
        options: Opt<JsWindowOptions>,
    ) -> Result<()> {
        let _options = options.clone().unwrap_or_default();

        /*
        let h = Rc::new(ui::ImageWindow::new().unwrap());

        let image = image.to_inner().to_rgba8();

        let primary_display = self.displays.primary_display().into_js(&ctx)?;
        let center = primary_display.rect.center();

        h.show().unwrap();

        h.window()
            .set_position(LogicalPosition::new(center.x as f32, center.y as f32));
        //h.window().set_size(LogicalSize::new(
        //   image.width() as f32,
        //   image.height() as f32,
        //));
        h.set_window_width(image.width() as f32);
        h.set_window_height(image.height() as f32);

        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            image.as_raw(),
            image.width(),
            image.height(),
        );

        h.on_closed({
            let h = h.clone();
            move || {
                h.hide().unwrap(); // Hides the dialog
            }
        });

        //h.on_ok_clicked(|| h.hide().unwrap());

        h.set_image(Image::from_rgba8(buffer));
        /*
        let (tx, mut rx) = watch::channel(());
        h.on_closed(move || tx.send(()).unwrap());
        let local_cancellation_token = self.runtime.cancellation_token().clone();
        h.show().unwrap();

        Runtime::block_on(async {
            select! {
                _ = rx.changed() => {},
                _ = local_cancellation_token.cancelled() => {},
            }
        });
        */
        h.run().unwrap();

        */

        Ok(())
    }
}
