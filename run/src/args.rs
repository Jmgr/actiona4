use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 🤖 runs a script
    Run {
        /// file path to the script
        #[arg(value_hint = ValueHint::FilePath)]
        filepath: PathBuf,
    },

    /// 🧪 evaluates code
    Eval {
        /// the code to evaluate
        #[arg(trailing_var_arg = true)]
        code: Vec<String>,
    },

    /// 💻 starts the interactive terminal (REPL)
    Repl {
        /// Use the `actiona` namespace instead of globals
        #[arg(long)]
        no_globals: bool,
    },

    /// ⚙️ initializes a new script project
    Init {
        /// directory to initialize (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// 🐚 outputs shell completions to stdout
    Completions {
        /// the shell to generate completions for
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Parser)]
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
    pub disable_updates: Option<bool>,

    /// Should Actiona send anonymous telemetry data?
    /// Default is false.
    #[arg(long, env, default_value_t = true)]
    pub disable_telemetry: bool,

    #[cfg(unix)]
    #[arg(long, env, help = "X11 display to use (Linux/X11 only 🐧)")]
    pub display: Option<String>,
}
