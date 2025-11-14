#![windows_subsystem = "console"]

use std::{fs, path::PathBuf};

use actiona_ng::runtime::Runtime;
use clap::Parser;
use color_eyre::{Result, config::HookBuilder, eyre::Context};
#[cfg(windows)]
use windows::{
    Wdk::System::SystemServices::RtlGetVersion, Win32::System::SystemInformation::OSVERSIONINFOW,
};

#[derive(Debug, Parser)]
struct Args {
    filepath: PathBuf,

    /// Show debug information
    #[cfg(debug_assertions)]
    #[arg(long, default_value_t = true)]
    debug: bool,

    /// Show debug information
    #[cfg(not(debug_assertions))]
    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[cfg(windows)]
fn is_windows10_1607_or_newer() -> Option<bool> {
    const WINDOWS_1607_BUILD: u32 = 14393;

    let mut info = OSVERSIONINFOW::default();
    unsafe { RtlGetVersion(&mut info).ok().ok()? };

    Some(info.dwBuildNumber >= WINDOWS_1607_BUILD)
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.debug {
        color_eyre::install()?;
    } else {
        let (panic_hook, eyre_hook) = HookBuilder::default()
            .capture_span_trace_by_default(false)
            .display_location_section(false)
            .display_env_section(false)
            .into_hooks();

        eyre_hook.install()?;
        panic_hook.install();
    }

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

    // Read the input file
    let script = fs::read_to_string(args.filepath).context("reading input file")?;

    Runtime::run_with_ui(
        |_runtime, script_engine| async move { script_engine.eval_async::<()>(&script).await },
        tauri::generate_context!(),
    )?;

    Ok(())
}
