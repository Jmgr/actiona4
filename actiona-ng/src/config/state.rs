use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use strum::Display;
use time::OffsetDateTime;
use versions::SemVer;

use crate::config::store::Store;

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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct State {
    #[serde(with = "time::serde::iso8601::option")]
    pub next_update_check: Option<OffsetDateTime>,
    pub new_version_available: Option<VersionInfo>,
}

impl State {
    #[must_use]
    pub fn new_store(project_dirs: &ProjectDirs) -> Store<Self> {
        let directory = {
            #[cfg(linux)]
            {
                project_dirs
                    .state_dir()
                    .unwrap_or_else(|| project_dirs.config_local_dir())
                    .to_path_buf()
            }
            #[cfg(not(linux))]
            {
                project_dirs.config_local_dir().to_path_buf()
            }
        };

        Store::new(directory, "state.toml")
    }
}
