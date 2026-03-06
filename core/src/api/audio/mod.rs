use std::{
    fmt::Display,
    fs::File,
    path::Path,
    sync::{
        Arc, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use color_eyre::{Result, eyre::ensure};
use macros::{FromJsObject, options};
use parking_lot::Mutex;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use tokio::{
    select,
    sync::{Notify, futures::Notified},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{api::js::duration::JsDuration, types::display::DisplayFields};

pub mod js;

/// Tracks active players so the runner can wait for all sounds to finish.
/// Held by both `Audio` (to register players) and `Runtime` (to query).
#[derive(Debug, Default)]
pub struct PlayingSoundsTracker {
    players: Mutex<Vec<Weak<Player>>>,
    notify: Notify,
}

impl PlayingSoundsTracker {
    /// Returns true if any registered player still has audio queued.
    pub fn has_playing_sounds(&self) -> bool {
        let mut players = self.players.lock();

        players.retain(|weak| weak.upgrade().is_some_and(|player| !player.empty()));

        !players.is_empty()
    }

    fn register(&self, player: Weak<Player>) {
        self.players.lock().push(player);
    }

    pub fn notified(&self) -> Notified<'_> {
        self.notify.notified()
    }

    pub fn notify_finished(&self) {
        self.notify.notify_waiters();
    }
}

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
#[options]
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct PlaySoundOptions {
    /// Volume to play the sound at
    #[default(1.0)]
    pub volume: f32,

    /// Speed to play the sound at
    #[default(1.0)]
    pub playback_rate: f32,

    /// Should the sound start paused
    pub paused: bool,

    /// Should the sound loop
    pub r#loop: bool,

    /// Fade in duration
    #[default(ts = "0")]
    pub fade_in: Option<JsDuration>,

    /// Fade out duration
    #[default(ts = "0")]
    pub fade_out: Option<JsDuration>,
}

#[derive(Default)]
struct OutputStreamCell(Mutex<Option<MixerDeviceSink>>);

impl OutputStreamCell {
    fn create_player(&self) -> Result<Player> {
        let mut output_stream = self.0.lock();

        if output_stream.is_none() {
            *output_stream = Some(DeviceSinkBuilder::open_default_sink()?);
        }

        let output_stream = output_stream
            .as_ref()
            .expect("output stream should be initialized");

        Ok(Player::connect_new(output_stream.mixer()))
    }

    fn clear_if_idle(
        &self,
        playing_sounds_tracker: &PlayingSoundsTracker,
        inflight_play_requests: &AtomicUsize,
    ) {
        if inflight_play_requests.load(Ordering::Acquire) == 0
            && !playing_sounds_tracker.has_playing_sounds()
        {
            self.0.lock().take();
        }
    }

    #[cfg(test)]
    fn is_initialized(&self) -> bool {
        self.0.lock().is_some()
    }
}

struct InflightPlayGuard(Arc<AtomicUsize>);

impl InflightPlayGuard {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        counter.fetch_add(1, Ordering::AcqRel);
        Self(counter)
    }
}

impl Drop for InflightPlayGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

#[derive(Clone)]
pub struct Audio {
    output_stream: Arc<OutputStreamCell>,
    inflight_play_requests: Arc<AtomicUsize>,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
    playing_sounds_tracker: Arc<PlayingSoundsTracker>,
}

impl Display for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default().finish(f)
    }
}

