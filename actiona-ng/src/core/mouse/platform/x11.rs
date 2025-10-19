use std::sync::Arc;

use tokio::select;
use tokio_util::sync::CancellationToken;
use x11rb_async::protocol::xinput::{Device, DeviceType, xi_query_device, xi_query_pointer};

use super::{Button, MouseImplTrait, Result};
use crate::{
    core::mouse::{ButtonConditions, MouseError},
    error::CommonError,
    runtime::{Runtime, events::MouseButtonEvent},
};

#[derive(Debug)]
pub struct MouseImpl {
    runtime: Arc<Runtime>,
    master_pointer_device_id: u16,
}

impl Button {
    const fn into_button_mask(self) -> u32 {
        match self {
            Self::Left => 1 << 1,
            Self::Middle => 1 << 2,
            Self::Right => 1 << 3,
            Self::Back => 1 << 8,
            Self::Forward => 1 << 9,
        }
    }
}

impl MouseImpl {
    pub async fn new(runtime: Arc<Runtime>) -> Result<Self> {
        let x11_connection = runtime.platform().x11_connection();
        let cookie = xi_query_device(x11_connection.async_connection(), Device::ALL_MASTER)
            .await
            .unwrap();
        let reply = cookie.reply().await.unwrap();

        let mut master_pointer_device_id = None;

        for info in &reply.infos {
            if info.type_ == DeviceType::MASTER_POINTER {
                master_pointer_device_id = Some(info.deviceid);
            }
        }

        let master_pointer_device_id = master_pointer_device_id.unwrap(); // TODO

        Ok(Self {
            runtime,
            master_pointer_device_id,
        })
    }
}

impl MouseImplTrait for MouseImpl {
    async fn is_button_pressed(&self, button: Button) -> Result<bool> {
        let x11_connection = self.runtime.platform().x11_connection();
        let master_pointer_device_id = self.master_pointer_device_id;

        let cookie = xi_query_pointer(
            x11_connection.async_connection(),
            x11_connection.screen().root,
            master_pointer_device_id,
        )
        .await
        .map_err(|err| MouseError::ConnectionError(err.to_string()))?;

        let buttons = cookie
            .reply()
            .await
            .map_err(|err| MouseError::ReplyError(err.to_string()))?
            .buttons;

        let mask = buttons.first().ok_or_else(|| {
            MouseError::Unexpected("button mask should have at least one entry".into())
        })?;

        Ok(mask & button.into_button_mask() != 0)
    }

    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent> {
        let guard = self.runtime.platform().mouse_buttons().subscribe();
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
}
