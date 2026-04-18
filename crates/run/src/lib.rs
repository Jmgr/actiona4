#![warn(clippy::all, clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::future_not_send)]
#![allow(clippy::too_many_arguments)]
#![allow(rustdoc::invalid_html_tags)]

use std::{
    env,
    ffi::OsString,
    io::{IsTerminal, stdin},
    path::Path,
    sync::Arc,
};

use ::config::{
    Config,
    settings::{DEFAULT_TELEMETRY, DEFAULT_UPDATE_CHECK},
};
use actiona_common::sentry::setup_crash_reporting;
use actiona_core::{
    format_js_value_for_console,
    runtime::{Runtime, RuntimeOptions, RuntimePlatformSetup, WaitAtEnd},
    scripting::{self},
};
use clap::{CommandFactory, Parser, error::ErrorKind};
use color_eyre::{
    Result,
    config::HookBuilder,
    eyre::{Context, OptionExt, eyre},
};
use dialoguer::{Confirm, theme::ColorfulTheme};
use installer_tools::path::{PathScope, add_directory_to_path, remove_directory_from_path};
#[cfg(not(windows))]
use rfd::{MessageButtons, MessageLevel};
use tracing_subscriber::{
    EnvFilter, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};
#[cfg(windows)]
use windows::{
    Wdk::System::SystemServices::RtlGetVersion, Win32::System::SystemInformation::OSVERSIONINFOW,
};

use crate::{
    args::{Args, Commands, CrashType, MacrosCommands},
    repl::repl,
    updates::{check_updates, check_updates_now},
};

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod args;
mod config;
mod init;
mod macros;
mod repl;
mod updates;

#[cfg(windows)]
fn is_windows10_1607_or_newer() -> Option<bool> {
    const WINDOWS_1607_BUILD: u32 = 14393;

    let mut info = OSVERSIONINFOW::default();
    unsafe { RtlGetVersion(&mut info).ok().ok()? };

    Some(info.dwBuildNumber >= WINDOWS_1607_BUILD)
}

/// Returned when a script fails. The diagnostic has already been printed;
/// callers must not print this error again.
#[derive(Debug)]
pub struct ScriptFailed;

impl std::fmt::Display for ScriptFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("script failed")
    }
}

impl std::error::Error for ScriptFailed {}

pub const NO_ARGS_MESSAGE: &str = "Actiona Run is a command-line tool.\n\nUse it from a terminal, for example:\n  actiona-run script.ts\n  actiona-run repl\n\nFor full help, run:\n  actiona-run --help";

pub fn run_cli() -> Result<()> {
    let _guard = setup_crash_reporting(built_info::PKG_NAME)?;

    init_tracing();

    // When the OS re-launches this binary as a deep-link handler (e.g. Snipping
    // Tool redirecting to `actiona-run://…`), forward the URL to the running
    // first instance and exit before arg parsing attempts to interpret the URI.
    #[cfg(windows)]
    if Runtime::relay_deep_link_if_needed() {
        return Ok(());
    }

    // When launched without arguments outside a terminal (e.g. double-clicking
    // the AppImage on Linux), show a dialog explaining this is a CLI tool.
    // On Windows this is handled by actiona-runw.exe before actiona-run.exe runs.
    #[cfg(not(windows))]
    if std::env::args_os().len() == 1 && !stdin().is_terminal() {
        rfd::MessageDialog::new()
            .set_title("Actiona Run")
            .set_description(NO_ARGS_MESSAGE)
            .set_level(MessageLevel::Info)
            .set_buttons(MessageButtons::Ok)
            .show();
        return Ok(());
    }

    let args = parse_args_with_default_run();

    // Handle crash-test before run_cli_with_args to avoid the X11 check and
    // runtime setup — we only need the crash handler to be in place.
    if let args::Commands::CrashTest { crash_type } = args.command {
        trigger_crash(crash_type);
    }

    run_cli_with_args(args)
}

#[cfg(unix)]
fn effective_x11_display<'a>(
    cli_display_name: Option<&'a str>,
    env_display_name: Option<&'a str>,
) -> Option<&'a str> {
    cli_display_name
        .filter(|display_name| !display_name.trim().is_empty())
        .or_else(|| env_display_name.filter(|display_name| !display_name.trim().is_empty()))
}

#[cfg(unix)]
fn ensure_x11_session_available(
    cli_display_name: Option<&str>,
    env_display_name: Option<&str>,
) -> Result<()> {
    let display_name =
        effective_x11_display(cli_display_name, env_display_name).ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "No X11 session is available. Set the DISPLAY environment variable or pass --display to an active X11 server."
            )
        })?;

    actiona_core::runtime::ensure_x11_session_available(Some(display_name)).with_context(|| {
        format!(
            "No X11 session is available on display `{display_name}`. Start an X11 session, or pass --display to a reachable X11 server."
        )
    })?;

    Ok(())
}

