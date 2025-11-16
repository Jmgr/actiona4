use std::sync::Arc;

use color_eyre::Result;
use enigo::Key;
use x11rb_async::{
    connection::Connection, protocol::xproto::ConnectionExt, rust_connection::RustConnection,
};
use xkeysym::Keysym;

use crate::runtime::Runtime;

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
