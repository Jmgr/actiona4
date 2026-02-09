#![windows_subsystem = "console"]

use std::sync::Arc;

use actiona_core::{
    config::Config,
    runtime::{Runtime, RuntimeOptions, WaitAtEnd},
};
use clap::Parser;
use color_eyre::{Result, config::HookBuilder, eyre::Context};
use tracing_subscriber::{
    EnvFilter, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};
#[cfg(windows)]
use windows::{
    Wdk::System::SystemServices::RtlGetVersion, Win32::System::SystemInformation::OSVERSIONINFOW,
};

use crate::{
    args::{Args, Commands},
    repl::repl,
    updates::check_updates,
};

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod args;
mod init;
mod repl;
mod setup;
mod updates;

#[cfg(windows)]
fn is_windows10_1607_or_newer() -> Option<bool> {
    const WINDOWS_1607_BUILD: u32 = 14393;

    let mut info = OSVERSIONINFOW::default();
    unsafe { RtlGetVersion(&mut info).ok().ok()? };

    Some(info.dwBuildNumber >= WINDOWS_1607_BUILD)
}

fn main() -> Result<()> {
    init_tracing();

    let args = Arc::new(Args::parse());

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

    // Handle commands that don't need the runtime
    match &args.command {
        Commands::Init { path } => return init::run(path),
        Commands::Completions { shell } => {
            let mut cmd = <Args as clap::CommandFactory>::command();
            clap_complete::generate(*shell, &mut cmd, "actiona4-run", &mut std::io::stdout());
            return Ok(());
        }
        _ => {}
    }

    // Automatic platform-specific setup (e.g. Windows notification registration)
    setup::ensure_platform_setup();

    let runtime_options = RuntimeOptions {
        #[cfg(unix)]
        display_name: args.display.clone(),
    };

    Runtime::run_with_ui(
        move |runtime, script_engine| async move {
            let config = Arc::new(Config::new().await?);

            check_updates(
                &args,
                config,
                runtime.cancellation_token(),
                runtime.task_tracker(),
            )
            .await?;

            match &args.command {
                Commands::Run { filepath } => {
                    init::ensure_index_dts(filepath)?;

                    let script: String = tokio::fs::read_to_string(&filepath)
                        .await
                        .context("reading input file")?;

                    let filename = filepath.to_string_lossy();
                    if let Err(err) = script_engine
                        .eval_async_with_filename::<()>(&script, Some(&filename))
                        .await
                    {
                        eprintln!("Error: {err}");
                    }
                }
                Commands::Eval { code } => {
                    let code = code.join("\n");

                    if let Err(err) = script_engine.eval_async::<()>(&code).await {
                        eprintln!("Error: {err}");
                    }
                }
                Commands::Repl => {
                    runtime.set_wait_at_end(WaitAtEnd::No);

                    repl(script_engine).await?;
                }
                Commands::Init { .. } | Commands::Completions { .. } => {
                    unreachable!("handled before runtime startup")
                }
            };

            Ok(())
        },
        runtime_options,
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
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .init();
}
