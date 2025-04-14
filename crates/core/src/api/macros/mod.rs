use std::{
    collections::{HashMap, HashSet},
    fmt,
    io::{Read, Write},
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use color_eyre::{Result, eyre::eyre};
use enigo::{Coordinate, Key};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use time::OffsetDateTime;
use tokio::{select, sync::mpsc, time::sleep};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::warn;

use crate::{
    api::{
        displays::Displays,
        keyboard::{KeyExt, Keyboard, js::JsKey},
        mouse::{Axis, Button, Mouse, PressOptions},
        point::point,
    },
    cancel_on,
    error::CommonError,
    runtime::Runtime,
    types::{
        display::{DisplayFields, display_with_type},
        input::Direction,
        si32::Si32,
        su32::Su32,
    },
};

pub mod js;
pub mod player;

pub const MACRO_VERSION: u32 = 1;

/// A single recorded event with its timestamp.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MacroEvent {
    MouseMove {
        #[serde(rename = "timeMs")]
        time_ms: u64,
        x: Si32,
        y: Si32,
    },
    MouseButton {
        #[serde(rename = "timeMs")]
        time_ms: u64,
        button: Button,
        direction: Direction,
    },
    MouseScroll {
        #[serde(rename = "timeMs")]
        time_ms: u64,
        axis: Axis,
        length: Si32,
    },
    KeyboardKey {
        #[serde(rename = "timeMs")]
        time_ms: u64,
        key: JsKey,
        direction: Direction,
    },
}

impl MacroEvent {
    #[must_use]
    pub const fn time_ms(&self) -> u64 {
        match self {
            Self::MouseMove { time_ms, .. }
            | Self::MouseButton { time_ms, .. }
            | Self::MouseScroll { time_ms, .. }
            | Self::KeyboardKey { time_ms, .. } => *time_ms,
        }
    }
}

impl fmt::Display for MacroEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MouseMove { x, y, .. } => {
                let fields = DisplayFields::default()
                    .display("x", x)
                    .display("y", y)
                    .finish_as_string();
                formatter.write_str(&display_with_type("MouseMove", fields))
            }
            Self::MouseButton {
                button, direction, ..
            } => {
                let fields = DisplayFields::default()
                    .display("button", button)
                    .display("direction", direction)
                    .finish_as_string();
                formatter.write_str(&display_with_type("MouseButton", fields))
            }
            Self::MouseScroll { axis, length, .. } => {
                let fields = DisplayFields::default()
                    .display("axis", axis)
                    .display("length", length)
                    .finish_as_string();
                formatter.write_str(&display_with_type("MouseScroll", fields))
            }
            Self::KeyboardKey { key, direction, .. } => {
                let fields = DisplayFields::default()
                    .display("key", key)
                    .display("direction", direction)
                    .finish_as_string();
                formatter.write_str(&display_with_type("KeyboardKey", fields))
            }
        }
    }
}

/// Snapshot of a single display's configuration at recording time.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroDisplayInfo {
    pub x: Si32,
    pub y: Si32,
    pub width: Su32,
    pub height: Su32,
    pub scale_factor: f32,
    pub is_primary: bool,
}

impl From<&crate::runtime::events::DisplayInfo> for MacroDisplayInfo {
    fn from(d: &crate::runtime::events::DisplayInfo) -> Self {
        Self {
            x: d.rect.top_left.x,
            y: d.rect.top_left.y,
            width: d.rect.size.width,
            height: d.rect.size.height,
            scale_factor: d.scale_factor,
            is_primary: d.is_primary,
        }
    }
}

/// Metadata stored alongside the recorded events.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroMetadata {
    /// Duration of the recording in milliseconds.
    pub duration_ms: u64,
    /// Timestamp of when the recording was made.
    #[serde(with = "time::serde::rfc3339")]
    pub recorded_at: OffsetDateTime,
    /// Platform on which the macro was recorded (`"linux"` or `"windows"`).
    pub platform: String,
    /// Display configuration at recording time.
    pub displays: Vec<MacroDisplayInfo>,
}

/// A recorded macro: metadata plus an ordered list of timed events.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroData {
    pub version: u32,
    pub metadata: MacroMetadata,
    pub events: Vec<MacroEvent>,
}

