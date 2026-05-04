use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};
use satint::{SaturatingInto, Su32};
use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;
use types::{
    point::Point,
    rect::Rect,
    size::{Size, size},
};
use x11rb_async::{
    connection::Connection, protocol::xproto::ImageFormat, rust_connection::RustConnection,
};

use crate::Capture;

mod shm;

pub use shm::ShmSegment;

#[derive(Debug)]
struct X11Inner {
    connection: RustConnection,
    root: u32,
    full_screen_width: u16,
    full_screen_height: u16,
    root_depth: u8,
}

/// X11 screen capture handle. Owns a dedicated `x11rb-async` connection.
///
/// Cheap to clone — the connection is shared via an `Arc`.
#[derive(Clone, Debug)]
pub struct Screen {
    inner: Arc<X11Inner>,
}

impl Screen {
    /// Open a new X11 connection and spawn its driver task on the current
    /// tokio runtime. Returns `Err` if no display is reachable.
    pub async fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let (connection, screen_index, packet_reader) = RustConnection::connect(None).await?;
        let setup_screen = connection
            .setup()
            .roots
            .get(screen_index)
            .ok_or_else(|| eyre!("invalid X11 screen index {screen_index}"))?
            .clone();

        task_tracker.spawn(async move {
            select! {
                _ = cancellation_token.cancelled() => {},
                result = packet_reader => {
                    let Err(err) = result;
                    error!("X11 packet reader exited with error: {err}");
                },
            }
        });

        Ok(Self {
            inner: Arc::new(X11Inner {
                connection,
                root: setup_screen.root,
                full_screen_width: setup_screen.width_in_pixels,
                full_screen_height: setup_screen.height_in_pixels,
                root_depth: setup_screen.root_depth,
            }),
        })
    }

    /// The X11 root depth (typically 24 for true-color displays).
    pub fn root_depth(&self) -> u8 {
        self.inner.root_depth
    }

    /// Width of the full root window in pixels.
    pub fn full_screen_width(&self) -> u32 {
        u32::from(self.inner.full_screen_width)
    }

    /// Height of the full root window in pixels.
    pub fn full_screen_height(&self) -> u32 {
        u32::from(self.inner.full_screen_height)
    }

    pub(crate) fn connection(&self) -> &RustConnection {
        &self.inner.connection
    }

    pub(crate) fn root(&self) -> u32 {
        self.inner.root
    }

    /// Capture the entire root window. Always uses XGetImage (no SHM).
    pub async fn capture_full_screen(&self) -> Result<Capture> {
        let width: Su32 = self.inner.full_screen_width.saturating_into();
        let height: Su32 = self.inner.full_screen_height.saturating_into();
        let size = size(width, height);
        let bgra = get_image_raw(self, Point::ZERO, size).await?;
        Ok(Capture { size, bgra })
    }

    /// Capture a rectangle of the root window. Always uses XGetImage (no SHM).
    pub async fn capture_rect(&self, rect: Rect) -> Result<Capture> {
        let bgra = get_image_raw(self, rect.top_left, rect.size).await?;
        Ok(Capture {
            size: rect.size,
            bgra,
        })
    }
}

async fn get_image_raw(screen: &Screen, position: Point, size: Size) -> Result<Vec<u8>> {
    use x11rb_async::protocol::xproto::ConnectionExt;

    let reply = screen
        .connection()
        .get_image(
            ImageFormat::Z_PIXMAP,
            screen.root(),
            position.x.into_inner().try_into()?,
            position.y.into_inner().try_into()?,
            size.width.into_inner().try_into()?,
            size.height.into_inner().try_into()?,
            u32::MAX,
        )
        .await?
        .reply()
        .await?;

    Ok(reply.data)
}
