use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::store::Store;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub update_check: bool,
    pub telemetry: Option<Uuid>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            update_check: true,
            telemetry: None,
        }
    }
}

impl Settings {
    #[must_use]
    pub fn new_store(project_dirs: &ProjectDirs) -> Store<Self> {
        let directory = project_dirs.preference_dir().to_path_buf();

        Store::new(directory, "settings.toml")
    }

    pub fn set_telemetry(&mut self, enabled: bool) {
        if enabled && self.telemetry.is_none() {
            self.telemetry = Some(Uuid::new_v4());
        }
    }
}