impl MacroData {
    /// Saves this macro to a gzip-compressed JSON file.
    pub async fn save(&self, path: impl AsRef<Path>, task_tracker: TaskTracker) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        let json = serde_json::to_vec(self)?;
        let compressed = task_tracker
            .spawn_blocking(move || {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&json)?;
                Result::<Vec<u8>>::Ok(encoder.finish()?)
            })
            .await
            .map_err(|err| eyre!("Task join error: {err}"))??;
        tokio::fs::write(path, compressed).await?;
        Ok(())
    }

    /// Loads a macro from a gzip-compressed JSON file previously written by `save()`.
    pub async fn load(path: impl AsRef<Path>, task_tracker: TaskTracker) -> Result<Self> {
        let compressed = tokio::fs::read(path.as_ref()).await?;

        task_tracker
            .spawn_blocking(move || {
                let mut decoder = GzDecoder::new(compressed.as_slice());
                let mut json = Vec::new();
                decoder.read_to_end(&mut json)?;
                Result::<Self>::Ok(serde_json::from_slice(&json)?)
            })
            .await
            .map_err(|err| eyre!("Task join error: {err}"))?
    }
}

/// Configuration for `record_impl`.
pub struct RecordConfig {
    pub stop_keys: HashSet<enigo::Key>,
    pub timeout: Option<Duration>,
    pub mouse_position_interval: Duration,
    pub mouse_buttons: bool,
    pub mouse_position: bool,
    pub mouse_scroll: bool,
    pub keyboard_keys: bool,
}

/// Configuration for `play_impl`.
#[derive(Clone, Copy, Debug)]
pub struct PlayConfig {
    pub speed: f64,
    pub mouse_buttons: bool,
    pub mouse_position: bool,
    /// When true, mouse movements are offset by the delta between the current cursor position
    /// and the first recorded cursor position, making playback position-independent.
    pub relative_mouse_position: bool,
    pub mouse_scroll: bool,
    pub keyboard_keys: bool,
}

/// Progress of a `play_impl` operation.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlayProgress {
    pub events_done: u64,
    pub total_events: u64,
}

