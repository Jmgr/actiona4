use std::{
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::{Result, eyre::eyre};
use installer_tools::package::packaged_files;

use crate::{
    constants::{
        RUN_FILE_DESCRIPTION, SIGNING_CERTIFICATE_SHA1, SIGNING_PRODUCT_URL, SIGNING_TIMESTAMP_URL,
    },
    util::run_command,
};

pub fn sign_binaries(workspace_root: &Path) -> Result<()> {
    sign_files(
        RUN_FILE_DESCRIPTION,
        release_file_paths(workspace_root),
        "Failed to sign release binaries.",
    )
}

pub fn release_file_paths(workspace_root: &Path) -> Vec<PathBuf> {
    packaged_files()
        .iter()
        .filter(|packaged_file| packaged_file.should_sign)
        .map(|packaged_file| workspace_root.join(packaged_file.source_path))
        .collect()
}

pub(crate) fn signing_arguments(file_description: &str) -> Vec<String> {
    vec![
        "sign".to_owned(),
        "/v".to_owned(),
        "/d".to_owned(),
        file_description.to_owned(),
        "/du".to_owned(),
        SIGNING_PRODUCT_URL.to_owned(),
        "/fd".to_owned(),
        "sha256".to_owned(),
        "/sha1".to_owned(),
        SIGNING_CERTIFICATE_SHA1.to_owned(),
        "/tr".to_owned(),
        SIGNING_TIMESTAMP_URL.to_owned(),
        "/td".to_owned(),
        "sha256".to_owned(),
    ]
}

fn sign_files<I, P>(file_description: &str, file_paths: I, failure_message: &str) -> Result<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut unsigned_file_paths = Vec::new();

    for file_path in file_paths {
        let file_path = file_path.as_ref();

        if !file_path.is_file() {
            return Err(eyre!("File not found: {}", file_path.display()));
        }

        if is_authenticode_signed(file_path)? {
            continue;
        }

        unsigned_file_paths.push(file_path.to_path_buf());
    }

    if unsigned_file_paths.is_empty() {
        return Ok(());
    }

    let mut command = Command::new("signtool");
    command.args(signing_arguments(file_description));

    for file_path in unsigned_file_paths {
        command.arg(file_path);
    }

    run_command(&mut command, failure_message)
}

fn is_authenticode_signed(file_path: &Path) -> Result<bool> {
    let status = Command::new("signtool")
        .arg("verify")
        .arg("/pa")
        .arg(file_path)
        .status()?;

    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    fn is_signable_binary_path(file_path: &Path) -> bool {
        let Some(extension) = file_path
            .extension()
            .and_then(|extension| extension.to_str())
        else {
            return false;
        };

        extension.eq_ignore_ascii_case("exe") || extension.eq_ignore_ascii_case("dll")
    }

    #[test]
    fn signable_binary_path_accepts_exe_and_dll_extensions_case_insensitively() {
        assert!(is_signable_binary_path(Path::new("actiona-run.exe")));
        assert!(is_signable_binary_path(Path::new("selection.DLL")));
    }

    #[test]
    fn signable_binary_path_rejects_other_extensions() {
        assert!(!is_signable_binary_path(Path::new("actiona-run.pdb")));
        assert!(!is_signable_binary_path(Path::new("actiona-run")));
    }
}
