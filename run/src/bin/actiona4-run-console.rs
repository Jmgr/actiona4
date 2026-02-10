#![cfg_attr(windows, windows_subsystem = "console")]

use color_eyre::Result;

fn main() -> Result<()> {
    run::run_cli()
}
