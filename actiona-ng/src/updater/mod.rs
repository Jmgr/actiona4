use std::sync::Arc;

use color_eyre::Result;
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
    config: Arc<Config>,
    app: String,
    app_version: SemVer,
}

impl Updater {
    #[must_use]
    pub fn new(
        config: Arc<Config>,
        enabled: bool,
        app: &str,
        app_version: SemVer,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> (Arc<Self>, oneshot::Receiver<()>) {
        let result = Arc::new(Self {
            config: config.clone(),
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
                let next_update_check = config.state(|state| state.next_update_check);
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
                                && let Err(err) = config
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

                    if let Err(err) = config
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
                if cancel_on(&cancellation_token, sleep(next_check.try_into().unwrap()))
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
        params.insert("os_distribution", System::distribution_id()); // TODO: remove option?

        if let Some(os_version) = os_version {
            params.insert("os_version", os_version);
        }

        params.insert("os_arch", System::cpu_arch());
        params.insert(
            "os_locale",
            get_locale().unwrap_or_else(|| String::from("en")),
        ); // TODO: set as option

        if let Some(os_display) = os_display {
            params.insert("os_display", os_display);
        }

        params.insert("app_distribution", "unknown".to_string()); // TODO: set as option
        params.insert("app_locale", "en".to_string()); // TODO: set as option

        let response = client.get(UPDATER_URL).query(&params).send().await?;
        let response = response.json::<UpdateResponse>().await?;

        Ok(response)
    }
}
