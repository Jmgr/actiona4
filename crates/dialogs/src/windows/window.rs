//! A dialog window of our own, for the dialogs Windows does not provide: a prompt, one control,
//! and OK and Cancel buttons, built from standard controls so that it looks native.
//!
//! The window runs its own message loop, which ends on `WM_QUIT`, so [`super::thread`] closes it
//! like the system's dialogs. `IsDialogMessageW` gives it a dialog's keyboard handling: Tab moves
//! between controls, Enter accepts and Escape cancels. Closing the window cancels the dialog.

use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    ptr,
    sync::Once,
};

use tracing::debug;
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
        Graphics::Gdi::{
            COLOR_BTNFACE, CreateFontIndirectW, DT_CALCRECT, DT_EXPANDTABS, DT_NOPREFIX,
            DT_WORDBREAK, DeleteObject, DrawTextW, GetDC, GetMonitorInfoW, GetSysColorBrush,
            GetTextMetricsW, HDC, HFONT, HGDIOBJ, MONITOR_DEFAULTTOPRIMARY, MONITORINFO,
            MonitorFromPoint, ReleaseDC, SelectObject, TEXTMETRICW,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            SystemServices::{SS_LEFT, SS_NOPREFIX},
        },
        UI::{
            Controls::{
                ICC_DATE_CLASSES, ICC_LISTVIEW_CLASSES, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX,
                InitCommonControlsEx, NMHDR,
            },
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow, SystemParametersInfoForDpi},
            Input::KeyboardAndMouse::{EnableWindow, GetFocus, IsWindowEnabled, SetFocus},
            WindowsAndMessaging::{
                BS_DEFPUSHBUTTON, BS_PUSHBUTTON, CREATESTRUCTW, CreateWindowExW, DC_HASDEFID,
                DM_GETDEFID, DefWindowProcW, DestroyWindow, DispatchMessageW, GWLP_USERDATA,
                GetClientRect, GetCursorPos, GetMessageW, GetWindowLongPtrW, HMENU, IDC_ARROW,
                IDCANCEL, IDOK, IsDialogMessageW, LoadCursorW, MINMAXINFO, MSG, MoveWindow,
                NONCLIENTMETRICSW, RegisterClassExW, SPI_GETNONCLIENTMETRICS, SW_SHOW,
                SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetForegroundWindow, SetWindowLongPtrW,
                SetWindowPos, ShowWindow, TranslateMessage, WA_INACTIVE, WINDOW_EX_STYLE,
                WINDOW_STYLE, WM_ACTIVATE, WM_CLOSE, WM_COMMAND, WM_DPICHANGED, WM_GETFONT,
                WM_GETMINMAXINFO, WM_NCCREATE, WM_NOTIFY, WM_SETFONT, WM_SIZE, WNDCLASSEXW,
                WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN, WS_EX_CONTROLPARENT, WS_EX_DLGMODALFRAME,
                WS_MAXIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU, WS_TABSTOP, WS_THICKFRAME, WS_VISIBLE,
            },
        },
    },
    core::{Error as WindowsError, HSTRING, PCWSTR, w},
};

use super::struct_size;
use crate::Result;

/// Sizes in pixels at 96 DPI, from the Windows layout guidelines.
const MARGIN: i32 = 11;
const SPACING: i32 = 7;
const BUTTON_WIDTH: i32 = 75;
const BUTTON_HEIGHT: i32 = 23;
/// Width of the dialog's content, unless its control needs more.
pub const CONTENT_WIDTH: i32 = 320;

const CLASS_NAME: PCWSTR = w!("ActionaDialog");
const CONTROL_ID: i32 = 100;
const LABEL_ID: i32 = 101;

/// The reply to `DM_GETDEFID`: OK is the default button, in the low word.
#[allow(clippy::cast_possible_wrap)]
const DEFAULT_BUTTON: isize = ((DC_HASDEFID << 16) | IDOK.0.cast_unsigned()) as isize;

#[allow(clippy::cast_possible_truncation)]
const fn low_word(wparam: WPARAM) -> u16 {
    wparam.0 as u16
}

