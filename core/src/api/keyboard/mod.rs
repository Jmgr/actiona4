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

    pub async fn wait_for_keys(
        &self,
        keys: &HashSet<Key>,
        exclusive: bool, // TODO: options
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        if keys.is_empty() {
            return Ok(());
        }

        let guard = self.runtime.keyboard_keys();
        let mut receiver = guard.subscribe();
        let mut pressed_keys = HashSet::with_capacity(keys.len());

        loop {
            let event = cancel_on(&cancellation_token, receiver.recv()).await??;
            if event.is_injected || event.is_repeat {
                continue;
            }

            // Ignore keys that are not part of the list
            if !keys.contains(&event.key) {
                continue;
            }

            // Remove released keys
            if event.direction.is_release() {
                pressed_keys.remove(&event.key);
                continue;
            }

            pressed_keys.insert(event.key);

            if exclusive {
                if pressed_keys == *keys {
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

                println!(
                    "pressed: {}",
                    keyboard.is_key_pressed(Key::Return).await.unwrap()
                );
            }
        });
    }
}
