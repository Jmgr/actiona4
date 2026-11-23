#![allow(unsafe_code, dead_code)]

use color_eyre::Result;
use satint::SaturatingInto;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess},
        UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowTextLengthW, GetWindowTextW,
            GetWindowThreadProcessId, IsWindowVisible, SendNotifyMessageW, WM_CLOSE,
        },
    },
    core::{BOOL, Error},
};

use crate::platform::win::safe_handle::SafeHandle;

#[allow(clippy::as_conversions)] // pointer casts required by Windows callback API
unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let vec_ptr = lparam.0 as *mut Vec<HWND>;
    // SAFETY: `lparam` was created from a live mutable `Vec<HWND>` in `all_windows`.
    unsafe {
        let vec = &mut *vec_ptr;
        vec.push(hwnd);
    }

    true.into()
}

#[allow(clippy::as_conversions)] // pointer casts required by Windows EnumWindows API
pub fn all_windows() -> Result<Vec<HWND>> {
    let mut result = Vec::new();
    let result_ptr = &raw mut result;
    // SAFETY: EnumWindows invokes the callback synchronously while `result` remains alive.
    unsafe {
        EnumWindows(Some(enum_proc), LPARAM(result_ptr as isize))?;
    }

    Ok(result)
}

pub fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0;
    // SAFETY: `pid` is valid writable storage for the process identifier.
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&raw mut pid));
    }
    pid
}

pub fn windows_for_pid(pid: u32) -> Result<Vec<HWND>> {
    Ok(all_windows()?
        .into_iter()
        .filter(|hwnd| window_pid(*hwnd) == pid)
        .collect::<Vec<_>>())
}

pub fn window_title(hwnd: HWND) -> String {
    // SAFETY: `hwnd` is caller-provided and the API takes no additional pointers.
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return String::new();
    }

    let mut buffer = vec![0; (len + 1).saturating_into()];

    // SAFETY: `buffer` has space for the text length and its terminating NUL.
    let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    if len == 0 {
        return String::new();
    }

    String::from_utf16_lossy(&buffer[..len.saturating_into()])
}

pub fn window_classname(hwnd: HWND) -> Result<String> {
    let mut buffer = [0_u16; 256]; // safe per WNDCLASS/E[X] docs
    // SAFETY: `buffer` is valid writable storage for the class-name UTF-16 string.
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
    if len == 0 {
        return Err(Error::from_thread().into());
    }

    Ok(String::from_utf16_lossy(&buffer[..len.saturating_into()]))
}

pub fn is_window_visible(hwnd: HWND) -> bool {
    // SAFETY: `hwnd` comes from a Win32 window enumeration or caller-provided handle.
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

pub fn send_close_message_to_window(hwnd: HWND) -> Result<()> {
    // SAFETY: `hwnd` is caller-provided and the default parameters are valid for WM_CLOSE.
    unsafe {
        SendNotifyMessageW(hwnd, WM_CLOSE, WPARAM::default(), LPARAM::default())?;
    }
    Ok(())
}

pub fn terminate_process_by_pid(pid: u32) -> Result<()> {
    // SAFETY: the opened process handle is immediately wrapped and terminated before it is dropped.
    unsafe {
        let handle = SafeHandle::try_new(OpenProcess(PROCESS_TERMINATE, false, pid)?)?;
        TerminateProcess(handle.as_raw(), 1)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn subsystem() {
        all_windows()
            .unwrap()
            .into_iter()
            .filter(|hwnd| is_window_visible(*hwnd))
            .map(|hwnd| (hwnd, window_title(hwnd)))
            .filter(|(_hwnd, title)| title.contains("Notepad"))
            .map(|(hwnd, _)| send_close_message_to_window(hwnd))
            //.map(window_title)
            .collect::<Result<Vec<_>>>()
            .unwrap();
    }
}
