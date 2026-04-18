use std::fmt::Debug;

use color_eyre::{Result, eyre::eyre};
use derive_more::Constructor;
use macros::{FromJsObject, FromSerde, IntoSerde, options};
use rfd::{
    AsyncMessageDialog, MessageButtons as RfdMessageButtons,
    MessageDialogResult as RfdMessageDialogResult, MessageLevel as RfdMessageLevel,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::api::dialogs::js::JsMessageBoxButtons;

pub mod file_dialog;
pub mod js;
pub mod native_dialog;

#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
/// @category Dialogs
/// @expand
pub enum MessageBoxIcon {
    #[default]
    /// `MessageBoxIcon.Info`
    Info,
    /// `MessageBoxIcon.Warning`
    Warning,
    /// `MessageBoxIcon.Error`
    Error,
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

#[derive(
    Clone, Debug, Deserialize, Display, EnumIter, Eq, FromSerde, IntoSerde, PartialEq, Serialize,
)]
/// @category Dialogs
/// @expand
pub enum MessageBoxResult {
    /// `MessageBoxResult.Yes`
    Yes,
    /// `MessageBoxResult.No`
    No,
    /// `MessageBoxResult.Ok`
    Ok,
    /// `MessageBoxResult.Cancel`
    Cancel,
}

/// Message box options
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct MessageBoxOptions {
    /// Title displayed in the message box title bar.
    pub title: Option<String>,

    /// Buttons displayed in the message box.
    #[default(ts = "MessageBoxButtons.ok()")]
    pub buttons: Option<JsMessageBoxButtons>,

    /// Icon displayed in the message box.
    #[default(ts = "MessageBoxIcon.Info")]
    pub icon: Option<MessageBoxIcon>,
}

#[derive(Constructor, Debug)]
pub struct Dialogs {}

impl Dialogs {
    pub async fn message_box(
        text: impl Into<String>,
        options: Option<MessageBoxOptions>,
    ) -> Result<MessageBoxResult> {
        let options = options.unwrap_or_default();
        let buttons = options.buttons.unwrap_or_default().into_inner();
        let dialog_result = AsyncMessageDialog::new()
            .set_title(options.title.unwrap_or_default())
            .set_description(text.into())
            .set_level(message_box_icon_to_rfd_level(
                options.icon.unwrap_or_default(),
            ))
            .set_buttons(message_box_buttons_to_rfd_buttons(&buttons))
            .show()
            .await;

        message_box_result_from_rfd(&buttons, dialog_result)
    }
}

const fn message_box_icon_to_rfd_level(icon: MessageBoxIcon) -> RfdMessageLevel {
    match icon {
        MessageBoxIcon::Info => RfdMessageLevel::Info,
        MessageBoxIcon::Warning => RfdMessageLevel::Warning,
        MessageBoxIcon::Error => RfdMessageLevel::Error,
    }
}

fn message_box_buttons_to_rfd_buttons(buttons: &MessageBoxButtons) -> RfdMessageButtons {
    match buttons {
        MessageBoxButtons::Ok => RfdMessageButtons::Ok,
        MessageBoxButtons::OkCancel => RfdMessageButtons::OkCancel,
        MessageBoxButtons::YesNo => RfdMessageButtons::YesNo,
        MessageBoxButtons::YesNoCancel => RfdMessageButtons::YesNoCancel,
        MessageBoxButtons::OkCustom(ok_label) => RfdMessageButtons::OkCustom(ok_label.clone()),
        MessageBoxButtons::OkCancelCustom(ok_label, cancel_label) => {
            RfdMessageButtons::OkCancelCustom(ok_label.clone(), cancel_label.clone())
        }
        MessageBoxButtons::YesNoCancelCustom(yes_label, no_label, cancel_label) => {
            RfdMessageButtons::YesNoCancelCustom(
                yes_label.clone(),
                no_label.clone(),
                cancel_label.clone(),
            )
        }
    }
}

