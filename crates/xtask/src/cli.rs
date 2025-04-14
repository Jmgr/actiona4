use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "xtask", version, about = "Workspace automation tasks")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Build a signed AppImage for actiona-run (Linux only).
    #[cfg(unix)]
    AppImage,
    /// Build an unsigned AppImage for actiona-run (Linux only).
    #[cfg(unix)]
    AppImageNoSign,
    /// Generate rustdoc JSON and TypeScript declaration files.
    Doc,
    /// Generate Breakpad symbol files (.sym) and strip release binaries.
    Symbols,
    /// Symbolicate a crash dump archive or loose minidump using the release symbol files.
    Symbolicate {
        /// Path to the .zip archive or .dmp file to analyse.
        dump: PathBuf,
    },
    #[cfg(windows)]
    /// Build the Actiona Run installer with Inno Setup.
    Installer,
    #[cfg(windows)]
    /// Build the Actiona Run installer with Inno Setup without signing.
    InstallerNoSign,
    #[cfg(windows)]
    /// Build a zip archive with the installer payload minus the Inno plugin.
    Archive,
    #[cfg(windows)]
    /// Build a zip archive with the installer payload minus the Inno plugin without signing.
    ArchiveNoSign,
    #[cfg(windows)]
    /// Sign the actiona-run release executables with signtool.
    SignBinaries,
}
