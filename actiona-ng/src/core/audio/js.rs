use std::path::Path;

use rquickjs::{
    JsLifetime, Result,
    class::{Trace, Tracer},
    prelude::*,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    IntoJsResult,
    core::{
        audio::{Audio, PlayingSound},
        js::classes::{HostClass, SingletonClass, register_host_class},
    },
};

pub type JsPlaySoundOptions = super::PlaySoundOptions;

/// @singleton
#[derive(JsLifetime)]
#[rquickjs::class(rename = "Audio")]
pub struct JsAudio {
    inner: Audio,
}

impl<'js> Trace<'js> for JsAudio {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsAudio {
    fn register_dependencies(ctx: &Ctx<'js>) -> Result<()> {
        register_host_class::<JsPlayingSound>(ctx)?;

        Ok(())
    }
}

impl JsAudio {
    /// @skip
    pub fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> color_eyre::Result<Self> {
        Ok(Self {
            inner: Audio::new(cancellation_token, task_tracker)?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsAudio {
    pub fn play_file(
        &self,
        ctx: Ctx<'_>,
        path: String,
        options: Opt<JsPlaySoundOptions>,
    ) -> Result<JsPlayingSound> {
        let options = options.unwrap_or_default();
        let playing_sound = self
            .inner
            .play_file(Path::new(&path), options)
            .into_js_result(&ctx)?;
        Ok(JsPlayingSound::new(playing_sound))
    }

    pub async fn play_file_and_wait(
        &self,
        ctx: Ctx<'_>,
        path: String,
        options: Opt<JsPlaySoundOptions>,
    ) -> Result<()> {
        let options = options.unwrap_or_default();
        self.inner
            .play_file_and_wait(Path::new(&path), options)
            .await
            .into_js_result(&ctx)?;
        Ok(())
    }
}

/// PlayingSound
///
/// @prop volume: number = `1` // Sound volume
/// @prop playbackRate: number = `1` // Sound playing speed
#[derive(JsLifetime)]
#[rquickjs::class(rename = "PlayingSound")]
pub struct JsPlayingSound {
    inner: PlayingSound,
}

impl<'js> HostClass<'js> for JsPlayingSound {}

impl<'js> Trace<'js> for JsPlayingSound {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsPlayingSound {
    /// @skip
    #[must_use]
    pub const fn new(inner: PlayingSound) -> Self {
        Self { inner }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsPlayingSound {
    /// Is the sound paused
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn paused(&self) -> bool {
        self.inner.is_paused()
    }

    pub fn pause(&self) {
        self.inner.pause();
    }

    pub fn resume(&self) {
        self.inner.resume();
    }

    pub fn stop(&self) {
        self.inner.stop();
    }

    /// @skip
    #[qjs(get, rename = "volume")]
    #[must_use]
    pub fn get_volume(&self) -> f32 {
        self.inner.volume()
    }

    /// @skip
    #[qjs(set, rename = "volume")]
    pub fn set_volume(&mut self, volume: f32) {
        self.inner.set_volume(volume);
    }

    /// @skip
    #[qjs(get, rename = "playbackRate")]
    #[must_use]
    pub fn get_playback_rate(&self) -> f32 {
        self.inner.playback_rate()
    }

    /// @skip
    #[qjs(set, rename = "playbackRate")]
    pub fn set_playback_rate(&mut self, playback_rate: f32) {
        self.inner.set_playback_rate(playback_rate);
    }

    /// The duration of the sound in seconds
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn duration(&self) -> Option<f64> {
        let duration = self.inner.duration()?;
        Some(duration.as_secs_f64())
    }

    /// Promise allowing to wait until the sound has finished playing
    /// @get
    #[qjs(get)]
    #[must_use]
    pub async fn finished(&self) {
        self.inner.wait_finished().await;
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::runtime::Runtime;

    #[test]
    #[traced_test]
    #[ignore]
    fn test_set_text() {
        Runtime::test_with_script_engine(|_script_engine| async move {});
    }
}
