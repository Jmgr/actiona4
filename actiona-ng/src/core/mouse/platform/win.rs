use std::sync::Arc;

use tokio::select;
use tokio_util::sync::CancellationToken;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2,
};

use super::{Button, MouseImplTrait, Result};
use crate::{
    core::mouse::ButtonConditions,
    error::CommonError,
    runtime::{Runtime, events::MouseButtonEvent},
};

#[derive(Debug)]
pub struct MouseImpl {
    runtime: Arc<Runtime>,
}

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
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { runtime })
    }
}

#[allow(unsafe_code)]
impl MouseImplTrait for MouseImpl {
    async fn is_button_pressed(&self, button: Button) -> Result<bool> {
        Ok(unsafe { GetAsyncKeyState(button.into_vkey()) as u16 & 0x8000u16 != 0 })
    }

    // TODO: put the logic in the crossplatform module
    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent> {
        let guard = self.runtime.mouse_buttons();
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

            let button_result = match conditions.button {
                None => true,
                Some(button) => button == event.button,
            };
            let direction_result = match conditions.direction {
                None => true,
                Some(direction) => direction == event.direction,
            };

            if button_result && direction_result {
                return Ok(event);
            }
        }

        Err(CommonError::Cancelled.into())
    }
}
