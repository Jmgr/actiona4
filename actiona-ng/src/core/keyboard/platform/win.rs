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
    pub async fn is_key_pressed(&self, key: Key) -> Result<bool> {
        let key = VIRTUAL_KEY::try_from(key).map_err(|err| eyre!("invalid key: {err}"))?;

        Ok(unsafe { GetAsyncKeyState(key.0.into()) as u16 & 0x8000u16 != 0 })
    }
}
