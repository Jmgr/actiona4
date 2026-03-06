use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use color_eyre::Result;
use enigo::Key;
use macros::options;
use parking_lot::Mutex;
use rquickjs::{AsyncContext, async_with};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    api::js::event_handle::{HandleId, HandleRegistry},
    cancel_on,
    runtime::{Runtime, WithUserData, events::KeyboardKeyEvent},
    scripting::callbacks::FunctionKey,
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
    context: AsyncContext,
    function_key: FunctionKey,
    options: OnKeysOptions,
}

type TriggerMap = HashMap<Vec<Key>, Vec<KeyHandler>>;

#[derive(Clone)]
pub struct KeyTriggers {
    /// Map from sorted normalized key list to a list of handlers.
    triggers: Arc<Mutex<TriggerMap>>,
    runtime: Arc<Runtime>,
}

impl fmt::Debug for KeyTriggers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyTriggers").finish_non_exhaustive()
    }
}

impl KeyTriggers {
    pub fn new(
        runtime: Arc<Runtime>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let triggers: Arc<Mutex<TriggerMap>> = Arc::new(Mutex::new(HashMap::new()));

        let local_runtime = runtime.clone();
        let local_triggers = triggers.clone();

        task_tracker.spawn(async move {
            let guard = local_runtime.keyboard_keys();
            let mut receiver = guard.subscribe();
            let mut pressed_keys: HashSet<Key> = HashSet::new();
            // Track which (trigger, handle_id) pairs have already fired this press cycle.
            let mut fired: HashSet<(Vec<Key>, HandleId)> = HashSet::new();

            loop {
                let Ok(event) = cancel_on(&cancellation_token, receiver.recv()).await? else {
                    break;
                };
                Self::on_key(event, &mut pressed_keys, &mut fired, &local_triggers).await?;
            }

            Result::<()>::Ok(())
        });

        Self { triggers, runtime }
    }

    async fn on_key(
        event: KeyboardKeyEvent,
        pressed_keys: &mut HashSet<Key>,
        fired: &mut HashSet<(Vec<Key>, HandleId)>,
        triggers: &Arc<Mutex<TriggerMap>>,
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
        let to_fire: Vec<(Vec<Key>, HandleId, AsyncContext, FunctionKey)> = {
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
                        handler.context.clone(),
                        handler.function_key,
                    ));
                }
            }

            pending_callbacks
        };

        for (trigger_keys, handle_id, context, function_key) in to_fire {
            fired.insert((trigger_keys, handle_id));

            // Use call_sync so the async_with! closure completes without yielding.
            // If the closure were to .await (as callbacks.call() does), WithFuture::poll
            // would reach opaque.poll(cx) and overwrite the scheduler's queue waker with
            // this task's waker. That would prevent eval_async from driving wait_for_keys
            // when no trigger fires for a subsequent key (e.g. Escape after F8).
            async_with!(context => |ctx| {
                ctx.user_data().callbacks().call_sync(&ctx, function_key);
            })
            .await;
        }

        Ok(())
    }

    pub fn add(
        &self,
        id: HandleId,
        keys: Vec<Key>,
        context: AsyncContext,
        function_key: FunctionKey,
        options: OnKeysOptions,
    ) {
        let mut triggers = self.triggers.lock();
        let was_empty = triggers.is_empty();

        let mut normalized: Vec<Key> = keys.iter().map(|key| key.normalize()).collect();
        normalized.sort_by_cached_key(|key| format!("{key:?}"));
        normalized.dedup();

        triggers.entry(normalized).or_default().push(KeyHandler {
            id,
            context,
            function_key,
            options,
        });

        if was_empty {
            self.runtime.increase_background_tasks_counter();
        }
    }

    pub fn remove(&self, id: HandleId) {
        let mut triggers = self.triggers.lock();
        let was_empty = triggers.is_empty();

        triggers.retain(|_, handlers| {
            handlers.retain(|handler| handler.id != id);
            !handlers.is_empty()
        });

        if !was_empty && triggers.is_empty() {
            self.runtime.decrease_background_tasks_counter();
        }
    }

    pub fn clear(&self) {
        let mut triggers = self.triggers.lock();
        if triggers.is_empty() {
            return;
        }
        triggers.clear();
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

pub(super) trait KeyExt {
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
