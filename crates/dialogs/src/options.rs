//! Dialog options and results.

use std::path::PathBuf;

use jiff::civil::Date;
use strum::EnumIs;
use types::Color;

/// Icon shown in a message box.
#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, PartialEq)]
pub enum MessageBoxIcon {
    #[default]
    Info,
    Warning,
    Error,
}

/// Set of buttons shown in a message box.
#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, PartialEq)]
pub enum MessageBoxButtons {
    #[default]
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

impl MessageBoxButtons {
    #[must_use]
    pub const fn has_cancel(self) -> bool {
        matches!(self, Self::OkCancel | Self::YesNoCancel)
    }

    #[must_use]
    pub const fn has_no(self) -> bool {
        matches!(self, Self::YesNo | Self::YesNoCancel)
    }

    /// The result reported when the user closes the dialog without pressing a button: Cancel if
    /// that button is shown, otherwise No, otherwise Ok.
    #[must_use]
    pub const fn dismissed_result(self) -> MessageBoxResult {
        if self.has_cancel() {
            MessageBoxResult::Cancel
        } else if self.has_no() {
            MessageBoxResult::No
        } else {
            MessageBoxResult::Ok
        }
    }

    /// The result of the affirmative button (OK or Yes).
    #[must_use]
    pub const fn accepted_result(self) -> MessageBoxResult {
        match self {
            Self::Ok | Self::OkCancel => MessageBoxResult::Ok,
            Self::YesNo | Self::YesNoCancel => MessageBoxResult::Yes,
        }
    }
}

/// Overrides the label of a button, if that button is shown.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ButtonLabels {
    pub ok: Option<String>,
    pub cancel: Option<String>,
    pub yes: Option<String>,
    pub no: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MessageBoxOptions {
    pub title: String,
    pub text: String,
    pub icon: MessageBoxIcon,
    pub buttons: MessageBoxButtons,
    pub labels: ButtonLabels,
}

/// The button the user pressed. Closing the dialog without pressing a button is reported as
/// [`MessageBoxButtons::dismissed_result`].
#[derive(Clone, Copy, Debug, EnumIs, Eq, PartialEq)]
pub enum MessageBoxResult {
    Ok,
    Cancel,
    Yes,
    No,
}

#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, PartialEq)]
pub enum TextInputMode {
    #[default]
    SingleLine,
    MultiLine,
    Password,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextInputOptions {
    pub title: String,
    /// Prompt shown above the input. Not shown by zenity in multi-line mode.
    pub text: String,
    /// Initial value. Ignored by kdialog in password mode.
    pub value: String,
    pub mode: TextInputMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColorPickerOptions {
    pub title: String,
    /// Initially selected colour. Its alpha channel is ignored.
    pub value: Color,
}

impl Default for ColorPickerOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            value: Color::new(0, 0, 0, 255),
        }
    }
}

/// A named set of file extensions, such as `Images` with `png` and `jpg`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileFilter {
    pub name: String,
    /// Extensions without the leading dot.
    pub extensions: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileDialogOptions {
    pub title: String,
    /// Folder the dialog starts in.
    pub directory: Option<PathBuf>,
    /// Initial file name. Only used when saving.
    ///
    /// zenity versions from 4.0 to before 4.1.99 ignore it unless the file already exists, though
    /// they still open its folder. The portal and kdialog always show it.
    pub file_name: Option<String>,
    /// Restricts the files that can be picked. Ignored when picking folders.
    pub filters: Vec<FileFilter>,
}

/// What an open dialog selects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(windows, allow(dead_code))]
pub struct OpenMode {
    pub multiple: bool,
    pub directory: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectOptions {
    pub title: String,
    pub text: String,
    pub items: Vec<String>,
    /// Indices of the initially selected items. When selecting a single item, only the first
    /// counts, and the first item is selected if this is empty.
    pub selected: Vec<usize>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DateOptions {
    pub title: String,
    pub text: String,
    /// Initially selected date. Today if `None`.
    pub value: Option<Date>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProgressOptions {
    pub title: String,
    pub text: String,
    /// Whether the dialog has a Cancel button.
    pub cancellable: bool,
    /// Initial progress, between 0 and 1, or `None` to show activity instead.
    pub value: Option<f64>,
}
