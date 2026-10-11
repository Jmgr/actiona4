use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::{Result, eyre::eyre};
use installer_tools::package::{PackageKind, PackagedFilePlatform, packaged_files};
use tokio::fs as tokio_fs;

use crate::package_docs::stage_packaged_files;

const LINUXDEPLOY_URL_BASE: &str =
    "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous";
const LINUXDEPLOY_PLUGIN_APPIMAGE_URL_BASE: &str =
    "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous";
const WEBKIT_PACKAGE: &str = "webkitgtk-6.0";
const WEBKIT_INJECTED_BUNDLE_DIR: &str = "injected-bundle";
const WEBKIT_RUNTIME_FILES: [&str; 4] = [
    "WebKitGPUProcess",
    "WebKitNetworkProcess",
    "WebKitWebProcess",
    "injected-bundle/libwebkitgtkinjectedbundle.so",
];

/// The WebKit helper processes and injected bundle, which `libwebkitgtk` loads from a directory
/// compiled into it rather than through the dynamic linker.
struct WebkitRuntime {
    /// Absolute host directory, e.g. `/usr/lib/x86_64-linux-gnu/webkitgtk-6.0`.
    dir: PathBuf,
    files: Vec<PathBuf>,
}

impl WebkitRuntime {
    /// Where the runtime is staged, relative to the AppDir root.
    fn relative_dir(&self) -> Result<&Path> {
        self.dir.strip_prefix("/").map_err(|_| {
            eyre!(
                "WebKit runtime directory must be absolute: {}",
                self.dir.display()
            )
        })
    }
}