/// Scales `value`, in pixels at 96 DPI, to `dpi`.
pub fn scale(value: i32, dpi: u32) -> i32 {
    let scaled = i64::from(value) * i64::from(dpi);
    i32::try_from((scaled + 48) / 96).unwrap_or(i32::MAX)
}

/// The control hosted by a dialog window.
pub trait Control {
    /// Creates the control, with [`create_control`].
    fn create(&self, parent: HWND) -> Result<HWND>;

    /// The control's size, in pixels at `dpi`, once its font is set. A control that stretches is
    /// given this size initially and grows with the window; the window is not resizable otherwise.
    fn size(&self, control: HWND, dpi: u32) -> SIZE;

    fn stretches(&self) -> bool {
        false
    }

    /// Called after the control is moved or resized.
    fn resized(&self, _control: HWND) {}

    /// Whether OK is enabled. Checked again after each notification from the control.
    fn can_accept(&self, _control: HWND) -> bool {
        true
    }

    /// Whether a notification from the control accepts the dialog, such as double-clicking an
    /// item.
    fn accepts(&self, _notification: &NMHDR) -> bool {
        false
    }
}

/// Sends `message` to `window`, which must not keep pointers passed in `lparam` after returning.
pub fn send(window: HWND, message: u32, wparam: usize, lparam: isize) -> isize {
    // SAFETY: pointers passed in `lparam` outlive this synchronous call.
    unsafe { SendMessageW(window, message, Some(WPARAM(wparam)), Some(LPARAM(lparam))) }.0
}

fn instance() -> Result<HINSTANCE> {
    // SAFETY: gets the executable's handle, which stays valid.
    Ok(unsafe { GetModuleHandleW(PCWSTR::null()) }?.into())
}

fn create_child(
    parent: HWND,
    class: PCWSTR,
    text: &HSTRING,
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
    id: i32,
) -> Result<HWND> {
    let id = HMENU(ptr::without_provenance_mut(
        usize::try_from(id).unwrap_or_default(),
    ));
    // SAFETY: `parent` is a window of this thread, and `class` and `text` outlive the call.
    Ok(unsafe {
        CreateWindowExW(
            ex_style,
            class,
            text,
            WS_CHILD | WS_VISIBLE | style,
            0,
            0,
            0,
            0,
            Some(parent),
            Some(id),
            Some(instance()?),
            None,
        )
    }?)
}

/// Creates a dialog window's control: a child of `parent` of the window class `class`, which can
/// be reached with Tab.
pub fn create_control(
    parent: HWND,
    class: PCWSTR,
    style: u32,
    ex_style: WINDOW_EX_STYLE,
) -> Result<HWND> {
    create_child(
        parent,
        class,
        &HSTRING::new(),
        WS_TABSTOP | WINDOW_STYLE(style),
        ex_style,
        CONTROL_ID,
    )
}

/// Runs `f` with a device context for `window`, with the window's font selected.
fn with_font<T>(window: HWND, f: impl FnOnce(HDC) -> T) -> T {
    let font = HGDIOBJ(send(window, WM_GETFONT, 0, 0) as *mut c_void);
    // SAFETY: the device context is released, and the font it had restored, before returning.
    unsafe {
        let dc = GetDC(Some(window));
        let previous = SelectObject(dc, font);
        let result = f(dc);
        SelectObject(dc, previous);
        ReleaseDC(Some(window), dc);
        result
    }
}

/// The height of a line of text in `window`'s font, in pixels.
pub fn text_height(window: HWND) -> i32 {
    with_font(window, |dc| {
        let mut metrics = TEXTMETRICW::default();
        // SAFETY: `dc` is valid, and `metrics` is writable.
        _ = unsafe { GetTextMetricsW(dc, &raw mut metrics) };
        metrics.tmHeight
    })
}

/// The system's font for dialogs, at a given DPI.
struct Font(HFONT);

impl Font {
    fn message(dpi: u32) -> Result<Self> {
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: struct_size::<NONCLIENTMETRICSW>(),
            ..NONCLIENTMETRICSW::default()
        };
        // SAFETY: `metrics` is writable, and its size is given.
        unsafe {
            SystemParametersInfoForDpi(
                SPI_GETNONCLIENTMETRICS.0,
                metrics.cbSize,
                Some(ptr::from_mut(&mut metrics).cast()),
                0,
                dpi,
            )
        }?;

