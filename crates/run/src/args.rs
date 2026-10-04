use std::path::PathBuf;

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum, ValueHint};
use strum::EnumIs;

#[derive(ClapArgs, Clone, Debug)]
pub struct MacroInputFilter {
    /// do not record/replay mouse button press/release events
    #[arg(long)]
    pub no_mouse_buttons: bool,

    /// do not record/replay mouse cursor position
    #[arg(long)]
    pub no_mouse_position: bool,

    /// do not record/replay mouse scroll wheel events
    #[arg(long)]
    pub no_mouse_scroll: bool,

    /// do not record/replay keyboard key press/release events
    #[arg(long)]
    pub no_keyboard_keys: bool,
}

/// 🔴 Records and replays input macros.
///
/// Examples:
/// - `actiona-run macros record recording.amac`
/// - `actiona-run macros play recording.amac --speed 2.0`
#[derive(Debug, Subcommand)]
#[command(verbatim_doc_comment)]
pub enum MacrosCommands {
    /// Records user input and saves the macro to a file
    ///
    /// Examples:
    /// - `actiona-run macros record recording.amac`
    /// - `actiona-run macros record recording.amac --stop-key Escape --stop-key Control`
    /// - `actiona-run macros record recording.amac --timeout 30s --no-mouse-position`
    #[command(verbatim_doc_comment)]
    Record {
        /// file path to save the recorded macro to
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,

        /// key that stops recording; can be specified multiple times (all listed keys must be
        /// pressed simultaneously to stop). Key names: Escape, Control, Alt, Shift, Space, etc.
        #[arg(long = "stop-key", value_name = "KEY", default_values = ["Escape"])]
        stop_keys: Vec<String>,

        /// maximum recording duration before stopping automatically (e.g. 30s, 1m, 2m30s)
        #[arg(long)]
        timeout: Option<String>,

        /// how often to sample the mouse cursor position (e.g. 16ms, 33ms)
        #[arg(long, default_value = "16ms")]
        mouse_position_interval: String,

        #[command(flatten)]
        filter: MacroInputFilter,
    },

    /// Replays a macro from a file
    ///
    /// Examples:
    /// - `actiona-run macros play recording.amac`
    /// - `actiona-run macros play recording.amac --speed 2.0`
    /// - `actiona-run macros play recording.amac --relative-mouse-position`
    #[command(verbatim_doc_comment)]
    Play {
        /// file path of the macro to replay
        #[arg(value_hint = ValueHint::FilePath)]
        file: PathBuf,

        /// playback speed multiplier (1.0 = real-time, 2.0 = twice as fast)
        #[arg(long, default_value = "1.0")]
        speed: f64,

        /// replay mouse movements relative to the current cursor position instead of the
        /// absolute screen coordinates that were recorded
        #[arg(long)]
        relative_mouse_position: bool,

        #[command(flatten)]
        filter: MacroInputFilter,
    },
}

#[derive(ClapArgs, Clone, Debug)]
pub struct RunArgs {
    /// Seed the random number generator for deterministic runs
    #[arg(long)]
    pub seed: Option<u64>,
}

/// Options for commands that run a script in the background (`run` and `eval`), as opposed to
/// the interactive REPL.
#[derive(ClapArgs, Clone, Debug)]
pub struct ScriptArgs {
    /// Do not display the system tray icon (the script can then only be stopped with the stop
    /// hotkey, Ctrl+C or by ending the process)
    #[arg(long)]
    pub no_tray: bool,

    /// Global hotkey that stops the script, and any other running script using the same hotkey
    /// (e.g. Ctrl+Alt+Shift+Q, Ctrl+F12); defaults to the `stop_hotkey` setting, or
    /// Ctrl+Alt+Shift+Q
    #[arg(long, value_name = "HOTKEY")]
    pub stop_hotkey: Option<String>,

    /// Do not register the global stop hotkey
    #[arg(long, conflicts_with = "stop_hotkey")]
    pub no_stop_hotkey: bool,
}

/// Run Actiona 4 automation scripts from the command line.
///
/// Examples:
/// - `actiona-run ./scripts/hello.ts`
/// - `actiona-run eval "console.log('hello')"`
/// - `actiona-run repl`
/// - `actiona-run init ./my-script`
#[derive(Debug, EnumIs, Subcommand)]
#[command(verbatim_doc_comment)]
pub enum Commands {
    /// 🤖 runs a script (default)
    Run {
        /// file path to the script
        #[arg(value_hint = ValueHint::FilePath)]
        filepath: PathBuf,

        #[command(flatten)]
        run_args: RunArgs,

        #[command(flatten)]
        script_args: ScriptArgs,
    },

    /// 🧪 evaluates code
    Eval {
        /// the code to evaluate
        #[arg(trailing_var_arg = true)]
        code: Vec<String>,

        #[command(flatten)]
        run_args: RunArgs,

        #[command(flatten)]
        script_args: ScriptArgs,
    },

    /// 💻 starts the interactive terminal (REPL)
    Repl {
        #[command(flatten)]
        run_args: RunArgs,
    },

    /// ⚙️ initializes a new script project
    Init {
        /// directory to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// 🔄 checks for updates now
    Update,

    /// 🐚 outputs shell completions to stdout
    Completions {
        /// the shell to generate completions for
        shell: clap_complete::Shell,
    },

    /// ⚙️ gets or sets configuration values
    ///
    /// Examples:
    /// - `actiona-run config update_check true`
    /// - `actiona-run config telemetry false`
    /// - `actiona-run config stop_hotkey Ctrl+Shift+F12` (`none` disables it, `default` resets it)
    /// - `actiona-run config update_check` (prints current value)
    #[command(verbatim_doc_comment)]
    Config {
        /// the setting name
        key: ConfigKey,

        /// the value to set; omit to read the current value
        value: Option<String>,
    },

    /// 🔴 records and replays input macros
    Macros {
        #[command(subcommand)]
        command: MacrosCommands,
    },

    /// ⚙️ performs application setup
    Setup,

    /// 💥 triggers a synthetic crash to test crash dump collection (hidden, for development use)
    #[command(hide = true)]
    CrashTest {
        /// The type of crash to trigger
        #[arg(long, value_enum, default_value = "segfault")]
        crash_type: CrashType,
    },
}

/// Settings that `actiona-run config` can read and change.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum ConfigKey {
    /// check for updates once per day (true or false)
    UpdateCheck,
    /// send anonymous telemetry data (true or false)
    Telemetry,
    /// global hotkey that stops running scripts (e.g. Ctrl+Alt+Shift+Q, none, default)
    StopHotkey,
}

/// Crash types available via `crash-test --crash-type`.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CrashType {
    Abort,
    Segfault,
    #[cfg(unix)]
    Bus,
    DivideByZero,
    Illegal,
    Trap,
    StackOverflow,
    Panic,
}

#[derive(Debug, Parser)]
#[command(
    name = "actiona-run",
    version,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Show debug information
    #[cfg(debug_assertions)]
    #[arg(long, default_value_t = true)]
    pub debug: bool,

    /// Show debug information
    #[cfg(not(debug_assertions))]
    #[arg(long, default_value_t = false)]
    pub debug: bool,

    /// Should Actiona check for updates once per day?
    /// Default is true.
    #[arg(long, env)]
    pub update_check: Option<bool>,

    /// Should Actiona send anonymous telemetry data?
    /// Default is false.
    #[arg(long, env, default_value_t = true)]
    pub disable_telemetry: bool,

    #[cfg(unix)]
    #[arg(long, env, help = "X11 display to use (Linux/X11 only 🐧)")]
    pub display: Option<String>,
}