pub async fn build_appimages(workspace_root: &Path, sign: bool) -> Result<()> {
    let appimage_dir = workspace_root.join("target");
    let tools_dir = appimage_dir.join("tools");
    let version = read_version(workspace_root).await?;
    let arch = appimage_arch()?;

    fs::create_dir_all(&appimage_dir)?;
    fs::create_dir_all(&tools_dir)?;

    let linuxdeploy = ensure_linuxdeploy(arch, &tools_dir).await?;
    let appimage_plugin = ensure_linuxdeploy_appimage_plugin(arch, &tools_dir).await?;

    for package_kind in PackageKind::ALL {
        build_appimage(
            workspace_root,
            &linuxdeploy,
            &appimage_plugin,
            &tools_dir,
            &version,
            arch,
            package_kind,
            sign,
        )
        .await?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn build_appimage(
    workspace_root: &Path,
    linuxdeploy: &Path,
    appimage_plugin: &Path,
    tools_dir: &Path,
    version: &str,
    arch: &str,
    package_kind: PackageKind,
    sign: bool,
) -> Result<()> {
    let release_dir = workspace_root.join("target").join("release");
    let appimage_dir = workspace_root.join("target");
    let app_dir = appimage_dir.join("AppDir");
    let docs_dir = app_dir
        .join("usr")
        .join("share")
        .join("doc")
        .join(format!("actiona-{}", package_kind.artifact_name()));
    let metainfo_dir = app_dir.join("usr").join("share").join("metainfo");

    let output_path = appimage_dir.join(format!(
        "actiona-{}-{version}-{arch}.AppImage",
        package_kind.artifact_name()
    ));
    let packaged_files: Vec<_> = packaged_files(workspace_root, package_kind)?
        .into_iter()
        .filter(|packaged_file| packaged_file.include_in_appimage)
        .collect();

    reset_app_dir(&app_dir)?;
    stage_packaged_files(
        workspace_root,
        &docs_dir,
        &packaged_files,
        PackagedFilePlatform::Linux,
    )
    .await?;
    stage_appstream_metainfo(workspace_root, &metainfo_dir, package_kind)?;
    remove_output_if_exists(&output_path)?;

    run_linuxdeploy(
        linuxdeploy,
        appimage_plugin,
        tools_dir,
        workspace_root,
        &release_dir,
        &app_dir,
        &output_path,
        version,
        package_kind,
        sign,
    )?;

    println!("AppImage written to: {}", output_path.display());

    Ok(())
}

fn stage_appstream_metainfo(
    workspace_root: &Path,
    metainfo_dir: &Path,
    package_kind: PackageKind,
) -> Result<()> {
    let filename = format!("app.actiona.{}.appdata.xml", package_kind.artifact_name());
    let source_path = workspace_root.join("assets").join(&filename);
    let source_file = require_file(&source_path)?;
    let destination_path = metainfo_dir.join(filename);

    fs::create_dir_all(metainfo_dir)?;
    fs::copy(source_file, &destination_path)?;
    validate_appstream_metainfo(&destination_path)?;

    Ok(())
}

fn validate_appstream_metainfo(metainfo_path: &Path) -> Result<()> {
    if which_appstreamcli().is_err() {
        eprintln!("appstreamcli not found on PATH, skipping AppStream validation");
        return Ok(());
    }

    let status = Command::new("appstreamcli")
        .arg("validate")
        .arg("--no-net")
        .arg(metainfo_path)
        .status()
        .map_err(|error| eyre!("Failed to run appstreamcli: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(eyre!(
            "AppStream validation failed for {}",
            metainfo_path.display()
        ))
    }
}

fn reset_app_dir(app_dir: &Path) -> Result<()> {
    if app_dir.exists() {
        fs::remove_dir_all(app_dir)?;
    }

    fs::create_dir_all(app_dir)?;
    Ok(())
}

fn remove_output_if_exists(output_path: &Path) -> Result<()> {
    if output_path.exists() {
        fs::remove_file(output_path)?;
    }

    Ok(())
}

async fn ensure_linuxdeploy(arch: &str, tools_dir: &Path) -> Result<PathBuf> {
    if let Ok(path) = which_linuxdeploy() {
        return Ok(path);
    }

    let tool_path = tools_dir.join(format!("linuxdeploy-{arch}.AppImage"));

    if !tool_path.exists() {
        download_linuxdeploy(arch, &tool_path).await?;
    }

    Ok(tool_path)
}

async fn ensure_linuxdeploy_appimage_plugin(arch: &str, tools_dir: &Path) -> Result<PathBuf> {
    if let Ok(path) = which_linuxdeploy_appimage_plugin() {
        return Ok(path);
    }

    let plugin_path = tools_dir.join(format!("linuxdeploy-plugin-appimage-{arch}.AppImage"));
    if !plugin_path.exists() {
        download_linuxdeploy_appimage_plugin(arch, &plugin_path).await?;
    }

    Ok(plugin_path)
}

fn which_linuxdeploy() -> Result<PathBuf> {
    let output = Command::new("which").arg("linuxdeploy").output()?;
    if output.status.success() {
        let path = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok(PathBuf::from(path))
    } else {
        Err(eyre!("linuxdeploy not found on PATH"))
    }
}

fn which_linuxdeploy_appimage_plugin() -> Result<PathBuf> {
    let output = Command::new("which")
        .arg("linuxdeploy-plugin-appimage")
        .output()?;
    if output.status.success() {
        let path = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok(PathBuf::from(path))
    } else {
        Err(eyre!("linuxdeploy-plugin-appimage not found on PATH"))
    }
}

fn which_appstreamcli() -> Result<PathBuf> {
    let output = Command::new("which").arg("appstreamcli").output()?;
    if output.status.success() {
        let path = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok(PathBuf::from(path))
    } else {
        Err(eyre!("appstreamcli not found on PATH"))
    }
}

async fn download_linuxdeploy(arch: &str, dest: &Path) -> Result<()> {
    let url = format!("{LINUXDEPLOY_URL_BASE}/linuxdeploy-{arch}.AppImage");
    eprintln!("Downloading linuxdeploy from {url}...");

    download_file(&url, dest).await
}

async fn download_linuxdeploy_appimage_plugin(arch: &str, dest: &Path) -> Result<()> {
    let url = format!(
        "{LINUXDEPLOY_PLUGIN_APPIMAGE_URL_BASE}/linuxdeploy-plugin-appimage-{arch}.AppImage"
    );
    eprintln!("Downloading linuxdeploy AppImage plugin from {url}...");

    download_file(&url, dest).await
}

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let bytes = reqwest::get(url)
        .await
        .map_err(|error| eyre!("Failed to download {url}: {error}"))?
        .error_for_status()
        .map_err(|error| eyre!("Download failed for {url}: {error}"))?
        .bytes()
        .await
        .map_err(|error| eyre!("Failed to read download response for {url}: {error}"))?;

    tokio_fs::write(dest, &bytes).await?;
    tokio_fs::set_permissions(dest, fs::Permissions::from_mode(0o755)).await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_linuxdeploy(
    tool_path: &Path,
    appimage_plugin: &Path,
    tools_dir: &Path,
    workspace_root: &Path,
    release_dir: &Path,
    app_dir: &Path,
    output_path: &Path,
    version: &str,
    package_kind: PackageKind,
    sign: bool,
) -> Result<()> {
    let run_binary = require_binary(release_dir, "actiona-run")?;
    let selection_binary = require_binary(release_dir, "extension-selection")?;
    let opencv_binary = require_binary(release_dir, "extension-opencv")?;
    let desktop_file_path = workspace_root.join("assets").join(format!(
        "app.actiona.{}.desktop",
        package_kind.artifact_name()
    ));
    let desktop_file = require_file(&desktop_file_path)?;
    let icon_source_path = workspace_root
        .join("crates")
        .join("core")
        .join("icons")
        .join("icon.png");
    let icon_file =
        prepare_linuxdeploy_icon(require_file(&icon_source_path)?, tools_dir, package_kind)?;
    let webkit_runtime = if package_kind == PackageKind::Editor {
        let webkit_runtime = webkit_runtime()?;
        stage_webkit_runtime(app_dir, &webkit_runtime)?;
        Some(webkit_runtime)
    } else {
        None
    };

    // APPIMAGE_EXTRACT_AND_RUN=1 avoids the FUSE requirement when linuxdeploy
    // and its AppImage output plugin are distributed as AppImages.
    let mut cmd = Command::new(tool_path);
    cmd.arg("--appdir")
        .arg(app_dir)
        .arg("--desktop-file")
        .arg(desktop_file)
        .arg("--icon-file")
        .arg(icon_file)
        .arg("--executable")
        .arg(run_binary)
        .arg("--executable")
        .arg(selection_binary)
        .arg("--executable")
        .arg(opencv_binary);

    if let Some(webkit_runtime) = &webkit_runtime {
        let staged_dir = app_dir.join(webkit_runtime.relative_dir()?);
        cmd.arg("--executable")
            .arg(require_binary(release_dir, "actiona-editor")?)
            // linuxdeploy does not know about the WebKit helpers on its own: this bundles their
            // dependencies and points their RUNPATH at the bundled libraries, so they never load
            // the host's WebKit.
            .arg("--deploy-deps-only")
            .arg(&staged_dir)
            .arg("--deploy-deps-only")
            .arg(staged_dir.join(WEBKIT_INJECTED_BUNDLE_DIR));
    }

    configure_linuxdeploy_environment(&mut cmd, tools_dir, output_path, version)?;

    run_linuxdeploy_command(&mut cmd)?;

    if let Some(webkit_runtime) = &webkit_runtime {
        clear_webkit_runtime_paths(app_dir, webkit_runtime)?;
        install_editor_apprun(app_dir, webkit_runtime)?;
    }

    let mut output_cmd = Command::new(appimage_plugin);
    output_cmd.arg(format!("--appdir={}", app_dir.display()));
    configure_linuxdeploy_environment(&mut output_cmd, tools_dir, output_path, version)?;

    if sign {
        output_cmd.env("LDAI_SIGN", "1");
        if let Ok(key) = env::var("ACTIONA_GPG_KEY") {
            output_cmd.env("LDAI_SIGN_KEY", key);
        }
    }

    run_linuxdeploy_command(&mut output_cmd)
}

fn configure_linuxdeploy_environment(
    cmd: &mut Command,
    tools_dir: &Path,
    output_path: &Path,
    version: &str,
) -> Result<()> {
    cmd.env("ARCH", appimage_arch()?)
        .env(
            "APPIMAGE_EXTRACT_AND_RUN",
            env::var("APPIMAGE_EXTRACT_AND_RUN").unwrap_or_else(|_| "1".to_owned()),
        )
        // The metainfo is validated offline by validate_appstream_metainfo.
        .env("LDAI_NO_APPSTREAM", "1")
        .env("LDAI_OUTPUT", output_path)
        .env("LINUXDEPLOY_OUTPUT_VERSION", version)
        .env("PATH", prepend_to_path(tools_dir)?);

    Ok(())
}

fn run_linuxdeploy_command(cmd: &mut Command) -> Result<()> {
    let status = cmd
        .status()
        .map_err(|error| eyre!("Failed to run linuxdeploy: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(eyre!("linuxdeploy exited with status {status}"))
    }
}

fn webkit_runtime() -> Result<WebkitRuntime> {
    let output = Command::new("pkg-config")
        .args(["--variable=libdir", WEBKIT_PACKAGE])
        .output()
        .map_err(|error| eyre!("Failed to run pkg-config for {WEBKIT_PACKAGE}: {error}"))?;

    if !output.status.success() {
        return Err(eyre!(
            "pkg-config could not locate {WEBKIT_PACKAGE}; install its development package"
        ));
    }

    let lib_dir = PathBuf::from(String::from_utf8(output.stdout)?.trim());
    let dir = lib_dir.join(WEBKIT_PACKAGE);
    let files = WEBKIT_RUNTIME_FILES
        .iter()
        .map(|filename| {
            let path = dir.join(filename);
            require_file(&path).map(Path::to_path_buf)
        })
        .collect::<Result<_>>()?;

    Ok(WebkitRuntime { dir, files })
}

fn stage_webkit_runtime(app_dir: &Path, webkit_runtime: &WebkitRuntime) -> Result<()> {
    let staged_dir = app_dir.join(webkit_runtime.relative_dir()?);

    for source_path in &webkit_runtime.files {
        let relative_path = source_path.strip_prefix(&webkit_runtime.dir)?;
        let destination_path = staged_dir.join(relative_path);
        let destination_dir = destination_path.parent().ok_or_else(|| {
            eyre!(
                "WebKit runtime path has no parent: {}",
                destination_path.display()
            )
        })?;

        fs::create_dir_all(destination_dir)?;
        fs::copy(source_path, destination_path)?;
    }

    Ok(())
}

/// Clears the helper and injected bundle directories compiled into the bundled `libwebkitgtk`.
///
/// With an empty helper directory WebKit looks its helper processes up in `PATH`, and with an
/// empty injected bundle directory it relies on `WEBKIT_INJECTED_BUNDLE_PATH`. The editor AppRun
/// sets both to the bundled copies. Only these two exact strings are cleared: other embedded
/// paths (GStreamer helpers, library search paths) keep pointing at the host.
fn clear_webkit_runtime_paths(app_dir: &Path, webkit_runtime: &WebkitRuntime) -> Result<()> {
    let helper_dir = webkit_runtime.dir.to_string_lossy().into_owned();
    let injected_bundle_dir = format!("{helper_dir}/{WEBKIT_INJECTED_BUNDLE_DIR}/");
    let mut cleared_helper_dirs = 0;
    let mut cleared_injected_bundle_dirs = 0;

    for entry in fs::read_dir(app_dir.join("usr").join("lib"))? {
        let path = entry?.path();
        let is_webkit_library = path
            .file_name()
            .and_then(|filename| filename.to_str())
            .is_some_and(|filename| {
                filename.starts_with("libwebkitgtk-") && filename.contains(".so")
            });
        if !is_webkit_library || !path.is_file() {
            continue;
        }

        let mut contents = fs::read(&path)?;
        cleared_helper_dirs += clear_c_string(&mut contents, helper_dir.as_bytes());
        cleared_injected_bundle_dirs +=
            clear_c_string(&mut contents, injected_bundle_dir.as_bytes());
        fs::write(path, contents)?;
    }

    for (cleared, value) in [
        (cleared_helper_dirs, &helper_dir),
        (cleared_injected_bundle_dirs, &injected_bundle_dir),
    ] {
        if cleared == 0 {
            return Err(eyre!(
                "\"{value}\" was not found in the bundled libwebkitgtk; the editor AppImage \
                 cannot redirect WebKit to its bundled runtime"
            ));
        }
    }

    Ok(())
}

/// Overwrites every NUL-terminated string in `contents` that is exactly `value` with NULs, and
/// returns how many were cleared. Longer strings that merely start or end with `value` are left
/// untouched.
fn clear_c_string(contents: &mut [u8], value: &[u8]) -> usize {
    let mut cleared = 0;

    for start in 1..contents.len().saturating_sub(value.len()) {
        let end = start + value.len();
        if contents[start - 1] == 0 && contents[end] == 0 && &contents[start..end] == value {
            contents[start..end].fill(0);
            cleared += 1;
        }
    }

    cleared
}

fn install_editor_apprun(app_dir: &Path, webkit_runtime: &WebkitRuntime) -> Result<()> {
    let apprun_path = app_dir.join("AppRun");
    if apprun_path.exists() || apprun_path.is_symlink() {
        fs::remove_file(&apprun_path)?;
    }

    // WebKit finds its helpers through PATH and its injected bundle through
    // WEBKIT_INJECTED_BUNDLE_PATH (see clear_webkit_runtime_paths), so the editor keeps the
    // caller's working directory.
    let webkit_dir = webkit_runtime.relative_dir()?.display();
    let script = format!(
        r#"#!/bin/sh
set -eu
APPDIR="${{APPDIR:-$(dirname -- "$(readlink -f -- "$0")")}}"
webkit_dir="$APPDIR/{webkit_dir}"
export PATH="$webkit_dir${{PATH:+:$PATH}}"
export WEBKIT_INJECTED_BUNDLE_PATH="$webkit_dir/{WEBKIT_INJECTED_BUNDLE_DIR}"
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1
exec "$APPDIR/usr/bin/actiona-editor" "$@"
"#
    );

    fs::write(&apprun_path, script)?;
    fs::set_permissions(apprun_path, fs::Permissions::from_mode(0o755))?;

    Ok(())
}

fn prepare_linuxdeploy_icon(
    icon_source: &Path,
    tools_dir: &Path,
    package_kind: PackageKind,
) -> Result<PathBuf> {
    let icon_path = tools_dir.join(format!("actiona-{}.png", package_kind.artifact_name()));
    fs::copy(icon_source, &icon_path)?;
    fs::set_permissions(&icon_path, fs::Permissions::from_mode(0o644))?;
    Ok(icon_path)
}

fn require_binary(release_dir: &Path, name: &str) -> Result<PathBuf> {
    let path = release_dir.join(name);
    if path.exists() {
        Ok(path)
    } else {
        Err(eyre!(
            "Binary not found: {}. Run `cargo make release` first.",
            path.display()
        ))
    }
}

fn require_file(path: &Path) -> Result<&Path> {
    if path.exists() {
        Ok(path)
    } else {
        Err(eyre!("Required file not found: {}", path.display()))
    }
}

fn prepend_to_path(dir: &Path) -> Result<String> {
    let mut paths = vec![dir.to_path_buf()];
    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }

    env::join_paths(paths)
        .map_err(|error| eyre!("Failed to construct PATH for linuxdeploy: {error}"))
        .map(|value| value.to_string_lossy().into_owned())
}

pub fn appimage_arch() -> Result<&'static str> {
    match env::consts::ARCH {
        "x86_64" => Ok("x86_64"),
        "aarch64" => Ok("aarch64"),
        "x86" => Ok("i686"),
        "arm" => Ok("armhf"),
        other => Err(eyre!("Unsupported architecture for AppImage: {other}")),
    }
}

pub async fn read_version(workspace_root: &Path) -> Result<String> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let contents = tokio_fs::read_to_string(&cargo_toml_path).await?;
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

#[cfg(test)]
mod tests {
    use super::clear_c_string;

    #[test]
    fn clear_c_string_only_clears_exact_nul_terminated_matches() {
        let dir = b"/usr/lib/x86_64-linux-gnu/webkitgtk-6.0";
        let mut contents = [
            b"\0/usr/lib\0".as_slice(),
            b"/usr/libexec/gstreamer-1.0/gst-plugin-scanner\0",
            dir,
            b"\0",
            dir,
            b"/injected-bundle/\0",
            b"/opt",
            dir,
            b"\0",
        ]
        .concat();
        let mut expected = contents.clone();
        let exact_match_start = expected
            .windows(dir.len())
            .position(|window| window == dir)
            .unwrap();
        expected[exact_match_start..exact_match_start + dir.len()].fill(0);

        assert_eq!(clear_c_string(&mut contents, dir), 1);
        assert_eq!(contents, expected);
    }
}
