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
mod signing;
mod symbols;
mod typescript;
mod util;
mod workspace;

use clap::Parser;
use color_eyre::Result;

#[cfg(windows)]
use crate::{
    archive::build_archive,
    installer::build_installer,
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
        Commands::AppImage => appimage::build_appimage(&workspace_root, true).await?,
        #[cfg(unix)]
        Commands::AppImageNoSign => appimage::build_appimage(&workspace_root, false).await?,
        Commands::Doc => generate_docs(&workspace_root).await?,
        Commands::LintTs => lint_e2e_typescript(&workspace_root)?,
        Commands::Symbols => generate_symbols(&workspace_root)?,
        Commands::Symbolicate { dump } => symbolicate(&workspace_root, &dump)?,
        #[cfg(windows)]
        Commands::Installer => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            let notification_package_info = read_notification_package_info(&workspace_root).await?;
            build_installer(
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
            build_installer(
                &workspace_root,
                &workspace_package_info,
                &notification_package_info,
                false,
            )
            .await?;
        }
        #[cfg(windows)]
        Commands::Archive => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            build_archive(&workspace_root, &workspace_package_info).await?;
        }
        #[cfg(windows)]
        Commands::ArchiveNoSign => {
            let workspace_package_info = read_workspace_package_info(&workspace_root).await?;
            build_archive(&workspace_root, &workspace_package_info).await?;
        }
        #[cfg(windows)]
        Commands::SignBinaries => sign_binaries(&workspace_root)?,
    }

    Ok(())
}
