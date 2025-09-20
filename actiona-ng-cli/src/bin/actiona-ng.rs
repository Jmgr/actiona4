#![windows_subsystem = "windows"]

use std::{fs, path::PathBuf};

use actiona_ng::runtime::Runtime;
use clap::Parser;
use eyre::Result;

#[derive(Debug, Parser)]
struct Args {
    filepath: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read the input file
    let script = fs::read_to_string(args.filepath)?;

    Runtime::run(async move |_runtime, script_engine| {
        script_engine.eval_async::<()>(&script).await?;
        Ok(())
    })?;

    Ok(())
}
