use std::{fs::File, path::Path, sync::Arc, time::Duration};

use color_eyre::Result;
use macros::FromJsObject;
use once_cell::sync::OnceCell;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;

use crate::core::js::duration::JsDuration;

pub mod js;

pub struct PlayingSound {
    sink: Arc<Sink>,
    duration: Option<Duration>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl PlayingSound {
    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    #[must_use]
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_playback_rate(&self, playback_rate: f32) {
        self.sink.set_speed(playback_rate);
    }

    #[must_use]
    pub fn playback_rate(&self) -> f32 {
        self.sink.speed()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub async fn wait_finished(&self) {
        let sink = self.sink.clone();

        let handle = self.task_tracker.spawn_blocking(move || {
            sink.sleep_until_end();
        });

        select! {
            _ = self.cancellation_token.cancelled() => { self.sink.stop() },
            _ = handle => {},
        }
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
struct OutputStreamCell(OnceCell<OutputStream>);

impl OutputStreamCell {
    fn get_or_try_init(&self) -> Result<&OutputStream> {
        Ok(self
            .0
            .get_or_try_init(OutputStreamBuilder::open_default_stream)?)
    }
}

pub struct Audio {
    output_stream: Arc<OutputStreamCell>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl Audio {
    pub fn new(cancellation_token: CancellationToken, task_tracker: TaskTracker) -> Result<Self> {
        let output_stream = Arc::new(OutputStreamCell::default());

        // Delayed initialization
        let local_output_stream = output_stream.clone();
        task_tracker.spawn_blocking(move || {
            if let Err(err) = local_output_stream.get_or_try_init() {
                error!("open_default_stream failed: {err}");
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
        let sink = Sink::connect_new(output_stream.mixer());

        sink.set_volume(options.volume);
        sink.set_speed(options.playback_rate);

        if let Some(fade_in) = options.fade_in {
            source = Box::new(source.fade_in(fade_in.into()));
        }

        if let Some(fade_out) = options.fade_out {
            source = Box::new(source.fade_out(fade_out.into()));
        }

        if options.r#loop {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        if options.paused {
            sink.pause();
        }

        Ok(PlayingSound {
            sink: Arc::new(sink),
            duration,
            cancellation_token: self.cancellation_token.clone(),
            task_tracker: self.task_tracker.clone(),
        })
    }

    pub async fn play_file_and_wait(&self, path: &Path, options: PlaySoundOptions) -> Result<()> {
        let playing_sound = self.play_file(path, options)?;

        playing_sound.wait_finished().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Arc, time::Duration};

    use tokio::time::sleep;
    use tracing_test::traced_test;

    use crate::{
        core::audio::{Audio, PlaySoundOptions},
        runtime::Runtime,
    };

    #[test]
    #[traced_test]
    fn test_audio() {
        Runtime::test(async |runtime| {
            let audio = Audio::new(runtime.cancellation_token(), runtime.task_tracker()).unwrap();
            let sound = Arc::new(
                audio
                    .play_file(
                        Path::new("/home/jmgr/Music/sample.mp3"),
                        PlaySoundOptions::default(),
                    )
                    .unwrap(),
            );
            println!("DUR {:?}", sound.duration());
            let local_sound = sound.clone();
            runtime.task_tracker().spawn(async move {
                sleep(Duration::from_secs(2)).await;
                local_sound.stop();
            });
            sound.wait_finished().await;
            //sleep(Duration::from_secs(10)).await;
        });
    }
}
