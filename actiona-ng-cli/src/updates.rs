use std::{
    io::{self, IsTerminal},
    sync::Arc,
};

use actiona_ng::{config::Config, updater::Updater};
use color_eyre::{Result, eyre::OptionExt, owo_colors::OwoColorize};
use console::Emoji;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::warn;
use versions::SemVer;

use crate::{args::Args, built_info};

pub async fn check_updates(
    args: &Args,
    config: Arc<Config>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
) -> Result<()> {
    // CLI and env have a higher priority than settings
    let updates_enabled = args.disable_updates.map_or_else(
        || config.settings(|settings| !settings.disable_updates),
        |enabled| enabled,
    );

    let app_version =
        SemVer::new(built_info::PKG_VERSION).ok_or_eyre("failed to parse crate version")?;
    let (_updater, ready) = Updater::new(
        config.clone(),
        updates_enabled,
        built_info::PKG_NAME,
        app_version.clone(),
        cancellation_token,
        task_tracker,
    );

    if !updates_enabled {
        return Ok(());
    }

    // Wait for the updater to be ready
    _ = ready.await;

    if let Some(version_info) = config.state(|state| state.new_version_available.clone()) {
        if version_info.version <= app_version {
            // The new version is older or the same as the one we are currently running
            if let Err(err) = config
                .state_mut(|state| state.new_version_available = None)
                .await
            {
                warn!("saving state failed: {err}");
            }
        } else if io::stdout().is_terminal() {
            // Display a message saying there is a new version available
            let warning_sign = Emoji("⚠️", "/!\\");
            let up_arrow = Emoji("🠱", "^");
            let down_arrow = Emoji("🠳", "v");
            let left_arrow = Emoji("🠰", "<");
            let right_arrow = Emoji("🠲", ">");
            for _ in 0..80 {
                print!("{}", down_arrow);
            }
            println!();
            println!(
                "{}  {} {}",
                warning_sign,
                "NEW VERSION AVAILABLE".bold(),
                warning_sign
            );
            println!(
                "You are running {} version {}, latest version is {},\nreleased {:.0} ago.",
                built_info::PKG_NAME,
                built_info::PKG_VERSION.bold(),
                version_info.version.bold(),
                (OffsetDateTime::now_utc() - version_info.release_date).max(time::Duration::ZERO)
            );
            println!(
                "Download: {}  {}  {}",
                right_arrow,
                version_info.download_url.bright_blue().underline(),
                left_arrow
            );
            for _ in 0..80 {
                print!("{}", up_arrow);
            }
            println!();
        } else {
            // TODO
        }
    }

    Ok(())
}
