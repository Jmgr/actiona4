use std::{path::Path, process::Command};

use color_eyre::{Result, eyre::eyre};
use tempfile::tempdir;

use crate::util::run_command;

fn validate_typescript_declarations(workspace_root: &Path, path: &Path) -> Result<()> {
    let e2e_dir = workspace_root.join("crates/e2e");
    let base_tsconfig_path = workspace_root.join("crates/run/assets/tsconfig.json");
    let tsc_path = if cfg!(windows) {
        e2e_dir.join("node_modules/.bin/tsc.cmd")
    } else {
        e2e_dir.join("node_modules/.bin/tsc")
    };

    if !tsc_path.exists() {
        return Err(eyre!(
            "TypeScript compiler not found at {}. Run `npm ci --prefix crates/e2e` first.",
            tsc_path.display()
        ));
    }

    let temp_dir = tempdir()?;
    let tsconfig_path = temp_dir.path().join("tsconfig.json");
    let tsconfig = serde_json::json!({
        "extends": base_tsconfig_path,
        "files": [path]
    });
    std::fs::write(&tsconfig_path, serde_json::to_vec_pretty(&tsconfig)?)?;

    run_command(
        Command::new(&tsc_path)
            .arg("-p")
            .arg(&tsconfig_path)
            .current_dir(workspace_root),
        "Generated TypeScript declarations failed TypeScript validation.",
    )
}

pub async fn generate_docs(workspace_root: &Path) -> Result<()> {
    let rustdoc_json_path = workspace_root.join("target/doc/actiona_core.json");
    let output_path = workspace_root.join("target/doc/index.d.ts");
    let run_assets_directory = workspace_root.join("crates/run/assets");

    run_command(
        Command::new("cargo")
            .arg("+nightly")
            .arg("rustdoc")
            .arg("--package")
            .arg("core")
            .arg("--lib")
            .arg("--")
            .arg("--output-format")
            .arg("json")
            .arg("-Z")
            .arg("unstable-options")
            .current_dir(workspace_root),
        "Failed to generate rustdoc JSON for the core crate.",
    )?;

    run_command(
        Command::new("cargo")
            .arg("run")
            .arg("--package")
            .arg("doc-generator")
            .arg("--")
            .arg(&rustdoc_json_path)
            .arg(&output_path)
            .current_dir(workspace_root),
        "Failed to generate TypeScript declarations.",
    )?;

    validate_typescript_declarations(workspace_root, &output_path)?;

    tokio::fs::create_dir_all(&run_assets_directory).await?;
    tokio::fs::copy(&output_path, run_assets_directory.join("index.d.ts")).await?;

    Ok(())
}
