use rquickjs::{
    Ctx, JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};

use crate::{
    IntoJsResult,
    core::{
        js::classes::{SingletonClass, ValueClass, register_enum, register_value_class},
        ui::{MessageBoxButtons, MessageBoxOptions, Ui},
    },
    runtime::WithUserData,
};

pub type JsMessageBoxIcon = super::MessageBoxIcon;
pub type JsMessageBoxResult = super::MessageBoxResult;

/// @singleton
#[derive(Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "Ui")]
pub struct JsUi {}

impl SingletonClass<'_> for JsUi {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_value_class::<JsMessageBoxButtons>(ctx)?;
        register_enum::<JsMessageBoxIcon>(ctx)?;
        register_enum::<JsMessageBoxResult>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsUi {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsUi {
    /// @skip
    #[qjs(skip)]
    pub const fn new() -> eyre::Result<Self> {
        Ok(Self {})
    }

    #[qjs(static)]
    pub async fn message_box<'js>(
        ctx: Ctx<'js>,
        text: String,
        options: Opt<MessageBoxOptions>,
    ) -> Result<JsMessageBoxResult> {
        let app_handle = ctx.user_data().app_handle();

        Ui::message_box(app_handle, text, options.0)
            .await
            .into_js_result(&ctx)
    }
}

#[derive(Clone, Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "MessageBoxButtons")]
pub struct JsMessageBoxButtons {
    inner: MessageBoxButtons,
}

impl<'js> Trace<'js> for JsMessageBoxButtons {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl ValueClass<'_> for JsMessageBoxButtons {}

impl JsMessageBoxButtons {
    #[must_use]
    pub fn into_inner(self) -> MessageBoxButtons {
        self.inner
    }
}

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
