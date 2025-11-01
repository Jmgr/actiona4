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
    UI::WindowsAndMessaging::{
        GetClassNameW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible, SET_WINDOW_POS_FLAGS, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
        SetWindowPos,
    },
};
use windows_result::BOOL;

use crate::{
    core::{
        point::Point,
        rect::Rect,
        size::Size,
        windows::platform::{Error, Registry, Result, WindowId, WindowsHandler},
    },
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

    fn classname(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.get_handle(id)?;

        let mut buffer = [0; 255];
        let len = unsafe { GetClassNameW(**handle, &mut buffer) };
        if len == 0 {
            return Ok(String::new());
        }

        Ok(String::from_utf16_lossy(&buffer[..len as usize]))
    }

    fn close(&self, id: WindowId) -> Result<()> {
        todo!()
    }

    fn process_id(&self, id: WindowId) -> Result<u32> {
        let handle = self.inner.get_handle(id)?;
        let mut process_id = 0;

        unsafe { GetWindowThreadProcessId(**handle, Some(&mut process_id)) };

        Ok(process_id as u32)
    }

    fn rect(&self, id: WindowId) -> Result<Rect> {
        todo!()
    }

    fn set_active(&self, id: WindowId) -> Result<()> {
        todo!()
    }

    fn minimize(&self, id: WindowId) -> Result<()> {
        todo!()
    }

    fn maximize(&self, id: WindowId) -> Result<()> {
        todo!()
    }

    fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        let handle = self.inner.get_handle(id)?;

        unsafe {
            SetWindowPos(
                **handle,
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

    fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        todo!()
    }

    fn size(&self, id: WindowId) -> Result<Size> {
        todo!()
    }

    fn is_active(&self, id: WindowId) -> Result<bool> {
        todo!()
    }

    fn active_window(&mut self) -> Result<WindowId> {
        todo!()
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