async fn setup(config: &Config, current_path: &Path) -> Result<()> {
    println!("*** First time setup ***");

    let path_added = if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add actiona-run to the PATH?")
        .default(true)
        .show_default(true)
        .interact()?
    {
        add_directory_to_path(PathScope::User, &current_path.to_string_lossy())?
    } else {
        remove_directory_from_path(PathScope::User, &current_path.to_string_lossy())?;
        false
    };

    let mut update_check = config.settings(|settings| settings.update_check);
    update_check = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Enable automatic update checks?")
        .default(update_check)
        .show_default(true)
        .interact()?;
    config
        .settings_mut(|settings| settings.update_check = update_check)
        .await?;

    let mut telemetry = config.settings(|settings| settings.telemetry.is_some());
    telemetry = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Help us improve Actiona (anonymous usage data)?")
        .default(telemetry)
        .show_default(true)
        .interact()?;
    config
        .settings_mut(|settings| settings.set_telemetry(telemetry))
        .await?;

    if path_added {
        #[cfg(unix)]
        println!(
            "To use actiona-run in this session, run:\n  export PATH=\"${{PATH}}:{}\"",
            current_path.display()
        );
        #[cfg(windows)]
        println!("Please restart your terminal for the PATH change to take effect.");
    }

    println!("Setup finished");

    Ok(())
}

fn run_cli_with_args(args: Args) -> Result<()> {
    let args = Arc::new(args);

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
                "Warning: Unable to determine your version of Windows. Actiona Run is only supported on Windows 10 1607 or newer."
            )
        }
    }

    let show_tray_icon = args.command.is_run() || args.command.is_eval();
    let platform = RuntimePlatformSetup::new(show_tray_icon)?;

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    tokio_runtime.block_on(async move {
        let config = Config::new().await?;

        // Perform first-time setup if needed
        if stdin().is_terminal()
            && !args.command.is_completions()
            && !args.command.is_setup()
            && !config.state(|state| state.first_time_init)
        {
            let current_path =
                env::current_exe().context("finding current executable directory")?;
            let current_path = current_path
                .parent()
                .ok_or_eyre(eyre!("failed to get the current directory"))?;

            setup(&config, current_path).await?;
        }

        // Handle commands that don't need the runtime
        match &args.command {
            Commands::Init { path } => return init::run(path),
            Commands::Config { key, value } => {
                return config::run(&config, key, *value).await;
            }
            Commands::Update => {
                return check_updates_now(&config).await;
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
            Commands::Setup => {
                let current_path =
                    env::current_exe().context("finding current executable directory")?;
                let current_path = current_path
                    .parent()
                    .ok_or_eyre(eyre!("failed to get the current directory"))?;

                if !stdin().is_terminal() {
                    // We are not in an interactive terminal, so just set the defaults.
                    add_directory_to_path(PathScope::User, &current_path.to_string_lossy())?;
                    config
                        .settings_mut(|settings| {
                            settings.update_check = DEFAULT_UPDATE_CHECK;
                            settings.set_telemetry(DEFAULT_TELEMETRY);
                        })
                        .await?;

                    return Ok(());
                }

                setup(&config, current_path).await?;

                return Ok(());
            }
            _ => {}
        }

        #[cfg(unix)]
        ensure_x11_session_available(
            args.display.as_deref(),
            std::env::var("DISPLAY").ok().as_deref(),
        )?;

        let runtime_options = RuntimeOptions {
            #[cfg(unix)]
            display_name: args.display.clone(),
            install_ctrl_c_handler: !args.command.is_repl(),
            show_tray_icon,
            seed: seed_from_command(&args.command),
        };

        Runtime::run(
            platform,
            move |runtime, script_engine| async move {
                check_updates(
                    &args,
                    &config,
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
                        {
                            if !scripting::try_emit_script_diagnostic(&err, &script) {
                                eprintln!("Error: {err}");
                            }
                            return Err(ScriptFailed.into());
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
                            Err(err) => {
                                if !scripting::try_emit_script_diagnostic(&err, &code) {
                                    eprintln!("Error: {err}");
                                }
                                return Err(ScriptFailed.into());
                            }
                        }
                    }
                    Commands::Repl { .. } => {
                        runtime.set_wait_at_end(WaitAtEnd::No);

                        repl(script_engine, runtime.cancellation_token()).await?;
                    }
                    Commands::Macros { command } => match command {
                        MacrosCommands::Record {
                            file,
                            stop_keys,
                            timeout,
                            mouse_position_interval,
                            filter,
                        } => {
                            macros::run_record(
                                runtime,
                                file,
                                stop_keys,
                                timeout.as_deref(),
                                mouse_position_interval,
                                !filter.no_mouse_buttons,
                                !filter.no_mouse_position,
                                !filter.no_mouse_scroll,
                                !filter.no_keyboard_keys,
                            )
                            .await?;
                        }
                        MacrosCommands::Play {
                            file,
                            speed,
                            relative_mouse_position,
                            filter,
                        } => {
                            macros::run_play(
                                runtime,
                                file,
                                *speed,
                                !filter.no_mouse_buttons,
                                !filter.no_mouse_position,
                                *relative_mouse_position,
                                !filter.no_mouse_scroll,
                                !filter.no_keyboard_keys,
                            )
                            .await?;
                        }
                    },
                    Commands::Init { .. }
                    | Commands::Update
                    | Commands::Completions { .. }
                    | Commands::Config { .. }
                    | Commands::CrashTest { .. }
                    | Commands::Setup => {
                        unreachable!("handled before runtime startup")
                    }
                };

                Ok(())
            },
            runtime_options,
        )
        .await?;

        Ok(())
    })
}

