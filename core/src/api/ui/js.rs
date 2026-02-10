use macros::FromJsObject;
use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};

use crate::{
    IntoJsResult,
    api::{
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

/// Message box options.
///
/// ```ts
/// await Ui.messageBox("Delete this file?", {
///   title: "Confirm",
///   buttons: MessageBoxButtons.yesNo(),
///   icon: MessageBoxIcon.Warning,
/// });
/// ```
/// @category UI
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsMessageBoxOptions {
    /// Title displayed in the message box title bar.
    /// @default `undefined`
    pub title: Option<String>,

    /// Buttons displayed in the message box.
    /// @default `MessageBoxButtons.ok()`
    pub buttons: Option<JsMessageBoxButtons>,

    /// Icon displayed in the message box.
    /// @default `MessageBoxIcon.Info`
    pub icon: Option<super::MessageBoxIcon>,

    /// Abort signal to cancel the message box.
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

/// User interface utilities.
///
/// Provides methods for displaying message boxes and other UI elements.
/// Only available when running with the Tauri UI.
///
/// ```ts
/// const result = await Ui.messageBox("Hello, world!");
/// ```
///
/// ```ts
/// const result = await Ui.messageBox("Delete this file?", {
///   title: "Confirm",
///   buttons: MessageBoxButtons.yesNo(),
///   icon: MessageBoxIcon.Warning,
/// });
/// if (result === MessageBoxResult.Yes) {
///   println("Confirmed");
/// }
/// ```
///
/// @category UI
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

    /// Displays a message box and returns the user's response.
    ///
    /// ```ts
    /// const result = await Ui.messageBox("Operation complete");
    /// ```
    ///
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

/// Button configurations for message boxes.
///
/// Use the static factory methods to create button sets.
///
/// ```ts
/// const buttons = MessageBoxButtons.ok();
/// const buttons2 = MessageBoxButtons.yesNoCancel();
/// const buttons3 = MessageBoxButtons.okCancelCustom("Save", "Discard");
/// ```
/// @category UI
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

    /// Creates an OK button.
    #[qjs(static)]
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            inner: MessageBoxButtons::Ok,
        }
    }

    /// Creates an OK button with a custom label.
    #[qjs(static)]
    #[must_use]
    pub const fn ok_custom(ok_label: String) -> Self {
        Self {
            inner: MessageBoxButtons::OkCustom(ok_label),
        }
    }

    /// Creates OK and Cancel buttons.
    #[qjs(static)]
    #[must_use]
    pub const fn ok_cancel() -> Self {
        Self {
            inner: MessageBoxButtons::OkCancel,
        }
    }

    /// Creates OK and Cancel buttons with custom labels.
    #[qjs(static)]
    #[must_use]
    pub const fn ok_cancel_custom(ok_label: String, cancel_label: String) -> Self {
        Self {
            inner: MessageBoxButtons::OkCancelCustom(ok_label, cancel_label),
        }
    }

    /// Creates Yes and No buttons.
    #[qjs(static)]
    #[must_use]
    pub const fn yes_no() -> Self {
        Self {
            inner: MessageBoxButtons::YesNo,
        }
    }

    /// Creates Yes, No, and Cancel buttons.
    #[qjs(static)]
    #[must_use]
    pub const fn yes_no_cancel() -> Self {
        Self {
            inner: MessageBoxButtons::YesNoCancel,
        }
    }

    /// Creates Yes, No, and Cancel buttons with custom labels.
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
