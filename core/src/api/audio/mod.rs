use std::{fmt::Display, fs::File, path::Path, sync::Arc, time::Duration};

use color_eyre::{Result, eyre::ensure};
use macros::FromJsObject;
use once_cell::sync::OnceCell;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;

use crate::{api::js::duration::JsDuration, types::display::DisplayFields};

pub mod js;

#[derive(Clone)]
pub struct PlayingSound {
    player: Arc<Player>,
    filename: Option<String>,
    duration: Option<Duration>,
    source_sample_rate: u32,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl PlayingSound {
    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn set_volume(&self, volume: f32) -> Result<()> {
        validate_volume(volume)?;
        self.player.set_volume(volume);
        Ok(())
    }

    #[must_use]
    pub fn volume(&self) -> f32 {
        self.player.volume()
    }

    pub fn set_playback_rate(&self, playback_rate: f32) -> Result<()> {
        validate_playback_rate(playback_rate, self.source_sample_rate)?;
        self.player.set_speed(playback_rate);
        Ok(())
    }

    #[must_use]
    pub fn playback_rate(&self) -> f32 {
        self.player.speed()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub async fn wait_finished(&self, cancellation_token: CancellationToken) {
        let player = self.player.clone();

        let handle = self.task_tracker.spawn_blocking(move || {
            player.sleep_until_end();
        });

        select! {
            _ = self.cancellation_token.cancelled() => { self.player.stop() },
            _ = cancellation_token.cancelled() => { self.player.stop() },
            _ = handle => {},
        }
    }
}

impl Display for PlayingSound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display_if_some("filename", &self.filename)
            .finish(f)
    }
}

/// Play sound options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct PlaySoundOptions {
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
}

impl Default for PlaySoundOptions {
    fn default() -> Self {
        Self {
            volume: 1.0,
            playback_rate: 1.0,
            paused: false,
            r#loop: false,
            fade_in: None,
            fade_out: None,
        }
    }
}

#[derive(Default)]
struct OutputStreamCell(OnceCell<MixerDeviceSink>);

impl OutputStreamCell {
    fn get_or_try_init(&self) -> Result<&MixerDeviceSink> {
        Ok(self
            .0
            .get_or_try_init(DeviceSinkBuilder::open_default_sink)?)
    }
}

#[derive(Clone)]
pub struct Audio {
    output_stream: Arc<OutputStreamCell>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl Display for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default().finish(f)
    }
}

impl Audio {
    pub fn new(cancellation_token: CancellationToken, task_tracker: TaskTracker) -> Result<Self> {
        let output_stream = Arc::new(OutputStreamCell::default());

        // Delayed initialization
        let local_output_stream = output_stream.clone();
        task_tracker.spawn_blocking(move || {
            if let Err(err) = local_output_stream.get_or_try_init() {
                error!("local_output_stream open failed: {err}");
            }
        });

        Ok(Self {
            output_stream,
            cancellation_token,
            task_tracker,
        })
    }

    pub fn play_file(&self, path: &Path, options: PlaySoundOptions) -> Result<PlayingSound> {
        let output_stream = self.output_stream.get_or_try_init()?;

        let file = File::open(path)?;
        let mut source: Box<dyn Source<Item = f32> + Send> = Box::new(Decoder::try_from(file)?);
        let duration = source.total_duration();
        let source_sample_rate = source.sample_rate();
        let source_channels = source.channels();
        validate_source_format(source_sample_rate.get(), source_channels.get())?;
        validate_volume(options.volume)?;
        validate_playback_rate(options.playback_rate, source_sample_rate.get())?;
        let player = Player::connect_new(output_stream.mixer());

        player.set_volume(options.volume);
        player.set_speed(options.playback_rate);

        if let Some(fade_in) = options.fade_in {
            source = Box::new(source.fade_in(fade_in.into()));
        }

        if let Some(fade_out) = options.fade_out {
            source = Box::new(source.fade_out(fade_out.into()));
        }

        if options.r#loop {
            player.append(source.repeat_infinite());
        } else {
            player.append(source);
        }

        if options.paused {
            player.pause();
        }

        Ok(PlayingSound {
            player: Arc::new(player),
            filename: path.file_name().map(|n| n.to_string_lossy().into_owned()),
            duration,
            source_sample_rate: source_sample_rate.get(),
            cancellation_token: self.cancellation_token.clone(),
            task_tracker: self.task_tracker.clone(),
        })
    }

    pub async fn play_file_and_wait(
        &self,
        path: &Path,
        options: PlaySoundOptions,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let playing_sound = self.play_file(path, options)?;

        playing_sound.wait_finished(cancellation_token).await;

        Ok(())
    }
}

fn validate_source_format(sample_rate: u32, channels: u16) -> Result<()> {
    ensure!(
        sample_rate > 0,
        "audio stream reports invalid sample rate: 0"
    );
    ensure!(
        channels > 0,
        "audio stream reports invalid channel count: 0"
    );
    Ok(())
}

fn validate_volume(volume: f32) -> Result<()> {
    ensure!(volume.is_finite(), "audio volume must be a finite number");
    ensure!(
        volume >= 0.0,
        "audio volume must be greater than or equal to 0"
    );
    Ok(())
}

fn validate_playback_rate(playback_rate: f32, source_sample_rate: u32) -> Result<()> {
    ensure!(
        playback_rate.is_finite(),
        "audio playback rate must be a finite number"
    );
    ensure!(
        playback_rate > 0.0,
        "audio playback rate must be greater than 0"
    );

    #[allow(clippy::as_conversions)]
    let effective_sample_rate = source_sample_rate as f64 * playback_rate as f64;
    ensure!(
        effective_sample_rate >= 1.0,
        "audio playback rate is too small for this source sample rate"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_playback_rate, validate_volume};

    #[test]
    fn validate_volume_accepts_valid_values() {
        assert!(validate_volume(0.0).is_ok());
        assert!(validate_volume(1.0).is_ok());
    }

    #[test]
    fn validate_volume_rejects_invalid_values() {
        assert!(validate_volume(-0.1).is_err());
        assert!(validate_volume(f32::NAN).is_err());
        assert!(validate_volume(f32::INFINITY).is_err());
    }

    #[test]
    fn validate_playback_rate_accepts_valid_values() {
        assert!(validate_playback_rate(1.0, 44_100).is_ok());
        assert!(validate_playback_rate(2.0, 48_000).is_ok());
    }

    #[test]
    fn validate_playback_rate_rejects_invalid_values() {
        assert!(validate_playback_rate(0.0, 44_100).is_err());
        assert!(validate_playback_rate(-1.0, 44_100).is_err());
        assert!(validate_playback_rate(f32::NAN, 44_100).is_err());
        assert!(validate_playback_rate(f32::INFINITY, 44_100).is_err());
        assert!(validate_playback_rate(1.0e-8, 44_100).is_err());
    }
}
