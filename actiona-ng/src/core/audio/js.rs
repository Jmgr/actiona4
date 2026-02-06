use std::{path::Path, sync::Arc};

use macros::FromJsObject;
use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::*,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::instrument;

use crate::{
    IntoJsResult,
    core::{
        audio::{Audio, PlayingSound},
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_host_class},
            duration::JsDuration,
            task::{task, task_with_token},
        },
    },
};

/// Play sound options
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsPlaySoundOptions {
    /// Volume to play the sound at
    /// @default `1.0`
    pub volume: f32,

    /// Speed to play the sound at
    /// @default `1.0`
    pub playback_rate: f32,

    /// Should the sound start paused
    /// @default `false`
    pub paused: bool,

    /// Should the sound loop
    /// @default `false`
    pub r#loop: bool,

    /// Fade in duration
    /// @default `0`
    pub fade_in: Option<JsDuration>,

    /// Fade out duration
    /// @default `0`
    pub fade_out: Option<JsDuration>,

    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl JsPlaySoundOptions {
    fn into_inner(self) -> super::PlaySoundOptions {
        super::PlaySoundOptions {
            volume: self.volume,
            playback_rate: self.playback_rate,
            paused: self.paused,
            r#loop: self.r#loop,
            fade_in: self.fade_in,
            fade_out: self.fade_out,
        }
    }
}

/// @singleton
#[derive(JsLifetime)]
#[rquickjs::class(rename = "Audio")]
pub struct JsAudio {
    inner: Arc<Audio>,
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
    #[instrument(skip_all)]
    pub fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> color_eyre::Result<Self> {
        Ok(Self {
            inner: Arc::new(Audio::new(cancellation_token, task_tracker)?),
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
        let options = options.0.unwrap_or_default();
        let playing_sound = self
            .inner
            .play_file(Path::new(&path), options.into_inner())
            .into_js_result(&ctx)?;
        Ok(JsPlayingSound::new(playing_sound))
    }

    /// @returns Task<void>
    pub fn play_file_and_wait<'js>(
        &self,
        ctx: Ctx<'js>,
        path: String,
        options: Opt<JsPlaySoundOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let local_audio = self.inner.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            local_audio
                .play_file_and_wait(Path::new(&path), options.into_inner(), token)
                .await
                .into_js_result(&ctx)
        })
    }
}

/// PlayingSound
///
/// @prop volume: number = `1` // Sound volume
/// @prop playbackRate: number = `1` // Sound playing speed
#[derive(JsLifetime)]
#[rquickjs::class(rename = "PlayingSound")]
pub struct JsPlayingSound {
    inner: Arc<PlayingSound>,
}

impl<'js> HostClass<'js> for JsPlayingSound {}

impl<'js> Trace<'js> for JsPlayingSound {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsPlayingSound {
    /// @skip
    #[must_use]
    pub fn new(inner: PlayingSound) -> Self {
        Self {
            inner: Arc::new(inner),
        }
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

    /// Await to wait until the sound has finished playing
    /// @get
    /// @returns Promise<void>
    #[qjs(get)]
    pub fn finished<'js>(&self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
        let local_sound = self.inner.clone();

        task(ctx, async move |_ctx, token| {
            local_sound.wait_finished(token).await;
            Ok(())
        })
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
