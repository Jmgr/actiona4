use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2,
};

use super::{Button, MouseImplTrait, Result};
use crate::{
    core::mouse::ButtonConditions,
    runtime::{Runtime, events::MouseButtonEvent},
};

#[derive(Debug, Default)]
pub struct MouseImpl;

impl Button {
    fn into_vkey(self) -> i32 {
        match self {
            Button::Left => VK_LBUTTON,
            Button::Middle => VK_MBUTTON,
            Button::Right => VK_RBUTTON,
            Button::Back => VK_XBUTTON1,
            Button::Forward => VK_XBUTTON2,
        }
        .0
        .into()
    }
}

impl MouseImpl {
    pub async fn new(_runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self)
    }
}

#[allow(unsafe_code)]
impl MouseImplTrait for MouseImpl {
    async fn is_button_pressed(&self, button: Button) -> Result<bool> {
        Ok(unsafe { GetAsyncKeyState(button.into_vkey()) as u16 & 0x8000u16 != 0 })
    }

    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent> {
        todo!()
    }
}