impl Audio {
    pub fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        playing_sounds_tracker: Arc<PlayingSoundsTracker>,
    ) -> Result<Self> {
        let output_stream = Arc::new(OutputStreamCell::default());
        let inflight_play_requests = Arc::new(AtomicUsize::new(0));

        // If audio goes idle, drop the stream so background backend errors from an
        // otherwise-unused stream do not surface in long-lived sessions (e.g. REPL).
        {
            let local_output_stream = output_stream.clone();
            let local_tracker = playing_sounds_tracker.clone();
            let local_cancellation_token = cancellation_token.clone();
            let local_inflight = inflight_play_requests.clone();

            task_tracker.spawn(async move {
                loop {
                    select! {
                        _ = local_cancellation_token.cancelled() => break,
                        _ = local_tracker.notified() => {
                            local_output_stream.clear_if_idle(&local_tracker, &local_inflight);
                        },
                    }
                }
            });
        }

        Ok(Self {
            // Open the audio sink lazily when a sound is first played. This avoids
            // spawning an idle output stream in sessions that never use audio (e.g. REPL).
            output_stream,
            inflight_play_requests,
            cancellation_token,
            task_tracker,
            playing_sounds_tracker,
        })
    }

    pub fn play_file(&self, path: &Path, options: PlaySoundOptions) -> Result<PlayingSound> {
        let _inflight_play_guard = InflightPlayGuard::new(self.inflight_play_requests.clone());

        let file = File::open(path)?;
        let mut source: Box<dyn Source<Item = f32> + Send> = Box::new(Decoder::try_from(file)?);
        let duration = source.total_duration();
        let source_sample_rate = source.sample_rate();
        let source_channels = source.channels();
        validate_source_format(source_sample_rate.get(), source_channels.get())?;
        validate_volume(options.volume)?;
        validate_playback_rate(options.playback_rate, source_sample_rate.get())?;
        let player = self.output_stream.create_player()?;

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

        let player = Arc::new(player);

        self.playing_sounds_tracker
            .register(Arc::downgrade(&player));

        // Stop the player if the runtime is cancelled (fire-and-forget path).
        let cancel_player = player.clone();
        let cancel_token = self.cancellation_token.clone();
        self.task_tracker.spawn(async move {
            cancel_token.cancelled().await;
            cancel_player.stop();
        });

        let local_player = player.clone();
        let local_tracker = self.playing_sounds_tracker.clone();
        self.task_tracker.spawn_blocking(move || {
            local_player.sleep_until_end();
            local_tracker.notify_finished();
        });

        Ok(PlayingSound {
            player,
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
    use std::{
        path::PathBuf,
        sync::Arc,
        time::{Duration, Instant},
    };

    use tokio::time::sleep;
    use tokio_util::{sync::CancellationToken, task::TaskTracker};

    use super::{
        Audio, PlaySoundOptions, PlayingSoundsTracker, validate_playback_rate, validate_volume,
    };

    fn test_mp3() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/test.mp3")
    }

    fn make_audio() -> (Audio, Arc<PlayingSoundsTracker>) {
        let tracker = Arc::new(PlayingSoundsTracker::default());
        let audio = Audio::new(
            CancellationToken::new(),
            TaskTracker::new(),
            tracker.clone(),
        )
        .unwrap();
        (audio, tracker)
    }

    #[tokio::test]
    async fn new_does_not_initialize_output_stream() {
        let tracker = Arc::new(PlayingSoundsTracker::default());
        let audio = Audio::new(CancellationToken::new(), TaskTracker::new(), tracker).unwrap();

        assert!(
            !audio.output_stream.is_initialized(),
            "output stream should stay uninitialized until first playback"
        );
    }

    /// Play a sound and wait for it to finish — the tracker should see no playing sounds after.
    #[tokio::test]
    #[ignore]
    async fn play_file_and_wait_finishes() {
        let (audio, tracker) = make_audio();
        let sound = audio
            .play_file(&test_mp3(), PlaySoundOptions::default())
            .unwrap();
        assert!(tracker.has_playing_sounds());
        sound.wait_finished(CancellationToken::new()).await;
        assert!(!tracker.has_playing_sounds());
    }

    /// play_file_and_wait resolves only after the sound ends.
    #[tokio::test]
    #[ignore]
    async fn play_file_and_wait_blocks_for_duration() {
        let (audio, _tracker) = make_audio();
        let start = Instant::now();
        audio
            .play_file_and_wait(
                &test_mp3(),
                PlaySoundOptions::default(),
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(
            start.elapsed().as_millis() >= 900,
            "sound finished too early"
        );
    }

    /// Stopping a sound makes the tracker consider it done.
    #[tokio::test]
    #[ignore]
    async fn stop_clears_tracker() {
        let (audio, tracker) = make_audio();
        let sound = audio
            .play_file(&test_mp3(), PlaySoundOptions::default())
            .unwrap();
        assert!(tracker.has_playing_sounds());
        sound.stop();
        // sleep_until_end wakes within ~5ms after stop(); give it a moment
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(!tracker.has_playing_sounds());
    }

    /// Cancelling via the token stops the sound and wakes the notify.
    #[tokio::test]
    #[ignore]
    async fn cancellation_token_stops_playback() {
        let token = CancellationToken::new();
        let tracker = Arc::new(PlayingSoundsTracker::default());
        let audio = Audio::new(token.clone(), TaskTracker::new(), tracker.clone()).unwrap();
        let _sound = audio
            .play_file(&test_mp3(), PlaySoundOptions::default())
            .unwrap();
        assert!(tracker.has_playing_sounds());
        token.cancel();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(!tracker.has_playing_sounds());
    }

    /// Two sounds playing concurrently: tracker stays active until both finish.
    #[tokio::test]
    #[ignore]
    async fn two_sounds_tracker_waits_for_both() {
        let (audio, tracker) = make_audio();
        let _s1 = audio
            .play_file(&test_mp3(), PlaySoundOptions::default())
            .unwrap();
        sleep(Duration::from_millis(250)).await;
        let _s2 = audio
            .play_file(&test_mp3(), PlaySoundOptions::default())
            .unwrap();
        assert!(tracker.has_playing_sounds());
        // Wait for both via notify loop
        while tracker.has_playing_sounds() {
            tracker.notified().await;
        }
        assert!(!tracker.has_playing_sounds());
    }

    /// JS integration: `await audio.playFileAndWait(...)` resolves after playback ends.
    #[test]
    #[ignore]
    fn js_play_file_and_wait() {
        use crate::runtime::Runtime;
        Runtime::test_with_script_engine(|script_engine| async move {
            let path = test_mp3();
            let start = Instant::now();
            script_engine
                .eval_async::<()>(&format!(
                    r#"await audio.playFileAndWait({:?})"#,
                    path.display()
                ))
                .await
                .unwrap();
            assert!(
                start.elapsed().as_millis() >= 900,
                "sound finished too early"
            );
        });
    }

    /// JS integration: fire-and-forget `audio.playFile(...)` — the runner exits only after
    /// the sound finishes (WaitAtEnd::Automatic via the playing sounds tracker).
    #[test]
    #[ignore]
    fn js_play_file_runner_waits() {
        use crate::runtime::Runtime;
        let start = Instant::now();
        Runtime::test_with_script_engine(|script_engine| async move {
            let path = test_mp3();
            // Fire and forget — do not await
            script_engine
                .eval_async::<()>(&format!(r#"audio.playFile({:?})"#, path.display()))
                .await
                .unwrap();
        });
        // The runner should have waited for the sound to finish before returning
        assert!(
            start.elapsed().as_millis() >= 900,
            "runner exited before sound finished"
        );
    }

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
