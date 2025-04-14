use std::collections::HashSet;

use color_eyre::{Result, eyre::eyre};
use enigo::Key;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VIRTUAL_KEY, VK_CONTROL, VK_LBUTTON, VK_MBUTTON, VK_MENU, VK_RBUTTON,
    VK_SHIFT, VK_XBUTTON1, VK_XBUTTON2,
};

use crate::runtime::platform::win::events::input::keyboard::{
    get_keystate, vk_to_enigo_key_with_snapshot,
};

#[derive(Clone, Debug, Default)]
pub struct KeyboardImpl {}

impl KeyboardImpl {
    pub fn is_key_pressed(&self, key: Key) -> Result<bool> {
        let key = VIRTUAL_KEY::try_from(key).map_err(|err| eyre!("invalid key: {err}"))?;

        Ok(is_virtual_key_pressed(key))
    }

    pub fn get_pressed_keys(&self) -> Result<Vec<Key>> {
        let keystate = get_keystate();
        let keys: HashSet<Key> = (0u16..=255)
            .filter(|&vk| !skip_virtual_key(vk))
            .filter(|&vk| is_virtual_key_pressed(VIRTUAL_KEY(vk)))
            .map(|vk| vk_to_enigo_key_with_snapshot(u32::from(vk), &keystate))
            .collect();

        Ok(keys.into_iter().collect())
    }
}

#[allow(unsafe_code)]
fn is_virtual_key_pressed(key: VIRTUAL_KEY) -> bool {
    #[allow(clippy::as_conversions)] // i16 → u16 bitwise check, not a numeric conversion
    unsafe {
        GetAsyncKeyState(key.0.into()) as u16 & 0x8000u16 != 0
    }
}

const fn skip_virtual_key(virtual_key: u16) -> bool {
    matches!(
        VIRTUAL_KEY(virtual_key),
        // Generic modifiers duplicate the side-specific variants.
        VK_SHIFT | VK_CONTROL | VK_MENU
            // Mouse buttons are not part of keyboard state.
            | VK_LBUTTON
            | VK_RBUTTON
            | VK_MBUTTON
            | VK_XBUTTON1
            | VK_XBUTTON2
    )
}
