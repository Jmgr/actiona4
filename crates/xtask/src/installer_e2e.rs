use std::{
    path::Path,
    process::{Command, Stdio},
};

use color_eyre::{Result, eyre::eyre};
use tempfile::TempDir;

use crate::{installer::installer_path, workspace::WorkspacePackageInfo};

pub fn run_installer_e2e(
    workspace_root: &Path,
    workspace_package_info: &WorkspacePackageInfo,
) -> Result<()> {
    let installer_path = installer_path(workspace_root, workspace_package_info);
    let install_directory = tempfile::Builder::new()
        .prefix("actiona-run-installer-e2e-")
        .tempdir()?;

    let test_result = install_and_test(&installer_path, install_directory.path(), workspace_root);
    let uninstall_result = uninstall(install_directory.path());
    let remove_result = remove_install_directory(install_directory);

    test_result?;
    uninstall_result?;
    remove_result
}

fn install_and_test(
    installer_path: &Path,
    install_directory: &Path,
    workspace_root: &Path,
) -> Result<()> {
    if !installer_path.is_file() {
        return Err(eyre!("Installer not found: {}", installer_path.display()));
    }

    let status = Command::new(installer_path)
        .args([
            "/VERYSILENT",
            "/SUPPRESSMSGBOXES",
            "/NORESTART",
            "/SP-",
            "/CURRENTUSER",
            &format!("/DIR={}", install_directory.display()),
            r#"/TASKS="""#,
        ])
        .status()?;
    if !status.success() {
        return Err(eyre!("Installer exited with status {status}."));
    }

    let actiona_run = install_directory.join("actiona-run.exe");
    if !actiona_run.is_file() {
        return Err(eyre!(
            "Installed actiona-run executable not found: {}",
            actiona_run.display()
        ));
    }

    let status = Command::new("cargo")
        .args(["test", "--locked", "-p", "e2e"])
        .current_dir(workspace_root)
        .env("ACTIONA4_E2E_RUNNER", &actiona_run)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(eyre!("Installer E2E tests exited with status {status}."))
    }
}

fn uninstall(install_directory: &Path) -> Result<()> {
    let uninstaller_path = install_directory.join("unins000.exe");
    if !uninstaller_path.is_file() {
        return Ok(());
    }

    let status = Command::new(uninstaller_path)
        .args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(eyre!("Installer uninstaller exited with status {status}."))
    }
}

fn remove_install_directory(install_directory: TempDir) -> Result<()> {
    install_directory.close()?;
    Ok(())
}
