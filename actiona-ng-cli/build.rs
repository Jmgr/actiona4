use std::{env, fs};

use cfg_aliases::cfg_aliases;
use clap::CommandFactory;
use clap_complete::{Shell, generate_to};

include!("src/args.rs");

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }

    tauri_build::build();

    built::write_built_file().expect("Failed to acquire build-time information");

    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("completions");

    fs::create_dir_all(&out_dir).unwrap();

    let mut cmd = Args::command();
    let bin_name = "actiona-ng-cli";

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        generate_to(shell, &mut cmd, bin_name, &out_dir).expect("failed to generate completion");
    }

    println!("cargo:rerun-if-changed=src/args.rs");
}
