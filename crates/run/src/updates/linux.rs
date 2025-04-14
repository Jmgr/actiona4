use actiona_core::api::notification::{Notification as DesktopNotification, NotificationOptions};
use color_eyre::Result;
use config::{Config, state::State};
use time::OffsetDateTime;
use tokio_util::task::TaskTracker;
use tracing::warn;
use versions::SemVer;

use super::{
    UPDATE_CHECK_FAILURE_URL, should_notify_update_check_failure, update_available_for_notification,
};

pub(super) async fn maybe_notify_update_available(
    config: &Config,
    state: &State,
    app_version: &SemVer,
    task_tracker: TaskTracker,
) {
    let Some(version_info) = update_available_for_notification(state, app_version) else {
        return;
    };

    let notified_version = version_info.version.clone();
    if let Err(error) = config
        .state_mut(|state| state.last_notified_version = Some(notified_version))
        .await
    {
        warn!("saving state failed: {error}");
    }

    let title = "Actiona Run update available";
    let body = format!(
        "Version {} is available. Download: {}",
        version_info.version, version_info.download_url
    );

    if let Err(error) = show_notification(title, &body, task_tracker).await {
        warn!("showing update notification failed: {error}");
    }
}

pub(super) async fn maybe_notify_update_check_failure(
    config: &Config,
    state: &State,
    app_version: &SemVer,
    task_tracker: TaskTracker,
) {
    if !should_notify_update_check_failure(state, app_version, OffsetDateTime::now_utc()) {
        return;
    }

    let noticed_at = OffsetDateTime::now_utc();
    if let Err(error) = config
        .state_mut(|state| state.last_update_check_failure_notice = Some(noticed_at))
        .await
    {
        warn!("saving state failed: {error}");
    }

    let title = "Unable to check for Actiona Run updates";
    let body = format!(
        "Actiona Run has been unable to check for updates recently. Visit {UPDATE_CHECK_FAILURE_URL} to see if a new version is available."
    );

    if let Err(error) = show_notification(title, &body, task_tracker).await {
        warn!("showing update failure notification failed: {error}");
    }
}

async fn show_notification(title: &str, body: &str, task_tracker: TaskTracker) -> Result<()> {
    if !desktop_notifications_available() {
        return Ok(());
    }

    let notification = DesktopNotification::new(task_tracker);
    _ = notification
        .show(NotificationOptions {
            title: Some(title.to_owned()),
            body: Some(body.to_owned()),
            app_name: Some("Actiona".to_owned()),
            transient: Some(false),
            ..NotificationOptions::default()
        })
        .await?;

    Ok(())
}

fn desktop_notifications_available() -> bool {
    std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some()
        || std::env::var_os("DISPLAY").is_some()
        || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

#[cfg(test)]
mod tests {
    use std::time::Duration as StdDuration;

    use config::{Channel, VersionInfo};
    use time::OffsetDateTime;
    use tokio_util::task::TaskTracker;
    use versions::SemVer;

    use super::{UPDATE_CHECK_FAILURE_URL, desktop_notifications_available, show_notification};

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
    #[ignore]
    fn show_update_available_notification_preview() {
        let version_info = version_info("1.2.3");
        let title = "Actiona Run update available";
        let body = format!(
            "Version {} is available. Download: {}",
            version_info.version, version_info.download_url
        );

        preview_notification(title, &body);
    }

    #[test]
    #[ignore]
    fn show_update_check_failure_notification_preview() {
        let title = "Unable to check for Actiona Run updates";
        let body = format!(
            "Actiona Run has been unable to check for updates recently. Visit {UPDATE_CHECK_FAILURE_URL} to see if a new version is available."
        );

        preview_notification(title, &body);
    }

    fn preview_notification(title: &str, body: &str) {
        if !desktop_notifications_available() {
            eprintln!(
                "Skipping notification preview because no Linux desktop session was detected."
            );
            return;
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            show_notification(title, body, TaskTracker::new())
                .await
                .unwrap();
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        });
    }
}
