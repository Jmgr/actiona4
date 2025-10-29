use std::ffi::{CString, NulError};

use thiserror::Error;
use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use x11rb::{rust_connection::RustConnection, xcb_ffi::XCBConnection};
use x11rb_async::{
    connection::Connection as AsyncConnection,
    errors::{ConnectError, ConnectionError, ReplyError},
    protocol::xproto::Screen,
    rust_connection::RustConnection as AsyncRustConnection,
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum X11Error {
    #[error("Connecting to the X11 server failed: {0}")]
    ConnectError(String),

    #[error("Connection to the X11 server failed: {0}")]
    ConnectionError(String),

    #[error("X11 reply error: {0}")]
    ReplyError(String),

    #[error(transparent)]
    NulErr(#[from] NulError),
}

impl From<ConnectError> for X11Error {
    fn from(value: ConnectError) -> Self {
        Self::ConnectError(value.to_string())
    }
}

impl From<ConnectionError> for X11Error {
    fn from(value: ConnectionError) -> Self {
        Self::ConnectionError(value.to_string())
    }
}

impl From<ReplyError> for X11Error {
    fn from(value: ReplyError) -> Self {
        Self::ReplyError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, X11Error>;

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
            select! {
                _ = local_cancellation_token.cancelled() => {}
                _ = packet_reader => {},
            }
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
