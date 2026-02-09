use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::store::Store;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub disable_updates: bool,
    pub telemetry: Option<Uuid>,
}

impl Settings {
    #[must_use]
    pub fn new_store(project_dirs: &ProjectDirs) -> Store<Self> {
        let directory = project_dirs.preference_dir().to_path_buf();

        Store::new(directory, "settings.toml")
    }
}
