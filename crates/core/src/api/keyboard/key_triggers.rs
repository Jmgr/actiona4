use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use color_eyre::Result;
use enigo::Key;
use macros::options;
use parking_lot::Mutex;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    api::{
        js::event_handle::{HandleId, HandleRegistry},
        macros::player::MacroPlayer,
        triggers::TriggerAction,
    },
    cancel_on,
    runtime::{Runtime, events::KeyboardKeyEvent},
};

/// Options for a key trigger.
#[options]
#[derive(Clone, Copy, Debug)]
pub struct OnKeysOptions {
    /// Require exactly these keys and no others to be pressed.
    pub exclusive: bool,
}

struct KeyHandler {
    id: HandleId,
    action: TriggerAction,
    options: OnKeysOptions,
}

type TriggerMap = HashMap<Vec<Key>, Vec<KeyHandler>>;

#[derive(Clone)]
pub struct KeyTriggers {
    /// Map from sorted normalized key list to a list of handlers.
    triggers: Arc<Mutex<TriggerMap>>,
    runtime: Arc<Runtime>,
    macro_player: Arc<MacroPlayer>,
    task_tracker: TaskTracker,
    cancellation_token: CancellationToken,
    listener_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl fmt::Debug for KeyTriggers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyTriggers").finish_non_exhaustive()
    }
}

impl KeyTriggers {
    pub fn new(
        runtime: Arc<Runtime>,
        macro_player: Arc<MacroPlayer>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            triggers: Arc::new(Mutex::new(HashMap::new())),
            runtime,
            macro_player,
            task_tracker,
            cancellation_token,
            listener_cancellation_token: Arc::new(Mutex::new(None)),
        }
    }

    async fn on_key(
        event: KeyboardKeyEvent,
        pressed_keys: &mut HashSet<Key>,
        fired: &mut HashSet<(Vec<Key>, HandleId)>,
        triggers: &Arc<Mutex<TriggerMap>>,
        macro_player: &Arc<MacroPlayer>,
    ) -> Result<()> {
        if event.is_injected || event.is_repeat {
            return Ok(());
        }

        let key = event.key.normalize();

        if event.direction.is_release() {
            pressed_keys.remove(&key);
            fired.retain(|(trigger, _)| !trigger.contains(&key));
            return Ok(());
        }

        pressed_keys.insert(key);

        // Collect handlers to fire: those whose trigger matches and haven't fired yet.
        let to_fire: Vec<(Vec<Key>, HandleId, TriggerAction)> = {
            let trigger_registry = triggers.lock();
            let mut pending_callbacks = Vec::new();

            for (trigger_keys, handlers) in trigger_registry.iter() {
                for handler in handlers {
                    let key_id = (trigger_keys.clone(), handler.id);
                    if fired.contains(&key_id) {
                        continue;
                    }

                    if !keys_match(trigger_keys, pressed_keys, handler.options.exclusive) {
                        continue;
                    }

                    pending_callbacks.push((
                        trigger_keys.clone(),
                        handler.id,
                        handler.action.clone(),
                    ));
                }
            }

            pending_callbacks
        };

        for (trigger_keys, handle_id, action) in to_fire {
            fired.insert((trigger_keys, handle_id));
            action.fire(macro_player, "onKey/onKeys").await;
        }

        Ok(())
    }

    fn ensure_listener_started(&self) {
        let mut listener_cancellation_token = self.listener_cancellation_token.lock();
        if listener_cancellation_token.is_some() {
            return;
        }

        let worker_cancellation_token = self.cancellation_token.child_token();
        *listener_cancellation_token = Some(worker_cancellation_token.clone());
        drop(listener_cancellation_token);

        let local_runtime = self.runtime.clone();
        let local_triggers = self.triggers.clone();
        let local_macro_player = self.macro_player.clone();
        self.task_tracker.spawn(async move {
            let guard = local_runtime.keyboard_keys();
            let mut receiver = guard.subscribe();
            let mut pressed_keys: HashSet<Key> = HashSet::new();
            // Track which (trigger, handle_id) pairs have already fired this press cycle.
            let mut fired: HashSet<(Vec<Key>, HandleId)> = HashSet::new();

            loop {
                let event = match cancel_on(&worker_cancellation_token, receiver.recv()).await {
                    Ok(Ok(event)) => event,
                    Ok(Err(_)) | Err(_) => break,
                };

                Self::on_key(
                    event,
                    &mut pressed_keys,
                    &mut fired,
                    &local_triggers,
                    &local_macro_player,
                )
                .await?;
            }

            Result::<()>::Ok(())
        });
    }

    fn stop_listener_if_running(&self) {
        let cancellation_token = self.listener_cancellation_token.lock().take();
        if let Some(cancellation_token) = cancellation_token {
            cancellation_token.cancel();
        }
    }

    pub fn add(&self, id: HandleId, keys: Vec<Key>, action: TriggerAction, options: OnKeysOptions) {
        let was_empty = {
            let mut triggers = self.triggers.lock();
            let was_empty = triggers.is_empty();

            let mut normalized: Vec<Key> = keys.iter().map(|key| key.normalize()).collect();
            normalized.sort_by_cached_key(|key| format!("{key:?}"));
            normalized.dedup();

            triggers.entry(normalized).or_default().push(KeyHandler {
                id,
                action,
                options,
            });

            was_empty
        };

        if was_empty {
            self.ensure_listener_started();
            self.runtime.increase_background_tasks_counter();
        }
    }

    pub fn remove(&self, id: HandleId) {
        let became_empty = {
            let mut triggers = self.triggers.lock();
            let was_empty = triggers.is_empty();

            triggers.retain(|_, handlers| {
                handlers.retain(|handler| handler.id != id);
                !handlers.is_empty()
            });

            !was_empty && triggers.is_empty()
        };

        if became_empty {
            self.stop_listener_if_running();
            self.runtime.decrease_background_tasks_counter();
        }
    }

    pub fn clear(&self) {
        let had_triggers = {
            let mut triggers = self.triggers.lock();
            if triggers.is_empty() {
                return;
            }
            triggers.clear();
            true
        };

        if had_triggers {
            self.stop_listener_if_running();
        }
        self.runtime.decrease_background_tasks_counter();
    }
}

