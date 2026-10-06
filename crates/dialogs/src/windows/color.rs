//! Colour picker, shown with `ChooseColorW`.

use std::ptr;

use types::Color;
use windows::{
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, WPARAM},
        UI::{
            Controls::Dialogs::{
                CC_ENABLEHOOK, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW, COMMON_DLG_ERRORS,
                ChooseColorW, CommDlgExtendedError,
            },
            WindowsAndMessaging::{SetWindowTextW, WM_INITDIALOG},
        },
    },
    core::HSTRING,
};

use super::struct_size;
use crate::{ColorPickerOptions, Error, Result};

/// `COLORREF` is `0x00BBGGRR`.
fn to_colorref(color: Color) -> COLORREF {
    COLORREF(u32::from_le_bytes([color[0], color[1], color[2], 0]))
}

const fn from_colorref(color: COLORREF) -> Color {
    let [red, green, blue, _] = color.0.to_le_bytes();
    Color::new(red, green, blue, 255)
}

/// Shows the colour picker on the current thread. It is closed by the thread's `WM_QUIT`.
pub fn color_picker(options: &ColorPickerOptions) -> Result<Option<Color>> {
    let title = HSTRING::from(&options.title);
    let mut custom_colors = [COLORREF(0x00FF_FFFF); 16];

    let mut choose = CHOOSECOLORW {
        lStructSize: struct_size::<CHOOSECOLORW>(),
        rgbResult: to_colorref(options.value),
        lpCustColors: custom_colors.as_mut_ptr(),
        Flags: CC_RGBINIT | CC_FULLOPEN | CC_ENABLEHOOK,
        lCustData: LPARAM(ptr::from_ref(&title) as isize),
        lpfnHook: Some(set_title),
        ..CHOOSECOLORW::default()
    };

    // SAFETY: `choose` and everything it points to outlive the call.
    if unsafe { ChooseColorW(&raw mut choose) }.as_bool() {
        return Ok(Some(from_colorref(choose.rgbResult)));
    }

    // SAFETY: no preconditions.
    match unsafe { CommDlgExtendedError() } {
        COMMON_DLG_ERRORS(0) => Ok(None),
        error => Err(Error::Backend(format!(
            "the colour picker failed with error {:#x}",
            error.0
        ))),
    }
}

/// The colour picker has no title option: the hook sets it as the dialog opens. An empty title
/// keeps the default one.
unsafe extern "system" fn set_title(
    window: HWND,
    message: u32,
    _: WPARAM,
    lparam: LPARAM,
) -> usize {
    if message == WM_INITDIALOG {
        // SAFETY: on `WM_INITDIALOG`, `lparam` is the `CHOOSECOLORW` passed to `ChooseColorW`,
        // whose `lCustData` is the title, and both outlive the dialog.
        let title = unsafe {
            let choose = &*(lparam.0 as *const CHOOSECOLORW);
            &*(choose.lCustData.0 as *const HSTRING)
        };
        if !title.is_empty() {
            // SAFETY: `window` is the dialog being initialised.
            _ = unsafe { SetWindowTextW(window, title) };
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use types::Color;
    use windows::Win32::Foundation::COLORREF;

    use super::{from_colorref, to_colorref};

    #[test]
    fn converts_colorrefs() {
        assert_eq!(to_colorref(Color::new(1, 2, 3, 4)), COLORREF(0x0003_0201));
        assert_eq!(
            from_colorref(COLORREF(0x0003_0201)),
            Color::new(1, 2, 3, 255)
        );
    }
}