fn parse_args_with_default_run() -> Args {
    try_parse_args_with_default_run().unwrap_or_else(|error| error.exit())
}

fn try_parse_args_with_default_run() -> clap::error::Result<Args> {
    try_parse_args_with_default_run_from(std::env::args_os().collect())
}

fn try_parse_args_with_default_run_from(args: Vec<OsString>) -> clap::error::Result<Args> {
    let args = maybe_insert_default_run(args);

    match Args::try_parse_from(&args) {
        Err(error) if should_render_help_for_missing_subcommand(&args, &error) => {
            let mut command = Args::command();
            let help = command.render_help().to_string();

            Err(command.error(ErrorKind::DisplayHelp, help))
        }
        result => result,
    }
}

fn should_render_help_for_missing_subcommand(args: &[OsString], error: &clap::Error) -> bool {
    args.len() == 1 && error.kind() == ErrorKind::MissingSubcommand
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

const fn seed_from_command(command: &Commands) -> Option<u64> {
    use Commands::*;
    match command {
        Run { run_args, .. } => run_args.seed,
        Eval { run_args, .. } => run_args.seed,
        Repl { run_args } => run_args.seed,
        Init { .. }
        | Update
        | Completions { .. }
        | Config { .. }
        | Macros { .. }
        | Setup
        | CrashTest { .. } => todo!(),
    }
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

#[allow(unsafe_code)]
fn trigger_crash(crash_type: CrashType) -> ! {
    let flavor = match crash_type {
        CrashType::Abort => sadness_generator::SadnessFlavor::Abort,
        CrashType::Segfault => sadness_generator::SadnessFlavor::Segfault,
        #[cfg(unix)]
        CrashType::Bus => sadness_generator::SadnessFlavor::Bus,
        CrashType::DivideByZero => sadness_generator::SadnessFlavor::DivideByZero,
        CrashType::Illegal => sadness_generator::SadnessFlavor::Illegal,
        CrashType::Trap => sadness_generator::SadnessFlavor::Trap,
        CrashType::StackOverflow => sadness_generator::SadnessFlavor::StackOverflow {
            non_rust_thread: false,
            long_jumps: false,
        },
        CrashType::Panic => panic!("*test* panic, don't panic ;)"),
    };
    // SAFETY: intentionally crashes the process for testing purposes.
    unsafe { flavor.make_sad() }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn,enigo::platform::x11=off,minidump_writer=error"));

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
    #[cfg(unix)]
    use color_eyre::Result;

    use super::maybe_insert_default_run;
    #[cfg(unix)]
    use super::{effective_x11_display, ensure_x11_session_available};
    use crate::args::Args;

    #[cfg(unix)]
    #[test]
    fn x11_preflight_fails_with_clear_message_when_no_display_is_available() -> Result<()> {
        let error = ensure_x11_session_available(None, None).unwrap_err();

        assert_eq!(
            error.to_string(),
            "No X11 session is available. Set the DISPLAY environment variable or pass --display to an active X11 server."
        );

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn cli_display_takes_priority_over_display_environment_variable() {
        assert_eq!(effective_x11_display(Some(":42"), Some(":0")), Some(":42"));
    }

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
            crate::args::Commands::Repl { run_args } => {
                assert_eq!(run_args.seed, Some(99));
            }
            other => panic!("expected repl command, got {other:?}"),
        }
    }
}
