use std::sync::Arc;

use tokio::select;
use x11rb_async::protocol::xinput::{
    Device, DeviceType, XIEventMask, xi_query_device, xi_query_pointer,
};

use super::{JsButton, MouseImplTrait, Result};
use crate::{core::mouse::MouseError, runtime::Runtime};

#[derive(Debug)]
pub struct MouseImpl {
    runtime: Arc<Runtime>,
    master_pointer_device_id: u16,
}

impl JsButton {
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
        let cookie = xi_query_device(x11_connection.connection(), Device::ALL_MASTER)
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

        let mut event_receiver = runtime.subcribe_events();

        let cancellation_token = runtime.cancellation_token();

        runtime.task_tracker().spawn(async move {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    event = event_receiver.recv() => {
                        let Ok(_event) = event else {
                            break;
                        };

                        /*
                        match event {
                            RecordEvent::MouseButton(_button, _direction) => {
                                // TODO
                            }
                            _ => (),
                        }
                        */
                    }
                }
            }
        });

        Ok(Self {
            runtime,
            master_pointer_device_id,
        })
    }

    pub fn xinput_event_mask() -> XIEventMask {
        XIEventMask::RAW_BUTTON_PRESS | XIEventMask::RAW_BUTTON_RELEASE
    }
}

impl MouseImplTrait for MouseImpl {
    fn is_button_pressed(&mut self, button: JsButton) -> Result<bool> {
        let x11_connection = self.runtime.platform().x11_connection();
        let master_pointer_device_id = self.master_pointer_device_id;

        let buttons = Runtime::block_on(async move {
            let cookie = xi_query_pointer(
                x11_connection.connection(),
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

            Result::Ok(buttons)
        })
        .unwrap();

        let mask = buttons.first().ok_or_else(|| {
            MouseError::Unexpected("button mask should have at least one entry".into())
        })?;

        Ok(mask & button.into_button_mask() != 0)
    }
}
