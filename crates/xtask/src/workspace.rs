use std::{
    env,
    path::{Path, PathBuf},
};

use color_eyre::{Result, eyre::eyre};
#[cfg(any(test, windows))]
use versions::SemVer;

#[cfg(windows)]
pub struct WorkspacePackageInfo {
    pub version: String,
    pub file_version: String,
    pub publisher: String,
    pub documentation_url: String,
}

#[cfg(windows)]
pub struct NotificationPackageInfo {
    pub aumid: String,
}

pub fn workspace_root() -> Result<PathBuf> {
    let manifest_directory =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|error| eyre!(error))?);

    manifest_directory
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| eyre!("Failed to resolve workspace root from CARGO_MANIFEST_DIR."))
}

#[cfg(windows)]
pub async fn read_workspace_package_info(workspace_root: &Path) -> Result<WorkspacePackageInfo> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let cargo_toml_contents = tokio::fs::read_to_string(&cargo_toml_path).await?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_contents)?;
    let package_table = cargo_toml
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table)
        .ok_or_else(|| eyre!("Failed to read workspace.package from Cargo.toml."))?;

    let version = package_table
        .get("version")
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| eyre!("Failed to read workspace.package.version from Cargo.toml."))?;
    let file_version = normalize_windows_file_version(&version)?;

    let publisher = package_table
        .get("authors")
        .and_then(toml::Value::as_array)
        .and_then(|authors| authors.first())
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| eyre!("Failed to read workspace.package.authors[0] from Cargo.toml."))?;

    let documentation_url = package_table
        .get("documentation")
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| eyre!("Failed to read workspace.package.documentation from Cargo.toml."))?;

    Ok(WorkspacePackageInfo {
        version,
        file_version,
        publisher,
        documentation_url,
    })
}

#[cfg(any(test, windows))]
fn normalize_windows_file_version(version: &str) -> Result<String> {
    let semantic_version =
        SemVer::new(version).ok_or_else(|| eyre!("Invalid semantic version '{version}'."))?;

    let major = u16::try_from(semantic_version.major).map_err(|error| {
        eyre!(
            "Major version component '{}' in '{version}' is too large for Windows file versions: {error}",
            semantic_version.major
        )
    })?;
    let minor = u16::try_from(semantic_version.minor).map_err(|error| {
        eyre!(
            "Minor version component '{}' in '{version}' is too large for Windows file versions: {error}",
            semantic_version.minor
        )
    })?;
    let patch = u16::try_from(semantic_version.patch).map_err(|error| {
        eyre!(
            "Patch version component '{}' in '{version}' is too large for Windows file versions: {error}",
            semantic_version.patch
        )
    })?;

    Ok(format!("{major}.{minor}.{patch}.0"))
}

#[cfg(windows)]
pub async fn read_notification_package_info(
    workspace_root: &Path,
) -> Result<NotificationPackageInfo> {
    let cargo_toml_path = workspace_root
        .join("crates")
        .join("core")
        .join("Cargo.toml");
    let cargo_toml_contents = tokio::fs::read_to_string(&cargo_toml_path).await?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_contents)?;

    let aumid = cargo_toml
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("metadata"))
        .and_then(toml::Value::as_table)
        .and_then(|metadata| metadata.get("actiona"))
        .and_then(toml::Value::as_table)
        .and_then(|actiona| actiona.get("notification-aumid"))
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            eyre!(
                "Failed to read core package.metadata.actiona.notification-aumid from Cargo.toml."
            )
        })?;

    Ok(NotificationPackageInfo { aumid })
}

#[cfg(test)]
mod tests {
    use super::normalize_windows_file_version;

    #[test]
    fn normalizes_three_part_semver() {
        assert_eq!(normalize_windows_file_version("0.1.8").unwrap(), "0.1.8.0");
    }

    #[test]
    fn strips_pre_release_and_build_metadata() {
        assert_eq!(
            normalize_windows_file_version("1.2.3-beta.1+abc123").unwrap(),
            "1.2.3.0"
        );
    }

    #[test]
    fn rejects_non_semver_versions() {
        assert!(normalize_windows_file_version("1.2.3.4").is_err());
    }
}
