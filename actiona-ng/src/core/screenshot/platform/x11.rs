#![allow(unsafe_code)]

// TODO: https://chatgpt.com/share/68cbf389-ce58-8001-bbd8-4e0b543ae240

use std::{
    collections::HashMap,
    ffi::c_void,
    num::NonZero,
    os::fd::{AsFd, OwnedFd},
    ptr::NonNull,
    sync::Arc,
};

use eyre::{Result, eyre};
use image::{DynamicImage, RgbaImage};
use memfd::{FileSeal, MemfdOptions};
use nix::sys::mman::{MapFlags, ProtFlags, mmap, munmap};
use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};
use tokio::{select, sync::Mutex};
use x11rb_async::{connection::Connection, protocol::xproto::ImageFormat};

use super::ScreenshotImplTrait;
use crate::{
    core::{
        color::Color,
        displays::Displays,
        image::Image,
        point::Point,
        rect::{Rect, rect},
    },
    platform::x11::X11Connection,
    runtime::{DisplayInfo, DisplayInfoVec, RecordEvent, Runtime},
};

#[derive(Debug)]
struct ShmData {
    shm_segment_id: u32,
    mapped_ptr: NonNull<c_void>,
    size: usize,
    x11_connection: Arc<X11Connection>,
}

// TODO: Safety?
unsafe impl Send for ShmData {}
unsafe impl Sync for ShmData {}

impl Drop for ShmData {
    fn drop(&mut self) {
        use x11rb_async::protocol::shm::ConnectionExt;

        let _ = Runtime::block_on(
            self.x11_connection
                .connection()
                .shm_detach(self.shm_segment_id),
        );
        unsafe {
            let _ = munmap(self.mapped_ptr, self.size);
        }
    }
}

#[derive(Debug)]
struct Display {
    rect: Rect,
    shm_data: Option<ShmData>,
    x11_connection: Arc<X11Connection>,
}

impl Display {
    async fn new(
        runtime: Arc<Runtime>,
        display_info: &DisplayInfo,
        x11_connection: Arc<X11Connection>,
    ) -> Result<Self> {
        const BYTES_PER_PIXEL: usize = 4;
        let rect = display_info.rect;
        let image_size = (rect.width as usize) * (rect.height as usize) * BYTES_PER_PIXEL;

        let shm_data = if runtime.platform().has_shm() {
            let shm_segment_id = x11_connection.connection().generate_id().await?;

            let memfd = MemfdOptions::default()
                .allow_sealing(true)
                .create("x11_screenshot")?;

            let memfd_file = memfd.as_file();
            memfd_file.set_len(image_size as u64)?;

            memfd.add_seals(&[FileSeal::SealGrow, FileSeal::SealShrink, FileSeal::SealSeal])?;

            let mapped_ptr = unsafe {
                mmap(
                    None,
                    NonZero::new(image_size).unwrap(),
                    ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                    MapFlags::MAP_SHARED,
                    memfd_file.as_fd(),
                    0,
                )?
            };

            let result = ShmData {
                shm_segment_id,
                mapped_ptr,
                size: image_size,
                x11_connection: x11_connection.clone(),
            };

            use x11rb_async::protocol::shm::ConnectionExt;

            let owned_fd: OwnedFd = memfd.into_file().into();
            x11_connection
                .connection()
                .shm_attach_fd(shm_segment_id, owned_fd, false)
                .await?;

            Some(result)
        } else {
            None
        };

        Ok(Self {
            rect,
            shm_data,
            x11_connection,
        })
    }

    async fn capture_shm_get_image(&self, shm_data: &ShmData) -> Result<Image> {
        use x11rb_async::protocol::shm::ConnectionExt;

        let root = self.x11_connection.screen().root;

        self.x11_connection
            .connection()
            .shm_get_image(
                root,
                self.rect.x as i16,
                self.rect.y as i16,
                self.rect.width as u16,
                self.rect.height as u16,
                u32::MAX, // plane mask (capture all planes)
                ImageFormat::Z_PIXMAP.into(),
                shm_data.shm_segment_id,
                0, // offset into the shared memory region
            )
            .await?
            .reply()
            .await?;

        // Extract pixel data from shared memory
        let pixel_data = unsafe {
            std::slice::from_raw_parts(shm_data.mapped_ptr.as_ptr() as *const u8, shm_data.size)
        };

        Ok(image_from_bgr_data(
            pixel_data,
            self.rect.width,
            self.rect.height,
        ))
    }

    pub async fn capture(&self) -> Result<Image> {
        if let Some(shm_data) = &self.shm_data {
            self.capture_shm_get_image(shm_data).await
        } else {
            capture_get_image(self.x11_connection.clone(), self.rect).await
        }
    }
}

#[derive(Debug)]
pub struct ScreenshotImpl {
    display_map: Arc<Mutex<HashMap<u32, Display>>>,
    x11_connection: Arc<X11Connection>,
}

