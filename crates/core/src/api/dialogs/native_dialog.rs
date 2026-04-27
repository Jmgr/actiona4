use color_eyre::Result;
use macros::{FromSerde, IntoSerde};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tokio_util::task::TaskTracker;

use super::Dialogs;
use crate::api::color::Color;

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

impl From<TextInputMode> for rustydialogs::TextInputMode {
    fn from(mode: TextInputMode) -> Self {
        match mode {
            TextInputMode::SingleLine => Self::SingleLine,
            TextInputMode::MultiLine => Self::MultiLine,
            TextInputMode::Password => Self::Password,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextInputOptions {
    pub title: String,
    pub message: String,
    pub value: String,
    pub mode: TextInputMode,
}

#[derive(Clone, Debug)]
pub struct ColorPickerOptions {
    pub title: String,
    pub value: Color,
}

fn color_to_dialog_color(color: Color) -> rustydialogs::ColorValue {
    rustydialogs::ColorValue {
        red: color[0],
        green: color[1],
        blue: color[2],
    }
}

const fn color_from_dialog_color(color: rustydialogs::ColorValue) -> Color {
    Color::new(color.red, color.green, color.blue, 255)
}

impl Dialogs {
    pub async fn text_input(
        options: TextInputOptions,
        task_tracker: TaskTracker,
    ) -> Result<Option<String>> {
        let result = task_tracker
            .spawn_blocking(move || {
                rustydialogs::TextInput {
                    title: &options.title,
                    message: &options.message,
                    value: &options.value,
                    mode: options.mode.into(),
                    owner: None,
                }
                .show()
            })
            .await?;
        Ok(result)
    }

    pub async fn color_picker(
        options: ColorPickerOptions,
        task_tracker: TaskTracker,
    ) -> Result<Option<Color>> {
        let result = task_tracker
            .spawn_blocking(move || {
                rustydialogs::ColorPicker {
                    title: &options.title,
                    value: color_to_dialog_color(options.value),
                    owner: None,
                }
                .show()
            })
            .await?;
        Ok(result.map(color_from_dialog_color))
    }
}
