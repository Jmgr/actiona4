use macros::FromJsObject;
use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};

use crate::{
    IntoJsResult,
    core::{
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_enum, register_host_class},
            task::task_with_token,
        },
        ui::{MessageBoxButtons, Ui},
    },
    runtime::WithUserData,
};

pub type JsMessageBoxIcon = super::MessageBoxIcon;
pub type JsMessageBoxResult = super::MessageBoxResult;

/// Message box options
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsMessageBoxOptions {
    /// @default `undefined`
    pub title: Option<String>,

    /// @default `MessageBoxButtons.ok()`
    pub buttons: Option<JsMessageBoxButtons>,

    /// @default `MessageBoxIcon.Info`
    pub icon: Option<super::MessageBoxIcon>,

    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl JsMessageBoxOptions {
    fn into_inner(self) -> super::MessageBoxOptions {
        super::MessageBoxOptions {
            title: self.title,
            buttons: self.buttons,
            icon: self.icon,
        }
    }
}

/// @singleton
#[derive(Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "Ui")]
pub struct JsUi {}

impl SingletonClass<'_> for JsUi {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_host_class::<JsMessageBoxButtons>(ctx)?;
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
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// @returns Task<MessageBoxResult>
    #[qjs(static)]
    pub fn message_box<'js>(
        ctx: Ctx<'js>,
        text: String,
        options: Opt<JsMessageBoxOptions>,
    ) -> Result<Promise<'js>> {
        let app_handle = ctx.user_data().app_handle();
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();

        task_with_token(ctx, signal, async move |ctx, _token| {
            Ui::message_box(app_handle, text, Some(options.into_inner()))
                .await
                .into_js_result(&ctx)
        })
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

impl HostClass<'_> for JsMessageBoxButtons {}

impl JsMessageBoxButtons {
    /// @skip
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
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "MessageBoxButtons cannot be instantiated directly",
        ))
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
