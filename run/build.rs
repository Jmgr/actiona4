use cfg_aliases::cfg_aliases;

include!("src/args.rs");

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }

    tauri_build::build();

    built::write_built_file().expect("Failed to acquire build-time information");

    println!("cargo:rerun-if-changed=src/args.rs");
}
