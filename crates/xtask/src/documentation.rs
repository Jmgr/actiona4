use std::{path::Path, process::Command};

use color_eyre::Result;

use crate::util::run_command;

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

    tokio::fs::create_dir_all(&run_assets_directory).await?;
    tokio::fs::copy(&output_path, run_assets_directory.join("index.d.ts")).await?;

    Ok(())
}
