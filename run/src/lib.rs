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
        Commands::Run { filepath, .. } => {
            let script = std::fs::read_to_string(filepath).context("reading input file")?;
            parse_pragmas(&script).no_globals
        }
        Commands::Eval { code, .. } => {
            let code = code.join("\n");
            parse_pragmas(&code).no_globals
        }
        Commands::Repl { no_globals, .. } => *no_globals,
        _ => false,
    };

    let seed = match &args.command {
        Commands::Run { run_args, .. }
        | Commands::Eval { run_args, .. }
        | Commands::Repl { run_args, .. } => run_args.seed,
        _ => None,
    };

    let runtime_options = RuntimeOptions {
        #[cfg(unix)]
        display_name: args.display.clone(),
        no_globals,
        install_ctrl_c_handler: !args.command.is_repl(),
        show_tray_icon: !args.command.is_repl(),
        seed,
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
                Commands::Run { filepath, .. } => {
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
                Commands::Eval { code, .. } => {
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
    // Top-level flags and which of them consume a following value.
    let top_level_flags: Vec<(Option<char>, Option<&str>, bool)> = cmd
        .get_arguments()
        .map(|a| (a.get_short(), a.get_long(), a.get_action().takes_values()))
        .collect();

    // Flags defined on the default `run` subcommand. If one of those appears
    // before a subcommand, we inject `run` before that flag.
    let run_flags: Vec<(Option<char>, Option<&str>)> = cmd
        .get_subcommands()
        .find(|subcommand| subcommand.get_name() == "run")
        .map(|run| {
            run.get_arguments()
                .map(|a| (a.get_short(), a.get_long()))
                .collect()
        })
        .unwrap_or_default();

    let flag_matches = |flag: &str, short: Option<char>, long: Option<&str>| {
        // Keep matching logic aligned with the existing parser behavior:
        // exact `-x` / `--long` matches only.
        long.is_some_and(|l| flag.strip_prefix("--") == Some(l))
            || short.is_some_and(|s| flag.len() == 2 && flag.starts_with('-') && flag.ends_with(s))
    };

    let top_takes_value = |flag: &str| -> bool {
        top_level_flags
            .iter()
            .any(|(short, long, takes_value)| *takes_value && flag_matches(flag, *short, *long))
    };

    let is_top_level_flag = |flag: &str| -> bool {
        top_level_flags
            .iter()
            .any(|(short, long, _)| flag_matches(flag, *short, *long))
    };

    let is_run_flag = |flag: &str| -> bool {
        run_flags
            .iter()
            .any(|(short, long)| flag_matches(flag, *short, *long))
    };

    let mut index = 1;
    while index < args.len() {
        let arg = args[index].to_string_lossy();

        if arg == "--" {
            return (index + 1 < args.len()).then_some(index + 1);
        }

        if arg.starts_with('-') {
            if top_takes_value(&arg) {
                index += 2;
                continue;
            }

            if is_top_level_flag(&arg) {
                index += 1;
                continue;
            }

            if is_run_flag(&arg) {
                return Some(index);
            }

            index += 1;
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

    use clap::{Parser, error::ErrorKind};

    use super::maybe_insert_default_run;
    use crate::args::Args;

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
            OsString::from("--update-check"),
            OsString::from("false"),
            OsString::from("script.ts"),
        ];

        let args = maybe_insert_default_run(args);
        assert_eq!(
            args,
            vec!["actiona-run", "--update-check", "false", "run", "script.ts"]
        );
    }

    #[test]
    fn inserts_run_before_run_seed_option() {
        let args = vec![
            OsString::from("actiona-run"),
            OsString::from("--seed"),
            OsString::from("42"),
            OsString::from("script.ts"),
        ];

        let args = maybe_insert_default_run(args);
        assert_eq!(
            args,
            vec!["actiona-run", "run", "--seed", "42", "script.ts"]
        );
    }

    #[test]
    fn inserts_run_after_top_level_options_and_before_run_seed_option() {
        let args = vec![
            OsString::from("actiona-run"),
            OsString::from("--update-check"),
            OsString::from("false"),
            OsString::from("--seed"),
            OsString::from("42"),
            OsString::from("script.ts"),
        ];

        let args = maybe_insert_default_run(args);
        assert_eq!(
            args,
            vec![
                "actiona-run",
                "--update-check",
                "false",
                "run",
                "--seed",
                "42",
                "script.ts"
            ]
        );
    }

    #[test]
    fn keeps_version_flag_without_default_subcommand() {
        let args = vec![OsString::from("actiona-run"), OsString::from("--version")];

        let args = maybe_insert_default_run(args);
        assert_eq!(args, vec!["actiona-run", "--version"]);
    }

    #[test]
    fn parses_version_flag() {
        let args = vec![OsString::from("actiona-run"), OsString::from("--version")];
        let args = maybe_insert_default_run(args);

        let err = Args::try_parse_from(args).expect_err("`--version` should trigger Clap exit");
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn parses_default_run_with_seed() {
        let args = vec![
            OsString::from("actiona-run"),
            OsString::from("--seed"),
            OsString::from("42"),
            OsString::from("script.ts"),
        ];
        let args = maybe_insert_default_run(args);

        let parsed = Args::try_parse_from(args).expect("parse args");

        match parsed.command {
            crate::args::Commands::Run { filepath, run_args } => {
                assert_eq!(filepath, std::path::PathBuf::from("script.ts"));
                assert_eq!(run_args.seed, Some(42));
            }
            other => panic!("expected run command, got {other:?}"),
        }
    }

    #[test]
    fn parses_eval_with_seed() {
        let parsed =
            Args::try_parse_from(["actiona-run", "eval", "--seed", "123", "console.log('hi')"])
                .expect("parse args");

        match parsed.command {
            crate::args::Commands::Eval { code, run_args } => {
                assert_eq!(run_args.seed, Some(123));
                assert_eq!(code, vec!["console.log('hi')"]);
            }
            other => panic!("expected eval command, got {other:?}"),
        }
    }

    #[test]
    fn parses_repl_with_seed() {
        let parsed =
            Args::try_parse_from(["actiona-run", "repl", "--seed", "99"]).expect("parse args");

        match parsed.command {
            crate::args::Commands::Repl {
                no_globals,
                run_args,
            } => {
                assert!(!no_globals);
                assert_eq!(run_args.seed, Some(99));
            }
            other => panic!("expected repl command, got {other:?}"),
        }
    }
}
