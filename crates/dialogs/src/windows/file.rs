//! File dialogs, shown with the common item dialogs (`IFileOpenDialog` and `IFileSaveDialog`).

use std::{
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    path::{self, PathBuf},
};

use windows::{
    Win32::{
        Foundation::ERROR_CANCELLED,
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FOS_ALLOWMULTISELECT, FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS,
            FileOpenDialog, FileSaveDialog, IFileDialog, IFileOpenDialog, IFileSaveDialog,
            IShellItem, SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
        },
    },
    core::{HRESULT, HSTRING, PCWSTR},
};

use crate::{FileDialogOptions, Result, options::OpenMode};

/// Initialises COM on the current thread for as long as it lives.
struct Com;

impl Com {
    fn initialize() -> Result<Self> {
        // SAFETY: the dialog's thread has not initialised COM yet.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) }.ok()?;
        Ok(Self)
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        // SAFETY: balances the successful `CoInitializeEx` in `initialize`.
        unsafe { CoUninitialize() };
    }
}

/// `*.png;*.jpg`, the form the dialogs expect.
fn filter_spec(extensions: &[String]) -> String {
    extensions
        .iter()
        .map(|extension| format!("*.{extension}"))
        .collect::<Vec<_>>()
        .join(";")
}

/// Sets the title, the initial folder and, unless picking folders, the filters.
fn configure(dialog: &IFileDialog, options: &FileDialogOptions, filters: bool) -> Result<()> {
    if !options.title.is_empty() {
        // SAFETY: the dialog copies the string.
        unsafe { dialog.SetTitle(&HSTRING::from(&options.title)) }?;
    }

    // A folder that does not exist is left out rather than failing the dialog.
    if let Some(directory) = &options.directory
        && let Ok(directory) = path::absolute(directory)
    {
        let directory = HSTRING::from(directory.as_path());
        // SAFETY: `directory` is a valid string.
        let folder = unsafe { SHCreateItemFromParsingName::<_, _, IShellItem>(&directory, None) };
        if let Ok(folder) = folder {
            // SAFETY: `folder` is a valid shell item.
            unsafe { dialog.SetFolder(&folder) }?;
        }
    }

    if filters && !options.filters.is_empty() {
        let strings: Vec<(HSTRING, HSTRING)> = options
            .filters
            .iter()
            .map(|filter| {
                (
                    HSTRING::from(&filter.name),
                    HSTRING::from(filter_spec(&filter.extensions)),
                )
            })
            .collect();
        let specs: Vec<COMDLG_FILTERSPEC> = strings
            .iter()
            .map(|(name, spec)| COMDLG_FILTERSPEC {
                pszName: PCWSTR(name.as_ptr()),
                pszSpec: PCWSTR(spec.as_ptr()),
            })
            .collect();
        // SAFETY: the dialog copies the filters.
        unsafe { dialog.SetFileTypes(&specs) }?;
    }

    Ok(())
}

/// Shows the dialog. Returns `false` if the user cancelled.
fn show(dialog: &IFileDialog) -> Result<bool> {
    // SAFETY: the dialog is configured, on the thread that created it.
    match unsafe { dialog.Show(None) } {
        Ok(()) => Ok(true),
        Err(error) if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn item_path(item: &IShellItem) -> Result<PathBuf> {
    // SAFETY: `item` is a valid shell item. The returned string is freed below.
    let name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }?;
    // SAFETY: `name` is a valid null-terminated string.
    let path = OsString::from_wide(unsafe { name.as_wide() });
    // SAFETY: `name` was allocated by the shell, and is not used after this.
    unsafe { CoTaskMemFree(Some(name.0.cast_const().cast())) };
    Ok(path.into())
}

/// Shows an open dialog on the current thread. It is closed by the thread's `WM_QUIT`.
pub fn open(options: &FileDialogOptions, mode: OpenMode) -> Result<Option<Vec<PathBuf>>> {
    let _com = Com::initialize()?;
    // SAFETY: COM is initialised on this thread.
    let dialog: IFileOpenDialog =
        unsafe { CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER) }?;

    configure(&dialog, options, !mode.directory)?;
    // SAFETY: the dialog is valid.
    let mut flags = unsafe { dialog.GetOptions() }? | FOS_FORCEFILESYSTEM;
    if mode.multiple {
        flags |= FOS_ALLOWMULTISELECT;
    }
    if mode.directory {
        flags |= FOS_PICKFOLDERS;
    }
    // SAFETY: the dialog is valid.
    unsafe { dialog.SetOptions(flags) }?;

    if !show(&dialog)? {
        return Ok(None);
    }

    // SAFETY: the dialog was accepted, so it has results.
    let items = unsafe { dialog.GetResults() }?;
    // SAFETY: `items` is valid.
    let count = unsafe { items.GetCount() }?;
    (0..count)
        // SAFETY: `index` is below the item count.
        .map(|index| item_path(&unsafe { items.GetItemAt(index) }?))
        .collect::<Result<Vec<_>>>()
        .map(Some)
}

/// Shows a save dialog on the current thread. It is closed by the thread's `WM_QUIT`.
pub fn save(options: &FileDialogOptions) -> Result<Option<PathBuf>> {
    let _com = Com::initialize()?;
    // SAFETY: COM is initialised on this thread.
    let dialog: IFileSaveDialog =
        unsafe { CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER) }?;

    configure(&dialog, options, true)?;
    if let Some(name) = &options.file_name {
        // SAFETY: the dialog copies the string.
        unsafe { dialog.SetFileName(&HSTRING::from(name)) }?;
    }
    // SAFETY: the dialog is valid.
    let flags = unsafe { dialog.GetOptions() }? | FOS_FORCEFILESYSTEM;
    // SAFETY: the dialog is valid.
    unsafe { dialog.SetOptions(flags) }?;

    if !show(&dialog)? {
        return Ok(None);
    }

    // SAFETY: the dialog was accepted, so it has a result.
    let item = unsafe { dialog.GetResult() }?;
    item_path(&item).map(Some)
}

#[cfg(test)]
mod tests {
    use super::filter_spec;

    #[test]
    fn filter_specs() {
        assert_eq!(
            filter_spec(&["png".to_owned(), "jpg".to_owned()]),
            "*.png;*.jpg"
        );
    }
}
