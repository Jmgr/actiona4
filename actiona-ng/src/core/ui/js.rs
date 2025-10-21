use std::sync::Arc;

use macros::FromJsObject;
use rquickjs::{
    Ctx, JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tauri::AppHandle;

use crate::{
    IntoJsResult,
    core::{
        displays::{Displays, js::JsDisplayInfo},
        image::js,
        js::classes::{SingletonClass, ValueClass, register_value_class},
        point::js::JsPoint,
        ui::{MessageBox, MessageBoxButtons},
    },
    runtime::{Runtime, WithUserData},
};

pub type JsMessageBoxIcon = super::MessageBoxIcon;
pub type JsMessageBoxResult = super::MessageBoxResult;

/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Ui")]
pub struct JsUi {
    _runtime: Arc<Runtime>,
    displays: Arc<Displays>,
}

impl SingletonClass<'_> for JsUi {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_value_class::<JsMessageBox>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsUi {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> Trace<'js> for super::Ui {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[derive(Clone, Debug, Default, JsLifetime, Trace)]
#[rquickjs::class(rename = "MessageBoxButtons")]
pub struct JsMessageBoxButtons {
    inner: MessageBoxButtons,
}

impl ValueClass<'_> for JsMessageBoxButtons {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMessageBoxButtons {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[qjs(static)]
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            inner: MessageBoxButtons::Ok,
        }
    }

    #[qjs(static)]
    #[must_use]
    pub const fn ok_custom(ok_label: String) -> Self {
        Self {
            inner: MessageBoxButtons::OkCustom(ok_label),
        }
    }

    #[qjs(static)]
    #[must_use]
    pub const fn ok_cancel() -> Self {
        Self {
            inner: MessageBoxButtons::OkCancel,
        }
    }

    #[qjs(static)]
    #[must_use]
    pub const fn ok_cancel_custom(ok_label: String, cancel_label: String) -> Self {
        Self {
            inner: MessageBoxButtons::OkCancelCustom(ok_label, cancel_label),
        }
    }

    #[qjs(static)]
    #[must_use]
    pub const fn yes_no() -> Self {
        Self {
            inner: MessageBoxButtons::YesNo,
        }
    }

    #[qjs(static)]
    #[must_use]
    pub const fn yes_no_cancel() -> Self {
        Self {
            inner: MessageBoxButtons::YesNoCancel,
        }
    }

    #[qjs(static)]
    #[must_use]
    pub const fn yes_no_cancel_custom(
        yes_label: String,
        no_label: String,
        cancel_label: String,
    ) -> Self {
        Self {
            inner: MessageBoxButtons::YesNoCancelCustom(yes_label, no_label, cancel_label),
        }
    }
}

#[derive(Clone, Debug, JsLifetime)]
#[rquickjs::class(rename = "MessageBox")]
pub struct JsMessageBox {
    app_handle: AppHandle,
}

impl ValueClass<'_> for JsMessageBox {
    fn register_dependencies(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        register_value_class::<JsMessageBoxButtons>(ctx)?;
        JsMessageBoxIcon::register(ctx)?;
        JsMessageBoxResult::register(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsMessageBox {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMessageBox {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    #[must_use]
    pub fn new<'js>(ctx: Ctx<'js>) -> Self {
        Self {
            app_handle: ctx.user_data().app_handle(),
        }
    }

    #[qjs(static)]
    pub async fn show<'js>(
        ctx: Ctx<'js>,
        text: String,
        title: String,
        buttons: Opt<JsMessageBoxButtons>,
        icon: Opt<JsMessageBoxIcon>,
    ) -> Result<JsMessageBoxResult> {
        let buttons = buttons.0.unwrap_or_default();
        let icon = icon.0.unwrap_or_default();
        let app_handle = ctx.user_data().app_handle();

        MessageBox::builder()
            .app_handle(app_handle)
            .text(text)
            .title(title)
            .buttons(buttons.inner)
            .icon(icon)
            .build()
            .show()
            .await
            .into_js_result(&ctx)
    }
}

/// Window options
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsWindowOptions {
    display: Option<JsDisplayInfo>, // TODO: add position
    position: Option<JsPoint>,
}

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
