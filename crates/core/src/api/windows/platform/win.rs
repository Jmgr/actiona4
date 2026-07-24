#![allow(unsafe_code)]

use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    sync::Arc,
};

use derive_more::Deref;
use parking_lot::Mutex;
use satint::{SaturatingInto, Su32};
use tokio_util::sync::CancellationToken;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    System::StationsAndDesktops::{
        DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS, EnumDesktopWindows, OpenInputDesktop,
    },
    UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible, SW_MAXIMIZE, SW_MINIMIZE, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SendNotifyMessageW, SetForegroundWindow,
        SetWindowPos, ShowWindow, WM_CLOSE,
    },
};
use windows_result::BOOL;

use crate::{
    api::{
        point::{Point, point},
        rect::Rect,
        size::{Size, size},
        windows::platform::{Registry, Result, WindowId, WindowsHandler},
    },
    cancel_on,
    platform::win::safe_handle::SafeDesktopHandle,
    runtime::{
        Runtime,
        platform::win::events::{WindowEvent, WindowHandle as WinWindowHandle},
    },
};

#[derive(Clone, Deref, Eq, PartialEq)]
struct WindowHandle(HWND);

impl Debug for WindowHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Window").field(&self.0).finish()
    }
}

impl Hash for WindowHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[allow(clippy::as_conversions)] // pointer-to-integer for hashing
        (self.0.0 as usize).hash(state);
    }
}

// SAFETY: HWND is an opaque handle (integer-like) safe to send/share across threads on Windows.
unsafe impl Send for WindowHandle {}
// SAFETY: HWND is an opaque handle (integer-like) safe to send/share across threads on Windows.
unsafe impl Sync for WindowHandle {}

#[derive(Debug)]
pub struct WindowsWindowHandler {
    inner: Mutex<Registry<WindowHandle>>,
}

impl Default for WindowsWindowHandler {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Registry::default()),
        }
    }
}

#[allow(clippy::as_conversions)] // pointer casts required by Windows callback API
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let vec_ptr = lparam.0 as *mut Vec<HWND>;
    // SAFETY: `lparam` was created from a mutable `Vec<HWND>` pointer in `all` and the callback
    // is invoked synchronously by `EnumDesktopWindows` while that vector remains alive.
    unsafe {
        let vec = &mut *vec_ptr;
        vec.push(hwnd);
    }

    true.into()
}

impl WindowsHandler for WindowsWindowHandler {
    #[allow(clippy::as_conversions)] // pointer casts required by Windows EnumDesktopWindows API
    fn all(&self) -> Result<Vec<WindowId>> {
        let mut result = Vec::new();
        let result_ptr = &raw mut result;
        // SAFETY: The desktop handle is valid while enumerating, and `result_ptr` points to the
        // local vector kept alive for the synchronous callback.
        unsafe {
            let hdesk = SafeDesktopHandle::try_new(OpenInputDesktop(
                DESKTOP_CONTROL_FLAGS::default(),
                false,
                DESKTOP_READOBJECTS,
            )?)?;

            EnumDesktopWindows(
                Some(hdesk.as_raw()),
                Some(enum_proc),
                LPARAM(result_ptr as isize),
            )?;
        }

        Ok(self
            .inner
            .lock()
            .update(result.into_iter().map(WindowHandle)))
    }

