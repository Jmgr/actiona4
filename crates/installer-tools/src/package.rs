#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackagedFile {
    pub source_path: &'static str,
    pub destination_name: &'static str,
    pub destination_dir: &'static str,
    pub include_in_archive: bool,
    pub include_in_installer: bool,
    pub should_sign: bool,
}

const PACKAGED_FILES: [PackagedFile; 5] = [
    PackagedFile {
        source_path: "target/i686-pc-windows-msvc/release/inno_plugin.dll",
        destination_name: "inno_plugin.dll",
        destination_dir: "{app}",
        include_in_archive: false,
        include_in_installer: true,
        should_sign: true,
    },
    PackagedFile {
        source_path: "target/release/actiona-runw.exe",
        destination_name: "actiona-runw.exe",
        destination_dir: "{app}",
        include_in_archive: true,
        include_in_installer: true,
        should_sign: true,
    },
    PackagedFile {
        source_path: "target/release/actiona-run.exe",
        destination_name: "actiona-run.exe",
        destination_dir: "{app}",
        include_in_archive: true,
        include_in_installer: true,
        should_sign: true,
    },
    PackagedFile {
        source_path: "target/release/selection-tool.exe",
        destination_name: "selection-tool.exe",
        destination_dir: "{app}",
        include_in_archive: true,
        include_in_installer: true,
        should_sign: true,
    },
    PackagedFile {
        source_path: "LICENSE",
        destination_name: "LICENSE.txt",
        destination_dir: "{app}",
        include_in_archive: true,
        include_in_installer: true,
        should_sign: false,
    },
];

pub fn packaged_files() -> &'static [PackagedFile] {
    &PACKAGED_FILES
}
