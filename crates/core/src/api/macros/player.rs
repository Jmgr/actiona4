use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use color_eyre::Result;
use parking_lot::Mutex;
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use super::{MacroData, PlayConfig, PlayProgress, play_impl};
use crate::{
    api::{displays::Displays, keyboard::Keyboard, mouse::Mouse},
    error::CommonError,
    runtime::Runtime,
};

#[derive(Debug)]
struct CurrentPlayback {
    id: u64,
    token: CancellationToken,
    finished: watch::Receiver<bool>,
}

#[derive(Debug, Default)]
struct PlaybackState {
    current: Option<CurrentPlayback>,
}

#[derive(Debug)]
struct PlaybackRunGuard {
    id: u64,
    finished_tx: watch::Sender<bool>,
}

impl PlaybackRunGuard {
    fn finish(self, player: &MacroPlayer) {
        self.finished_tx.send_replace(true);

        let mut state = player.state.lock();
        if state
            .current
            .as_ref()
            .is_some_and(|current| current.id == self.id)
        {
            state.current = None;
        }
    }
}

#[derive(Debug)]
pub struct MacroPlayer {
    runtime: Arc<Runtime>,
    displays: Displays,
    keyboard: Keyboard,
    mouse: Mouse,
    next_id: AtomicU64,
    state: Mutex<PlaybackState>,
}

impl MacroPlayer {
    pub async fn new(runtime: Arc<Runtime>, keyboard: Keyboard, mouse: Mouse) -> Result<Self> {
        Ok(Self {
            displays: runtime.displays(),
            keyboard,
            runtime,
            mouse,
            next_id: AtomicU64::new(1),
            state: Mutex::new(PlaybackState::default()),
        })
    }

    pub async fn play(
        &self,
        data: Arc<MacroData>,
        config: PlayConfig,
        progress_tx: mpsc::UnboundedSender<PlayProgress>,
        token: CancellationToken,
    ) -> Result<()> {
        let guard = self.begin_playback(token.clone()).await;

        let result = if token.is_cancelled() {
            Err(CommonError::Cancelled.into())
        } else {
            play_impl(
                self.keyboard.clone(),
                self.mouse.clone(),
                &data,
                &config,
                self.displays.clone(),
                progress_tx,
                token,
            )
            .await
        };

        guard.finish(self);
        result
    }

    pub fn play_detached(self: &Arc<Self>, data: Arc<MacroData>, config: PlayConfig) {
        if data.events.is_empty() {
            return;
        }

        let player = self.clone();
        let task_tracker = self.runtime.task_tracker();
        let token = self.runtime.cancellation_token().child_token();
        let (progress_tx, _) = mpsc::unbounded_channel::<PlayProgress>();

        task_tracker.spawn(async move {
            if let Err(error) = player.play(data, config, progress_tx, token).await {
                if matches!(
                    error.downcast_ref::<CommonError>(),
                    Some(CommonError::Cancelled)
                ) {
                    return;
                }

                warn!(error = %error, "triggered macro playback failed");
            }
        });
    }

    async fn begin_playback(&self, token: CancellationToken) -> PlaybackRunGuard {
        let (finished_tx, finished_rx) = watch::channel(false);
        let playback_id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let previous_finished = {
            let mut state = self.state.lock();
            let previous = state.current.replace(CurrentPlayback {
                id: playback_id,
                token: token.clone(),
                finished: finished_rx,
            });

            if let Some(previous) = previous {
                previous.token.cancel();
                Some(previous.finished)
            } else {
                None
            }
        };

        if let Some(mut previous_finished) = previous_finished
            && !*previous_finished.borrow()
        {
            let _ = previous_finished.changed().await;
        }

        PlaybackRunGuard {
            id: playback_id,
            finished_tx,
        }
    }
}
