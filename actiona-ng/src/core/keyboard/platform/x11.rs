use std::sync::Arc;

use enigo::Key;
use eyre::Result;
use tokio::select;
use tokio_util::sync::CancellationToken;
use x11rb_async::{
    connection::Connection, protocol::xproto::ConnectionExt, rust_connection::RustConnection,
};
use xkeysym::Keysym;

use crate::{error::CommonError, runtime::Runtime};

#[derive(Debug)]
pub struct KeyboardImpl {
    runtime: Arc<Runtime>,
}

impl KeyboardImpl {
    pub const fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { runtime })
    }

    pub async fn is_key_pressed(&self, key: Key) -> Result<bool> {
        let x11_connection = self.runtime.platform().x11_connection();
        let connection = x11_connection.async_connection();
        let keysym: Keysym = key.into();
        let keycode = keysym_to_keycode(connection, keysym).await?;

        let Some(keycode) = keycode else {
            return Ok(false);
        };

        is_key_pressed(connection, keycode).await
    }

    pub async fn wait_for_key(
        &self,
        //conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<Key> {
        // MouseButtonEvent
        let guard = self.runtime.platform().keyboard_keys().subscribe();
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

            if event.key == Key::Escape {
                runtime_cancellation_token.cancel();
            }

            return Ok(event.key);

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

async fn keysym_to_keycode(connection: &RustConnection, keysym: Keysym) -> Result<Option<u8>> {
    let setup = connection.setup();
    let min = setup.min_keycode;
    let max = setup.max_keycode;
    let mapping = connection
        .get_keyboard_mapping(min, max - min + 1)
        .await?
        .reply()
        .await?;

    for (i, syms) in mapping
        .keysyms
        .chunks(mapping.keysyms_per_keycode.into())
        .enumerate()
    {
        if syms.contains(&keysym.raw()) {
            return Ok(Some(min + u8::try_from(i)?));
        }
    }
    Ok(None)
}

async fn is_key_pressed(connection: &RustConnection, keycode: u8) -> Result<bool> {
    let reply = connection.query_keymap().await?.reply().await?;
    let byte_index = usize::from(keycode) / 8;
    let bit_index = keycode % 8;
    Ok((reply.keys[byte_index] & (1 << bit_index)) != 0)
}
