#[cfg(target_os = "windows")]
use std::{env, fs, path::PathBuf};

/// Embeds the nearest parent `assets/windows-manifest.xml`
/// into the binary via the MSVC linker's `/MANIFEST:EMBED` mechanism.
///
/// Call this from a Windows `build.rs` to ensure Common Controls v6 and DPI awareness
/// are enabled.
#[cfg(target_os = "windows")]
pub fn embed_windows_manifest() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set"));
    let source = manifest_dir
        .ancestors()
        .map(|dir| dir.join("assets/windows-manifest.xml"))
        .find(|path| path.exists())
        .unwrap_or_else(|| {
            panic!(
                "Failed to find assets/windows-manifest.xml above {}",
                manifest_dir.display()
            )
        });
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should be set");
    let dest = PathBuf::from(out_dir).join("windows.manifest");

    let contents = fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", source.display()));
    fs::write(&dest, contents)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", dest.display()));

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", dest.display());
}
