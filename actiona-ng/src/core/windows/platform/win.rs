#![allow(unsafe_code)]

use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use derive_more::Deref;
use windows::Win32::{
    Foundation::{HWND, LPARAM},
    System::StationsAndDesktops::{
        DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS, EnumDesktopWindows, OpenInputDesktop,
    },
    UI::WindowsAndMessaging::{GetWindowTextLengthW, GetWindowTextW, IsWindowVisible},
};
use windows_result::BOOL;

use crate::{
    core::windows::platform::{Error, Registry, Result, WindowId, WindowsHandler},
    platform::win::safe_handle::SafeDesktopHandle,
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
        (self.0.0 as usize).hash(state);
    }
}

#[derive(Debug, Default)]
pub struct WindowsWindowHandler {
    inner: Registry<WindowHandle>,
}

impl From<windows_result::Error> for Error {
    fn from(value: windows_result::Error) -> Self {
        Self::Other(value.into())
    }
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let vec_ptr = lparam.0 as *mut Vec<HWND>;
    unsafe {
        let vec = &mut *vec_ptr;
        vec.push(hwnd);
    }

    true.into()
}

impl WindowsHandler for WindowsWindowHandler {
    fn all(&mut self) -> Result<Vec<WindowId>> {
        let mut result = Vec::new();
        let result_ptr = &mut result as *mut Vec<HWND>;
        unsafe {
            let hdesk: SafeDesktopHandle =
                OpenInputDesktop(DESKTOP_CONTROL_FLAGS::default(), false, DESKTOP_READOBJECTS)?
                    .into();

            EnumDesktopWindows(Some(*hdesk), Some(enum_proc), LPARAM(result_ptr as isize))?;
        }

        Ok(self
            .inner
            .update(result.into_iter().map(|window| WindowHandle(window))))
    }

    fn is_visible(&self, id: WindowId) -> Result<bool> {
        let handle = self.inner.get_handle(id)?;
        Ok(unsafe { IsWindowVisible(**handle).as_bool() })
    }

    fn title(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.get_handle(id)?;

        let len = unsafe { GetWindowTextLengthW(**handle) };
        if len == 0 {
            return Ok(String::new());
        }

        let mut buffer = vec![0; (len + 1) as usize];

        let len = unsafe { GetWindowTextW(**handle, &mut buffer) };
        if len == 0 {
            return Ok(String::new());
        }

        Ok(String::from_utf16_lossy(&buffer[..len as usize]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subsystem() {
        let mut handler = WindowsWindowHandler::default();
        println!(
            "{:?}",
            handler
                .all_windows()
                .unwrap()
                .into_iter()
                .filter(|id| handler.is_window_visible(*id).unwrap())
                .map(|id| handler.window_title(id).unwrap())
                .filter(|name| !name.is_empty())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }
}