fn message_box_result_from_rfd(
    buttons: &MessageBoxButtons,
    dialog_result: RfdMessageDialogResult,
) -> Result<MessageBoxResult> {
    match dialog_result {
        RfdMessageDialogResult::Yes => Ok(MessageBoxResult::Yes),
        RfdMessageDialogResult::No => Ok(MessageBoxResult::No),
        RfdMessageDialogResult::Ok => Ok(MessageBoxResult::Ok),
        RfdMessageDialogResult::Cancel => Ok(MessageBoxResult::Cancel),
        RfdMessageDialogResult::Custom(selected_label) => {
            message_box_custom_result_from_rfd(buttons, selected_label)
        }
    }
}

fn message_box_custom_result_from_rfd(
    buttons: &MessageBoxButtons,
    selected_label: String,
) -> Result<MessageBoxResult> {
    match buttons {
        MessageBoxButtons::OkCustom(_) => Ok(MessageBoxResult::Ok),
        MessageBoxButtons::OkCancelCustom(ok_label, cancel_label) => {
            if selected_label == *ok_label {
                Ok(MessageBoxResult::Ok)
            } else if selected_label == *cancel_label {
                Ok(MessageBoxResult::Cancel)
            } else {
                Err(eyre!(
                    "unsupported message box button result: {selected_label}"
                ))
            }
        }
        MessageBoxButtons::YesNoCancelCustom(yes_label, no_label, cancel_label) => {
            if selected_label == *yes_label {
                Ok(MessageBoxResult::Yes)
            } else if selected_label == *no_label {
                Ok(MessageBoxResult::No)
            } else if selected_label == *cancel_label {
                Ok(MessageBoxResult::Cancel)
            } else {
                Err(eyre!(
                    "unsupported message box button result: {selected_label}"
                ))
            }
        }
        _ => Err(eyre!(
            "unexpected custom message box button result: {selected_label}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MessageBoxButtons, MessageBoxResult, message_box_buttons_to_rfd_buttons,
        message_box_result_from_rfd,
    };

    #[test]
    fn converts_custom_buttons_to_rfd_buttons() {
        assert!(matches!(
            message_box_buttons_to_rfd_buttons(&MessageBoxButtons::OkCancelCustom(
                "Save".to_owned(),
                "Discard".to_owned(),
            )),
            rfd::MessageButtons::OkCancelCustom(ok_label, cancel_label)
                if ok_label == "Save" && cancel_label == "Discard"
        ));
    }

    #[test]
    fn normalizes_custom_ok_cancel_results() {
        assert_eq!(
            message_box_result_from_rfd(
                &MessageBoxButtons::OkCancelCustom("Save".to_owned(), "Discard".to_owned()),
                rfd::MessageDialogResult::Custom("Save".to_owned()),
            )
            .unwrap(),
            MessageBoxResult::Ok
        );
        assert_eq!(
            message_box_result_from_rfd(
                &MessageBoxButtons::OkCancelCustom("Save".to_owned(), "Discard".to_owned()),
                rfd::MessageDialogResult::Custom("Discard".to_owned()),
            )
            .unwrap(),
            MessageBoxResult::Cancel
        );
    }

    #[test]
    fn normalizes_custom_yes_no_cancel_results() {
        assert_eq!(
            message_box_result_from_rfd(
                &MessageBoxButtons::YesNoCancelCustom(
                    "Proceed".to_owned(),
                    "Skip".to_owned(),
                    "Stop".to_owned(),
                ),
                rfd::MessageDialogResult::Custom("Proceed".to_owned()),
            )
            .unwrap(),
            MessageBoxResult::Yes
        );
        assert_eq!(
            message_box_result_from_rfd(
                &MessageBoxButtons::YesNoCancelCustom(
                    "Proceed".to_owned(),
                    "Skip".to_owned(),
                    "Stop".to_owned(),
                ),
                rfd::MessageDialogResult::Custom("Skip".to_owned()),
            )
            .unwrap(),
            MessageBoxResult::No
        );
        assert_eq!(
            message_box_result_from_rfd(
                &MessageBoxButtons::YesNoCancelCustom(
                    "Proceed".to_owned(),
                    "Skip".to_owned(),
                    "Stop".to_owned(),
                ),
                rfd::MessageDialogResult::Custom("Stop".to_owned()),
            )
            .unwrap(),
            MessageBoxResult::Cancel
        );
    }
}
