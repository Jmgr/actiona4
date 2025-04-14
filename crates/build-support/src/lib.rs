#[cfg(target_os = "windows")]
use std::{env, fs, path::Path};

/// Embeds `assets/windows-manifest.xml` (two directories above `CARGO_MANIFEST_DIR`)
/// into the binary via the MSVC linker's `/MANIFEST:EMBED` mechanism.
///
/// Call this from a Windows `build.rs` to ensure Common Controls v6 and DPI awareness
/// are enabled.
#[cfg(target_os = "windows")]
pub fn embed_windows_manifest() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");
    let source = Path::new(&manifest_dir).join("../../assets/windows-manifest.xml");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should be set");
    let dest = Path::new(&out_dir).join("windows.manifest");

    let contents = fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", source.display()));
    fs::write(&dest, contents)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", dest.display()));

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", dest.display());
}
