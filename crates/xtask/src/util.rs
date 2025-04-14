#[cfg(windows)]
use std::path::Path;
use std::process::Command;

use color_eyre::{Result, eyre::eyre};

pub fn run_command(command: &mut Command, failure_message: &str) -> Result<()> {
    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(eyre!(
            "{failure_message} Command exited with status {status}."
        ))
    }
}

#[cfg(windows)]
pub async fn remove_file_if_exists(file_path: &Path) -> Result<()> {
    if tokio::fs::try_exists(file_path).await? {
        tokio::fs::remove_file(file_path).await?;
    }

    Ok(())
}
