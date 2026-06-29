#![warn(clippy::all, clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::unwrap_used)]
#![deny(unsafe_code)]

use color_eyre::{Result, eyre::OptionExt};
use directories::ProjectDirs;
use serde::{Serialize, de::DeserializeOwned};

mod settings;
mod state;
mod store;

pub use crate::{
    settings::CommonSettings,
    settings::DEFAULT_TELEMETRY,
    settings::DEFAULT_UPDATE_CHECK,
    state::CommonState,
    state::{Channel, VersionInfo},
    store::Store,
};

/// Canonical application identity. All Actiona apps (the runner, the editor,
/// ...) share the same on-disk configuration directories so that common
/// settings such as `update_check` and `telemetry` are stored only once.
const QUALIFIER: &str = "app.actiona";
const ORGANIZATION: &str = "Actiona";
const APPLICATION: &str = "Actiona";

/// Resolve the shared per-user directories for the application.
fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_eyre("failed to get project directories")
}

/// Create a [`Store`] for user *settings*, backed by a file in the shared
/// settings directory.
///
/// Use this for user-editable configuration that should live next to the
/// common settings. App-specific crates (e.g. the editor) use this to add
/// their own settings sections without duplicating directory logic.
pub fn settings_store<T>(filename: &'static str) -> Result<Store<T>>
where
    T: Serialize + DeserializeOwned + Default + Send + Sync,
{
    let project_dirs = project_dirs()?;
    let directory = project_dirs.preference_dir().to_path_buf();

    Ok(Store::new(directory, filename))
}

/// Create a [`Store`] backed by a file in the shared *state* directory.
///
/// State is machine-local, app-managed data (caches, bookkeeping) as opposed
/// to user settings.
pub fn state_store<T>(filename: &'static str) -> Result<Store<T>>
where
    T: Serialize + DeserializeOwned + Default + Send + Sync,
{
    let project_dirs = project_dirs()?;

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

    Ok(Store::new(directory, filename))
}

/// Configuration sections shared by every Actiona application.
///
/// Each application composes its top-level configuration around this core:
/// the runner uses it directly, while the editor embeds it alongside its own
/// [`Store`]s created via [`settings_store`] / [`state_store`].
#[derive(Clone, Debug)]
pub struct CommonConfig {
    settings: Store<CommonSettings>,
    state: Store<CommonState>,
}

impl CommonConfig {
    pub async fn new() -> Result<Self> {
        let result = Self {
            settings: CommonSettings::new_store()?,
            state: CommonState::new_store()?,
        };

        result.settings.load().await?;
        result.state.load().await?;

        Ok(result)
    }

    pub fn settings<R>(&self, operation: impl FnOnce(&CommonSettings) -> R) -> R {
        self.settings.with(operation)
    }

    pub async fn settings_mut<R>(
        &self,
        operation: impl FnOnce(&mut CommonSettings) -> R + Send,
    ) -> Result<R>
    where
        R: Send,
    {
        let result = self.settings.with_mut(operation).await?;

        // Consider that the first time initialization is done if we have already changed a setting
        self.state_mut(|state| state.first_time_init = true).await?;

        Ok(result)
    }

    pub fn state<R>(&self, operation: impl FnOnce(&CommonState) -> R) -> R {
        self.state.with(operation)
    }

    pub async fn state_mut<R>(&self, operation: impl FnOnce(&mut CommonState) -> R + Send) -> Result<R>
    where
        R: Send,
    {
        self.state.with_mut(operation).await
    }
}
