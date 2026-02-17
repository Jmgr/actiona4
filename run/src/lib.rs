#![warn(clippy::all, clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::future_not_send)]
#![allow(clippy::too_many_arguments)]
#![allow(rustdoc::invalid_html_tags)]

use std::{ffi::OsString, path::Path, sync::Arc};

use actiona_core::{
    config::Config,
    format_js_value_for_console,
    runtime::{Runtime, RuntimeOptions, WaitAtEnd},
    scripting,
    scripting::pragma::parse_pragmas,
};
use clap::{CommandFactory, Parser};
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
    updates::{check_updates, check_updates_now},
};

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod args;
mod config;
mod init;
mod repl;
mod updates;

#[cfg(windows)]
fn is_windows10_1607_or_newer() -> Option<bool> {
    const WINDOWS_1607_BUILD: u32 = 14393;

    let mut info = OSVERSIONINFOW::default();
    unsafe { RtlGetVersion(&mut info).ok().ok()? };

    Some(info.dwBuildNumber >= WINDOWS_1607_BUILD)
}

pub fn run_cli() -> Result<()> {
    init_tracing();

    let args = Arc::new(parse_args_with_default_run());

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
                "Warning: You are running an unsupported version of Windows (older than 10 1607). Some features may not work properly."
            )
        }
        None => {
            eprintln!(
                "Warning: Unable to determine your version of Windows. Actiona is only supported on Windows 10 1607 or newer."
            )
        }
    }

    // Handle commands that don't need the runtime
    match &args.command {
        Commands::Init { path } => return init::run(path),
        Commands::Config { key, value } => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("creating tokio runtime for config")?;

            return runtime.block_on(async {
                let cfg = Config::new().await?;
                config::run(cfg, key, *value).await
            });
        }
        Commands::Update => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .context("creating tokio runtime for update check")?;

            return runtime.block_on(async {
                let config = Config::new().await?;
                check_updates_now(config).await
            });
        }
        Commands::Completions { shell } => {
            let mut cmd = <Args as clap::CommandFactory>::command();
            let bin_name = std::env::args_os()
                .next()
                .and_then(|arg0| {
                    Path::new(&arg0)
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().into_owned())
                })
                .unwrap_or_else(|| "actiona-run".to_owned());

            clap_complete::generate(*shell, &mut cmd, &bin_name, &mut std::io::stdout());
            return Ok(());
        }
        _ => {}
    }

    // Determine no_globals before creating the runtime, since it affects registration.
    let no_globals = match &args.command {
        Commands::Run { filepath } => {
            let script = std::fs::read_to_string(filepath).context("reading input file")?;
            parse_pragmas(&script).no_globals
        }
        Commands::Eval { code } => {
            let code = code.join("\n");
            parse_pragmas(&code).no_globals
        }
        Commands::Repl { no_globals } => *no_globals,
        _ => false,
    };

    let runtime_options = RuntimeOptions {
        #[cfg(unix)]
        display_name: args.display.clone(),
        no_globals,
        install_ctrl_c_handler: !args.command.is_repl(),
    };

    Runtime::run_with_ui(
        move |runtime, script_engine| async move {
            let config = Config::new().await?;

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
                        && !scripting::try_emit_script_diagnostic(&err, &script)
                    {
                        eprintln!("Error: {err}");
                    }
                }
                Commands::Eval { code } => {
                    let code = code.join("\n");
                    let value = script_engine.eval_async_fn::<Option<String>>(&code, |value| {
                        Ok(if value.is_undefined() {
                            None
                        } else if value.is_promise() {
                            let rendered = format_js_value_for_console(value.clone());
                            Some(format!("{rendered} (hint: wrap with `await (...)`)"))
                        } else {
                            Some(format_js_value_for_console(value))
                        })
                    });

                    match value.await {
                        Ok(Some(value)) => println!("{value}"),
                        Ok(None) => {}
                        Err(err) if !scripting::try_emit_script_diagnostic(&err, &code) => {
                            eprintln!("Error: {err}");
                        }
                        Err(_) => {}
                    }
                }
                Commands::Repl { .. } => {
                    runtime.set_wait_at_end(WaitAtEnd::No);

                    repl(script_engine, runtime.cancellation_token()).await?;
                }
                Commands::Init { .. }
                | Commands::Update
                | Commands::Completions { .. }
                | Commands::Config { .. } => {
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

fn parse_args_with_default_run() -> Args {
    let args = maybe_insert_default_run(std::env::args_os().collect());
    Args::parse_from(args)
}

fn maybe_insert_default_run(mut args: Vec<OsString>) -> Vec<OsString> {
    let cmd = Args::command();

    let Some(index) = first_positional_index(&args, &cmd) else {
        return args;
    };

    if let Some(name) = args[index].to_str()
        && !cmd
            .get_subcommands()
            .map(|s| s.get_name())
            .any(|subcommand| subcommand == name)
    {
        args.insert(index, OsString::from("run"));
    }

    args
}

fn first_positional_index(args: &[OsString], cmd: &clap::Command) -> Option<usize> {
    // Build a set of flags that consume a following value.
    let value_flags: Vec<(Option<char>, Option<&str>)> = cmd
        .get_arguments()
        .filter(|a| a.get_action().takes_values())
        .map(|a| (a.get_short(), a.get_long()))
        .collect();

    let takes_value = |flag: &str| -> bool {
        value_flags.iter().any(|(short, long)| {
            long.is_some_and(|l| flag == format!("--{l}"))
                || short.is_some_and(|s| flag == format!("-{s}"))
        })
    };

    let mut index = 1;
    while index < args.len() {
        let arg = args[index].to_string_lossy();

        if arg == "--" {
            return (index + 1 < args.len()).then_some(index + 1);
        }

        if arg.starts_with('-') {
            index += if takes_value(&arg) { 2 } else { 1 };
            continue;
        }

        return Some(index);
    }

    None
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn,enigo::platform::x11=off"));

    let stdout_layer = tracing_subscriber::fmt::layer()
        // Keep command output deterministic on stdout (e.g. scripts/tests) by
        // sending tracing logs to stderr.
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .init();
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::maybe_insert_default_run;

    #[test]
    fn defaults_to_run_for_script_path() {
        let args = vec![OsString::from("actiona-run"), OsString::from("script.ts")];

        let args = maybe_insert_default_run(args);
        assert_eq!(args, vec!["actiona-run", "run", "script.ts"]);
    }

    #[test]
    fn keeps_explicit_subcommand() {
        let args = vec![
            OsString::from("actiona-run"),
            OsString::from("run"),
            OsString::from("script.ts"),
        ];

        let args = maybe_insert_default_run(args);
        assert_eq!(args, vec!["actiona-run", "run", "script.ts"]);
    }

    #[test]
    fn keeps_update_subcommand() {
        let args = vec![OsString::from("actiona-run"), OsString::from("update")];

        let args = maybe_insert_default_run(args);
        assert_eq!(args, vec!["actiona-run", "update"]);
    }

    #[test]
    fn respects_top_level_options_before_default_subcommand() {
        let args = vec![
            OsString::from("actiona-run"),
            OsString::from("--disable-updates"),
            OsString::from("false"),
            OsString::from("script.ts"),
        ];

        let args = maybe_insert_default_run(args);
        assert_eq!(
            args,
            vec![
                "actiona-run",
                "--disable-updates",
                "false",
                "run",
                "script.ts"
            ]
        );
    }
}
