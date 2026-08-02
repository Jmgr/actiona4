#[cfg(unix)]
mod appimage;
#[cfg(windows)]
mod archive;
mod cli;
#[cfg(windows)]
mod constants;
mod documentation;
#[cfg(windows)]
mod installer;
#[cfg(windows)]
mod installer_e2e;
mod package_docs;
#[cfg(windows)]
mod signing;
mod symbols;
mod typescript;
mod util;
#[cfg(unix)]
mod windows_lint;
mod workspace;

use clap::Parser;
use color_eyre::Result;

#[cfg(windows)]
use crate::{
    archive::build_archives,
    installer::build_installers,
    installer_e2e::run_installer_e2e,
    signing::sign_binaries,
    workspace::{read_notification_package_info, read_workspace_package_info},
};
use crate::{
    cli::{Cli, Commands},
    documentation::generate_docs,
    symbols::{generate_symbols, symbolicate},
    typescript::lint_e2e_typescript,
    workspace::workspace_root,
};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let workspace_root = workspace_root()?;

    match cli.command {
        #[cfg(unix)]
        Commands::AppImage => appimage::build_appimages(&workspace_root, true).await?,
        #[cfg(unix)]
        Commands::AppImageNoSign => appimage::build_appimages(&workspace_root, false).await?,
        Commands::Doc => generate_docs(&workspace_root).await?,
        Commands::LintTs => lint_e2e_typescript(&workspace_root)?,
        #[cfg(unix)]
        Commands::LintWindows => windows_lint::lint_windows(&workspace_root).await?,
        Commands::Symbols => generate_symbols(&workspace_root)?,
        Commands::Symbolicate { dump } => symbolicate(&workspace_root, &dump)?,
        #[cfg(windows)]
        Commands::Installer => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            let notification_package_info = read_notification_package_info(&workspace_root).await?;
            build_installers(
                &workspace_root,
                &workspace_package_info,
                &notification_package_info,
                true,
            )
            .await?;
        }
        #[cfg(windows)]
        Commands::InstallerNoSign => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            let notification_package_info = read_notification_package_info(&workspace_root).await?;
            build_installers(
                &workspace_root,
                &workspace_package_info,
                &notification_package_info,
                false,
            )
            .await?;
        }
        #[cfg(windows)]
        Commands::InstallerE2e => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            run_installer_e2e(&workspace_root, &workspace_package_info)?;
        }
        #[cfg(windows)]
        Commands::Archive => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            build_archives(&workspace_root, &workspace_package_info).await?;
        }
        #[cfg(windows)]
        Commands::ArchiveNoSign => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            build_archives(&workspace_root, &workspace_package_info).await?;
        }
        #[cfg(windows)]
        Commands::SignBinaries => sign_binaries(&workspace_root)?,
    }

    Ok(())
}
