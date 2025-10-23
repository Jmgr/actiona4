use std::fmt::Debug;

use derive_more::Constructor;
use eyre::Result;
use macros::{FromJsObject, FromSerde, IntoSerde};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogResult};
use tokio::sync::oneshot;

use crate::core::ui::js::JsMessageBoxButtons;

pub mod js;

#[derive(
    Clone,
    Debug,
    Default,
    Display,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    EnumIter,
    IntoSerde,
    FromSerde,
)]
pub enum MessageBoxIcon {
    #[default]
    Info,
    Warning,
    Error,
}

impl From<MessageBoxIcon> for tauri_plugin_dialog::MessageDialogKind {
    fn from(value: MessageBoxIcon) -> Self {
        match value {
            MessageBoxIcon::Info => Self::Info,
            MessageBoxIcon::Warning => Self::Warning,
            MessageBoxIcon::Error => Self::Error,
        }
    }
}

#[derive(Clone, Debug, Default, Display, Eq, PartialEq)]
pub enum MessageBoxButtons {
    #[default]
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
    OkCustom(String),
    OkCancelCustom(String, String),
    YesNoCancelCustom(String, String, String),
}

impl From<MessageBoxButtons> for tauri_plugin_dialog::MessageDialogButtons {
    fn from(value: MessageBoxButtons) -> Self {
        match value {
            MessageBoxButtons::Ok => Self::Ok,
            MessageBoxButtons::OkCancel => Self::OkCancel,
            MessageBoxButtons::YesNo => Self::YesNo,
            MessageBoxButtons::YesNoCancel => Self::YesNoCancel,
            MessageBoxButtons::OkCustom(ok) => Self::OkCustom(ok),
            MessageBoxButtons::OkCancelCustom(ok, cancel) => Self::OkCancelCustom(ok, cancel),
            MessageBoxButtons::YesNoCancelCustom(yes, no, cancel) => {
                Self::YesNoCancelCustom(yes, no, cancel)
            }
        }
    }
}

#[derive(
    Clone, Debug, Display, Eq, PartialEq, Serialize, Deserialize, EnumIter, IntoSerde, FromSerde,
)]
pub enum MessageBoxResult {
    Yes,
    No,
    Ok,
    Cancel,
}

impl From<tauri_plugin_dialog::MessageDialogResult> for MessageBoxResult {
    fn from(value: tauri_plugin_dialog::MessageDialogResult) -> Self {
        match value {
            MessageDialogResult::Yes => Self::Yes,
            MessageDialogResult::No => Self::No,
            MessageDialogResult::Ok => Self::Ok,
            MessageDialogResult::Cancel => Self::Cancel,
            MessageDialogResult::Custom(_) => todo!(),
        }
    }
}

/// Message box options
/// @options
#[derive(Clone, Debug, FromJsObject, Default)]
pub struct MessageBoxOptions {
    /// @default null
    title: Option<String>,

    /// @default MessageBoxButtons.ok()
    buttons: Option<JsMessageBoxButtons>,

    /// @default MessageBoxIcon.Info
    icon: Option<MessageBoxIcon>,
}

#[derive(Constructor, Debug)]
pub struct Ui {}

impl Ui {
    pub async fn message_box(
        app_handle: AppHandle,
        text: impl Into<String>,
        options: Option<MessageBoxOptions>,
    ) -> Result<MessageBoxResult> {
        let options = options.unwrap_or_default();
        let mut dialog = app_handle.dialog().message(text);
        let local_buttons = options.buttons.clone().map(|buttons| buttons.into_inner());

        if let Some(title) = options.title {
            dialog = dialog.title(title);
        }
        if let Some(buttons) = options.buttons {
            dialog = dialog.buttons(buttons.into_inner().into());
        }
        if let Some(icon) = options.icon {
            dialog = dialog.kind(icon.into());
        }

        let (sender, receiver) = oneshot::channel();

        dialog.show_with_result(|result| {
            let result = match result {
                MessageDialogResult::Yes => MessageBoxResult::Yes,
                MessageDialogResult::No => MessageBoxResult::No,
                MessageDialogResult::Ok => MessageBoxResult::Ok,
                MessageDialogResult::Cancel => MessageBoxResult::Cancel,
                MessageDialogResult::Custom(label) => {
                    local_buttons.map_or(MessageBoxResult::Ok, |buttons| match buttons {
                        MessageBoxButtons::Ok
                        | MessageBoxButtons::OkCancel
                        | MessageBoxButtons::YesNo
                        | MessageBoxButtons::YesNoCancel => MessageBoxResult::Ok,
                        MessageBoxButtons::OkCustom(_) => MessageBoxResult::Ok,
                        MessageBoxButtons::OkCancelCustom(ok_label, cancel_label) => {
                            if label == ok_label {
                                MessageBoxResult::Ok
                            } else if label == cancel_label {
                                MessageBoxResult::Cancel
                            } else {
                                MessageBoxResult::Ok
                            }
                        }
                        MessageBoxButtons::YesNoCancelCustom(yes_label, no_label, cancel_label) => {
                            if label == yes_label {
                                MessageBoxResult::Yes
                            } else if label == no_label {
                                MessageBoxResult::No
                            } else if label == cancel_label {
                                MessageBoxResult::Cancel
                            } else {
                                MessageBoxResult::Ok
                            }
                        }
                    })
                }
            };

            sender.send(result).unwrap();
        });

        let result = receiver.await.unwrap();

        Ok(result)
    }
}
