use std::{
    io::{self, IsTerminal},
    time::Duration as StdDuration,
};

use color_eyre::{Result, eyre::OptionExt, owo_colors::OwoColorize};
use config::{Config, VersionInfo, state::State};
use console::Emoji;
use indicatif::HumanDuration;
use time::{Duration as TimeDuration, OffsetDateTime};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{instrument, warn};
use updater::Updater;
use versions::SemVer;

use crate::{args::Args, built_info};

#[cfg(linux)]
mod linux;
#[cfg(windows)]
mod windows;

#[cfg(linux)]
use linux as platform;
#[cfg(windows)]
use windows as platform;

const UPDATE_CHECK_FAILURE_NOTIFICATION_THRESHOLD: u32 = 10;
const UPDATE_CHECK_FAILURE_NOTIFICATION_COOLDOWN_DAYS: i64 = 7;
const UPDATE_CHECK_FAILURE_URL: &str = "https://actiona.app";

fn app_name() -> String {
    format!("actiona-{}", built_info::PKG_NAME)
}

#[instrument(skip_all)]
pub async fn check_updates(
    args: &Args,
    config: &Config,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
) -> Result<()> {
    // CLI and env have a higher priority than settings
    let update_check = args
        .update_check
        .unwrap_or_else(|| config.settings(|settings| settings.update_check));

    let app_version =
        SemVer::new(built_info::PKG_VERSION).ok_or_eyre("failed to parse crate version")?;
    let (_updater, ready) = Updater::new(
        config.clone(),
        update_check,
        &app_name(),
        app_version.clone(),
        built_info::PKG_NAME,
        cancellation_token,
        task_tracker.clone(),
    );

    if !update_check {
        return Ok(());
    }

    // Wait for the updater to be ready
    _ = ready.await;

    let mut state = config.state(|state| state.clone());

    if let Some(version_info) = &state.new_version_available {
        if version_info.version <= app_version {
            // The new version is older or the same as the one we are currently running
            if let Err(err) = config
                .state_mut(|state| state.new_version_available = None)
                .await
            {
                warn!("saving state failed: {err}");
            }
            state.new_version_available = None;
        } else {
            print_update_available(version_info, &app_version);
        }
    }

    platform::maybe_notify_update_available(config, &state, &app_version, task_tracker.clone())
        .await;
    platform::maybe_notify_update_check_failure(config, &state, &app_version, task_tracker).await;

    Ok(())
}

#[instrument(skip_all)]
pub async fn check_updates_now(config: &Config) -> Result<()> {
    let app_version =
        SemVer::new(built_info::PKG_VERSION).ok_or_eyre("failed to parse crate version")?;

    let version_info = Updater::check_once(
        config,
        &app_name(),
        app_version.clone(),
        built_info::PKG_NAME,
    )
    .await?;
    let version_info = version_info.filter(|info| info.version > app_version);

    if let Err(err) = config
        .state_mut(|state| {
            state.new_version_available = version_info.clone();
            state.consecutive_update_check_failures = 0;
            state.last_update_check_failure_notice = None;
        })
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

fn print_update_available(version_info: &VersionInfo, app_version: &SemVer) {
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
        let since = StdDuration::try_from(since).map_or_else(
            |_| "just now".to_string(),
            |since| format!("{} ago", HumanDuration(since)),
        );

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

fn update_available_for_notification<'a>(
    state: &'a State,
    app_version: &SemVer,
) -> Option<&'a VersionInfo> {
    let version_info = state.new_version_available.as_ref()?;

    if version_info.version <= *app_version {
        return None;
    }

    if state.last_notified_version.as_ref() == Some(&version_info.version) {
        return None;
    }

    Some(version_info)
}

fn should_notify_update_check_failure(
    state: &State,
    app_version: &SemVer,
    now: OffsetDateTime,
) -> bool {
    if state
        .new_version_available
        .as_ref()
        .is_some_and(|version_info| version_info.version > *app_version)
    {
        return false;
    }

    if state.consecutive_update_check_failures < UPDATE_CHECK_FAILURE_NOTIFICATION_THRESHOLD {
        return false;
    }

    state
        .last_update_check_failure_notice
        .is_none_or(|last_notice| {
            now - last_notice >= TimeDuration::days(UPDATE_CHECK_FAILURE_NOTIFICATION_COOLDOWN_DAYS)
        })
}

#[cfg(test)]
mod tests {
    use config::{Channel, VersionInfo, state::State};
    use time::{Duration as TimeDuration, OffsetDateTime};
    use versions::SemVer;

    use super::{
        UPDATE_CHECK_FAILURE_NOTIFICATION_COOLDOWN_DAYS,
        UPDATE_CHECK_FAILURE_NOTIFICATION_THRESHOLD, should_notify_update_check_failure,
        update_available_for_notification,
    };

    fn version_info(version: &str) -> VersionInfo {
        VersionInfo {
            app: "actiona-run".to_owned(),
            channel: Channel::Stable,
            version: SemVer::new(version).unwrap(),
            release_date: OffsetDateTime::UNIX_EPOCH,
            download_url: "https://example.com/download".to_owned(),
            changelog_url: "https://example.com/changelog".to_owned(),
        }
    }

    #[test]
    fn update_notification_is_shown_once_per_version() {
        let state = State {
            new_version_available: Some(version_info("1.2.3")),
            ..State::default()
        };
        let current_version = SemVer::new("1.2.2").unwrap();

        let notified_version = update_available_for_notification(&state, &current_version)
            .map(|version_info| version_info.version.clone());

        assert_eq!(notified_version, Some(SemVer::new("1.2.3").unwrap()));

        let state = State {
            last_notified_version: Some(SemVer::new("1.2.3").unwrap()),
            ..state
        };

        assert!(update_available_for_notification(&state, &current_version).is_none());
    }

    #[test]
    fn failure_notification_requires_threshold_and_cooldown() {
        let current_version = SemVer::new("1.2.2").unwrap();
        let now = OffsetDateTime::UNIX_EPOCH + TimeDuration::days(30);
        let below_threshold_state = State {
            consecutive_update_check_failures: UPDATE_CHECK_FAILURE_NOTIFICATION_THRESHOLD - 1,
            ..State::default()
        };

        assert!(!should_notify_update_check_failure(
            &below_threshold_state,
            &current_version,
            now
        ));

        let threshold_state = State {
            consecutive_update_check_failures: UPDATE_CHECK_FAILURE_NOTIFICATION_THRESHOLD,
            ..State::default()
        };

        assert!(should_notify_update_check_failure(
            &threshold_state,
            &current_version,
            now
        ));

        let recent_notice_state = State {
            last_update_check_failure_notice: Some(
                now - TimeDuration::days(UPDATE_CHECK_FAILURE_NOTIFICATION_COOLDOWN_DAYS - 1),
            ),
            ..threshold_state
        };

        assert!(!should_notify_update_check_failure(
            &recent_notice_state,
            &current_version,
            now
        ));
    }

    #[test]
    fn failure_notification_is_suppressed_when_an_update_is_known() {
        let state = State {
            new_version_available: Some(version_info("1.2.3")),
            consecutive_update_check_failures: UPDATE_CHECK_FAILURE_NOTIFICATION_THRESHOLD,
            ..State::default()
        };
        let current_version = SemVer::new("1.2.2").unwrap();
        let now = OffsetDateTime::UNIX_EPOCH + TimeDuration::days(30);

        assert!(!should_notify_update_check_failure(
            &state,
            &current_version,
            now
        ));
    }
}
