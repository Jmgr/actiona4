use std::os::fd::OwnedFd;

use color_eyre::Result;
use memfd::{FileSeal, MemfdOptions};
use memmap2::MmapMut;
use satint::SaturatingInto;
use tokio::sync::Mutex;
use types::rect::Rect;
use x11rb_async::{connection::Connection, protocol::xproto::ImageFormat};

use crate::{Capture, platform::x11::Screen};

const BYTES_PER_PIXEL: usize = 4;

/// Pre-allocated shared-memory segment for fast X11 screen captures.
///
/// One segment can be reused across captures whose buffer size fits within
/// its capacity. Concurrent captures into the same segment are serialised
/// internally so the buffer is not overwritten while it is being read.
#[derive(Debug)]
pub struct ShmSegment {
    segment_id: u32,
    map: MmapMut,
    capacity: usize,
    lock: Mutex<()>,
}

impl ShmSegment {
    /// Allocate a new SHM segment of `capacity` bytes attached to `screen`.
    #[allow(unsafe_code)]
    pub async fn new(screen: &Screen, capacity: usize) -> Result<Self> {
        let segment_id = screen.connection().generate_id().await?;

        let memfd = MemfdOptions::default()
            .allow_sealing(true)
            .create("x11_screenshot")?;

        let memfd_file = memfd.as_file();
        memfd_file.set_len(capacity.try_into()?)?;

        memfd.add_seals(&[FileSeal::SealGrow, FileSeal::SealShrink, FileSeal::SealSeal])?;

        let map = unsafe { MmapMut::map_mut(memfd_file)? };

        use x11rb_async::protocol::shm::ConnectionExt;

        let owned_fd: OwnedFd = memfd.into_file().into();
        screen
            .connection()
            .shm_attach_fd(segment_id, owned_fd, false)
            .await?;

        Ok(Self {
            segment_id,
            map,
            capacity,
            lock: Mutex::new(()),
        })
    }

    /// Capture `rect` of `screen` into this segment and return the pixels.
    ///
    /// Returns `Err` if the rect's pixel buffer is larger than the segment's
    /// capacity.
    pub async fn capture_rect(&self, screen: &Screen, rect: Rect) -> Result<Capture> {
        use x11rb_async::protocol::shm::ConnectionExt;

        let needed = Self::capacity_for_rect(rect);
        if needed > self.capacity {
            return Err(color_eyre::eyre::eyre!(
                "SHM segment too small: need {needed} bytes, have {}",
                self.capacity
            ));
        }

        let _guard = self.lock.lock().await;

        screen
            .connection()
            .shm_get_image(
                screen.root(),
                rect.top_left.x.saturating_into(),
                rect.top_left.y.saturating_into(),
                rect.size.width.saturating_into(),
                rect.size.height.saturating_into(),
                u32::MAX,
                ImageFormat::Z_PIXMAP.into(),
                self.segment_id,
                0,
            )
            .await?
            .reply()
            .await?;

        // Clone the buffer while the mutex is still held so a concurrent
        // capture cannot overwrite it before the caller is done with it.
        let bgra = self.map[..needed].to_vec();
        Ok(Capture {
            size: rect.size,
            bgra,
        })
    }

    /// Bytes required to back a capture of the given rectangle.
    #[allow(clippy::as_conversions)]
    pub fn capacity_for_rect(rect: Rect) -> usize {
        let width: u32 = rect.size.width.into();
        let height: u32 = rect.size.height.into();
        (width as usize) * (height as usize) * BYTES_PER_PIXEL
    }

    /// Capacity of this segment, in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
