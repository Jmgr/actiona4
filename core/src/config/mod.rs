use color_eyre::{Result, eyre::OptionExt};
use directories::ProjectDirs;

use crate::config::{settings::Settings, state::State, store::Store};

pub mod settings;
pub mod state;
pub mod store;

#[derive(Clone, Debug)]
pub struct Config {
    settings: Store<Settings>,
    state: Store<State>,
}

impl Config {
    pub async fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("app.actiona4-run", "Actiona", "Actiona4-run")
            .ok_or_eyre("failed to get project directories")?;

        let result = Self {
            settings: Settings::new_store(&project_dirs),
            state: State::new_store(&project_dirs),
        };

        result.settings.load().await?;
        result.state.load().await?;

        Ok(result)
    }

    pub fn settings<R>(&self, f: impl FnOnce(&Settings) -> R) -> R {
        self.settings.with(f)
    }

    pub async fn settings_mut<R>(&self, f: impl FnOnce(&mut Settings) -> R) -> Result<R> {
        self.settings.with_mut(f).await
    }

    pub fn state<R>(&self, f: impl FnOnce(&State) -> R) -> R {
        self.state.with(f)
    }

    pub async fn state_mut<R>(&self, f: impl FnOnce(&mut State) -> R) -> Result<R> {
        self.state.with_mut(f).await
    }
}
