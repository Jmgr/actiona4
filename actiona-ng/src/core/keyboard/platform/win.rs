use std::sync::Arc;

use enigo::Key;
use eyre::{Result, eyre};
use tokio::select;
use tokio_util::sync::CancellationToken;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VIRTUAL_KEY};

use crate::{error::CommonError, runtime::Runtime};

#[derive(Debug)]
pub struct KeyboardImpl {
    runtime: Arc<Runtime>,
}

impl KeyboardImpl {
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { runtime })
    }

    #[allow(unsafe_code)]
    pub fn is_key_pressed(&self, key: Key) -> Result<bool> {
        let key = VIRTUAL_KEY::try_from(key).map_err(|err| eyre!("invalid key: {err}"))?;

        Ok(unsafe { GetAsyncKeyState(key.0.into()) as u16 & 0x8000u16 != 0 })
    }

    pub async fn wait_for_key(
        &self,
        //conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<Key> {
        // MouseButtonEvent
        let guard = self.runtime.keyboard_keys();
        let mut receiver = guard.subscribe();
        let runtime_cancellation_token = self.runtime.cancellation_token();
        loop {
            let event = select! {
                _ = runtime_cancellation_token.cancelled() => { break; }
                _ = cancellation_token.cancelled() => { break; }
                event = receiver.recv() => { event }
            };

            let Ok(event) = event else {
                break;
            };

            if event.is_repeat {
                continue;
            }

            if event.key == Key::Escape {
                runtime_cancellation_token.cancel();
            }

            //println!("{:?}", event);

            return Ok(event.key); // TODO

            /*
            let button_result = conditions
                .button
                .is_none_or(|button| button == event.button);
            let direction_result = conditions
                .direction
                .is_none_or(|direction| direction == event.direction);

            if button_result && direction_result {
                return Ok(event);
            }
            */
        }

        Err(CommonError::Cancelled.into())
    }
}
