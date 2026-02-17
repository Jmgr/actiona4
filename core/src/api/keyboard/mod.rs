use std::{collections::HashSet, sync::Arc};

use color_eyre::Result;
use enigo::{Direction, Enigo, Key};
use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

pub(crate) mod platform;

pub mod js;

pub use enigo::Coordinate;
#[cfg(windows)]
use platform::win::KeyboardImpl;
#[cfg(unix)]
use platform::x11::KeyboardImpl;

use crate::{cancel_on, runtime::Runtime};

#[derive(Clone, Debug)]
pub struct Keyboard {
    runtime: Arc<Runtime>,
    enigo: Arc<Mutex<Enigo>>,
    implementation: KeyboardImpl,
}

impl Keyboard {
    #[instrument(skip_all)]
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        let enigo = runtime.enigo();

        #[cfg(unix)]
        let implementation = KeyboardImpl::new(runtime.clone())?;
        #[cfg(windows)]
        let implementation = KeyboardImpl::default();

        Ok(Self {
            runtime,
            enigo,
            implementation,
        })
    }

    #[instrument(skip(self), err, ret)]
    pub fn text(&self, text: &str) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().text(text)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn key(&self, key: Key, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().key(key, direction)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn raw(&self, keycode: u16, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().raw(keycode, direction)?;

        Ok(())
    }

    pub async fn is_key_pressed(&self, key: Key) -> Result<bool> {
        self.implementation.is_key_pressed(key).await
    }

    pub async fn get_pressed_keys(&self) -> Result<Vec<Key>> {
        self.implementation.get_pressed_keys().await
    }

    pub async fn wait_for_keys(
        &self,
        keys: &HashSet<Key>,
        exclusive: bool, // TODO: options
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }

        // Expand generic modifier keys (e.g. Control -> {LControl, RControl}) so that
        // either the left or right physical key satisfies the requirement.
        let keys = expand_generic_modifiers(keys);

        let guard = self.runtime.keyboard_keys();
        let mut receiver = guard.subscribe();
        let mut pressed_keys = HashSet::with_capacity(keys.len());

        loop {
            let event = cancel_on(&cancellation_token, receiver.recv()).await??;
            if event.is_injected || event.is_repeat {
                continue;
            }

            // Normalize the incoming key so that e.g. LControl matches a Control requirement
            let key = normalize_to_generic_modifier(event.key);

            // Ignore keys that are not part of the list
            if !keys.contains(&key) {
                continue;
            }

            // Remove released keys
            if event.direction.is_release() {
                pressed_keys.remove(&key);
                continue;
            }

            pressed_keys.insert(key);

            if exclusive {
                if pressed_keys == keys {
                    return Ok(());
                }
            } else if keys.is_subset(&pressed_keys) {
                return Ok(());
            }
        }
    }
}

/// Map left/right physical modifier keys to their generic counterpart.
/// Keys that are not side-specific modifiers are returned unchanged.
const fn normalize_to_generic_modifier(key: Key) -> Key {
    match key {
        Key::LControl | Key::RControl => Key::Control,
        Key::LShift | Key::RShift => Key::Shift,
        Key::LMenu => Key::Alt,
        #[cfg(target_os = "windows")]
        Key::RMenu => Key::Alt,
        _ => key,
    }
}

/// Expand generic modifier keys in the set into their generic form only.
/// For example, if the set contains `Key::Control`, it stays as `Key::Control`
/// (and incoming events are normalized via [`normalize_to_generic_modifier`]).
/// If the set contains `Key::LControl` specifically, it is replaced by the
/// generic `Key::Control` so that either physical key can satisfy it.
fn expand_generic_modifiers(keys: &HashSet<Key>) -> HashSet<Key> {
    keys.iter()
        .map(|key| normalize_to_generic_modifier(*key))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use enigo::Key;
    use tokio::time::sleep;
    use tracing_test::traced_test;

    use crate::{api::keyboard::Keyboard, runtime::Runtime};

    #[test]
    #[traced_test]
    #[ignore]
    fn test_keyboard() {
        Runtime::test(async |runtime| {
            let keyboard = Keyboard::new(runtime).unwrap();

            loop {
                sleep(Duration::from_secs(1)).await;

                println!(
                    "pressed: {}",
                    keyboard.is_key_pressed(Key::Return).await.unwrap()
                );
            }
        });
    }
}
