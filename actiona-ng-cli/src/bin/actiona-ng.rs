#![windows_subsystem = "console"]

use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    sync::Arc,
};

use actiona_ng::{config::Config, runtime::Runtime, updater::Updater};
use clap::Parser;
use color_eyre::{
    Result,
    config::HookBuilder,
    eyre::{Context, OptionExt},
    owo_colors::OwoColorize,
};
use console::Emoji;
use time::OffsetDateTime;
use tracing::warn;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use versions::SemVer;
#[cfg(windows)]
use windows::{
    Wdk::System::SystemServices::RtlGetVersion, Win32::System::SystemInformation::OSVERSIONINFOW,
};

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

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

    /// Should Actiona-ng check for updates once per day?
    /// Default is true.
    #[arg(long, env)]
    disable_updates: Option<bool>,

    /// Should Actiona-ng send anonymous telemetry data?
    /// Default is false.
    #[arg(long, env, default_value_t = true)]
    disable_telemetry: bool,
}

#[cfg(windows)]
fn is_windows10_1607_or_newer() -> Option<bool> {
    const WINDOWS_1607_BUILD: u32 = 14393;

    let mut info = OSVERSIONINFOW::default();
    unsafe { RtlGetVersion(&mut info).ok().ok()? };

    Some(info.dwBuildNumber >= WINDOWS_1607_BUILD)
}

fn main() -> Result<()> {
    init_tracing();

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

    Runtime::run_with_ui(
        move |runtime, script_engine| async move {
            let config = Arc::new(Config::new().await?);

            // CLI and env have a higher priority than settings
            let updates_enabled = args.disable_updates.map_or_else(
                || config.settings(|settings| !settings.disable_updates),
                |enabled| enabled,
            );

            let app_version =
                SemVer::new(built_info::PKG_VERSION).ok_or_eyre("failed to parse crate version")?;
            let (_updater, ready) = Updater::new(
                config.clone(),
                updates_enabled,
                built_info::PKG_NAME,
                app_version.clone(),
                runtime.cancellation_token(),
                runtime.task_tracker(),
            );

            if updates_enabled {
                _ = ready.await;

                if let Some(version_info) =
                    config.state(|state| state.new_version_available.clone())
                {
                    if version_info.version <= app_version {
                        // The new version is older or the same as the one we are currently running
                        if let Err(err) = config
                            .state_mut(|state| state.new_version_available = None)
                            .await
                        {
                            warn!("saving state failed: {err}");
                        }
                    } else if io::stdout().is_terminal() {
                        // Display a message saying there is a new version available
                        let warning_sign = Emoji("⚠️", "/!\\");
                        let up_arrow = Emoji("🠱", "^");
                        let down_arrow = Emoji("🠳", "v");
                        let left_arrow = Emoji("🠰", "<");
                        let right_arrow = Emoji("🠲", ">");
                        for _ in 0..80 {
                            print!("{}", down_arrow);
                        }
                        println!();
                        println!(
                            "{}  {} {}",
                            warning_sign,
                            "NEW VERSION AVAILABLE".bold(),
                            warning_sign
                        );
                        println!(
                            "You are running {} version {}, latest version is {},\nreleased {:.0} ago.",
                            built_info::PKG_NAME,
                            built_info::PKG_VERSION.bold(),
                            version_info.version.bold(),
                            (OffsetDateTime::now_utc() - version_info.release_date)
                                .max(time::Duration::ZERO)
                        );
                        println!(
                            "Download: {}  {}  {}",
                            right_arrow,
                            version_info.download_url.bright_blue().underline(),
                            left_arrow
                        );
                        for _ in 0..80 {
                            print!("{}", up_arrow);
                        }
                        println!();
                    } else {
                        // TODO
                    }
                }
            }

            let script: String = tokio::fs::read_to_string(&args.filepath)
                .await
                .context("reading input file")?;

            script_engine.eval_async::<()>(&script).await
        },
        tauri::generate_context!(),
    )?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .init();
}
