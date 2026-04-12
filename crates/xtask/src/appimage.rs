use std::{
    env,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::{Result, eyre::eyre};

use crate::workspace::workspace_root;

const APPIMAGETOOL_URL_BASE: &str =
    "https://github.com/AppImage/appimagetool/releases/download/continuous";

const APPRUN: &str = "\
#!/bin/sh
exec \"$(dirname \"$0\")/usr/bin/actiona-run\" \"$@\"
";

pub async fn build_appimage(workspace_root: &Path, sign: bool) -> Result<()> {
    let release_dir = workspace_root.join("target").join("release");
    let appimage_dir = workspace_root.join("target");
    let app_dir = appimage_dir.join("AppDir");

    let version = read_version(workspace_root).await?;
    let arch = appimage_arch()?;
    let output_path = appimage_dir.join(format!("actiona-run-{version}-{arch}.AppImage"));

    std::fs::create_dir_all(&appimage_dir)?;
    assemble_app_dir(workspace_root, &release_dir, &app_dir)?;

    let appimagetool = ensure_appimagetool(arch).await?;
    run_appimagetool(&appimagetool, &app_dir, &output_path, sign)?;

    println!("AppImage written to: {}", output_path.display());

    Ok(())
}

fn assemble_app_dir(workspace_root: &Path, release_dir: &Path, app_dir: &Path) -> Result<()> {
    let bin_dir = app_dir.join("usr").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    copy_binary(release_dir, &bin_dir, "actiona-run")?;
    copy_binary(release_dir, &bin_dir, "selection-tool")?;

    let icon_src = workspace_root
        .join("crates")
        .join("core")
        .join("icons")
        .join("icon.png");
    std::fs::copy(&icon_src, app_dir.join("actiona-run.png"))?;

    let desktop_src = workspace_root.join("assets").join("actiona-run.desktop");
    if !desktop_src.exists() {
        return Err(eyre!("Desktop file not found: {}", desktop_src.display()));
    }
    std::fs::copy(&desktop_src, app_dir.join("actiona-run.desktop"))?;

    let apprun_path = app_dir.join("AppRun");
    std::fs::write(&apprun_path, APPRUN)?;
    std::fs::set_permissions(&apprun_path, std::fs::Permissions::from_mode(0o755))?;

    Ok(())
}

fn copy_binary(release_dir: &Path, bin_dir: &Path, name: &str) -> Result<()> {
    let src = release_dir.join(name);
    if !src.exists() {
        return Err(eyre!(
            "Binary not found: {}. Run `cargo make release` first.",
            src.display()
        ));
    }
    let dst = bin_dir.join(name);
    std::fs::copy(&src, &dst)?;
    std::fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

async fn ensure_appimagetool(arch: &str) -> Result<PathBuf> {
    if let Ok(path) = which_appimagetool() {
        return Ok(path);
    }

    let tools_dir = workspace_root()?.join("target").join("tools");
    std::fs::create_dir_all(&tools_dir)?;
    let tool_path = tools_dir.join(format!("appimagetool-{arch}.AppImage"));

    if !tool_path.exists() {
        download_appimagetool(arch, &tool_path).await?;
    }

    Ok(tool_path)
}

fn which_appimagetool() -> Result<PathBuf> {
    let output = Command::new("which").arg("appimagetool").output()?;
    if output.status.success() {
        let path = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok(PathBuf::from(path))
    } else {
        Err(eyre!("appimagetool not found on PATH"))
    }
}

async fn download_appimagetool(arch: &str, dest: &Path) -> Result<()> {
    let url = format!("{APPIMAGETOOL_URL_BASE}/appimagetool-{arch}.AppImage");
    eprintln!("Downloading appimagetool from {url}...");

    let bytes = reqwest::get(&url)
        .await
        .map_err(|error| eyre!("Failed to download appimagetool: {error}"))?
        .error_for_status()
        .map_err(|error| eyre!("appimagetool download failed: {error}"))?
        .bytes()
        .await
        .map_err(|error| eyre!("Failed to read appimagetool response: {error}"))?;

    tokio::fs::write(dest, &bytes).await?;
    tokio::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755)).await?;

    Ok(())
}

fn run_appimagetool(
    tool_path: &Path,
    app_dir: &Path,
    output_path: &Path,
    sign: bool,
) -> Result<()> {
    // APPIMAGE_EXTRACT_AND_RUN=1 avoids the FUSE requirement — appimagetool
    // extracts itself to a temp dir and runs without needing /dev/fuse mounted.
    // This works in CI and any environment where FUSE is not available.
    let mut cmd = Command::new(tool_path);
    cmd.arg(app_dir)
        .arg(output_path)
        .env("ARCH", appimage_arch()?)
        .env(
            "APPIMAGE_EXTRACT_AND_RUN",
            env::var("APPIMAGE_EXTRACT_AND_RUN").unwrap_or_else(|_| "1".to_owned()),
        );

    if sign {
        cmd.arg("--sign");
        // If ACTIONA_GPG_KEY is set, use that specific key (e.g. a dedicated
        // signing subkey fingerprint). Otherwise appimagetool uses gpg's
        // default key.
        if let Ok(key) = env::var("ACTIONA_GPG_KEY") {
            cmd.arg("--sign-key").arg(key);
        }
    }

    let status = cmd
        .status()
        .map_err(|error| eyre!("Failed to run appimagetool: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(eyre!("appimagetool exited with status {status}"))
    }
}

fn appimage_arch() -> Result<&'static str> {
    match env::consts::ARCH {
        "x86_64" => Ok("x86_64"),
        "aarch64" => Ok("aarch64"),
        "x86" => Ok("i686"),
        "arm" => Ok("armhf"),
        other => Err(eyre!("Unsupported architecture for AppImage: {other}")),
    }
}

async fn read_version(workspace_root: &Path) -> Result<String> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let contents = tokio::fs::read_to_string(&cargo_toml_path).await?;
    let value: toml::Value = toml::from_str(&contents)?;

    value
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| eyre!("Failed to read workspace.package.version from Cargo.toml."))
}
