use std::sync::Arc;

use derive_more::Constructor;
use tracing::error;
use x11rb::protocol::randr::NotifyMask;
use x11rb_async::{connection::Connection, protocol::randr::ConnectionExt};

use crate::{
    platform::x11::X11Connection,
    runtime::events::{DisplayInfoVec, LatestOnlySignals, Topic},
};

#[derive(Debug, Constructor)]
pub struct ScreenChangeTopic {
    x11_connection: Arc<X11Connection>,
}

impl Topic for ScreenChangeTopic {
    type T = DisplayInfoVec;
    type Signal = LatestOnlySignals<Self::T>;

    async fn on_start(&self) {
        let connection = self.x11_connection.async_connection();

        if let Err(err) = connection
            .randr_select_input(self.x11_connection.screen().root, NotifyMask::SCREEN_CHANGE)
            .await
        {
            error!("randr_select_input failed: {err}");
        }

        if let Err(err) = connection.flush().await {
            error!("flush failed: {err}");
        }
    }

    async fn on_stop(&self) {
        let connection = self.x11_connection.async_connection();

        if let Err(err) = connection
            .randr_select_input(self.x11_connection.screen().root, NotifyMask::default())
            .await
        {
            error!("randr_select_input failed: {err}");
        }

        if let Err(err) = connection.flush().await {
            error!("flush failed: {err}");
        }
    }
}
