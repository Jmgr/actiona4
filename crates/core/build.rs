use std::{env, fs, path::PathBuf};

use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        linux: { all(unix, target_os = "linux") },
    }

    #[cfg(windows)]
    ensure_common_controls_v6_for_tests();

    write_notification_aumid();
    built::write_built_file().expect("Failed to acquire build-time information");
}

fn write_notification_aumid() {
    let manifest_directory =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set"));
    let cargo_toml_path = manifest_directory.join("Cargo.toml");
    let cargo_toml_contents =
        fs::read_to_string(&cargo_toml_path).expect("Failed to read core/Cargo.toml.");
    let cargo_toml: toml::Value =
        toml::from_str(&cargo_toml_contents).expect("Failed to parse core/Cargo.toml.");
    let notification_aumid = cargo_toml
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("metadata"))
        .and_then(toml::Value::as_table)
        .and_then(|metadata| metadata.get("actiona"))
        .and_then(toml::Value::as_table)
        .and_then(|actiona| actiona.get("notification-aumid"))
        .and_then(toml::Value::as_str)
        .expect("Missing package.metadata.actiona.notification-aumid in core/Cargo.toml.");

    println!("cargo:rustc-env=ACTIONA_NOTIFICATION_AUMID={notification_aumid}");
}

#[cfg(windows)]
fn ensure_common_controls_v6_for_tests() {
    let manifest_directory =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set"));
    let source_manifest_path = manifest_directory.join("../../assets/windows-manifest.xml");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));
    let manifest_path = out_dir.join("actiona_ng_tests.manifest");
    let manifest_content = fs::read_to_string(&source_manifest_path).unwrap_or_else(|error| {
        panic!(
            "Failed to read Windows manifest {}: {error}",
            source_manifest_path.display()
        )
    });

    if let Err(error) = fs::write(&manifest_path, manifest_content) {
        println!(
            "cargo:warning=Failed to write Windows test manifest {}: {}",
            manifest_path.display(),
            error
        );
        return;
    }

    println!("cargo:rerun-if-changed={}", source_manifest_path.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
        manifest_path.display()
    );
}
