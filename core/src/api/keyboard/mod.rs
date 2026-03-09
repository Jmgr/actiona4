use std::{collections::HashSet, fmt::Display, sync::Arc};

use color_eyre::Result;
use enigo::{Direction, Enigo, Key};
use parking_lot::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

pub(crate) mod platform;

pub mod js;
mod key_triggers;
mod text_replacements;

pub use enigo::Coordinate;
pub(crate) use key_triggers::KeyExt;
#[cfg(windows)]
use platform::win::KeyboardImpl;
#[cfg(unix)]
use platform::x11::KeyboardImpl;

use crate::{cancel_on, runtime::Runtime, types::display::DisplayFields};

#[derive(Clone, Debug)]
pub struct Keyboard {
    runtime: Arc<Runtime>,
    enigo: Arc<Mutex<Enigo>>,
    implementation: KeyboardImpl,
}

impl Display for Keyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default().finish(f)
    }
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

    pub fn check_platform(&self) -> color_eyre::Result<()> {
        self.runtime.require_not_wayland()
    }

    #[instrument(skip(self), err, ret)]
    pub fn text(&self, text: &str) -> Result<()> {
        use enigo::Keyboard;

        self.runtime.require_not_wayland()?;
        self.enigo.lock().text(text)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn press(&self, key: Key) -> Result<()> {
        use enigo::Keyboard;

        self.runtime.require_not_wayland()?;
        self.enigo.lock().key(key, Direction::Press)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn release(&self, key: Key) -> Result<()> {
        use enigo::Keyboard;

        self.runtime.require_not_wayland()?;
        self.enigo.lock().key(key, Direction::Release)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn tap(&self, key: Key) -> Result<()> {
        use enigo::Keyboard;

        self.runtime.require_not_wayland()?;
        self.enigo.lock().key(key, Direction::Click)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn press_raw(&self, keycode: u16) -> Result<()> {
        use enigo::Keyboard;

        self.runtime.require_not_wayland()?;
        self.enigo.lock().raw(keycode, Direction::Press)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn release_raw(&self, keycode: u16) -> Result<()> {
        use enigo::Keyboard;

        self.runtime.require_not_wayland()?;
        self.enigo.lock().raw(keycode, Direction::Release)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn tap_raw(&self, keycode: u16) -> Result<()> {
        use enigo::Keyboard;

        self.runtime.require_not_wayland()?;
        self.enigo.lock().raw(keycode, Direction::Click)?;

        Ok(())
    }

    pub fn is_key_pressed(&self, key: Key) -> Result<bool> {
        self.runtime.require_not_wayland()?;
        self.implementation.is_key_pressed(key)
    }

    pub fn get_pressed_keys(&self) -> Result<Vec<Key>> {
        self.runtime.require_not_wayland()?;
        self.implementation.get_pressed_keys()
    }

    pub async fn wait_for_keys(
        &self,
        keys: &HashSet<Key>,
        exclusive: bool,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        self.runtime.require_not_wayland()?;
        if keys.is_empty() {
            return Ok(());
        }

        // Normalize any physical modifier keys to their generic form (e.g. LControl -> Control),
        // matching how incoming events are normalized before comparison.
        let keys: HashSet<Key> = keys.iter().map(|key| key.normalize()).collect();

        let guard = self.runtime.keyboard_keys();
        let mut receiver = guard.subscribe();
        let mut pressed_keys = HashSet::with_capacity(keys.len());

        loop {
            let event = cancel_on(&cancellation_token, receiver.recv()).await??;
            if event.is_injected || event.is_repeat {
                continue;
            }

            // Normalize the incoming key so that e.g. LControl matches a Control requirement
            let key = event.key.normalize();

            // Ignore keys that are not part of the awaited set.
            if !keys.contains(&key) {
                continue;
            }

            // Remove released keys
            if event.direction.is_release() {
                pressed_keys.remove(&key);
                continue;
            } else {
                pressed_keys.insert(key);
            }

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

                println!("pressed: {}", keyboard.is_key_pressed(Key::Return).unwrap());
            }
        });
    }
}
