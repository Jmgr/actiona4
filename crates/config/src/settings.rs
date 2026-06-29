use color_eyre::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::store::Store;

pub const DEFAULT_UPDATE_CHECK: bool = true;
pub const DEFAULT_TELEMETRY: bool = false;

#[derive(Debug, Deserialize, Serialize)]
pub struct CommonSettings {
    pub update_check: bool,
    pub telemetry: Option<Uuid>,
}

impl Default for CommonSettings {
    fn default() -> Self {
        let mut settings = Self {
            update_check: DEFAULT_UPDATE_CHECK,
            telemetry: None,
        };

        settings.set_telemetry(DEFAULT_TELEMETRY);

        settings
    }
}

impl CommonSettings {
    pub fn new_store() -> Result<Store<Self>> {
        crate::settings_store("settings.toml")
    }

    pub fn set_telemetry(&mut self, enabled: bool) {
        if enabled {
            if self.telemetry.is_none() {
                self.telemetry = Some(Uuid::new_v4());
            }
        } else {
            self.telemetry = None;
        }
    }
}
