use std::{path::Path, sync::Arc};

use macros::FromJsObject;
use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::*,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        audio::{Audio, PlayingSound, PlayingSoundsTracker},
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_host_class},
            duration::JsDuration,
            task::{task, task_with_token},
        },
    },
    types::display::display_with_type,
};

/// Options for playing a sound file.
///
/// ```ts
/// // Play with default options
/// audio.playFile("sound.wav");
///
/// // Play at half volume, looping, with a fade in
/// audio.playFile("music.mp3", {
///     volume: 0.5,
///     loop: true,
///     fadeIn: 2000,
/// });
/// ```
///
/// @options
#[derive(Clone, Debug, FromJsObject)]
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

    /// Abort signal to cancel the sound playback.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl Default for JsPlaySoundOptions {
    fn default() -> Self {
        Self {
            volume: 1.0,
            playback_rate: 1.0,
            paused: false,
            r#loop: false,
            fade_in: None,
            fade_out: None,
            signal: None,
        }
    }
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

/// The global audio singleton for playing sound files.
///
/// ```ts
/// // Play a sound and forget about it
/// audio.playFile("notification.wav");
///
/// // Play a sound and wait for it to finish
/// await audio.playFileAndWait("alert.wav");
///
/// // Play with options and control playback
/// const sound = audio.playFile("music.mp3", { volume: 0.8, loop: true });
/// sound.pause();
/// sound.resume();
/// sound.stop();
/// ```
///
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
    #[instrument(skip_all)]
    pub fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        playing_sounds_tracker: Arc<PlayingSoundsTracker>,
    ) -> color_eyre::Result<Self> {
        Ok(Self {
            inner: Audio::new(cancellation_token, task_tracker, playing_sounds_tracker)?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsAudio {
    /// Plays a sound file and returns a `PlayingSound` handle for controlling playback.
    ///
    /// ```ts
    /// const sound = audio.playFile("music.mp3");
    /// sound.volume = 0.5;
    /// ```
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

    /// Plays a sound file and waits for it to finish.
    ///
    /// ```ts
    /// await audio.playFileAndWait("alert.wav");
    ///
    /// // With a fade out and abort signal
    /// const controller = new AbortController();
    /// await audio.playFileAndWait("long-track.mp3", {
    ///     fadeOut: 1000,
    ///     signal: controller.signal,
    /// });
    /// ```
    ///
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

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Audio", &self.inner)
    }
}

/// A handle to an actively playing sound, allowing control over playback.
///
/// ```ts
/// const sound = audio.playFile("music.mp3");
/// println(sound.duration);  // duration in seconds
/// sound.volume = 0.5;
/// sound.playbackRate = 1.5;
/// sound.pause();
/// sound.resume();
/// await sound.finished;  // wait until the sound ends
/// ```
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
    /// Whether the sound is currently paused.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn paused(&self) -> bool {
        self.inner.is_paused()
    }

    /// Pauses the sound. Use `resume()` to continue playback.
    pub fn pause(&self) {
        self.inner.pause();
    }

    /// Resumes a paused sound.
    pub fn resume(&self) {
        self.inner.resume();
    }

    /// Stops the sound permanently.
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
    pub fn set_volume(&mut self, ctx: Ctx<'_>, volume: f32) -> Result<()> {
        self.inner.set_volume(volume).into_js_result(&ctx)
    }

    /// @skip
    #[qjs(get, rename = "playbackRate")]
    #[must_use]
    pub fn get_playback_rate(&self) -> f32 {
        self.inner.playback_rate()
    }

    /// @skip
    #[qjs(set, rename = "playbackRate")]
    pub fn set_playback_rate(&mut self, ctx: Ctx<'_>, playback_rate: f32) -> Result<()> {
        self.inner
            .set_playback_rate(playback_rate)
            .into_js_result(&ctx)
    }

    /// The total duration of the sound in seconds, or `undefined` if unknown.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn duration(&self) -> Option<f64> {
        let duration = self.inner.duration()?;
        Some(duration.as_secs_f64())
    }

    /// A promise that resolves when the sound has finished playing.
    ///
    /// ```ts
    /// const sound = audio.playFile("music.mp3");
    /// await sound.finished;
    /// println("Sound finished!");
    /// ```
    ///
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

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("PlayingSound", &self.inner)
    }
}
