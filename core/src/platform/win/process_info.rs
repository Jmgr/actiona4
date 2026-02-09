#![allow(unsafe_code, dead_code)]

use std::path::Path;

use color_eyre::Result;
use pe_parser::pe::parse_portable_executable;
use tokio::fs;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::StationsAndDesktops::{
            DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS, EnumDesktopWindows, OpenInputDesktop,
        },
        UI::WindowsAndMessaging::{
            GetClassNameW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
            IsWindowVisible, SendNotifyMessageW, WM_CLOSE,
        },
    },
    core::{BOOL, Error},
};

use crate::{platform::win::safe_handle::SafeDesktopHandle, types::su32::Su32};

#[derive(Debug)]
pub enum ProcessType {
    Gui,
    Console,
    Service,
    Unknown,
}

pub async fn find_process_type(path: &Path) -> Result<Option<ProcessType>> {
    let bytes = fs::read(path).await?;
    let executable = parse_portable_executable(bytes.as_slice())?;
    let subsystem = executable
        .optional_header_64
        .as_ref()
        .and_then(|h| h.get_subsystem())
        .or_else(|| {
            executable
                .optional_header_32
                .as_ref()
                .and_then(|h| h.get_subsystem())
        });

    use pe_parser::optional::*;
    Ok(match subsystem {
        None => None,
        Some(Subsystem::WindowsGUI) => Some(ProcessType::Gui),
        Some(Subsystem::WindowsCUI) => Some(ProcessType::Console),
        Some(_) => Some(ProcessType::Unknown),
    })
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

#[allow(clippy::as_conversions)] // pointer casts required by Windows EnumDesktopWindows API
pub fn all_windows() -> Result<Vec<HWND>> {
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

    Ok(result)
}

pub fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
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
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return String::new();
    }

    let mut buffer = vec![0; usize::from(Su32::from(len + 1))];

    let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    if len == 0 {
        return String::new();
    }

    String::from_utf16_lossy(&buffer[..usize::from(Su32::from(len))])
}

pub fn window_classname(hwnd: HWND) -> Result<String> {
    let mut buffer = [0u16; 256]; // safe per WNDCLASS/E[X] docs
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) };
    if len == 0 {
        return Err(Error::from_thread().into());
    }
    Ok(String::from_utf16_lossy(
        &buffer[..usize::from(Su32::from(len))],
    ))
}

pub fn is_window_visible(hwnd: &HWND) -> bool {
    unsafe { IsWindowVisible(*hwnd).as_bool() }
}

pub fn send_close_message_to_window(hwnd: HWND) -> Result<()> {
    unsafe {
        SendNotifyMessageW(hwnd, WM_CLOSE, WPARAM::default(), LPARAM::default())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subsystem() {
        /*
        let result =
            find_process_type(Path::new(r#"C:\actiona-distribution\output64\actiona.exe"#))
                .await
                .unwrap();
            */

        all_windows()
            .unwrap()
            .into_iter()
            .filter(is_window_visible)
            .map(|hwnd| (hwnd, window_title(hwnd)))
            .filter(|(_hwnd, title)| title.contains("Notepad"))
            .map(|(hwnd, _)| send_close_message_to_window(hwnd))
            //.map(window_title)
            .collect::<Result<Vec<_>>>()
            .unwrap();

        //println!("{result:?}");
    }
}
