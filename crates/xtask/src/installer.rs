use std::{fmt::Write, path::Path, process::Command};

use color_eyre::{Result, eyre::eyre};
use installer_tools::package::{PackagedFile, PackagedFilePlatform, packaged_files};
use tokio::fs::{remove_dir_all, try_exists};

use crate::{
    constants::{INNO_SIGN_TOOL_NAME, INSTALLER_FILE_DESCRIPTION, RUN_FILE_DESCRIPTION},
    package_docs::{StagedPackagedFile, stage_packaged_files},
    signing::signing_arguments,
    util::run_command,
    workspace::{NotificationPackageInfo, WorkspacePackageInfo},
};

pub async fn build_installer(
    workspace_root: &Path,
    workspace_package_info: &WorkspacePackageInfo,
    notification_package_info: &NotificationPackageInfo,
    should_sign: bool,
) -> Result<()> {
    let installer_directory = workspace_root.join("installer");
    write_installer_files_include(workspace_root).await?;
    let sign_tool = if should_sign { INNO_SIGN_TOOL_NAME } else { "" };
    let mut command = Command::new("iscc");
    command
        .arg(format!("/DMyAppVersion={}", workspace_package_info.version))
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
        command.arg(inno_sign_tool_argument(INSTALLER_FILE_DESCRIPTION));
    }

    run_command(&mut command, "Failed to run Inno Setup compiler.")
}

async fn write_installer_files_include(workspace_root: &Path) -> Result<()> {
    let generated_include_path = workspace_root.join("target").join("files.iss");
    let staged_docs_directory = workspace_root
        .join("target")
        .join("package-docs")
        .join("windows");
    let packaged_files = packaged_files(workspace_root)?;
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
            installer_document_source_line(&staged_document)?
        )
        .map_err(|error| eyre!(error))?;
    }

    let parent_directory_path = generated_include_path
        .parent()
        .ok_or_else(|| eyre!("Generated installer include path has no parent directory."))?;
    tokio::fs::create_dir_all(parent_directory_path).await?;
    tokio::fs::write(generated_include_path, file_contents).await?;

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

fn installer_document_source_line(staged_file: &StagedPackagedFile) -> Result<String> {
    let source_path = staged_file
        .source_path
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
