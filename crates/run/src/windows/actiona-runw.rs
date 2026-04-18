#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    main_impl()
}

#[cfg(windows)]
use std::{
    env::{args_os, current_exe},
    os::windows::process::CommandExt,
    process::{Command, Stdio},
};

#[cfg(windows)]
use windows::{
    Win32::{
        System::Threading::CREATE_NO_WINDOW,
        UI::WindowsAndMessaging::{
            MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MESSAGEBOX_STYLE, MessageBoxW,
        },
    },
    core::{HSTRING, w},
};

#[cfg(windows)]
fn main_impl() -> ExitCode {
    if args_os().nth(1).is_none() {
        show_message_box(run::NO_ARGS_MESSAGE, MB_OK | MB_ICONINFORMATION);
        return ExitCode::FAILURE;
    }

    let exe_path = current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("actiona-run.exe")))
        .unwrap_or_else(|| "actiona-run.exe".into());

    let result = Command::new(&exe_path)
        .args(args_os().skip(1))
        .creation_flags(CREATE_NO_WINDOW.0)
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|child| child.wait_with_output());

    match result {
        Ok(output) if output.status.success() => ExitCode::SUCCESS,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = if stderr.trim().is_empty() {
                format!("actiona-run exited with status: {}", output.status)
            } else {
                stderr.trim().to_string()
            };
            show_message_box(&message, MB_OK | MB_ICONERROR);
            ExitCode::FAILURE
        }
        Err(err) => {
            show_message_box(&err.to_string(), MB_OK | MB_ICONERROR);
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main_impl() -> ExitCode {
    ExitCode::FAILURE
}

#[cfg(windows)]
fn show_message_box(message: &str, style: MESSAGEBOX_STYLE) {
    let message = HSTRING::from(message);

    unsafe {
        let _ = MessageBoxW(None, &message, w!("Actiona Run"), style);
    }
}