        // SAFETY: `lfMessageFont` is a valid font description.
        let font = unsafe { CreateFontIndirectW(&raw const metrics.lfMessageFont) };
        if font.is_invalid() {
            return Err(WindowsError::from_thread().into());
        }
        Ok(Self(font))
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        // SAFETY: the font is no longer selected anywhere: windows using it are destroyed or were
        // given another font.
        _ = unsafe { DeleteObject(self.0.into()) };
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Children {
    label: HWND,
    control: HWND,
    ok: HWND,
    cancel: HWND,
}

impl Children {
    const fn all(self) -> [HWND; 4] {
        [self.label, self.control, self.ok, self.cancel]
    }
}

/// State of a dialog window, owned by its thread. The window procedure can be reentered, so it is
/// only ever borrowed briefly.
struct Dialog<'a> {
    control: &'a dyn Control,
    text: HSTRING,
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
    children: Cell<Children>,
    font: RefCell<Option<Font>>,
    dpi: Cell<u32>,
    /// The window's smallest size, at the current DPI.
    minimum: Cell<SIZE>,
    /// The control that had the focus when the window was deactivated.
    focus: Cell<Option<HWND>>,
    /// Whether the user accepted the dialog, once they have answered.
    accepted: Cell<Option<bool>>,
}

impl<'a> Dialog<'a> {
    fn new(control: &'a dyn Control, text: &str) -> Self {
        let mut style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_CLIPCHILDREN;
        if control.stretches() {
            style |= WS_THICKFRAME | WS_MAXIMIZEBOX;
        }
        Self {
            control,
            text: HSTRING::from(text),
            style,
            ex_style: WS_EX_DLGMODALFRAME | WS_EX_CONTROLPARENT,
            children: Cell::default(),
            font: RefCell::default(),
            dpi: Cell::new(96),
            minimum: Cell::default(),
            focus: Cell::default(),
            accepted: Cell::default(),
        }
    }

    /// Creates the label, the control and the buttons, in Tab order. The label comes first so that
    /// accessibility tools name the control after it.
    fn create_children(&self, window: HWND) -> Result<()> {
        let label = create_child(
            window,
            w!("STATIC"),
            &self.text,
            WINDOW_STYLE(SS_LEFT.0 | SS_NOPREFIX.0),
            WINDOW_EX_STYLE(0),
            LABEL_ID,
        )?;
        let control = self.control.create(window)?;
        let button = |text, style, id| {
            create_child(
                window,
                w!("BUTTON"),
                &HSTRING::from(text),
                WS_TABSTOP | WINDOW_STYLE(style as u32),
                WINDOW_EX_STYLE(0),
                id,
            )
        };
        let ok = button("OK", BS_DEFPUSHBUTTON, IDOK.0)?;
        let cancel = button("Cancel", BS_PUSHBUTTON, IDCANCEL.0)?;
        self.children.set(Children {
            label,
            control,
            ok,
            cancel,
        });
        self.update_ok();
        Ok(())
    }

    /// Switches to the font and sizes of `dpi`.
    fn set_dpi(&self, dpi: u32) -> Result<()> {
        let font = Font::message(dpi)?;
        for child in self.children.get().all() {
            send(child, WM_SETFONT, font.0.0 as usize, 1);
        }
        // Drops the previous font, which is no longer used.
        self.font.replace(Some(font));
        self.dpi.set(dpi);
        self.minimum
            .set(self.window_size(self.ideal_client_size())?);
        Ok(())
    }