    fn is_visible(&self, id: WindowId) -> Result<bool> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        // SAFETY: `handle` is retrieved from this handler's registry of Win32 window handles.
        Ok(unsafe { IsWindowVisible(*handle).as_bool() })
    }

    fn title(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        // SAFETY: `handle` is a registered Win32 window handle.
        let len = unsafe { GetWindowTextLengthW(*handle) };
        if len == 0 {
            return Ok(String::new());
        }

        let buffer_len = len + 1;
        let mut buffer = vec![0; buffer_len.saturating_into()];

        // SAFETY: `buffer` has space for the reported text length plus its terminating NUL.
        let len = unsafe { GetWindowTextW(*handle, &mut buffer) };
        if len == 0 {
            return Ok(String::new());
        }

        let text_len: usize = len.saturating_into();
        Ok(String::from_utf16_lossy(&buffer[..text_len]))
    }

    fn classname(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        let mut buffer = [0; 255];
        // SAFETY: `buffer` is valid writable storage for the class-name UTF-16 string.
        let len = unsafe { GetClassNameW(*handle, &mut buffer) };
        if len == 0 {
            return Ok(String::new());
        }

        let text_len: usize = len.saturating_into();
        Ok(String::from_utf16_lossy(&buffer[..text_len]))
    }

    fn close(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        // SAFETY: `handle` is a registered window and the message parameters are defaults for WM_CLOSE.
        unsafe { SendNotifyMessageW(*handle, WM_CLOSE, WPARAM::default(), LPARAM::default())? };

        Ok(())
    }

    fn process_id(&self, id: WindowId) -> Result<Su32> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let mut process_id = 0;

        // SAFETY: `process_id` is valid writable storage and `handle` is registered.
        unsafe { GetWindowThreadProcessId(*handle, Some(&raw mut process_id)) };

        Ok(process_id.into())
    }

    fn rect(&self, id: WindowId) -> Result<Rect> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let mut win_rect = RECT::default();

        // SAFETY: `win_rect` is valid writable storage and `handle` is registered.
        unsafe {
            GetWindowRect(*handle, &raw mut win_rect)?;
        }

        let width = (win_rect.right - win_rect.left).max(0);
        let height = (win_rect.bottom - win_rect.top).max(0);

        Ok(Rect::new(
            point(win_rect.left, win_rect.top),
            size(width, height),
        ))
    }

    fn set_active(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        // SAFETY: `handle` is a registered window handle.
        unsafe {
            if !SetForegroundWindow(*handle).as_bool() {
                return Err(windows_result::Error::from_thread().into());
            }
        }

        Ok(())
    }

    fn minimize(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        // SAFETY: `handle` is a registered window handle.
        unsafe {
            if !ShowWindow(*handle, SW_MINIMIZE).as_bool() {
                return Err(windows_result::Error::from_thread().into());
            }
        }

        Ok(())
    }

    fn maximize(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        // SAFETY: `handle` is a registered window handle.
        unsafe {
            if !ShowWindow(*handle, SW_MAXIMIZE).as_bool() {
                return Err(windows_result::Error::from_thread().into());
            }
        }

        Ok(())
    }

    fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        // SAFETY: `handle` is a registered window and all position parameters are plain values.
        unsafe {
            SetWindowPos(
                *handle,
                None,
                position.x.into(),
                position.y.into(),
                0,
                0,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE,
            )?;
        };

        Ok(())
    }

    fn position(&self, id: WindowId) -> Result<Point> {
        Ok(self.rect(id)?.top_left())
    }

    fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        // SAFETY: `handle` is a registered window and all size parameters are plain values.
        unsafe {
            SetWindowPos(
                *handle,
                None,
                0,
                0,
                size.width.saturating_into(),
                size.height.saturating_into(),
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOMOVE,
            )?;
        };

        Ok(())
    }

    fn size(&self, id: WindowId) -> Result<Size> {
        Ok(self.rect(id)?.size())
    }

    fn is_active(&self, id: WindowId) -> Result<bool> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        // SAFETY: GetForegroundWindow takes no pointers and returns an opaque handle.
        let foreground = unsafe { GetForegroundWindow() };

        if foreground.0.is_null() {
            return Ok(false);
        }

        Ok(foreground == *handle)
    }

    fn active_window(&self) -> Result<Option<WindowId>> {
        // SAFETY: GetForegroundWindow takes no pointers and returns an opaque handle.
        let foreground = unsafe { GetForegroundWindow() };
        if foreground.0.is_null() {
            return Ok(None);
        }

        let window = WindowHandle(foreground);
        Ok(Some(self.inner.lock().get_or_insert(window)))
    }

    async fn wait_for_closed(
        &self,
        id: WindowId,
        runtime: Arc<Runtime>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let hwnd = **self.inner.lock().get_handle(id)?;

        #[allow(clippy::as_conversions)]
        let target = WinWindowHandle(hwnd.0 as isize);
        let mut receiver = runtime.platform().subscribe_window_events();

        loop {
            let event = cancel_on(&cancellation_token, receiver.recv()).await??;
            let WindowEvent::Closed(handle) = event;
            if handle == target {
                return Ok(());
            }
        }
    }
}
