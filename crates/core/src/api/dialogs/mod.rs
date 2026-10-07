use std::time::Duration;

use color_eyre::Result;
use macros::{FromSerde, IntoSerde};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::cancel_on;

pub mod js;

#[derive(
    Clone,
    Copy,
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

impl From<MessageBoxIcon> for ::dialogs::MessageBoxIcon {
    fn from(icon: MessageBoxIcon) -> Self {
        match icon {
            MessageBoxIcon::Info => Self::Info,
            MessageBoxIcon::Warning => Self::Warning,
            MessageBoxIcon::Error => Self::Error,
        }
    }
}

#[derive(
    Clone,
    Copy,
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
pub enum MessageBoxButtons {
    #[default]
    /// `MessageBoxButtons.Ok`
    Ok,
    /// `MessageBoxButtons.OkCancel`
    OkCancel,
    /// `MessageBoxButtons.YesNo`
    YesNo,
    /// `MessageBoxButtons.YesNoCancel`
    YesNoCancel,
}

impl From<MessageBoxButtons> for ::dialogs::MessageBoxButtons {
    fn from(buttons: MessageBoxButtons) -> Self {
        match buttons {
            MessageBoxButtons::Ok => Self::Ok,
            MessageBoxButtons::OkCancel => Self::OkCancel,
            MessageBoxButtons::YesNo => Self::YesNo,
            MessageBoxButtons::YesNoCancel => Self::YesNoCancel,
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
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
pub enum MessageBoxResult {
    /// `MessageBoxResult.Yes`
    Yes,
    /// `MessageBoxResult.No`
    No,
    /// `MessageBoxResult.Ok`
    Ok,
    /// `MessageBoxResult.Cancel`
    Cancel,
    /// `MessageBoxResult.Timeout`: the `timeout` option elapsed before the user pressed a button.
    Timeout,
}

impl From<::dialogs::MessageBoxResult> for MessageBoxResult {
    fn from(result: ::dialogs::MessageBoxResult) -> Self {
        match result {
            ::dialogs::MessageBoxResult::Ok => Self::Ok,
            ::dialogs::MessageBoxResult::Cancel => Self::Cancel,
            ::dialogs::MessageBoxResult::Yes => Self::Yes,
            ::dialogs::MessageBoxResult::No => Self::No,
        }
    }
}

#[derive(
    Clone,
    Copy,
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
pub enum TextInputMode {
    #[default]
    /// `TextInputMode.SingleLine`
    SingleLine,
    /// `TextInputMode.MultiLine`
    MultiLine,
    /// `TextInputMode.Password`
    Password,
}

impl From<TextInputMode> for ::dialogs::TextInputMode {
    fn from(mode: TextInputMode) -> Self {
        match mode {
            TextInputMode::SingleLine => Self::SingleLine,
            TextInputMode::MultiLine => Self::MultiLine,
            TextInputMode::Password => Self::Password,
        }
    }
}

/// Shows a dialog until the user answers it, `token` is cancelled, or `duration` elapses.
///
/// Returns `None` if `duration` elapsed. Either way, the dialog is closed by dropping its future.
pub(crate) async fn show<T>(
    token: &CancellationToken,
    duration: Option<Duration>,
    dialog: impl Future<Output = ::dialogs::Result<T>>,
) -> Result<Option<T>> {
    let result = match duration {
        Some(duration) => match cancel_on(token, timeout(duration, dialog)).await? {
            Ok(result) => result,
            Err(_elapsed) => return Ok(None),
        },
        None => cancel_on(token, dialog).await?,
    };
    Ok(Some(result?))
}

#[cfg(test)]
mod tests {
    use std::{future::pending, time::Duration};

    use tokio_util::sync::CancellationToken;

    use super::show;
    use crate::error::CommonError;

    #[tokio::test]
    async fn show_returns_the_answer() {
        let token = CancellationToken::new();
        let result = show(&token, Some(Duration::from_secs(60)), async {
            Ok::<_, dialogs::Error>(42)
        })
        .await
        .unwrap();
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn show_times_out() {
        let token = CancellationToken::new();
        let result = show(
            &token,
            Some(Duration::from_millis(10)),
            pending::<dialogs::Result<()>>(),
        )
        .await
        .unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn show_is_cancelled() {
        let token = CancellationToken::new();
        token.cancel();
        let error = show(&token, None, pending::<dialogs::Result<()>>())
            .await
            .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<CommonError>(),
            Some(CommonError::Cancelled)
        ));
    }
}
