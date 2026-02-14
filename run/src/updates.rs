use std::{
    io::{self, IsTerminal},
    time::Duration,
};

use actiona_core::{config::Config, updater::Updater};
use color_eyre::{Result, eyre::OptionExt, owo_colors::OwoColorize};
use console::Emoji;
use indicatif::HumanDuration;
use time::OffsetDateTime;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{instrument, warn};
use versions::SemVer;

use crate::{args::Args, built_info};

fn app_name() -> String {
    format!("actiona-{}", built_info::PKG_NAME)
}

#[instrument(skip_all)]
pub async fn check_updates(
    args: &Args,
    config: Config,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
) -> Result<()> {
    // CLI and env have a higher priority than settings
    let updates_enabled = args
        .disable_updates
        .unwrap_or_else(|| config.settings(|settings| !settings.disable_updates));

    let app_version =
        SemVer::new(built_info::PKG_VERSION).ok_or_eyre("failed to parse crate version")?;
    let (_updater, ready) = Updater::new(
        config.clone(),
        updates_enabled,
        &app_name(),
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
        } else {
            print_update_available(&version_info, &app_version);
        }
    }

    Ok(())
}

#[instrument(skip_all)]
pub async fn check_updates_now(config: Config) -> Result<()> {
    let app_version =
        SemVer::new(built_info::PKG_VERSION).ok_or_eyre("failed to parse crate version")?;

    let version_info =
        Updater::check_once(config.clone(), &app_name(), app_version.clone()).await?;
    let version_info = version_info.filter(|info| info.version > app_version);

    if let Err(err) = config
        .state_mut(|state| state.new_version_available = version_info.clone())
        .await
    {
        warn!("saving state failed: {err}");
    }

    if let Some(version_info) = version_info {
        print_update_available(&version_info, &app_version);
        return Ok(());
    }

    println!(
        "{} is up to date (version {}).",
        app_name(),
        built_info::PKG_VERSION
    );

    Ok(())
}

fn print_update_available(
    version_info: &actiona_core::config::state::VersionInfo,
    app_version: &SemVer,
) {
    if io::stdout().is_terminal() {
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

        let since = OffsetDateTime::now_utc() - version_info.release_date;
        let since = if let Ok(since) = Duration::try_from(since) {
            format!("{} ago", HumanDuration(since))
        } else {
            "just now".to_string()
        };

        println!(
            "You are running {} version {}, latest version is {},\nreleased {}.",
            app_name(),
            built_info::PKG_VERSION.bold(),
            version_info.version.bold(),
            since
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
        println!(
            "New version available for {}: {} (current {}). Download: {}",
            app_name(),
            version_info.version,
            app_version,
            version_info.download_url
        );
    }
}
