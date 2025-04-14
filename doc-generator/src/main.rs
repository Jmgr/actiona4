use std::{env, fs::read_to_string};

use eyre::{Result, eyre};
use rustdoc_types::Crate;
use types::File;

mod input;
mod output;
mod types;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() -> Result<()> {
    env_logger::init();

    let mut args = env::args();
    args.next();
    let input = args.next().ok_or(eyre!("expect input path"))?;
    let output = args.next().ok_or(eyre!("expect output path"))?;

    let json_string = read_to_string(input)?;
    let crate_ = serde_json::from_str::<Crate>(&json_string)?;
    let mut file: File = crate_.try_into()?;
    file.auto_generate_overloads()?;
    file.fix_duplicate_parameter_names();
    file.write(&output)?;

    Ok(())
}
