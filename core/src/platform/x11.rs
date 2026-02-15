use std::ffi::CString;

use color_eyre::Result;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use x11rb::{rust_connection::RustConnection, xcb_ffi::XCBConnection};
use x11rb_async::{
    connection::Connection as AsyncConnection, protocol::xproto::Screen,
    rust_connection::RustConnection as AsyncRustConnection,
};

use crate::cancel_on;

#[derive(Debug)]
pub struct X11Connection {
    async_connection: AsyncRustConnection,
    sync_connection: RustConnection,
    xcb_connection: XCBConnection,
    screen: Screen,
    screen_index: usize,
}

impl X11Connection {
    pub(crate) async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        display_name: Option<&str>,
    ) -> Result<Self> {
        let (async_connection, screen_index, packet_reader) =
            AsyncRustConnection::connect(display_name).await?;
        let screen = async_connection.setup().roots[screen_index].clone();
        let (sync_connection, _) = RustConnection::connect(display_name)?;

        let display_name = display_name.map(CString::new).transpose()?;
        let (xcb_connection, _) = XCBConnection::connect(display_name.as_deref())?;

        let local_cancellation_token = cancellation_token.clone();
        task_tracker.spawn(async move {
            _ = cancel_on(&local_cancellation_token, packet_reader).await;
        });

        Ok(Self {
            async_connection,
            sync_connection,
            xcb_connection,
            screen,
            screen_index,
        })
    }

    pub const fn async_connection(&self) -> &AsyncRustConnection {
        &self.async_connection
    }

    pub const fn sync_connection(&self) -> &RustConnection {
        &self.sync_connection
    }

    pub const fn xcb_connection(&self) -> &XCBConnection {
        &self.xcb_connection
    }

    pub const fn screen(&self) -> &Screen {
        &self.screen
    }

    pub const fn screen_index(&self) -> usize {
        self.screen_index
    }
}
