use color_eyre::Result;
use serde::{Deserialize, Serialize};
use strum::Display;
use time::OffsetDateTime;
use versions::SemVer;

use crate::store::Store;

#[derive(Clone, Copy, Debug, Deserialize, Display, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Channel {
    Stable,
    Beta,
    Dev,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VersionInfo {
    pub app: String,
    pub channel: Channel,
    pub version: SemVer,
    #[serde(with = "time::serde::iso8601")]
    pub release_date: OffsetDateTime,
    pub download_url: String,
    pub changelog_url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CommonState {
    #[serde(default, with = "time::serde::iso8601::option")]
    pub next_update_check: Option<OffsetDateTime>,
    #[serde(default)]
    pub new_version_available: Option<VersionInfo>,
    #[serde(default)]
    pub last_notified_version: Option<SemVer>,
    #[serde(default)]
    pub consecutive_update_check_failures: u32,
    #[serde(default, with = "time::serde::iso8601::option")]
    pub last_update_check_failure_notice: Option<OffsetDateTime>,
    #[serde(default)]
    pub first_time_init: bool,
}

impl CommonState {
    pub fn new_store() -> Result<Store<Self>> {
        crate::state_store("state.toml")
    }
}
