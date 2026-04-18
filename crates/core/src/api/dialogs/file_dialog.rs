use std::path::PathBuf;

use color_eyre::Result;
use itertools::Itertools;
use rfd::AsyncFileDialog;

use super::Dialogs;

#[derive(Clone, Debug, Default)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub directory: Option<String>,
    pub filters: Vec<FileFilter>,
}

fn build_dialog(options: &FileDialogOptions) -> AsyncFileDialog {
    let mut dialog = AsyncFileDialog::new();

    if let Some(title) = &options.title {
        dialog = dialog.set_title(title);
    }
    if let Some(directory) = &options.directory {
        dialog = dialog.set_directory(directory);
    }
    for filter in &options.filters {
        let extensions = filter.extensions.iter().map(String::as_str).collect_vec();
        dialog = dialog.add_filter(&filter.name, &extensions);
    }
    dialog
}

impl Dialogs {
    pub async fn pick_file(options: FileDialogOptions) -> Result<Option<PathBuf>> {
        let handle = build_dialog(&options).pick_file().await;
        Ok(handle.map(|h| h.path().to_path_buf()))
    }

    pub async fn pick_files(options: FileDialogOptions) -> Result<Vec<PathBuf>> {
        let handles = build_dialog(&options).pick_files().await;
        Ok(handles
            .unwrap_or_default()
            .into_iter()
            .map(|h| h.path().to_path_buf())
            .collect())
    }

    pub async fn pick_folder(options: FileDialogOptions) -> Result<Option<PathBuf>> {
        let handle = build_dialog(&options).pick_folder().await;
        Ok(handle.map(|h| h.path().to_path_buf()))
    }

    pub async fn pick_folders(options: FileDialogOptions) -> Result<Vec<PathBuf>> {
        let handles = build_dialog(&options).pick_folders().await;
        Ok(handles
            .unwrap_or_default()
            .into_iter()
            .map(|h| h.path().to_path_buf())
            .collect())
    }

    pub async fn save_file(options: FileDialogOptions) -> Result<Option<PathBuf>> {
        let handle = build_dialog(&options).save_file().await;
        Ok(handle.map(|h| h.path().to_path_buf()))
    }
}
