use std::{os::fd::OwnedFd, sync::Arc};

use color_eyre::Result;
use memfd::{FileSeal, MemfdOptions};
use memmap2::MmapMut;
use tokio::sync::Mutex;
use x11rb_async::{connection::Connection, protocol::xproto::ImageFormat};

use crate::{api::rect::Rect, platform::x11::X11Connection};

const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug)]
pub struct ShmCapture {
    segment_id: u32,
    map: MmapMut,
    lock: Mutex<()>,
}

impl ShmCapture {
    #[allow(unsafe_code)]
    pub async fn new(x11_connection: &X11Connection, size: usize) -> Result<Self> {
        let segment_id = x11_connection.async_connection().generate_id().await?;

        let memfd = MemfdOptions::default()
            .allow_sealing(true)
            .create("x11_screenshot")?;

        let memfd_file = memfd.as_file();
        memfd_file.set_len(size.try_into()?)?;

        memfd.add_seals(&[FileSeal::SealGrow, FileSeal::SealShrink, FileSeal::SealSeal])?;

        let map = unsafe { MmapMut::map_mut(memfd_file)? };

        use x11rb_async::protocol::shm::ConnectionExt;

        let owned_fd: OwnedFd = memfd.into_file().into();
        x11_connection
            .async_connection()
            .shm_attach_fd(segment_id, owned_fd, false)
            .await?;

        Ok(Self {
            segment_id,
            map,
            lock: Mutex::new(()),
        })
    }

    pub async fn capture(&self, x11_connection: &X11Connection, rect: Rect) -> Result<Vec<u8>> {
        use x11rb_async::protocol::shm::ConnectionExt;

        let root = x11_connection.screen().root;

        let _guard = self.lock.lock().await;

        x11_connection
            .async_connection()
            .shm_get_image(
                root,
                rect.top_left.x.into(),
                rect.top_left.y.into(),
                rect.size.width.into(),
                rect.size.height.into(),
                u32::MAX,
                ImageFormat::Z_PIXMAP.into(),
                self.segment_id,
                0,
            )
            .await?
            .reply()
            .await?;

        // Clone the SHM buffer while the mutex is still held so another capture
        // cannot overwrite it while callers are converting pixels or building
        // OpenCV Mats from the returned bytes.
        Ok(self.map.to_vec())
    }

    #[allow(clippy::as_conversions)]
    pub fn buffer_size_for_rect(rect: Rect) -> usize {
        let width: u32 = rect.size.width.into();
        let height: u32 = rect.size.height.into();
        (width as usize) * (height as usize) * BYTES_PER_PIXEL
    }
}

pub async fn get_image(x11_connection: Arc<X11Connection>, rect: Rect) -> Result<Vec<u8>> {
    use x11rb_async::protocol::xproto::ConnectionExt;

    let root = x11_connection.screen().root;

    let reply = x11_connection
        .async_connection()
        .get_image(
            ImageFormat::Z_PIXMAP,
            root,
            rect.top_left.x.into(),
            rect.top_left.y.into(),
            rect.size.width.into(),
            rect.size.height.into(),
            u32::MAX,
        )
        .await?
        .reply()
        .await?;

    Ok(reply.data)
}
