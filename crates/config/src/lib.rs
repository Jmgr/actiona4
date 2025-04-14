#![warn(clippy::all, clippy::nursery)]
#![warn(clippy::as_conversions)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::unwrap_used)]
#![deny(unsafe_code)]

use color_eyre::{Result, eyre::OptionExt};
use directories::ProjectDirs;

use crate::{settings::Settings, state::State, store::Store};

pub mod settings;
pub mod state;
pub mod store;

pub use crate::state::{Channel, VersionInfo};

#[derive(Clone, Debug)]
pub struct Config {
    settings: Store<Settings>,
    state: Store<State>,
}

impl Config {
    pub async fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("app.actiona-run", "Actiona", "Actiona-run")
            .ok_or_eyre("failed to get project directories")?;

        let result = Self {
            settings: Settings::new_store(&project_dirs),
            state: State::new_store(&project_dirs),
        };

        result.settings.load().await?;
        result.state.load().await?;

        Ok(result)
    }

    pub fn settings<R>(&self, operation: impl FnOnce(&Settings) -> R) -> R {
        self.settings.with(operation)
    }

    pub async fn settings_mut<R>(
        &self,
        operation: impl FnOnce(&mut Settings) -> R + Send,
    ) -> Result<R>
    where
        R: Send,
    {
        let result = self.settings.with_mut(operation).await?;

        // Consider that the first time initialization is done if we have already changed a setting
        self.state_mut(|state| state.first_time_init = true).await?;

        Ok(result)
    }

    pub fn state<R>(&self, operation: impl FnOnce(&State) -> R) -> R {
        self.state.with(operation)
    }

    pub async fn state_mut<R>(&self, operation: impl FnOnce(&mut State) -> R + Send) -> Result<R>
    where
        R: Send,
    {
        self.state.with_mut(operation).await
    }
}
