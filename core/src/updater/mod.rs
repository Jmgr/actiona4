use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};
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

use crate::{
    cancel_on,
    config::{
        Config,
        state::{Channel, VersionInfo},
    },
};

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
        config: Config,
        app: &str,
        app_version: SemVer,
    ) -> Result<Option<VersionInfo>> {
        let updater = Self {
            config,
            app: app.to_string(),
            app_version,
        };

        let result = updater.check_updates().await?;

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
        task_tracker.spawn(async move {
            loop {
                let now = OffsetDateTime::now_utc();
                let next_update_check = local_config.state(|state| state.next_update_check);
                let next_update_check = next_update_check.unwrap_or(now);
                let should_check = next_update_check <= now;

                info!("Updater should_check: {should_check}");

                let next_check = if should_check {
                    info!("Updater checking");
                    // Check
                    let result = local_updater.check_updates().await;

                    let next_check = match result {
                        Ok(result) => {
                            info!("Updater result {result:?}");

                            if result.update_available
                                && let Some(version_info) = result.info
                                && let Err(err) = local_config
                                    .state_mut(|state| {
                                        state.new_version_available = Some(version_info)
                                    })
                                    .await
                            {
                                warn!("saving state failed: {err}");
                            }

                            info!("Updater result saved");

                            // Check again tomorrow
                            Duration::days(24)
                        }
                        Err(err) => {
                            warn!("update check failed: {err}");

                            // Check again in one hour
                            Duration::hours(1)
                        }
                    };

                    if let Err(err) = local_config
                        .state_mut(|state| state.next_update_check = Some(now + next_check))
                        .await
                    {
                        warn!("saving state failed: {err}");
                    }

                    next_check
                } else {
                    (next_update_check - now).max(Duration::ZERO)
                };

                if let Some(sender) = sender.take() {
                    _ = sender.send(());
                }

                // Wait until next check
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

    async fn check_updates(&self) -> Result<UpdateResponse> {
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

        // TODO: detect app distribution, if possible.
        // params.insert("app_distribution", "unknown".to_string());

        // actiona-run is not translated for now.
        // params.insert("app_locale", "en".to_string());

        let request = client.get(UPDATER_URL).query(&params).build()?;
        let response = client
            .execute(request)
            .await
            .map_err(|err| eyre!("error sending update request\nnetwork error: {err}"))?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            if let Some(body) = summarize_response_body(&body) {
                return Err(eyre!(
                    "update server returned HTTP {status}\nresponse body:\n{body}"
                ));
            } else {
                return Err(eyre!(
                    "update server returned HTTP {status}\nresponse body:\n{body}"
                ));
            }
        }

        let response = serde_json::from_str::<UpdateResponse>(&body).map_err(|err| {
            summarize_response_body(&body)
                .map_or_else(|| eyre!("error decoding update response (HTTP {status}): {err}"),
                    |body| eyre!("error decoding update response (HTTP {status}): {err}\nresponse body:\n{body}"))
        })?;

        Ok(response)
    }
}

fn summarize_response_body(body: &str) -> Option<String> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    let mut summary = String::new();
    for (index, ch) in body.chars().enumerate() {
        if index >= MAX_ERROR_RESPONSE_LEN {
            summary.push_str("\n... (response truncated)");
            break;
        }
        summary.push(ch);
    }

    Some(summary)
}
