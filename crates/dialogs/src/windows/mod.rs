//! Windows backends: the system's own dialogs, or windows of our own built from standard controls
//! where Windows has none, each shown on a thread of its own.

use std::{mem, path::PathBuf};

use jiff::civil::Date;
use types::Color;

use crate::{
    ColorPickerOptions, DateOptions, FileDialogOptions, MessageBoxOptions, MessageBoxResult,
    Progress, ProgressOptions, Result, SelectOptions, TextInputOptions, options::OpenMode,
};

mod color;
mod date;
mod file;
mod input;
mod select;
mod task_dialog;
mod thread;
mod window;

/// The size of a Windows API structure, for its size field.
#[allow(clippy::cast_possible_truncation)]
const fn struct_size<T>() -> u32 {
    mem::size_of::<T>() as u32
}

#[derive(Clone, Copy, Debug)]
pub struct Backends;

#[allow(clippy::unused_self)]
impl Backends {
    pub const fn detect() -> Self {
        Self
    }

    pub async fn message_box(&self, options: &MessageBoxOptions) -> Result<MessageBoxResult> {
        let options = options.clone();
        thread::run(move |thread| task_dialog::message_box(&options, thread)).await
    }

    pub async fn text_input(&self, options: &TextInputOptions) -> Result<Option<String>> {
        let options = options.clone();
        thread::run(move |_| input::text_input(&options)).await
    }

    pub async fn color_picker(&self, options: &ColorPickerOptions) -> Result<Option<Color>> {
        let options = options.clone();
        thread::run(move |_| color::color_picker(&options)).await
    }

    pub async fn open(
        &self,
        options: &FileDialogOptions,
        mode: OpenMode,
    ) -> Result<Option<Vec<PathBuf>>> {
        let options = options.clone();
        thread::run(move |_| file::open(&options, mode)).await
    }

    pub async fn save(&self, options: &FileDialogOptions) -> Result<Option<PathBuf>> {
        let options = options.clone();
        thread::run(move |_| file::save(&options)).await
    }

    pub async fn select(
        &self,
        options: &SelectOptions,
        multiple: bool,
    ) -> Result<Option<Vec<usize>>> {
        let options = options.clone();
        thread::run(move |_| select::select(&options, multiple)).await
    }

    pub async fn date(&self, options: &DateOptions) -> Result<Option<Date>> {
        let options = options.clone();
        thread::run(move |_| date::date(&options)).await
    }

    pub async fn progress(&self, options: &ProgressOptions) -> Result<Progress> {
        task_dialog::progress(options).await
    }
}
