use std::{
    fmt::Write,
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::{Result, eyre::eyre};
use installer_tools::package::{PackageKind, PackagedFile, PackagedFilePlatform, packaged_files};
use tokio::fs::{create_dir_all, remove_dir_all, try_exists, write};

use crate::{
    constants::{INNO_SIGN_TOOL_NAME, RUN_FILE_DESCRIPTION},
    package_docs::{StagedPackagedFile, stage_packaged_files},
    signing::signing_arguments,
    util::run_command,
    workspace::{NotificationPackageInfo, WorkspacePackageInfo},
};

const RUN_APP_ID: &str = "{{A3D5D4F0-1AFA-4278-9E23-FA4A36632447}";
const EDITOR_APP_ID: &str = "{{9FD328F2-873A-4FC0-921A-42AD7CCCCDA}";

struct InstallerProduct {
    package_kind: PackageKind,
    app_id: &'static str,
    app_name: &'static str,
    app_exe_name: &'static str,
}

impl InstallerProduct {
    const fn from_package_kind(package_kind: PackageKind) -> Self {
        match package_kind {
            PackageKind::Run => Self {
                package_kind,
                app_id: RUN_APP_ID,
                app_name: "Actiona Run",
                app_exe_name: "actiona-runw.exe",
            },
            PackageKind::Editor => Self {
                package_kind,
                app_id: EDITOR_APP_ID,
                app_name: "Actiona Editor",
                app_exe_name: "actiona-editor.exe",
            },
        }
    }
}

pub async fn build_installers(
    workspace_root: &Path,
    workspace_package_info: &WorkspacePackageInfo,
    notification_package_info: &NotificationPackageInfo,
    should_sign: bool,
) -> Result<()> {
    for package_kind in PackageKind::ALL {
        build_installer(
            workspace_root,
            workspace_package_info,
            notification_package_info,
            InstallerProduct::from_package_kind(package_kind),
            should_sign,
        )
        .await?;
    }

    Ok(())
}

async fn build_installer(
    workspace_root: &Path,
    workspace_package_info: &WorkspacePackageInfo,
    notification_package_info: &NotificationPackageInfo,
    product: InstallerProduct,
    should_sign: bool,
) -> Result<()> {
    let installer_directory = workspace_root.join("installer");
    write_installer_files_include(workspace_root, product.package_kind).await?;
    let sign_tool = if should_sign { INNO_SIGN_TOOL_NAME } else { "" };
    let output_base_filename =
        installer_output_base_filename(workspace_package_info, product.package_kind);
    let mut command = Command::new("iscc");
    command
        .arg(format!("/DMyAppId={}", product.app_id))
        .arg(format!("/DMyAppName={}", product.app_name))
        .arg(format!("/DMyAppExeName={}", product.app_exe_name))
        .arg(format!("/DMyAppVersion={}", workspace_package_info.version))
        .arg(format!("/DMyOutputBaseFilename={output_base_filename}"))
        .arg(format!(
            "/DMyAppFileVersion={}",
            workspace_package_info.file_version
        ))
        .arg(format!(
            "/DMyAppPublisher={}",
            workspace_package_info.publisher
        ))
        .arg(format!(
            "/DMyAppURL={}",
            workspace_package_info.documentation_url
        ))
        .arg(format!(
            "/DMyNotificationAUMID={}",
            notification_package_info.aumid
        ))
        .arg(format!(
            "/DMyNotificationDisplayName={RUN_FILE_DESCRIPTION}"
        ))
        .arg(format!("/DMySignTool={sign_tool}"))
        .arg("main.iss")
        .current_dir(installer_directory);

    if should_sign {
        command.arg(inno_sign_tool_argument(product.app_name));
    }

    run_command(&mut command, "Failed to run Inno Setup compiler.")
}

#[must_use]
pub fn installer_output_base_filename(
    workspace_package_info: &WorkspacePackageInfo,
    package_kind: PackageKind,
) -> String {
    format!(
        "actiona-{}-{}-x86_64-setup",
        package_kind.artifact_name(),
        workspace_package_info.version
    )
}

#[must_use]
pub fn installer_path(
    workspace_root: &Path,
    workspace_package_info: &WorkspacePackageInfo,
) -> PathBuf {
    workspace_root.join("target").join(format!(
        "{}.exe",
        installer_output_base_filename(workspace_package_info, PackageKind::Run)
    ))
}

async fn write_installer_files_include(
    workspace_root: &Path,
    package_kind: PackageKind,
) -> Result<()> {
    let generated_include_path = workspace_root.join("target").join("files.iss");
    let staged_docs_directory = workspace_root
        .join("target")
        .join("package-docs")
        .join("windows");
    let packaged_files = packaged_files(workspace_root, package_kind)?;
    let mut file_contents = String::new();

    for packaged_file in packaged_files
        .iter()
        .filter(|packaged_file| packaged_file.include_in_installer)
        .filter(|packaged_file| !packaged_file.use_dos_line_feeds)
    {
        writeln!(
            file_contents,
            "{}",
            installer_source_line(packaged_file, PackagedFilePlatform::Windows)?
        )
        .map_err(|error| eyre!(error))?;
    }

    if try_exists(&staged_docs_directory).await? {
        remove_dir_all(&staged_docs_directory).await?;
    }

    for staged_document in stage_packaged_files(
        workspace_root,
        &staged_docs_directory,
        &packaged_files,
        PackagedFilePlatform::Windows,
    )
    .await?
    {
        writeln!(
            file_contents,
            "{}",
            installer_document_source_line(workspace_root, &staged_document)?
        )
        .map_err(|error| eyre!(error))?;
    }

    let parent_directory_path = generated_include_path
        .parent()
        .ok_or_else(|| eyre!("Generated installer include path has no parent directory."))?;
    create_dir_all(parent_directory_path).await?;
    write(generated_include_path, file_contents).await?;

    Ok(())
}

fn installer_source_line(
    packaged_file: &PackagedFile,
    platform: PackagedFilePlatform,
) -> Result<String> {
    let mut source_line = format!(
        "Source: \"..\\{}\"; DestDir: \"{}\"",
        inno_path(&packaged_file.source_path),
        packaged_file.destination_dir
    );

    if packaged_file.destination_name_for(platform)
        != packaged_file.source_path.rsplit('/').next().unwrap_or("")
    {
        write!(
            source_line,
            "; DestName: \"{}\"",
            packaged_file.destination_name_for(platform)
        )
        .map_err(|error| eyre!(error))?;
    }

    source_line.push_str("; Flags: ignoreversion");
    Ok(source_line)
}

fn installer_document_source_line(
    workspace_root: &Path,
    staged_file: &StagedPackagedFile,
) -> Result<String> {
    let source_path = staged_file
        .source_path
        .strip_prefix(workspace_root)
        .map_err(|_| {
            eyre!(
                "Staged installer document is outside the workspace: {}",
                staged_file.source_path.display()
            )
        })?
        .to_str()
        .ok_or_else(|| eyre!("Invalid UTF-8 path: {}", staged_file.source_path.display()))?;
    Ok(format!(
        "Source: \"..\\{}\"; DestDir: \"{{app}}\"; DestName: \"{}\"; Flags: ignoreversion",
        inno_path(source_path),
        staged_file.destination_name
    ))
}

fn inno_path(path: &str) -> String {
    path.replace('/', "\\")
}

fn inno_sign_tool_argument(file_description: &str) -> String {
    let mut sign_tool_definition = vec!["signtool".to_owned()];
    sign_tool_definition.extend(
        signing_arguments(file_description)
            .into_iter()
            .map(|argument| quote_inno_argument(&argument)),
    );
    sign_tool_definition.push("$f".to_owned());

    format!("/S{INNO_SIGN_TOOL_NAME}={}", sign_tool_definition.join(" "))
}

fn quote_inno_argument(argument: &str) -> String {
    if argument.contains(' ') || argument.contains('\t') {
        format!("$q{argument}$q")
    } else {
        argument.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use installer_tools::package::PackageKind;

    use super::{
        InstallerProduct, StagedPackagedFile, WorkspacePackageInfo, installer_document_source_line,
        installer_output_base_filename,
    };

    #[test]
    fn installer_products_have_distinct_names_executables_and_ids() {
        let run = InstallerProduct::from_package_kind(PackageKind::Run);
        let editor = InstallerProduct::from_package_kind(PackageKind::Editor);

        assert_eq!(run.app_name, "Actiona Run");
        assert_eq!(run.app_exe_name, "actiona-runw.exe");
        assert_eq!(editor.app_name, "Actiona Editor");
        assert_eq!(editor.app_exe_name, "actiona-editor.exe");
        assert_ne!(run.app_id, editor.app_id);
    }

    #[test]
    fn installer_output_names_differ_only_by_package_kind() {
        let package_info = WorkspacePackageInfo {
            version: "1.2.3".to_owned(),
            file_version: "1.2.3.0".to_owned(),
            publisher: "Publisher".to_owned(),
            documentation_url: "https://example.com".to_owned(),
        };

        assert_eq!(
            installer_output_base_filename(&package_info, PackageKind::Run),
            "actiona-run-1.2.3-x86_64-setup"
        );
        assert_eq!(
            installer_output_base_filename(&package_info, PackageKind::Editor),
            "actiona-editor-1.2.3-x86_64-setup"
        );
    }

    #[test]
    fn installer_document_source_line_uses_workspace_relative_path() {
        let workspace_root = Path::new(r"C:\rust\actiona4");
        let staged_file = StagedPackagedFile {
            source_path: PathBuf::from(r"C:\rust\actiona4\target\package-docs\windows\README.md"),
            destination_name: "README.md".to_owned(),
        };

        let source_line = installer_document_source_line(workspace_root, &staged_file).unwrap();

        assert_eq!(
            source_line,
            r#"Source: "..\target\package-docs\windows\README.md"; DestDir: "{app}"; DestName: "README.md"; Flags: ignoreversion"#
        );
    }

    #[test]
    fn installer_document_source_line_rejects_paths_outside_workspace() {
        let workspace_root = Path::new(r"C:\rust\actiona4");
        let staged_file = StagedPackagedFile {
            source_path: PathBuf::from(r"C:\elsewhere\README.md"),
            destination_name: "README.md".to_owned(),
        };

        let error = installer_document_source_line(workspace_root, &staged_file).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Staged installer document is outside the workspace")
        );
    }
}
