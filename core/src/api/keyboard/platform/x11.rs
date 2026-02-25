use std::{collections::HashSet, sync::Arc};

use color_eyre::Result;
use enigo::Key;
use x11rb::{
    connection::Connection, protocol::xproto::ConnectionExt, rust_connection::RustConnection,
};
use xkeysym::Keysym;

use crate::runtime::{Runtime, platform::x11::events::input::keysym_to_key};

#[derive(Clone, Debug)]
pub struct KeyboardImpl {
    runtime: Arc<Runtime>,
}

impl KeyboardImpl {
    pub const fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { runtime })
    }

    pub fn is_key_pressed(&self, key: Key) -> Result<bool> {
        let x11_connection = self.runtime.platform().x11_connection();
        let connection = x11_connection.sync_connection();
        let keysym: Keysym = key.into();
        let keycode = keysym_to_keycode(connection, keysym)?;

        let Some(keycode) = keycode else {
            return Ok(false);
        };

        is_key_pressed(connection, keycode)
    }

    pub fn get_pressed_keys(&self) -> Result<Vec<Key>> {
        let x11_connection = self.runtime.platform().x11_connection();
        let connection = x11_connection.sync_connection();
        let setup = connection.setup();
        let min = setup.min_keycode;
        let max = setup.max_keycode;

        let query_keymap = connection.query_keymap()?.reply()?;
        let mapping = connection
            .get_keyboard_mapping(min, max - min + 1)?
            .reply()?;

        let keys: HashSet<Key> = mapping
            .keysyms
            .chunks(mapping.keysyms_per_keycode.into())
            .enumerate()
            .filter_map(|(index, syms)| {
                let keycode = min + u8::try_from(index).ok()?;
                if !is_keycode_pressed(&query_keymap.keys, keycode) {
                    return None;
                }
                let raw_keysym = syms.iter().copied().find(|&ks| ks != 0)?;
                Some(keysym_to_key(Keysym::from(raw_keysym)))
            })
            .collect();

        Ok(keys.into_iter().collect())
    }
}

fn keysym_to_keycode(connection: &RustConnection, keysym: Keysym) -> Result<Option<u8>> {
    let setup = connection.setup();
    let min = setup.min_keycode;
    let max = setup.max_keycode;
    let mapping = connection
        .get_keyboard_mapping(min, max - min + 1)?
        .reply()?;

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

fn is_key_pressed(connection: &RustConnection, keycode: u8) -> Result<bool> {
    let reply = connection.query_keymap()?.reply()?;
    Ok(is_keycode_pressed(&reply.keys, keycode))
}

fn is_keycode_pressed(keymap: &[u8], keycode: u8) -> bool {
    let byte_index = usize::from(keycode) / 8;
    let bit_index = keycode % 8;
    (keymap[byte_index] & (1 << bit_index)) != 0
}
