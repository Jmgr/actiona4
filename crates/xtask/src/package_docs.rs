use std::path::{Path, PathBuf};

use color_eyre::Result;
use installer_tools::package::{PackagedFile, PackagedFilePlatform};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedPackagedFile {
    pub source_path: PathBuf,
    pub destination_name: String,
}

#[cfg(windows)]
pub async fn read_packaged_file_contents(
    workspace_root: &Path,
    packaged_file: &PackagedFile,
) -> Result<Vec<u8>> {
    let source_path = workspace_root.join(&packaged_file.source_path);

    if packaged_file.use_dos_line_feeds {
        read_document_contents_with_dos_line_feeds(&source_path).await
    } else {
        Ok(tokio::fs::read(&source_path).await?)
    }
}

pub async fn stage_packaged_files(
    workspace_root: &Path,
    destination_dir: &Path,
    packaged_files: &[PackagedFile],
    platform: PackagedFilePlatform,
) -> Result<Vec<StagedPackagedFile>> {
    tokio::fs::create_dir_all(destination_dir).await?;

    let mut staged_files = Vec::new();
    for packaged_file in packaged_files
        .iter()
        .filter(|packaged_file| packaged_file.use_dos_line_feeds)
    {
        let staged_source_path = destination_dir.join(packaged_file.destination_name_for(platform));
        let contents = read_document_contents_with_dos_line_feeds(
            &workspace_root.join(&packaged_file.source_path),
        )
        .await?;
        tokio::fs::write(&staged_source_path, contents).await?;
        staged_files.push(StagedPackagedFile {
            source_path: staged_source_path,
            destination_name: packaged_file.destination_name_for(platform).to_owned(),
        });
    }

    Ok(staged_files)
}

async fn read_document_contents_with_dos_line_feeds(source_path: &Path) -> Result<Vec<u8>> {
    let contents = tokio::fs::read_to_string(source_path).await?;
    Ok(normalize_to_crlf(&contents).into_bytes())
}

fn normalize_to_crlf(contents: &str) -> String {
    contents
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use installer_tools::package::{PackagedFilePlatform, packaged_files};
    use tempfile::tempdir;

    use super::{normalize_to_crlf, stage_packaged_files};

    #[test]
    fn packaged_files_enumerates_root_markdown_and_renames_license_on_windows() {
        let workspace = tempdir().unwrap();
        fs::write(workspace.path().join("README.md"), "readme\n").unwrap();
        fs::write(workspace.path().join("CHANGELOG.md"), "changelog\n").unwrap();
        fs::write(workspace.path().join("LICENSE"), "license\n").unwrap();
        fs::create_dir(workspace.path().join("docs")).unwrap();
        fs::write(workspace.path().join("docs").join("IGNORED.md"), "nested\n").unwrap();

        let packaged_files = packaged_files(workspace.path()).unwrap();
        let mut document_names: Vec<_> = packaged_files
            .iter()
            .filter(|packaged_file| packaged_file.use_dos_line_feeds)
            .map(|packaged_file| {
                packaged_file
                    .destination_name_for(PackagedFilePlatform::Windows)
                    .to_owned()
            })
            .collect();
        document_names.sort();

        assert_eq!(
            document_names,
            vec!["CHANGELOG.md", "LICENSE.md", "README.md"]
        );
    }

    #[test]
    fn normalize_to_crlf_rewrites_mixed_line_endings() {
        assert_eq!(normalize_to_crlf("a\nb\r\nc\rd"), "a\r\nb\r\nc\r\nd");
    }

    #[tokio::test]
    async fn stage_packaged_files_writes_crlf_copies() {
        let workspace = tempdir().unwrap();
        let destination = workspace.path().join("staged");
        fs::write(workspace.path().join("README.md"), "line1\nline2\n").unwrap();
        fs::write(workspace.path().join("LICENSE"), "license\n").unwrap();

        let packaged_files = packaged_files(workspace.path()).unwrap();
        let staged_files = stage_packaged_files(
            workspace.path(),
            &destination,
            &packaged_files,
            PackagedFilePlatform::Linux,
        )
        .await
        .unwrap();

        let mut staged_names: Vec<_> = staged_files
            .iter()
            .map(|staged_file| staged_file.destination_name.as_str())
            .collect();
        staged_names.sort();
        assert_eq!(staged_names, vec!["LICENSE", "README.md"]);
        assert_eq!(
            tokio::fs::read(destination.join("README.md"))
                .await
                .unwrap(),
            b"line1\r\nline2\r\n"
        );
    }
}
