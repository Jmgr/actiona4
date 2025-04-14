#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

use color_eyre::Result;
use tokio::select;
use tokio_util::sync::CancellationToken;

pub use super::Button;
use crate::{
    api::mouse::ButtonConditions,
    error::CommonError,
    runtime::{Runtime, events::MouseButtonEvent},
};

pub async fn wait_for_button(
    runtime: &Runtime,
    conditions: ButtonConditions,
    cancellation_token: CancellationToken,
) -> Result<MouseButtonEvent> {
    let guard = runtime.mouse_buttons();
    let mut receiver = guard.subscribe();
    let runtime_cancellation_token = runtime.cancellation_token();

    loop {
        let event = select! {
            _ = runtime_cancellation_token.cancelled() => { break; }
            _ = cancellation_token.cancelled() => { break; }
            event = receiver.recv() => { event }
        };

        let Ok(event) = event else {
            break;
        };

        let button_result = conditions
            .button
            .is_none_or(|button| button == event.button);
        let direction_result = conditions
            .direction
            .is_none_or(|direction| direction == event.direction);

        if button_result && direction_result {
            return Ok(event);
        }
    }

    Err(CommonError::Cancelled.into())
}

pub trait MouseImplTrait {
    fn is_button_pressed(&self, button: Button) -> Result<bool>;
    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent>;
}
