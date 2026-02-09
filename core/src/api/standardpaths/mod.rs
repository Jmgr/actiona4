use std::fmt::Display;

use directories::{BaseDirs, UserDirs};
use tracing::instrument;

use crate::types::{OptionalPath, display::DisplayFields};

pub mod js;

#[derive(Clone, Debug)]
pub struct StandardPaths {
    base: Option<BaseDirs>,
    user: Option<UserDirs>,
}

impl Default for StandardPaths {
    #[instrument(skip_all)]
    fn default() -> Self {
        Self {
            base: BaseDirs::new(),
            user: UserDirs::new(),
        }
    }
}

impl Display for StandardPaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display_if_some("home", &self.home())
            .display_if_some("cache", &self.cache())
            .display_if_some("downloads", &self.downloads())
            .display_if_some("desktop", &self.desktop())
            .display_if_some("config", &self.config())
            .display_if_some("pictures", &self.pictures())
            .display_if_some("local_config", &self.local_config())
            .display_if_some("videos", &self.videos())
            .display_if_some("public", &self.public())
            .display_if_some("documents", &self.documents())
            .display_if_some("music", &self.music())
            .finish(f)
    }
}

impl StandardPaths {
    #[must_use]
    pub fn home(&self) -> OptionalPath {
        self.base.as_ref().map(|base| base.home_dir()).into()
    }

    #[must_use]
    pub fn music(&self) -> OptionalPath {
        self.user.as_ref().and_then(|user| user.audio_dir()).into()
    }

    #[must_use]
    pub fn desktop(&self) -> OptionalPath {
        self.user
            .as_ref()
            .and_then(|user| user.desktop_dir())
            .into()
    }

    #[must_use]
    pub fn documents(&self) -> OptionalPath {
        self.user
            .as_ref()
            .and_then(|user| user.document_dir())
            .into()
    }

    #[must_use]
    pub fn downloads(&self) -> OptionalPath {
        self.user
            .as_ref()
            .and_then(|user| user.download_dir())
            .into()
    }

    #[must_use]
    pub fn pictures(&self) -> OptionalPath {
        self.user
            .as_ref()
            .and_then(|user| user.picture_dir())
            .into()
    }

    #[must_use]
    pub fn public(&self) -> OptionalPath {
        self.user.as_ref().and_then(|user| user.public_dir()).into()
    }

    #[must_use]
    pub fn videos(&self) -> OptionalPath {
        self.user.as_ref().and_then(|user| user.video_dir()).into()
    }

    #[must_use]
    pub fn cache(&self) -> OptionalPath {
        self.base.as_ref().map(|base| base.cache_dir()).into()
    }

    #[must_use]
    pub fn config(&self) -> OptionalPath {
        self.base.as_ref().map(|base| base.config_dir()).into()
    }

    #[must_use]
    pub fn local_config(&self) -> OptionalPath {
        self.base
            .as_ref()
            .map(|base| base.config_local_dir())
            .into()
    }
}
