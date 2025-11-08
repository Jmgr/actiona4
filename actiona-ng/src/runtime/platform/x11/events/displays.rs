use std::sync::Arc;

use derive_more::Constructor;
use eyre::Result;
use x11rb::protocol::randr::NotifyMask;
use x11rb_async::{connection::Connection, protocol::randr::ConnectionExt};

use crate::{
    platform::x11::X11Connection,
    runtime::events::{DisplayInfoVec, LatestOnlySignals, Topic},
};

#[derive(Constructor, Debug)]
pub struct ScreenChangeTopic {
    x11_connection: Arc<X11Connection>,
}

impl Topic for ScreenChangeTopic {
    type T = DisplayInfoVec;
    type Signal = LatestOnlySignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        let connection = self.x11_connection.async_connection();

        connection
            .randr_select_input(self.x11_connection.screen().root, NotifyMask::SCREEN_CHANGE)
            .await?;

        connection.flush().await?;

        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        let connection = self.x11_connection.async_connection();

        connection
            .randr_select_input(self.x11_connection.screen().root, NotifyMask::default())
            .await?;

        connection.flush().await?;

        Ok(())
    }
}
