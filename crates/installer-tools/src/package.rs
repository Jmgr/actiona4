use std::{collections::HashSet, ffi::OsStr, fs, path::Path};

use eyre::{Result, eyre};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageKind {
    Run,
    Editor,
}

impl PackageKind {
    pub const ALL: [Self; 2] = [Self::Run, Self::Editor];

    #[must_use]
    pub const fn artifact_name(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Editor => "editor",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackagedFile {
    pub source_path: String,
    pub destination_name: String,
    pub windows_destination_name: Option<String>,
    pub destination_dir: String,
    pub include_in_archive: bool,
    pub include_in_installer: bool,
    pub include_in_appimage: bool,
    pub should_sign: bool,
    pub use_dos_line_feeds: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackagedFilePlatform {
    Linux,
    Windows,
}

impl PackagedFile {
    #[must_use]
    pub fn destination_name_for(&self, platform: PackagedFilePlatform) -> &str {
        match platform {
            PackagedFilePlatform::Linux => &self.destination_name,
            PackagedFilePlatform::Windows => self
                .windows_destination_name
                .as_deref()
                .unwrap_or(&self.destination_name),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PackagedFileDefinition {
    source_path: &'static str,
    destination_name: Option<&'static str>,
    windows_destination_name: Option<&'static str>,
    destination_dir: &'static str,
    include_in_archive: bool,
    include_in_installer: bool,
    include_in_appimage: bool,
    should_sign: bool,
    use_dos_line_feeds: bool,
    enumerate_root_markdown_files: bool,
    editor_only: bool,
}

impl PackagedFileDefinition {
    const fn new(source_path: &'static str) -> Self {
        Self {
            source_path,
            destination_name: None,
            windows_destination_name: None,
            destination_dir: "{app}",
            include_in_archive: true,
            include_in_installer: true,
            include_in_appimage: false,
            should_sign: false,
            use_dos_line_feeds: false,
            enumerate_root_markdown_files: false,
            editor_only: false,
        }
    }

    const fn with_destination_name(mut self, destination_name: &'static str) -> Self {
        self.destination_name = Some(destination_name);
        self
    }

    const fn with_windows_destination_name(
        mut self,
        windows_destination_name: &'static str,
    ) -> Self {
        self.windows_destination_name = Some(windows_destination_name);
        self
    }

    const fn include_in_appimage(mut self) -> Self {
        self.include_in_appimage = true;
        self
    }

    const fn exclude_from_archive(mut self) -> Self {
        self.include_in_archive = false;
        self
    }

    const fn signed(mut self) -> Self {
        self.should_sign = true;
        self
    }

    const fn with_dos_line_feeds(mut self) -> Self {
        self.use_dos_line_feeds = true;
        self
    }

    const fn enumerate_root_markdown_files(mut self) -> Self {
        self.enumerate_root_markdown_files = true;
        self
    }

    const fn editor_only(mut self) -> Self {
        self.editor_only = true;
        self
    }
}

const PACKAGED_FILES: [PackagedFileDefinition; 8] = [
    PackagedFileDefinition::new("target/i686-pc-windows-msvc/release/inno_plugin.dll")
        .with_destination_name("inno_plugin.dll")
        .exclude_from_archive()
        .signed(),
    PackagedFileDefinition::new("target/release/actiona-runw.exe").signed(),
    PackagedFileDefinition::new("target/release/actiona-run.exe").signed(),
    PackagedFileDefinition::new("target/release/extension-selection.exe").signed(),
    PackagedFileDefinition::new("target/release/extension-opencv.exe").signed(),
    PackagedFileDefinition::new("target/release/editor.exe")
        .with_destination_name("actiona-editor.exe")
        .signed()
        .editor_only(),
    PackagedFileDefinition::new("*.md")
        .include_in_appimage()
        .with_dos_line_feeds()
        .enumerate_root_markdown_files(),
    PackagedFileDefinition::new("LICENSE")
        .with_windows_destination_name("LICENSE.txt")
        .include_in_appimage()
        .with_dos_line_feeds(),
];

pub fn packaged_files(
    workspace_root: &Path,
    package_kind: PackageKind,
) -> Result<Vec<PackagedFile>> {
    let mut packaged_files = Vec::new();

    for definition in PACKAGED_FILES {
        if definition.editor_only && package_kind == PackageKind::Run {
            continue;
        }

        if definition.enumerate_root_markdown_files {
            packaged_files.extend(expand_root_markdown_files(workspace_root, definition)?);
        } else {
            packaged_files.push(expand_packaged_file(definition.source_path, definition)?);
        }
    }

    ensure_unique_destination_names(&packaged_files)?;
    Ok(packaged_files)
}

fn expand_packaged_file(
    source_path: &str,
    definition: PackagedFileDefinition,
) -> Result<PackagedFile> {
    let source_file_name = Path::new(source_path)
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| eyre!("Invalid packaged file path: {source_path}"))?;

    Ok(PackagedFile {
        source_path: source_path.to_owned(),
        destination_name: definition
            .destination_name
            .unwrap_or(source_file_name)
            .to_owned(),
        windows_destination_name: definition.windows_destination_name.map(str::to_owned),
        destination_dir: definition.destination_dir.to_owned(),
        include_in_archive: definition.include_in_archive,
        include_in_installer: definition.include_in_installer,
        include_in_appimage: definition.include_in_appimage,
        should_sign: definition.should_sign,
        use_dos_line_feeds: definition.use_dos_line_feeds,
    })
}

fn expand_root_markdown_files(
    workspace_root: &Path,
    definition: PackagedFileDefinition,
) -> Result<Vec<PackagedFile>> {
    let mut packaged_files = Vec::new();

    for entry in fs::read_dir(workspace_root)? {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type()?.is_file()
            || !path
                .extension()
                .and_then(OsStr::to_str)
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            continue;
        }

        let source_path = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| eyre!("Invalid UTF-8 filename: {}", path.display()))?;
        packaged_files.push(expand_packaged_file(source_path, definition)?);
    }

    packaged_files.sort_by(|left, right| left.destination_name.cmp(&right.destination_name));
    Ok(packaged_files)
}

fn ensure_unique_destination_names(packaged_files: &[PackagedFile]) -> Result<()> {
    for platform in [PackagedFilePlatform::Linux, PackagedFilePlatform::Windows] {
        let mut seen_destination_names = HashSet::new();

        for packaged_file in packaged_files {
            let canonical_name = packaged_file
                .destination_name_for(platform)
                .to_ascii_lowercase();
            if !seen_destination_names.insert(canonical_name) {
                return Err(eyre!(
                    "Duplicate packaged file destination name for {:?}: {}",
                    platform,
                    packaged_file.destination_name_for(platform)
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, path::Path};

    use super::{PackageKind, PackagedFilePlatform, packaged_files};

    #[test]
    fn editor_package_adds_only_the_editor_to_the_run_payload() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let run_files = packaged_files(workspace_root, PackageKind::Run).unwrap();
        let editor_files = packaged_files(workspace_root, PackageKind::Editor).unwrap();
        let run_sources: HashSet<_> = run_files
            .iter()
            .map(|file| file.source_path.as_str())
            .collect();
        let editor_sources: HashSet<_> = editor_files
            .iter()
            .map(|file| file.source_path.as_str())
            .collect();

        assert_eq!(editor_sources.len(), run_sources.len() + 1);
        assert!(run_sources.is_subset(&editor_sources));

        let editor = editor_files
            .iter()
            .find(|file| file.source_path == "target/release/editor.exe")
            .unwrap();
        assert_eq!(
            editor.destination_name_for(PackagedFilePlatform::Windows),
            "actiona-editor.exe"
        );
    }
}
