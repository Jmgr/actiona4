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
    /// Build signed Actiona Run and Actiona Editor AppImages (Linux only).
    #[cfg(unix)]
    AppImage,
    /// Build unsigned Actiona Run and Actiona Editor AppImages (Linux only).
    #[cfg(unix)]
    AppImageNoSign,
    /// Generate rustdoc JSON and TypeScript declaration files.
    Doc,
    /// Lint and type-check end-to-end TypeScript scripts and shared declarations.
    LintTs,
    /// Cross-compile the workspace for Windows and run clippy on it (Linux only).
    #[cfg(unix)]
    LintWindows,
    /// Generate Breakpad symbol files (.sym) and strip release binaries.
    Symbols,
    /// Symbolicate a crash dump archive or loose minidump using the release symbol files.
    Symbolicate {
        /// Path to the .zip archive or .dmp file to analyse.
        dump: PathBuf,
    },
    #[cfg(windows)]
    /// Build the Actiona Run and Actiona Editor installers with Inno Setup.
    Installer,
    #[cfg(windows)]
    /// Build the Actiona Run and Actiona Editor installers without signing.
    InstallerNoSign,
    #[cfg(windows)]
    /// Install the unsigned Actiona Run installer and run end-to-end tests.
    InstallerE2e,
    #[cfg(windows)]
    /// Build Actiona Run and Actiona Editor zip archives.
    Archive,
    #[cfg(windows)]
    /// Build unsigned Actiona Run and Actiona Editor zip archives.
    ArchiveNoSign,
    #[cfg(windows)]
    /// Sign the packaged release executables with signtool.
    SignBinaries,
}
