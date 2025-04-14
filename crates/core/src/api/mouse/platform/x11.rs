use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};
use enigo::Mouse as _;
use tokio_util::sync::CancellationToken;
use x11rb::protocol::xinput::ConnectionExt as SyncConnectionExt;
use x11rb_async::protocol::xinput::{Device, DeviceType, xi_query_device};

use super::{Button, MouseImplTrait};
use crate::{
    api::{
        mouse::{ButtonConditions, Coordinate},
        point::{Point, point},
    },
    runtime::{Runtime, events::MouseButtonEvent},
};

#[derive(Clone, Debug)]
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
        let cookie = xi_query_device(x11_connection.async_connection(), Device::ALL_MASTER).await?;
        let reply = cookie.reply().await?;

        let mut master_pointer_device_id = None;

        for info in &reply.infos {
            if info.type_ == DeviceType::MASTER_POINTER {
                master_pointer_device_id = Some(info.deviceid);
            }
        }

        let master_pointer_device_id = master_pointer_device_id
            .ok_or_else(|| eyre!("could not find master pointer device"))?;

        Ok(Self {
            runtime,
            master_pointer_device_id,
        })
    }

    pub fn set_position(&self, position: Point, coordinate: Coordinate) -> Result<()> {
        Ok(self.runtime.enigo().lock().move_mouse(
            position.x.into(),
            position.y.into(),
            coordinate,
        )?)
    }

    pub fn position(&self) -> Result<Point> {
        let current_position = self.runtime.enigo().lock().location()?;
        Ok(point(current_position.0, current_position.1))
    }
}

impl MouseImplTrait for MouseImpl {
    fn is_button_pressed(&self, button: Button) -> Result<bool> {
        let x11_connection = self.runtime.platform().x11_connection();
        let master_pointer_device_id = self.master_pointer_device_id;

        let buttons = x11_connection
            .sync_connection()
            .xinput_xi_query_pointer(x11_connection.screen().root, master_pointer_device_id)?
            .reply()?
            .buttons;

        let mask = buttons
            .first()
            .ok_or_else(|| eyre!("button mask should have at least one entry"))?;

        Ok(mask & button.into_button_mask() != 0)
    }

    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent> {
        super::wait_for_button(self.runtime.as_ref(), conditions, cancellation_token).await
    }
}
