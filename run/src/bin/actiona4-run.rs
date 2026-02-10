#![cfg_attr(windows, windows_subsystem = "windows")]

use color_eyre::Result;

fn main() -> Result<()> {
    run::run_cli()
}