impl ScreenshotImpl {
    pub async fn new(runtime: Arc<Runtime>, displays: Arc<Displays>) -> Result<Self> {
        let mut event_receiver = runtime.subcribe_events();

        let display_map = Arc::new(Mutex::new(HashMap::new()));
        let local_display_map = display_map.clone();

        let local_runtime = runtime.clone();

        let x11_connection = runtime.platform().x11_connection();
        let local_x11_connection = x11_connection.clone();

        let cancellation_token = runtime.cancellation_token();

        runtime.task_tracker().spawn(async move {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    event = event_receiver.recv() => {
                        let Ok(event) = event else {
                            break;
                        };

                        if let RecordEvent::DisplayChanged(displays_info) = event {
                            Self::update_displays(
                                local_runtime.clone(),
                                local_display_map.clone(),
                                displays_info,
                                local_x11_connection.clone(),
                            )
                            .await
                            .unwrap();
                        }
                    }
                }
            }
        });

        let displays_info = {
            let displays_info = displays.displays_info().lock().unwrap();
            displays_info.clone()
        };

        Self::update_displays(
            runtime.clone(),
            display_map.clone(),
            displays_info,
            x11_connection.clone(),
        )
        .await?;

        let result = Self {
            display_map,
            x11_connection,
        };

        Ok(result)
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn update_displays(
        runtime: Arc<Runtime>,
        display_map: Arc<Mutex<HashMap<u32, Display>>>,
        displays_info: DisplayInfoVec,
        x11_connection: Arc<X11Connection>,
    ) -> Result<()> {
        let mut display_map = display_map.lock().await;

        display_map.clear();

        for display_info in displays_info.iter() {
            display_map.insert(
                display_info.id,
                Display::new(runtime.clone(), display_info, x11_connection.clone()).await?,
            );
        }

        Ok(())
    }
}

impl ScreenshotImplTrait for ScreenshotImpl {
    async fn capture_rect(&mut self, rect: Rect) -> Result<Image> {
        capture_get_image(self.x11_connection.clone(), rect).await
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn _capture_display(&mut self, display_id: u32) -> Result<Image> {
        let display_map = self.display_map.lock().await;
        let display = display_map
            .get(&display_id)
            .ok_or_else(|| eyre!("unknown display id: {display_id}"))?;

        display.capture().await
    }

    async fn _capture_pixel(&mut self, position: Point) -> Result<Color> {
        let result = self
            .capture_rect(rect(position.x, position.y, 1, 1))
            .await?;

        Ok((*result
            .as_rgba8()
            .expect("image should be RGBA")
            .get_pixel(0, 0))
        .into())
    }
}

fn image_from_bgr_data(data: &[u8], width: u32, height: u32) -> Image {
    let mut image = RgbaImage::new(width, height);

    let image_data: &mut [u8] = image.as_mut();

    image_data.copy_from_slice(data);

    // Convert from BGR to RGB
    image_data.par_chunks_exact_mut(4).for_each(|c| {
        c.swap(0, 2); // Swap R and B
        c[3] = 255; // Set alpha to 255 (fully opaque)
    });

    DynamicImage::ImageRgba8(image).into()
}

async fn capture_get_image(x11_connection: Arc<X11Connection>, rect: Rect) -> Result<Image> {
    use x11rb_async::protocol::xproto::ConnectionExt;

    let root = x11_connection.screen().root;

    let reply = x11_connection
        .connection()
        .get_image(
            ImageFormat::Z_PIXMAP,
            root,
            rect.x as i16,
            rect.y as i16,
            rect.width as u16,
            rect.height as u16,
            u32::MAX,
        )
        .await?
        .reply()
        .await?;

    Ok(image_from_bgr_data(&reply.data, rect.width, rect.height))
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Instant};

    use crate::{
        core::{
            displays::Displays,
            screenshot::platform::{ScreenshotImplTrait, x11::ScreenshotImpl},
        },
        runtime::Runtime,
    };

    #[test]
    fn test_screenshot() {
        Runtime::test(async |runtime| {
            let displays = Displays::new(runtime.clone()).unwrap();

            /*
            let rect2 = {
                let mut rect2 = rect(0, 0, 0, 0);
                let displays_info = displays.displays_info().lock().unwrap();

                for display in displays_info.iter() {
                    rect2 = rect2.union(rect(display.x, display.y, display.width, display.height));
                }
                rect2
            };

            let start = Instant::now();

            // SHM: 0.08528653
            // get_image: 0.42951524
            // GL: 0.11083711
            let image = Screenshot::new(runtime, Arc::new(displays))
                .await
                .unwrap()
                .capture_rect(rect2)
                .await
                .unwrap();

            println!("elapsed: {}", (Instant::now() - start).as_secs_f32());

            image.save("/tmp/test/test.bmp").unwrap();
            */

            /*
            SHM

            elapsed: 0.021464119
            elapsed: 0.010506714
            elapsed: 0.007166257

            getimage

            elapsed: 0.07405177
            elapsed: 0.014961153
            elapsed: 0.00688114
             */

            let displays_info = {
                let displays_info = displays.displays_info().lock().unwrap();
                displays_info.clone()
            };

            let displays = Arc::new(displays);

            let mut imp = ScreenshotImpl::new(runtime, displays).await.unwrap();

            for display in displays_info.iter() {
                let start = Instant::now();

                let image = imp._capture_display(display.id).await.unwrap();

                println!("elapsed: {}", (Instant::now() - start).as_secs_f32());

                image
                    .save(format!("/tmp/test/test{}.bmp", display.id))
                    .unwrap();
            }
        });
    }
}
