#![cfg_attr(windows, windows_subsystem = "console")]

fn main() -> std::process::ExitCode {
    match run::run_cli() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            if !err.is::<run::ScriptFailed>() {
                eprintln!("{err:?}");
            }
            std::process::ExitCode::FAILURE
        }
    }
}