impl HandleRegistry for KeyTriggers {
    fn remove_handle(&self, id: HandleId) {
        self.remove(id);
    }
}

fn keys_match(trigger_keys: &[Key], pressed_keys: &HashSet<Key>, exclusive: bool) -> bool {
    if exclusive {
        // Exact match: pressed keys must equal trigger keys exactly.
        pressed_keys.len() == trigger_keys.len()
            && trigger_keys
                .iter()
                .all(|trigger_key| pressed_keys.contains(trigger_key))
    } else {
        trigger_keys
            .iter()
            .all(|trigger_key| pressed_keys.contains(trigger_key))
    }
}

pub trait KeyExt {
    /// Normalizes a physical modifier key to its generic form.
    ///
    /// For example, `LControl` and `RControl` both become `Control`, so that
    /// either physical key can satisfy a trigger that specifies `Control`.
    fn normalize(self) -> Self;
}

impl KeyExt for Key {
    fn normalize(self) -> Self {
        if let Some(normalized_key) = normalize_windows_letter_digit_key(self) {
            return normalized_key;
        }

        match self {
            Self::LControl | Self::RControl => Self::Control,
            Self::LShift | Self::RShift => Self::Shift,
            Self::LMenu => Self::Alt,
            #[cfg(target_os = "windows")]
            Self::RMenu => Self::Alt,
            // Some layouts/reporting paths can surface Escape as a control character.
            // Canonicalize it so waitForKeys/onKey/onKeys behave consistently.
            Self::Unicode(character) if character == '\u{1b}' => Self::Escape,
            // Normalize uppercase ASCII letters to lowercase so that both "t" and "T"
            // in a JS trigger produce the same canonical key.
            Self::Unicode(c) if c.is_ascii_uppercase() => Self::Unicode(c.to_ascii_lowercase()),
            _ => self,
        }
    }
}

#[cfg(target_os = "windows")]
const fn normalize_windows_letter_digit_key(key: Key) -> Option<Key> {
    // The Windows keyboard hook maps physical letter/digit keys to named enigo variants
    // (Key::A..Key::Z, Key::Num0..Key::Num9), while JS string triggers are Unicode.
    // Normalize both paths to the same canonical Unicode representation.
    let normalized_key = match key {
        Key::A => Key::Unicode('a'),
        Key::B => Key::Unicode('b'),
        Key::C => Key::Unicode('c'),
        Key::D => Key::Unicode('d'),
        Key::E => Key::Unicode('e'),
        Key::F => Key::Unicode('f'),
        Key::G => Key::Unicode('g'),
        Key::H => Key::Unicode('h'),
        Key::I => Key::Unicode('i'),
        Key::J => Key::Unicode('j'),
        Key::K => Key::Unicode('k'),
        Key::L => Key::Unicode('l'),
        Key::M => Key::Unicode('m'),
        Key::N => Key::Unicode('n'),
        Key::O => Key::Unicode('o'),
        Key::P => Key::Unicode('p'),
        Key::Q => Key::Unicode('q'),
        Key::R => Key::Unicode('r'),
        Key::S => Key::Unicode('s'),
        Key::T => Key::Unicode('t'),
        Key::U => Key::Unicode('u'),
        Key::V => Key::Unicode('v'),
        Key::W => Key::Unicode('w'),
        Key::X => Key::Unicode('x'),
        Key::Y => Key::Unicode('y'),
        Key::Z => Key::Unicode('z'),
        Key::Num0 => Key::Unicode('0'),
        Key::Num1 => Key::Unicode('1'),
        Key::Num2 => Key::Unicode('2'),
        Key::Num3 => Key::Unicode('3'),
        Key::Num4 => Key::Unicode('4'),
        Key::Num5 => Key::Unicode('5'),
        Key::Num6 => Key::Unicode('6'),
        Key::Num7 => Key::Unicode('7'),
        Key::Num8 => Key::Unicode('8'),
        Key::Num9 => Key::Unicode('9'),
        _ => return None,
    };

    Some(normalized_key)
}

#[cfg(not(target_os = "windows"))]
const fn normalize_windows_letter_digit_key(_key: Key) -> Option<Key> {
    None
}
