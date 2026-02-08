use color_eyre::{Result, eyre::eyre};
use enigo::Key;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VIRTUAL_KEY};

#[derive(Debug, Default)]
pub struct KeyboardImpl {}

impl KeyboardImpl {
    #[allow(unsafe_code)]
    pub async fn is_key_pressed(&self, key: Key) -> Result<bool> {
        let key = VIRTUAL_KEY::try_from(key).map_err(|err| eyre!("invalid key: {err}"))?;

        #[allow(clippy::as_conversions)] // i16 → u16 bitwise check, not a numeric conversion
        Ok(unsafe { GetAsyncKeyState(key.0.into()) as u16 & 0x8000u16 != 0 })
    }
}
