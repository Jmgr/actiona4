#![warn(clippy::all, clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::unwrap_used)]
#![deny(unsafe_code)]

use std::{future::Future, sync::Arc};

use color_eyre::{Result, eyre::eyre};
use config::{Channel, Config, VersionInfo, state::State};
use indexmap::IndexMap;
use reqwest::Client;
use serde::Deserialize;
#[cfg(not(windows))]
use serde::Serialize;
#[cfg(not(windows))]
use strum::Display;
use sys_locale::get_locale;
use sysinfo::System;
use time::{Duration, OffsetDateTime};
use tokio::{sync::oneshot, time::sleep};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, warn};
use versions::SemVer;

const UPDATER_URL: &str = "https://updates.actiona.app/v1";
const MAX_ERROR_RESPONSE_LEN: usize = 128;

#[cfg(not(windows))]
#[derive(Clone, Copy, Debug, Display, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
enum LinuxDisplay {
    X11,
    XWayland,
    Wayland,
}

#[cfg(not(windows))]
impl LinuxDisplay {
    fn detect() -> Option<Self> {
        use std::env;

        match env::var("XDG_SESSION_TYPE").ok().as_deref() {
            Some("wayland") => Some(Self::Wayland),
            Some("x11") => Some(Self::X11),
            _ => {
                if env::var_os("WAYLAND_DISPLAY").is_some() {
                    Some(Self::XWayland)
                } else if env::var_os("DISPLAY").is_some() {
                    Some(Self::X11)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct UpdateResponse {
    update_available: bool,
    info: Option<VersionInfo>,
}

pub struct Updater {
    config: Config,
    app: String,
    app_version: SemVer,
}

impl Updater {
    pub async fn check_once(
        config: &Config,
        app: &str,
        app_version: SemVer,
        app_distribution: &str,
    ) -> Result<Option<VersionInfo>> {
        let updater = Self {
            config: config.clone(),
            app: app.to_string(),
            app_version,
        };

        let result = updater.check_updates(app_distribution).await?;

        if result.update_available {
            Ok(result.info)
        } else {
            Ok(None)
        }
    }

    #[must_use]
    pub fn new(
        config: Config,
        enabled: bool,
        app: &str,
        app_version: SemVer,
        app_distribution: &str,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> (Arc<Self>, oneshot::Receiver<()>) {
        let local_config = config.clone();
        let result = Arc::new(Self {
            config,
            app: app.to_string(),
            app_version,
        });

        info!("Updater enabled: {enabled}");

        let (sender, receiver) = oneshot::channel();

        if !enabled {
            _ = sender.send(());
            return (result, receiver);
        }

        let local_updater = result.clone();
        let mut sender = Some(sender);
        let app_distribution = app_distribution.to_string();
        task_tracker.spawn(async move {
            loop {
                let now = OffsetDateTime::now_utc();
                let next_update_check = local_config.state(|state| state.next_update_check);
                let next_update_check = next_update_check.unwrap_or(now);
                let should_check = next_update_check <= now;

                info!("Updater should_check: {should_check}");

                let next_check = if should_check {
                    info!("Updater checking");
                    let result = local_updater.check_updates(&app_distribution).await;

                    let next_check = match result {
                        Ok(result) => {
                            info!("Updater result {result:?}");

                            let new_version_available = if result.update_available {
                                result.info
                            } else {
                                None
                            };

                            if let Err(error) = local_config
                                .state_mut(|state| {
                                    apply_successful_check(state, new_version_available)
                                })
                                .await
                            {
                                warn!("saving state failed: {error}");
                            }

                            info!("Updater result saved");

                            Duration::days(1)
                        }
                        Err(error) => {
                            warn!("update check failed: {error}");

                            if let Err(save_error) =
                                local_config.state_mut(apply_failed_check).await
                            {
                                warn!("saving state failed: {save_error}");
                            }

                            Duration::hours(1)
                        }
                    };

                    if let Err(error) = local_config
                        .state_mut(|state| state.next_update_check = Some(now + next_check))
                        .await
                    {
                        warn!("saving state failed: {error}");
                    }

                    next_check
                } else {
                    (next_update_check - now).max(Duration::ZERO)
                };

                if let Some(sender) = sender.take() {
                    _ = sender.send(());
                }

                let Ok(next_check_duration) = next_check.try_into() else {
                    warn!("next update check duration is out of range: {next_check:?}");
                    break;
                };
                if cancel_on(&cancellation_token, sleep(next_check_duration))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        (result, receiver)
    }

    async fn check_updates(&self, app_distribution: &str) -> Result<UpdateResponse> {
        let client = Client::new();

        #[cfg(linux)]
        const OS_NAME: &str = "linux";

        #[cfg(windows)]
        const OS_NAME: &str = "windows";

        #[cfg(not(any(windows, linux)))]
        const OS_NAME: &str = "unknown";

        #[cfg(not(windows))]
        let display = LinuxDisplay::detect();

        #[cfg(windows)]
        let display: Option<String> = None;

        #[cfg(not(windows))]
        let distribution = Some(System::distribution_id());

        #[cfg(windows)]
        let distribution: Option<String> = None;

        let os_version = System::os_version();

        #[cfg(not(windows))]
        let os_display = display.map(|display| display.to_string());
        #[cfg(windows)]
        let os_display = display;

        let mut params = IndexMap::new();

        params.insert("app", self.app.clone());

        if let Some(client_id) = self.config.settings(|settings| settings.telemetry) {
            params.insert("client_id", client_id.to_string());
        }

        params.insert("app_channel", Channel::Stable.to_string());
        params.insert("app_version", self.app_version.to_string());
        params.insert("os_name", OS_NAME.to_string());

        if let Some(distribution) = distribution {
            params.insert("os_distribution", distribution);
        }

        if let Some(os_version) = os_version {
            params.insert("os_version", os_version);
        }

        params.insert("os_arch", System::cpu_arch());

        if let Some(os_locale) = get_locale() {
            params.insert("os_locale", os_locale);
        }

        if let Some(os_display) = os_display {
            params.insert("os_display", os_display);
        }

        params.insert("app_distribution", app_distribution.to_string());

        let request = client.post(UPDATER_URL).json(&params).build()?;
        let response = client
            .execute(request)
            .await
            .map_err(|error| eyre!("error sending update request\nnetwork error: {error}"))?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            let response_body = summarize_response_body(&body).unwrap_or(body);

            return Err(eyre!(
                "update server returned HTTP {status}\nresponse body:\n{response_body}"
            ));
        }

        let response = serde_json::from_str::<UpdateResponse>(&body).map_err(|error| {
            summarize_response_body(&body).map_or_else(
                || eyre!("error decoding update response (HTTP {status}): {error}"),
                |body| {
                    eyre!(
                        "error decoding update response (HTTP {status}): {error}\nresponse body:\n{body}"
                    )
                },
            )
        })?;

        Ok(response)
    }
}

fn apply_successful_check(state: &mut State, new_version_available: Option<VersionInfo>) {
    state.new_version_available = new_version_available;
    state.consecutive_update_check_failures = 0;
    state.last_update_check_failure_notice = None;
}

const fn apply_failed_check(state: &mut State) {
    state.consecutive_update_check_failures =
        state.consecutive_update_check_failures.saturating_add(1);
}

async fn cancel_on<T, F>(cancellation_token: &CancellationToken, future: F) -> Result<T>
where
    F: Future<Output = T>,
{
    tokio::select! {
        _ = cancellation_token.cancelled() => Err(eyre!("cancelled")),
        value = future => Ok(value),
    }
}

fn summarize_response_body(body: &str) -> Option<String> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    let mut summary = String::new();
    for (index, character) in body.chars().enumerate() {
        if index >= MAX_ERROR_RESPONSE_LEN {
            summary.push_str("\n... (response truncated)");
            break;
        }
        summary.push(character);
    }

    Some(summary)
}

#[cfg(test)]
mod tests {
    use config::{Channel, VersionInfo, state::State};
    use time::OffsetDateTime;
    use versions::SemVer;

    use super::{apply_failed_check, apply_successful_check};

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
    fn successful_check_stores_available_version_and_resets_failure_tracking() {
        let mut state = State {
            consecutive_update_check_failures: 12,
            last_update_check_failure_notice: Some(OffsetDateTime::UNIX_EPOCH),
            ..State::default()
        };

        apply_successful_check(&mut state, Some(version_info("1.2.3")));

        assert_eq!(
            state
                .new_version_available
                .as_ref()
                .map(|info| info.version.clone()),
            Some(SemVer::new("1.2.3").unwrap())
        );
        assert_eq!(state.consecutive_update_check_failures, 0);
        assert_eq!(state.last_update_check_failure_notice, None);
    }

    #[test]
    fn successful_check_without_update_clears_stale_state() {
        let mut state = State {
            new_version_available: Some(version_info("1.2.3")),
            consecutive_update_check_failures: 4,
            last_update_check_failure_notice: Some(OffsetDateTime::UNIX_EPOCH),
            ..State::default()
        };

        apply_successful_check(&mut state, None);

        assert!(state.new_version_available.is_none());
        assert_eq!(state.consecutive_update_check_failures, 0);
        assert_eq!(state.last_update_check_failure_notice, None);
    }

    #[test]
    fn failed_check_increments_failure_counter_without_touching_notice_timestamp() {
        let mut state = State {
            consecutive_update_check_failures: 9,
            last_update_check_failure_notice: Some(OffsetDateTime::UNIX_EPOCH),
            ..State::default()
        };

        apply_failed_check(&mut state);

        assert_eq!(state.consecutive_update_check_failures, 10);
        assert_eq!(
            state.last_update_check_failure_notice,
            Some(OffsetDateTime::UNIX_EPOCH)
        );
    }
}