    /// The window's size for a client area of `client`.
    fn window_size(&self, client: SIZE) -> Result<SIZE> {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: client.cx,
            bottom: client.cy,
        };
        // SAFETY: `rect` is writable.
        unsafe {
            AdjustWindowRectExForDpi(
                &raw mut rect,
                self.style,
                false,
                self.ex_style,
                self.dpi.get(),
            )
        }?;
        Ok(SIZE {
            cx: rect.right - rect.left,
            cy: rect.bottom - rect.top,
        })
    }

    /// The client area's size, with the content at its default width and the control at its own
    /// size.
    fn ideal_client_size(&self) -> SIZE {
        let dpi = self.dpi.get();
        let margin = scale(MARGIN, dpi);
        let control = self.control.size(self.children.get().control, dpi);
        let width = scale(CONTENT_WIDTH, dpi).max(control.cx);
        SIZE {
            cx: width + 2 * margin,
            cy: self.control_top(width) + control.cy + self.buttons_height(),
        }
    }

    /// The prompt's height, when it is `width` wide.
    fn label_height(&self, width: i32) -> i32 {
        if self.text.is_empty() {
            return 0;
        }
        let mut text = self.text.to_vec();
        let mut rect = RECT {
            right: width,
            ..RECT::default()
        };
        with_font(self.children.get().label, |dc| {
            // SAFETY: `dc` is valid, and `rect` is writable.
            _ = unsafe {
                DrawTextW(
                    dc,
                    &mut text,
                    &raw mut rect,
                    DT_CALCRECT | DT_WORDBREAK | DT_EXPANDTABS | DT_NOPREFIX,
                )
            };
        });
        rect.bottom
    }

    /// The control's top, when the content is `width` wide.
    fn control_top(&self, width: i32) -> i32 {
        let dpi = self.dpi.get();
        let label = self.label_height(width);
        let label = if label > 0 {
            label + scale(SPACING, dpi)
        } else {
            0
        };
        scale(MARGIN, dpi) + label
    }

    /// The height below the control: the buttons and the margins around them.
    fn buttons_height(&self) -> i32 {
        let dpi = self.dpi.get();
        2 * scale(MARGIN, dpi) + scale(BUTTON_HEIGHT, dpi)
    }

    fn layout(&self, window: HWND) {
        let children = self.children.get();
        if children.cancel.is_invalid() {
            // The window is still being created.
            return;
        }

        let dpi = self.dpi.get();
        let margin = scale(MARGIN, dpi);
        let mut client = RECT::default();
        // SAFETY: `client` is writable.
        _ = unsafe { GetClientRect(window, &raw mut client) };
        let width = client.right - 2 * margin;

        let place = |child, x, y, width: i32, height: i32| {
            // SAFETY: `child` is one of the window's children. Failure only leaves it where it was.
            _ = unsafe { MoveWindow(child, x, y, width.max(0), height.max(0), true) };
        };

        let control_top = self.control_top(width);
        place(
            children.label,
            margin,
            margin,
            width,
            self.label_height(width),
        );

        let button_width = scale(BUTTON_WIDTH, dpi);
        let button_height = scale(BUTTON_HEIGHT, dpi);
        let buttons_top = client.bottom - margin - button_height;
        let cancel_left = client.right - margin - button_width;
        let ok_left = cancel_left - scale(SPACING, dpi) - button_width;
        place(
            children.ok,
            ok_left,
            buttons_top,
            button_width,
            button_height,
        );
        place(
            children.cancel,
            cancel_left,
            buttons_top,
            button_width,
            button_height,
        );

        if self.control.stretches() {
            let height = client.bottom - self.buttons_height() - control_top;
            place(children.control, margin, control_top, width, height);
        } else {
            let size = self.control.size(children.control, dpi);
            let control_width = size.cx.min(width);
            let left = margin + (width - control_width) / 2;
            place(children.control, left, control_top, control_width, size.cy);
        }
        self.control.resized(children.control);
    }

    fn dpi_changed(&self, window: HWND, dpi: u32, suggested: &RECT) {
        if let Err(error) = self.set_dpi(dpi) {
            debug!("the dialog could not switch to {dpi} DPI: {error}");
            return;
        }

        let minimum = self.minimum.get();
        let size = if self.control.stretches() {
            SIZE {
                cx: (suggested.right - suggested.left).max(minimum.cx),
                cy: (suggested.bottom - suggested.top).max(minimum.cy),
            }
        } else {
            minimum
        };
        // SAFETY: `window` is this dialog's window.
        _ = unsafe {
            SetWindowPos(
                window,
                None,
                suggested.left,
                suggested.top,
                size.cx,
                size.cy,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn accept(&self) {
        if self.control.can_accept(self.children.get().control) {
            self.accepted.set(Some(true));
        }
    }

    fn update_ok(&self) {
        let children = self.children.get();
        let enabled = self.control.can_accept(children.control);
        // SAFETY: `ok` is the window's OK button.
        unsafe {
            if IsWindowEnabled(children.ok).as_bool() != enabled {
                _ = EnableWindow(children.ok, enabled);
            }
        }
    }

    fn handle(
        &self,
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Option<LRESULT> {
        match message {
            WM_COMMAND => {
                // The low word is the ID of the button, or IDOK and IDCANCEL for Enter and Escape.
                let id = i32::from(low_word(wparam));
                if id == IDOK.0 {
                    self.accept();
                } else if id == IDCANCEL.0 {
                    self.accepted.set(Some(false));
                } else {
                    return None;
                }
                Some(LRESULT(0))
            }
            WM_NOTIFY => {
                // SAFETY: `WM_NOTIFY` comes with an `NMHDR`, at the start of every notification.
                let notification = unsafe { &*(lparam.0 as *const NMHDR) };
                if notification.hwndFrom == self.children.get().control {
                    if self.control.accepts(notification) {
                        self.accept();
                    }
                    self.update_ok();
                }
                None
            }
            WM_CLOSE => {
                self.accepted.set(Some(false));
                Some(LRESULT(0))
            }
            // Makes Enter press OK, which `IsDialogMessageW` asks for.
            DM_GETDEFID => Some(LRESULT(DEFAULT_BUTTON)),
            // Gives the focus back to the control that had it, since only dialogs do that by
            // themselves.
            WM_ACTIVATE => {
                if u32::from(low_word(wparam)) == WA_INACTIVE {
                    // SAFETY: no preconditions.
                    let focus = unsafe { GetFocus() };
                    self.focus.set((!focus.is_invalid()).then_some(focus));
                } else {
                    let focus = self.focus.get().unwrap_or(self.children.get().control);
                    // SAFETY: `focus` is one of the window's children.
                    _ = unsafe { SetFocus(Some(focus)) };
                }
                Some(LRESULT(0))
            }
            WM_SIZE => {
                self.layout(window);
                Some(LRESULT(0))
            }
            WM_GETMINMAXINFO => {
                // SAFETY: `WM_GETMINMAXINFO` comes with a writable `MINMAXINFO`.
                let info = unsafe { &mut *(lparam.0 as *mut MINMAXINFO) };
                let minimum = self.minimum.get();
                info.ptMinTrackSize = POINT {
                    x: minimum.cx,
                    y: minimum.cy,
                };
                Some(LRESULT(0))
            }
            WM_DPICHANGED => {
                // SAFETY: `WM_DPICHANGED` comes with the suggested window rectangle.
                let suggested = unsafe { &*(lparam.0 as *const RECT) };
                self.dpi_changed(window, u32::from(low_word(wparam)), suggested);
                Some(LRESULT(0))
            }
            _ => None,
        }
    }

    /// Runs the message loop until the user answers, or the thread gets `WM_QUIT` because the
    /// dialog's future was dropped. Returns whether the user accepted the dialog.
    fn run(&self, window: HWND) -> Result<bool> {
        let mut message = MSG::default();
        while self.accepted.get().is_none() {
            // SAFETY: `message` is writable.
            match unsafe { GetMessageW(&raw mut message, None, 0, 0) }.0 {
                0 => return Ok(false),
                -1 => return Err(WindowsError::from_thread().into()),
                _ => {}
            }
            // SAFETY: `message` was just retrieved.
            unsafe {
                if !IsDialogMessageW(window, &raw const message).as_bool() {
                    _ = TranslateMessage(&raw const message);
                    DispatchMessageW(&raw const message);
                }
            }
        }
        Ok(self.accepted.get() == Some(true))
    }
}

unsafe extern "system" fn window_procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: the user data is either null or the window's `Dialog`, set from its creation
    // parameter, which outlives the window.
    unsafe {
        if message == WM_NCCREATE {
            let create = &*(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as isize);
        }

        let dialog = GetWindowLongPtrW(window, GWLP_USERDATA) as *const Dialog<'_>;
        if let Some(dialog) = dialog.as_ref()
            && let Some(result) = dialog.handle(window, message, wparam, lparam)
        {
            return result;
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}

fn register_class() -> Result<()> {
    static REGISTER: Once = Once::new();

    let instance = instance()?;
    REGISTER.call_once(|| {
        let controls = INITCOMMONCONTROLSEX {
            dwSize: struct_size::<INITCOMMONCONTROLSEX>(),
            dwICC: ICC_STANDARD_CLASSES | ICC_LISTVIEW_CLASSES | ICC_DATE_CLASSES,
        };
        // SAFETY: `controls` is valid. Failure shows when creating the controls.
        _ = unsafe { InitCommonControlsEx(&raw const controls) };

        let class = WNDCLASSEXW {
            cbSize: struct_size::<WNDCLASSEXW>(),
            lpfnWndProc: Some(window_procedure),
            hInstance: instance,
            // SAFETY: loads a system cursor, which need not be freed.
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default(),
            // SAFETY: no preconditions.
            hbrBackground: unsafe { GetSysColorBrush(COLOR_BTNFACE) },
            lpszClassName: CLASS_NAME,
            ..WNDCLASSEXW::default()
        };
        // SAFETY: `class` is valid. Failure shows when creating the window.
        _ = unsafe { RegisterClassExW(&raw const class) };
    });
    Ok(())
}

/// The work area of the monitor under the mouse cursor, where dialogs are shown.
fn work_area() -> RECT {
    let mut cursor = POINT::default();
    let mut info = MONITORINFO {
        cbSize: struct_size::<MONITORINFO>(),
        ..MONITORINFO::default()
    };
    // SAFETY: `cursor` and `info` are writable. Without a cursor position, this is the primary
    // monitor.
    unsafe {
        _ = GetCursorPos(&raw mut cursor);
        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTOPRIMARY);
        _ = GetMonitorInfoW(monitor, &raw mut info);
    }
    info.rcWork
}

struct DestroyOnDrop(HWND);

impl Drop for DestroyOnDrop {
    fn drop(&mut self) {
        // SAFETY: the window belongs to this thread.
        _ = unsafe { DestroyWindow(self.0) };
    }
}

/// Shows a dialog window with the prompt `text` above `control`, on the current thread. Returns
/// `accept`'s result, called with the control while it still exists, if the user accepted it, or
/// `None` if they cancelled.
pub fn show<T>(
    title: &str,
    text: &str,
    control: &dyn Control,
    accept: impl FnOnce(HWND) -> T,
) -> Result<Option<T>> {
    register_class()?;
    let dialog = Dialog::new(control, text);

    // The window starts empty in the middle of the work area, so that it gets that monitor's DPI.
    let area = work_area();
    let center = POINT {
        x: i32::midpoint(area.left, area.right),
        y: i32::midpoint(area.top, area.bottom),
    };
    // SAFETY: `dialog`, passed as the creation parameter, outlives the window, which is destroyed
    // before returning.
    let window = unsafe {
        CreateWindowExW(
            dialog.ex_style,
            CLASS_NAME,
            &HSTRING::from(title),
            dialog.style,
            center.x,
            center.y,
            0,
            0,
            None,
            None,
            Some(instance()?),
            Some(ptr::from_ref(&dialog).cast()),
        )
    }?;
    let _destroy = DestroyOnDrop(window);

    dialog.create_children(window)?;
    // SAFETY: `window` is valid.
    dialog.set_dpi(unsafe { GetDpiForWindow(window) })?;

    let size = dialog.minimum.get();
    let left = (center.x - size.cx / 2).max(area.left);
    let top = (center.y - size.cy / 2).max(area.top);
    // SAFETY: `window` is valid.
    unsafe {
        SetWindowPos(
            window,
            None,
            left,
            top,
            size.cx,
            size.cy,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
        _ = ShowWindow(window, SW_SHOW);
        _ = SetForegroundWindow(window);
    }

    let accepted = dialog.run(window)?;
    Ok(accepted.then(|| accept(dialog.children.get().control)))
}

#[cfg(test)]
mod tests {
    use super::scale;

    #[test]
    fn scales() {
        assert_eq!(scale(11, 96), 11);
        assert_eq!(scale(11, 144), 17);
        assert_eq!(scale(75, 120), 94);
    }
}
