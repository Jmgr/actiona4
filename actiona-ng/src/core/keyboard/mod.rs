use std::sync::{Arc, Mutex};

use enigo::{Direction, Enigo, Key};
use eyre::Result;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

pub(crate) mod platform;

pub mod js;

pub use enigo::Coordinate;
#[cfg(windows)]
use platform::win::KeyboardImpl;
#[cfg(unix)]
use platform::x11::KeyboardImpl;

use crate::runtime::Runtime;

#[derive(Debug)]
pub struct Keyboard {
    enigo: Arc<Mutex<Enigo>>,
    implementation: KeyboardImpl,
}

impl Keyboard {
    #[instrument]
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self {
            enigo: runtime.enigo(),
            implementation: KeyboardImpl::new(runtime)?,
        })
    }

    #[instrument(skip(self), err, ret)]
    pub fn text(&self, text: &str) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().unwrap().text(text)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn key(&self, key: Key, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().unwrap().key(key, direction)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn raw(&self, keycode: u16, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().unwrap().raw(keycode, direction)?;

        Ok(())
    }

    pub async fn is_key_pressed(&self, key: Key) -> Result<bool> {
        self.implementation.is_key_pressed(key).await
    }

    pub async fn wait_for_key(&self, cancellation_token: CancellationToken) -> Result<()> {
        self.implementation.wait_for_key(cancellation_token).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use enigo::Key;
    use tokio::time::sleep;
    use tracing_test::traced_test;

    use crate::{core::keyboard::Keyboard, runtime::Runtime};

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
