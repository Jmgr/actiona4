#![cfg_attr(windows, windows_subsystem = "console")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match run::run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if !err.is::<run::ScriptFailed>() {
                eprintln!("{err:?}");
            }
            ExitCode::FAILURE
        }
    }
}
