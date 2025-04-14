#![windows_subsystem = "windows"]

use std::{fs, path::PathBuf};

use clap::Parser;
use code::{runtime::Runtime, ts_to_js::TsToJs};
use eyre::{Result, eyre};

#[derive(Debug, Parser)]
struct Args {
    filepath: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read the input file
    let ts = fs::read_to_string(args.filepath)?;

    // Convert the TS code into JS
    let to_to_js = TsToJs::new(&ts)?;

    /*
    Runtime::run(move |_runtime, js_context| async move {
        js_context.with(|ctx| {
            ctx.eval::<(), _>(to_to_js.code()).map_err(|_| {
                let e = ctx.catch();
                eprintln!("err {:?}", e); // TMP
                eyre!(
                    "{}",
                    e.as_exception()
                        .expect("caught value should be an exception")
                        .message()
                        .expect("exception should have a message")
                )
            })
        })?;

        Ok(())
    })?;
    */
    Runtime::run_without_ui(move |_runtime, js_context| async move {
        js_context.with(|ctx| {
            ctx.eval::<(), _>(to_to_js.code()).map_err(|_| {
                let e = ctx.catch();
                eprintln!("err {:?}", e); // TMP
                eyre!(
                    "{}",
                    e.as_exception()
                        .expect("caught value should be an exception")
                        .message()
                        .expect("exception should have a message")
                )
            })
        })?;

        Ok(())
    })
    .unwrap();

    Ok(())
}
