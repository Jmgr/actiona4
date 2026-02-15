#![allow(unsafe_code)]

use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use color_eyre::eyre::eyre;
use derive_more::Deref;
use parking_lot::Mutex;
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
        size::Size,
        windows::platform::{Registry, Result, WindowId, WindowsHandler},
    },
    platform::win::safe_handle::SafeDesktopHandle,
    types::su32::Su32,
};

#[derive(Clone, Deref, Eq, PartialEq)]
struct WindowHandle(HWND);

impl Debug for WindowHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl From<windows_result::Error> for Error {
    fn from(value: windows_result::Error) -> Self {
        Self::Other(value.into())
    }
}

#[allow(clippy::as_conversions)] // pointer casts required by Windows callback API
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let vec_ptr = lparam.0 as *mut Vec<HWND>;
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
        let result_ptr = &mut result as *mut Vec<HWND>;
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
        Ok(unsafe { IsWindowVisible(*handle).as_bool() })
    }

    fn title(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        let len = unsafe { GetWindowTextLengthW(*handle) };
        if len == 0 {
            return Ok(String::new());
        }

        let mut buffer = vec![0; usize::from(Su32::from(len + 1))];

        let len = unsafe { GetWindowTextW(*handle, &mut buffer) };
        if len == 0 {
            return Ok(String::new());
        }

        Ok(String::from_utf16_lossy(
            &buffer[..usize::from(Su32::from(len))],
        ))
    }

    fn classname(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        let mut buffer = [0; 255];
        let len = unsafe { GetClassNameW(*handle, &mut buffer) };
        if len == 0 {
            return Ok(String::new());
        }

        Ok(String::from_utf16_lossy(
            &buffer[..usize::from(Su32::from(len))],
        ))
    }

    // TODO: untested
    fn close(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        unsafe { SendNotifyMessageW(*handle, WM_CLOSE, WPARAM::default(), LPARAM::default())? };

        Ok(())
    }

    fn process_id(&self, id: WindowId) -> Result<u32> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let mut process_id = 0;

        unsafe { GetWindowThreadProcessId(*handle, Some(&mut process_id)) };

        Ok(process_id)
    }

    // TODO: untested
    fn rect(&self, id: WindowId) -> Result<Rect> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let mut win_rect = RECT::default();

        unsafe {
            GetWindowRect(*handle, &mut win_rect)?;
        }

        let width = (win_rect.right - win_rect.left).max(0);
        let height = (win_rect.bottom - win_rect.top).max(0);

        Ok(Rect::new(
            point(win_rect.left, win_rect.top),
            Size::new(width.into(), height.into()),
        ))
    }

    // TODO: untested
    fn set_active(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        unsafe {
            if !SetForegroundWindow(*handle).as_bool() {
                return Err(windows_result::Error::from_thread().into());
            }
        }

        Ok(())
    }

    // TODO: untested
    fn minimize(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        unsafe {
            if !ShowWindow(*handle, SW_MINIMIZE).as_bool() {
                return Err(windows_result::Error::from_thread().into());
            }
        }

        Ok(())
    }

    // TODO: untested
    fn maximize(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        unsafe {
            if !ShowWindow(*handle, SW_MAXIMIZE).as_bool() {
                return Err(windows_result::Error::from_thread().into());
            }
        }

        Ok(())
    }

    fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        unsafe {
            SetWindowPos(
                *handle,
                None,
                position.x.into(),
                position.y.into(),
                0,
                0,
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE,
            )?
        };

        Ok(())
    }

    fn position(&self, id: WindowId) -> Result<Point> {
        Ok(self.rect(id)?.top_left())
    }

    // TODO: untested
    fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();

        unsafe {
            SetWindowPos(
                *handle,
                None,
                0,
                0,
                i32::from(size.width),
                i32::from(size.height),
                SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOMOVE,
            )?
        };

        Ok(())
    }

    // TODO: untested
    fn size(&self, id: WindowId) -> Result<Size> {
        Ok(self.rect(id)?.size())
    }

    // TODO: untested
    fn is_active(&self, id: WindowId) -> Result<bool> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let foreground = unsafe { GetForegroundWindow() };

        if foreground.0.is_null() {
            return Ok(false);
        }

        Ok(foreground == *handle)
    }

    // TODO: untested
    fn active_window(&self) -> Result<WindowId> {
        let foreground = unsafe { GetForegroundWindow() };
        if foreground.0.is_null() {
            return Err(eyre!("not found"));
        }

        let window = WindowHandle(foreground);
        Ok(self.inner.lock().get_or_insert(window))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subsystem() {
        let handler = WindowsWindowHandler::default();
        println!(
            "{:?}",
            handler
                .all()
                .unwrap()
                .into_iter()
                .filter(|id| handler.is_visible(*id).unwrap())
                .map(|id| handler.title(id).unwrap())
                .filter(|name| !name.is_empty())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
}
