#![windows_subsystem = "console"]

use std::{fs, path::PathBuf};

use actiona_ng::runtime::Runtime;
use clap::Parser;
use eyre::Result;
#[cfg(windows)]
use windows::{
    Wdk::System::SystemServices::RtlGetVersion, Win32::System::SystemInformation::OSVERSIONINFOW,
};

#[derive(Debug, Parser)]
struct Args {
    filepath: PathBuf,
}

#[cfg(windows)]
fn is_windows10_1607_or_newer() -> Option<bool> {
    const WINDOWS_1607_BUILD: u32 = 14393;

    let mut info = OSVERSIONINFOW::default();
    unsafe { RtlGetVersion(&mut info).ok().ok()? };

    Some(info.dwBuildNumber >= WINDOWS_1607_BUILD)
}

fn main() -> Result<()> {
    #[cfg(windows)]
    match is_windows10_1607_or_newer() {
        Some(true) => {}
        Some(false) => {
            eprintln!(
                "⚠️  You are running an unsupported version of Windows (older than 10 1607). Some features may not work properly."
            )
        }
        None => {
            eprintln!(
                "⚠️  Unable to determine your version of Windows. Actiona is only supported on Windows 10 1067 or never."
            )
        }
    }

    let args = Args::parse();

    // Read the input file
    let script = fs::read_to_string(args.filepath)?;

    Runtime::run_with_ui(
        |_runtime, script_engine| async move { script_engine.eval_async::<()>(&script).await },
        tauri::generate_context!(),
    )?;

    Ok(())
}
