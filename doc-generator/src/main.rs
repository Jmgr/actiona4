use std::fs::read_to_string;

use clap::Parser;
use color_eyre::Result;
use rustdoc_types::Crate;
use types::File;

mod input;
pub mod items;
mod output;
mod types;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug, Parser)]
struct Args {
    /// Wrap all declarations in a `declare namespace actiona { ... }` block
    #[arg(long)]
    no_globals: bool,

    /// Path to the rustdoc JSON input file
    input: String,

    /// Path to the output .d.ts file
    output: String,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let json_string = read_to_string(&args.input)?;
    let crate_ = serde_json::from_str::<Crate>(&json_string)?;
    let mut file: File = crate_.try_into()?;
    file.auto_generate_overloads()?;
    file.fix_duplicate_parameter_names();
    file.write(&args.output, args.no_globals)?;

    Ok(())
}
