//! Text input: an edit control in a dialog window of our own.

use windows::{
    Win32::{
        Foundation::{HWND, SIZE},
        UI::{
            Controls::{EM_SETLIMITTEXT, EM_SETSEL},
            WindowsAndMessaging::{
                ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_MULTILINE, ES_PASSWORD, ES_WANTRETURN,
                GetWindowTextLengthW, GetWindowTextW, SetWindowTextW, WS_EX_CLIENTEDGE, WS_VSCROLL,
            },
        },
    },
    core::{HSTRING, w},
};

use super::window::{self, CONTENT_WIDTH, Control, create_control, scale, send, text_height};
use crate::{Result, TextInputMode, TextInputOptions};

/// Lines shown by a multi-line input, before it is resized.
const MULTI_LINE_LINES: i32 = 6;

/// Edit controls separate lines with `\r\n`.
fn to_crlf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

fn window_text(window: HWND) -> String {
    // SAFETY: `window` is valid.
    let length = unsafe { GetWindowTextLengthW(window) };
    let mut text = vec![0; usize::try_from(length).unwrap_or_default() + 1];
    // SAFETY: `text` is writable, and its length is given.
    let length = unsafe { GetWindowTextW(window, &mut text) };
    text.truncate(usize::try_from(length).unwrap_or_default());
    String::from_utf16_lossy(&text)
}

struct TextInput<'a>(&'a TextInputOptions);

impl Control for TextInput<'_> {
    fn create(&self, parent: HWND) -> Result<HWND> {
        let style = match self.0.mode {
            TextInputMode::SingleLine => ES_AUTOHSCROLL,
            TextInputMode::Password => ES_AUTOHSCROLL | ES_PASSWORD,
            TextInputMode::MultiLine => {
                ES_MULTILINE | ES_WANTRETURN | ES_AUTOVSCROLL | WS_VSCROLL.0.cast_signed()
            }
        };
        let control = create_control(parent, w!("EDIT"), style.cast_unsigned(), WS_EX_CLIENTEDGE)?;

        // Lifts the default limit of 32767 characters.
        send(control, EM_SETLIMITTEXT, 0, 0);
        let value = if self.0.mode.is_multi_line() {
            to_crlf(&self.0.value)
        } else {
            self.0.value.clone()
        };
        // SAFETY: `control` was just created.
        unsafe { SetWindowTextW(control, &HSTRING::from(value)) }?;
        send(control, EM_SETSEL, 0, -1);
        Ok(control)
    }

    fn size(&self, control: HWND, dpi: u32) -> SIZE {
        let lines = if self.stretches() {
            MULTI_LINE_LINES
        } else {
            1
        };
        SIZE {
            cx: scale(CONTENT_WIDTH, dpi),
            cy: text_height(control) * lines + scale(8, dpi),
        }
    }

    fn stretches(&self) -> bool {
        self.0.mode.is_multi_line()
    }
}

/// Shows a text input on the current thread. It is closed by the thread's `WM_QUIT`.
pub fn text_input(options: &TextInputOptions) -> Result<Option<String>> {
    let multi_line = options.mode.is_multi_line();
    window::show(
        &options.title,
        &options.text,
        &TextInput(options),
        |control| {
            let text = window_text(control);
            if multi_line {
                text.replace("\r\n", "\n")
            } else {
                text
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::to_crlf;

    #[test]
    fn converts_line_endings() {
        assert_eq!(to_crlf("a\nb\r\nc"), "a\r\nb\r\nc");
        assert_eq!(to_crlf("single"), "single");
    }
}
