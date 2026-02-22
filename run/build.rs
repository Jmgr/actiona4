use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use cfg_aliases::cfg_aliases;
use flate2::{Compression, write::GzEncoder};

include!("src/args.rs");

fn build_compressed_index_dts() {
    let input_path = Path::new("assets").join("index.d.ts");
    let output_path = std::path::PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"))
        .join("index.d.ts.gz");

    let input = fs::read(&input_path).expect("Failed to read assets/index.d.ts");
    let output_file = File::create(&output_path)
        .unwrap_or_else(|err| panic!("Failed to create {}: {err}", output_path.display()));
    let mut encoder = GzEncoder::new(output_file, Compression::best());
    encoder
        .write_all(&input)
        .expect("Failed to write compressed index.d.ts");
    encoder.finish().expect("Failed to finish gzip stream");

    println!("cargo:rerun-if-changed={}", input_path.display());
}

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }

    build_compressed_index_dts();

    #[cfg(windows)]
    tauri_build::try_build(
        tauri_build::Attributes::new().windows_attributes(
            tauri_build::WindowsAttributes::new()
                .app_manifest(include_str!("windows-manifest.xml")),
        ),
    )
    .expect("tauri build failed");

    #[cfg(not(windows))]
    tauri_build::build();

    built::write_built_file().expect("Failed to acquire build-time information");

    println!("cargo:rerun-if-changed=src/args.rs");
}