/// Records user input, emitting a `MacroData` when done.
pub async fn record_impl(
    runtime: Arc<Runtime>,
    mouse: Mouse,
    config: RecordConfig,
    displays: Displays,
    token: CancellationToken,
) -> Result<MacroData> {
    let current_displays = displays
        .wait_get_info()
        .await
        .map(|infos| infos.iter().map(MacroDisplayInfo::from).collect::<Vec<_>>())
        .unwrap_or_default();
    let button_guard = runtime.mouse_buttons();
    let mut button_rx = button_guard.subscribe();
    let scroll_guard = runtime.mouse_scroll();
    let mut scroll_rx = scroll_guard.subscribe();
    let key_guard = runtime.keyboard_keys();
    let mut key_rx = key_guard.subscribe();

    let mut position_ticker = tokio::time::interval(config.mouse_position_interval);
    position_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    tokio::pin! {
        let timeout_fut = async {
            match config.timeout {
                Some(dur) => tokio::time::sleep(dur).await,
                None => std::future::pending::<()>().await,
            }
        };
    }

    let start = Instant::now();
    let elapsed_ms = || u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let mut events = Vec::new();
    let mut stop_key_press_idx: HashMap<enigo::Key, Vec<usize>> = HashMap::new();
    let mut pressed_stop_keys: HashSet<enigo::Key> = HashSet::new();
    let mut last_position = None;

    'record: loop {
        select! {
            biased;
            () = token.cancelled() => break 'record,
            () = &mut timeout_fut => break 'record,

            result = button_rx.recv(), if config.mouse_buttons => {
                let Ok(event) = result else { break 'record; };
                    let time_ms = elapsed_ms();
                    events.push(MacroEvent::MouseButton {
                        time_ms,
                        button: event.button,
                        direction: event.direction,
                    });
            }

            result = scroll_rx.recv(), if config.mouse_scroll => {
                let Ok(event) = result else { break 'record; };
                    let time_ms = elapsed_ms();
                    events.push(MacroEvent::MouseScroll {
                        time_ms,
                        axis: event.axis,
                        length: event.length.into(),
                    });
            }

            result = key_rx.recv() => {
                let Ok(event) = result else { break 'record; };
                if event.is_repeat {
                    continue;
                }
                let time_ms = elapsed_ms();
                let is_stop_key = config.stop_keys.contains(&event.key);

                if is_stop_key {
                    match event.direction {
                        Direction::Press => {
                            pressed_stop_keys.insert(event.key);

                            if config.keyboard_keys
                                && let Ok(js_key) = JsKey::try_from(event.key)
                            {
                                stop_key_press_idx.entry(event.key).or_default().push(events.len());
                                events.push(MacroEvent::KeyboardKey {
                                    time_ms,
                                    key: js_key,
                                    direction: Direction::Press,
                                });
                            }

                            if config.stop_keys.iter().all(|key| pressed_stop_keys.contains(key)) {
                                let mut remove_indices: Vec<usize> = stop_key_press_idx
                                    .values()
                                    .flat_map(|v| v.iter().copied())
                                    .collect();
                                remove_indices.sort_unstable_by(|left, right| right.cmp(left));
                                for idx in remove_indices {
                                    events.remove(idx);
                                }
                                break 'record;
                            }
                        }
                        Direction::Release => {
                            if config.keyboard_keys
                                && let Ok(js_key) = JsKey::try_from(event.key)
                            {
                                events.push(MacroEvent::KeyboardKey {
                                    time_ms,
                                    key: js_key,
                                    direction: Direction::Release,
                                });
                            }
                            pressed_stop_keys.remove(&event.key);
                            stop_key_press_idx
                                .entry(event.key)
                                .and_modify(|v| { v.pop(); });
                        }
                    }
                } else if config.keyboard_keys && let Ok(js_key) = JsKey::try_from(event.key) {
                    events.push(MacroEvent::KeyboardKey {
                        time_ms,
                        key: js_key,
                        direction: event.direction,
                    });
                }
            }

            _ = position_ticker.tick(), if config.mouse_position => {
                if let Ok(pos) = mouse.position()
                    && last_position != Some((pos.x, pos.y))
                {
                    last_position = Some((pos.x, pos.y));
                    let time_ms = elapsed_ms();
                    events.push(MacroEvent::MouseMove { time_ms, x: pos.x, y: pos.y });
                }
            }
        }
    }

    #[cfg(linux)]
    const OS_NAME: &str = "linux";
    #[cfg(windows)]
    const OS_NAME: &str = "windows";
    #[cfg(not(any(linux, windows)))]
    const OS_NAME: &str = "unknown";

    let metadata = MacroMetadata {
        duration_ms: elapsed_ms(),
        recorded_at: OffsetDateTime::now_utc(),
        platform: OS_NAME.to_string(),
        displays: current_displays,
    };

    Ok(MacroData {
        version: MACRO_VERSION,
        metadata,
        events,
    })
}

/// Computes the mouse offset for relative playback: `(current_pos - first_recorded_pos)`.
///
/// Returns `(Si32::ZERO, Si32::ZERO)` if relative mode is disabled, no mouse move events exist,
/// or the current position cannot be queried.
fn relative_mouse_offset(config: &PlayConfig, data: &MacroData, mouse: &Mouse) -> (Si32, Si32) {
    if !config.relative_mouse_position || !config.mouse_position {
        return (Si32::ZERO, Si32::ZERO);
    }
    let Some((fx, fy)) = data.events.iter().find_map(|e| {
        if let MacroEvent::MouseMove { x, y, .. } = e {
            Some((*x, *y))
        } else {
            None
        }
    }) else {
        return (Si32::ZERO, Si32::ZERO);
    };
    match mouse.position() {
        Ok(pos) => (pos.x - fx, pos.y - fy),
        Err(err) => {
            warn!(
                "Failed to get current mouse position for relative playback: {err}; \
                 falling back to absolute positions."
            );
            (Si32::ZERO, Si32::ZERO)
        }
    }
}

#[derive(Debug)]
struct PlaybackCleanup {
    keyboard: Keyboard,
    mouse: Mouse,
    initial_pressed_keys: HashSet<Key>,
    initial_pressed_buttons: HashSet<Button>,
    macro_pressed_keys: HashSet<Key>,
    macro_pressed_buttons: HashSet<Button>,
}

impl PlaybackCleanup {
    fn new(keyboard: Keyboard, mouse: Mouse) -> Result<Self> {
        let initial_pressed_keys = keyboard
            .get_pressed_keys()?
            .into_iter()
            .map(KeyExt::normalize)
            .collect();
        let initial_pressed_buttons = Button::iter()
            .filter_map(|button| match mouse.is_pressed(button) {
                Ok(true) => Some(Ok(button)),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<HashSet<_>>>()?;

        Ok(Self {
            keyboard,
            mouse,
            initial_pressed_keys,
            initial_pressed_buttons,
            macro_pressed_keys: HashSet::new(),
            macro_pressed_buttons: HashSet::new(),
        })
    }

    fn on_key_press(&mut self, key: Key) {
        if !self.initial_pressed_keys.contains(&key.normalize()) {
            self.macro_pressed_keys.insert(key);
        }
    }

    fn on_key_release(&mut self, key: Key) {
        self.macro_pressed_keys.remove(&key);
    }

    fn on_button_press(&mut self, button: Button) {
        if !self.initial_pressed_buttons.contains(&button) {
            self.macro_pressed_buttons.insert(button);
        }
    }

    fn on_button_release(&mut self, button: Button) {
        self.macro_pressed_buttons.remove(&button);
    }

    fn release_remaining_inputs(&mut self) {
        for key in self.macro_pressed_keys.drain() {
            if let Err(error) = self.keyboard.release(key) {
                warn!(?key, error = %error, "failed to release key during macro cleanup");
            }
        }

        for button in self.macro_pressed_buttons.drain() {
            if let Err(error) = self.mouse.release(Some(button)) {
                warn!(
                    ?button,
                    error = %error,
                    "failed to release mouse button during macro cleanup"
                );
            }
        }
    }
}

/// Replays a `MacroData`, reporting progress through `progress_tx`.
pub async fn play_impl(
    keyboard: Keyboard,
    mouse: Mouse,
    data: &MacroData,
    config: &PlayConfig,
    displays: Displays,
    progress_tx: mpsc::UnboundedSender<PlayProgress>,
    token: CancellationToken,
) -> Result<()> {
    let current_displays = displays
        .wait_get_info()
        .await
        .map(|infos| infos.iter().map(MacroDisplayInfo::from).collect::<Vec<_>>())
        .unwrap_or_default();
    if current_displays != data.metadata.displays.as_slice() {
        warn!(
            "Display configuration has changed since this macro was recorded. \
             Mouse positions may not replay correctly."
        );
    }

    let mut cleanup = PlaybackCleanup::new(keyboard.clone(), mouse.clone())?;
    let total_events = u64::try_from(data.events.len()).unwrap_or(u64::MAX);

    if data.events.is_empty() {
        let _ = progress_tx.send(PlayProgress {
            events_done: 0,
            total_events: 0,
        });
        return Ok(());
    }

    let mouse_offset = relative_mouse_offset(config, data, &mouse);

    let mut last_time_ms = 0u64;

    let playback_result: Result<()> = async {
        for (index, event) in data.events.iter().enumerate() {
            if token.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            let event_time_ms = event.time_ms();
            let delay_ms = event_time_ms.saturating_sub(last_time_ms);

            if delay_ms > 0 {
                #[allow(clippy::as_conversions)]
                let scaled_ms = (delay_ms as f64 / config.speed) as u64;
                cancel_on(&token, sleep(Duration::from_millis(scaled_ms)))
                    .await
                    .map_err(|_| CommonError::Cancelled)?;
            }

            last_time_ms = event_time_ms;

            match event {
                MacroEvent::MouseMove { x, y, .. } => {
                    if config.mouse_position {
                        mouse
                            .set_position(
                                point(*x + mouse_offset.0, *y + mouse_offset.1),
                                Coordinate::Abs,
                            )
                            .map_err(|err| eyre!("mouse move failed: {err}"))?;
                    }
                }
                MacroEvent::MouseButton {
                    button, direction, ..
                } => {
                    if config.mouse_buttons {
                        match direction {
                            Direction::Press => {
                                mouse
                                    .press(PressOptions {
                                        button: *button,
                                        position: None,
                                        relative_position: false,
                                    })
                                    .map_err(|err| eyre!("mouse press failed: {err}"))?;
                                cleanup.on_button_press(*button);
                            }
                            Direction::Release => {
                                mouse
                                    .release(Some(*button))
                                    .map_err(|err| eyre!("mouse release failed: {err}"))?;
                                cleanup.on_button_release(*button);
                            }
                        }
                    }
                }
                MacroEvent::MouseScroll { axis, length, .. } => {
                    if config.mouse_scroll {
                        mouse
                            .scroll((*length).into(), *axis)
                            .map_err(|err| eyre!("mouse scroll failed: {err}"))?;
                    }
                }
                MacroEvent::KeyboardKey { key, direction, .. } => {
                    if config.keyboard_keys {
                        match enigo::Key::try_from(*key) {
                            Ok(enigo_key) => match direction {
                                Direction::Press => {
                                    keyboard
                                        .press(enigo_key)
                                        .map_err(|err| eyre!("key press failed: {err}"))?;
                                    cleanup.on_key_press(enigo_key);
                                }
                                Direction::Release => {
                                    keyboard
                                        .release(enigo_key)
                                        .map_err(|err| eyre!("key release failed: {err}"))?;
                                    cleanup.on_key_release(enigo_key);
                                }
                            },
                            Err(err) => {
                                warn!("Skipping unsupported key during playback: {err:?}");
                            }
                        }
                    }
                }
            }

            let events_done = u64::try_from(index + 1).unwrap_or(u64::MAX);
            let _ = progress_tx.send(PlayProgress {
                events_done,
                total_events,
            });
        }

        Ok(())
    }
    .await;

    if playback_result.is_err() {
        cleanup.release_remaining_inputs();
    }

    playback_result
}
