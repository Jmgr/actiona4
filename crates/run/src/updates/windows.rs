use actiona_core::api::notification::{
    Notification as DesktopNotification, NotificationAction, NotificationActivationType,
    NotificationOptions,
};
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
    let actions = vec![protocol_action("Download", &version_info.download_url)];

    if let Err(error) = show_notification(
        title,
        &body,
        actions,
        Some(format!("update-available-{}", version_info.version)),
        task_tracker,
    )
    .await
    {
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
    let actions = vec![protocol_action("Open website", UPDATE_CHECK_FAILURE_URL)];

    if let Err(error) = show_notification(
        title,
        &body,
        actions,
        Some("update-check-failure".to_owned()),
        task_tracker,
    )
    .await
    {
        warn!("showing update failure notification failed: {error}");
    }
}

fn protocol_action(label: &str, url: &str) -> NotificationAction {
    NotificationAction {
        identifier: url.to_owned(),
        label: label.to_owned(),
        action_type: None,
        activation_type: Some(NotificationActivationType::Protocol),
        placement: None,
        button_style: None,
        input_id: None,
    }
}

async fn show_notification(
    title: &str,
    body: &str,
    actions: Vec<NotificationAction>,
    tag: Option<String>,
    task_tracker: TaskTracker,
) -> Result<()> {
    let notification = DesktopNotification::new(task_tracker);
    _ = notification
        .show(NotificationOptions {
            title: Some(title.to_owned()),
            body: Some(body.to_owned()),
            actions,
            tag,
            group: Some("updates".to_owned()),
            ..NotificationOptions::default()
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration as StdDuration;

    use actiona_core::api::notification::NotificationAction;
    use config::{Channel, VersionInfo};
    use time::OffsetDateTime;
    use tokio_util::task::TaskTracker;
    use versions::SemVer;

    use super::{UPDATE_CHECK_FAILURE_URL, protocol_action, show_notification};

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
        let actions = vec![protocol_action("Download", &version_info.download_url)];

        preview_notification(title, &body, actions, "update-available-preview");
    }

    #[test]
    #[ignore]
    fn show_update_check_failure_notification_preview() {
        let title = "Unable to check for Actiona Run updates";
        let body = format!(
            "Actiona Run has been unable to check for updates recently. Visit {UPDATE_CHECK_FAILURE_URL} to see if a new version is available."
        );
        let actions = vec![protocol_action("Open website", UPDATE_CHECK_FAILURE_URL)];

        preview_notification(title, &body, actions, "update-check-failure-preview");
    }

    fn preview_notification(title: &str, body: &str, actions: Vec<NotificationAction>, tag: &str) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            show_notification(
                title,
                body,
                actions,
                Some(tag.to_owned()),
                TaskTracker::new(),
            )
            .await
            .unwrap();
            tokio::time::sleep(StdDuration::from_secs(2)).await;
        });
    }
}
