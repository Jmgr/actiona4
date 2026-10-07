//! Native dialogs that close when their future is dropped.
//!
//! Every dialog is an `async fn`, and dropping its future closes the dialog.
//!
//! On Linux, dialogs are shown by `zenity` or `kdialog`, whichever is installed, except file
//! dialogs, which use the xdg-desktop-portal file chooser when it is available. The
//! `ACTIONA_DIALOGS_BACKEND` environment variable forces a backend: `zenity` or `kdialog` for
//! every dialog, or `portal` for file dialogs only.
//!
//! On Windows, dialogs are the system's own: task dialogs, the colour picker and the common
//! item dialogs. Text input, selection and date dialogs, which Windows does not provide, are
//! windows of our own built from standard controls.

use std::{io, path::PathBuf, result::Result as StdResult};

#[cfg(unix)]
mod linux;
mod options;
mod progress;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
use ::windows::core::Error as WindowsError;
use jiff::civil::Date;
#[cfg(unix)]
use linux::Linux as Backends;
#[cfg(unix)]
pub use linux::{KDialog, LinuxBackends, LinuxTool, Zenity};
use options::OpenMode;
pub use options::{
    ButtonLabels, ColorPickerOptions, DateOptions, FileDialogOptions, FileFilter,
    MessageBoxButtons, MessageBoxIcon, MessageBoxOptions, MessageBoxResult, ProgressOptions,
    SelectOptions, TextInputMode, TextInputOptions,
};
pub use progress::Progress;
use types::Color;
#[cfg(windows)]
use windows::Backends;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Neither zenity nor kdialog is installed.
    #[error("no dialog backend available: install zenity or kdialog")]
    NoBackend,

    /// The selected backend cannot show this dialog.
    #[error("{dialog} is not supported by {backend}")]
    Unsupported {
        dialog: &'static str,
        backend: &'static str,
    },

    /// The options cannot be shown, such as a selection without items.
    #[error("invalid dialog options: {0}")]
    InvalidOptions(&'static str),

    /// The backend failed or produced output that could not be understood.
    #[error("dialog backend failed: {0}")]
    Backend(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    /// Communication with the xdg-desktop-portal file chooser failed.
    #[cfg(unix)]
    #[error(transparent)]
    DBus(#[from] zbus::Error),

    /// A Windows API call failed.
    #[cfg(windows)]
    #[error(transparent)]
    Windows(#[from] WindowsError),
}

pub type Result<T> = StdResult<T, Error>;

/// Entry point for showing dialogs. Cheap to clone.
#[derive(Clone, Debug)]
pub struct Dialogs {
    backends: Backends,
}

impl Default for Dialogs {
    fn default() -> Self {
        Self::new()
    }
}

impl Dialogs {
    /// Detects the available backends. Never fails: if no backend is available, the error is
    /// reported by the first dialog that needs one.
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            backends: Backends::detect(),
        }
    }

    /// Uses the given backends instead of detecting them.
    #[cfg(unix)]
    #[must_use]
    pub fn with_backends(backends: LinuxBackends) -> Self {
        Self {
            backends: Backends::new(backends),
        }
    }

    /// Shows a message box and returns the button the user pressed.
    pub async fn message_box(&self, options: MessageBoxOptions) -> Result<MessageBoxResult> {
        self.backends.message_box(&options).await
    }

    /// Asks the user for some text. Returns `None` if the user cancelled.
    pub async fn text_input(&self, options: TextInputOptions) -> Result<Option<String>> {
        self.backends.text_input(&options).await
    }

    /// Asks the user to pick a colour. Returns `None` if the user cancelled. The returned colour
    /// is always opaque.
    pub async fn color_picker(&self, options: ColorPickerOptions) -> Result<Option<Color>> {
        self.backends.color_picker(&options).await
    }

    /// Asks the user to pick a file. Returns `None` if the user cancelled.
    pub async fn pick_file(&self, options: FileDialogOptions) -> Result<Option<PathBuf>> {
        let paths = self
            .backends
            .open(
                &options,
                OpenMode {
                    multiple: false,
                    directory: false,
                },
            )
            .await?;
        Ok(paths.and_then(|paths| paths.into_iter().next()))
    }

    /// Asks the user to pick one or more files. Returns `None` if the user cancelled.
    ///
    /// zenity and kdialog print one path per line, so with them, a path containing a line break
    /// comes back as several paths.
    pub async fn pick_files(&self, options: FileDialogOptions) -> Result<Option<Vec<PathBuf>>> {
        self.backends
            .open(
                &options,
                OpenMode {
                    multiple: true,
                    directory: false,
                },
            )
            .await
    }

    /// Asks the user to pick a folder. Returns `None` if the user cancelled.
    pub async fn pick_folder(&self, options: FileDialogOptions) -> Result<Option<PathBuf>> {
        let paths = self
            .backends
            .open(
                &options,
                OpenMode {
                    multiple: false,
                    directory: true,
                },
            )
            .await?;
        Ok(paths.and_then(|paths| paths.into_iter().next()))
    }

    /// Asks the user to pick one or more folders. Returns `None` if the user cancelled.
    ///
    /// Not supported by kdialog: without the portal, this fails with [`Error::Unsupported`].
    ///
    /// zenity prints one path per line, so with it, a path containing a line break comes back as
    /// several paths.
    pub async fn pick_folders(&self, options: FileDialogOptions) -> Result<Option<Vec<PathBuf>>> {
        self.backends
            .open(
                &options,
                OpenMode {
                    multiple: true,
                    directory: true,
                },
            )
            .await
    }

    /// Asks the user where to save a file. Returns `None` if the user cancelled.
    pub async fn save_file(&self, options: FileDialogOptions) -> Result<Option<PathBuf>> {
        self.backends.save(&options).await
    }

    /// Asks the user to pick one of `options.items`, and returns its index. Returns `None` if the
    /// user cancelled.
    pub async fn select_one(&self, options: SelectOptions) -> Result<Option<usize>> {
        Self::check_items(&options)?;
        let indices = self.backends.select(&options, false).await?;
        Ok(indices.and_then(|indices| indices.into_iter().next()))
    }

    /// Asks the user to pick any number of `options.items`, and returns their indices. Returns
    /// `None` if the user cancelled.
    pub async fn select_many(&self, options: SelectOptions) -> Result<Option<Vec<usize>>> {
        Self::check_items(&options)?;
        self.backends.select(&options, true).await
    }

    const fn check_items(options: &SelectOptions) -> Result<()> {
        if options.items.is_empty() {
            return Err(Error::InvalidOptions("there are no items to select from"));
        }
        Ok(())
    }

    /// Asks the user to pick a date. Returns `None` if the user cancelled.
    pub async fn date(&self, options: DateOptions) -> Result<Option<Date>> {
        self.backends.date(&options).await
    }

    /// Shows a progress dialog, which stays open until the returned handle is closed or
    /// dropped.
    pub async fn progress(&self, options: ProgressOptions) -> Result<Progress> {
        self.backends.progress(&options).await
    }
}
