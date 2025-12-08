use std::{collections::HashMap, ffi::c_void, os::fd::OwnedFd, sync::Arc, time::Instant};

use color_eyre::{Result, eyre::eyre};
use image::{DynamicImage, RgbaImage};
use memfd::{FileSeal, MemfdOptions};
use memmap2::MmapMut;
use opencv::{
    boxed_ref::BoxedRef,
    core::{CV_8UC1, CV_8UC3, CV_8UC4, Mat, MatTraitConst, Size as CvSize, Vector},
    imgcodecs::imwrite,
    imgproc::{COLOR_BGRA2BGR, COLOR_BGRA2GRAY, cvt_color},
};
use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};
use tokio::sync::Mutex;
use tracing::error;
use x11rb_async::{connection::Connection, protocol::xproto::ImageFormat};

use super::ScreenshotImplTrait;
use crate::{
    core::{
        color::Color,
        displays::Displays,
        image::Image,
        point::{Point, point},
        rect::{Rect, rect},
        size::size,
    },
    platform::x11::X11Connection,
    runtime::{
        Runtime,
        async_resource::AsyncResource,
        events::{DisplayInfo, DisplayInfoVec},
    },
    types::su32::su32,
};

const BYTES_PER_PIXEL: usize = 4;

#[derive(Debug)]
struct ShmData {
    shm_segment_id: u32,
    map: MmapMut,
}

#[derive(Debug)]
struct Display {
    rect: Rect,
    shm_data: Option<ShmData>,
    x11_connection: Arc<X11Connection>,
    shm_lock: Mutex<()>,
    rgb_mat_buffer: parking_lot::Mutex<Mat>,
    greyscale_mat_buffer: parking_lot::Mutex<Mat>,
}

impl Display {
    #[allow(unsafe_code)]
    async fn new(runtime: Arc<Runtime>, display_info: &DisplayInfo) -> Result<Self> {
        let rect = display_info.rect;
        let image_size = rect.size.width * rect.size.height * su32(BYTES_PER_PIXEL);
        let x11_connection = runtime.platform().x11_connection();

        let root_depth = x11_connection.screen().root_depth;
        if root_depth != 24 {
            return Err(eyre!("unsupported X11 depth: {}", root_depth));
        }

        let shm_data = if runtime.platform().has_shm() {
            let shm_segment_id = x11_connection.async_connection().generate_id().await?;

            let memfd = MemfdOptions::default()
                .allow_sealing(true)
                .create("x11_screenshot")?;

            let memfd_file = memfd.as_file();
            memfd_file.set_len(image_size.into())?;

            memfd.add_seals(&[FileSeal::SealGrow, FileSeal::SealShrink, FileSeal::SealSeal])?;

            let map = unsafe { MmapMut::map_mut(memfd_file)? };

            let result = ShmData {
                shm_segment_id,
                map,
            };

            use x11rb_async::protocol::shm::ConnectionExt;

            let owned_fd: OwnedFd = memfd.into_file().into();
            x11_connection
                .async_connection()
                .shm_attach_fd(shm_segment_id, owned_fd, false)
                .await?;

            Some(result)
        } else {
            None
        };

        let rgb_mat_buffer = unsafe {
            Mat::new_size(
                CvSize::new(rect.size.width.into(), rect.size.height.into()),
                CV_8UC3,
            )?
        };

        let greyscale_mat_buffer = unsafe {
            Mat::new_size(
                CvSize::new(rect.size.width.into(), rect.size.height.into()),
                CV_8UC1,
            )?
        };

        Ok(Self {
            rect,
            shm_data,
            x11_connection,
            shm_lock: Mutex::new(()),
            rgb_mat_buffer: parking_lot::Mutex::new(rgb_mat_buffer),
            greyscale_mat_buffer: parking_lot::Mutex::new(greyscale_mat_buffer),
        })
    }

    async fn capture_shm_get_image(&self, shm_data: &ShmData, rect: Rect) -> Result<Image> {
        use x11rb_async::protocol::shm::ConnectionExt;

        let root = self.x11_connection.screen().root;

        let _guard = self.shm_lock.lock().await;

        self.x11_connection
            .async_connection()
            .shm_get_image(
                root,
                rect.origin.x.into(),
                rect.origin.y.into(),
                rect.size.width.into(), // TODO: document that the max image size is u16::MAX
                rect.size.height.into(),
                u32::MAX, // plane mask (capture all planes)
                ImageFormat::Z_PIXMAP.into(),
                shm_data.shm_segment_id,
                0, // offset into the shared memory region
            )
            .await?
            .reply()
            .await?;

        // Extract pixel data from shared memory
        let pixel_data = &shm_data.map;

        image_from_bgr_data(
            pixel_data,
            rect.size.width.try_into()?,
            rect.size.height.try_into()?,
        )
    }

    pub async fn capture(&self) -> Result<Image> {
        if let Some(shm_data) = &self.shm_data {
            self.capture_shm_get_image(shm_data, self.rect).await
        } else {
            capture_get_image(self.x11_connection.clone(), self.rect).await
        }
    }

    pub async fn capture_mat(&self, use_color: bool) -> Result<()> {
        self.capture_image(self.x11_connection.clone(), self.rect, use_color, |mat| {
            imwrite(
                "/home/jmgr/rust/actiona-ng/output.png",
                &mat,
                &Vector::new(),
            )?;
            Result::<()>::Ok(())
        })
        .await??;
        Ok(())
    }

    #[allow(unsafe_code)]
    async fn capture_image<F, R>(
        &self,
        x11_connection: Arc<X11Connection>,
        rect: Rect,
        use_color: bool,
        f: F,
    ) -> Result<R>
    where
        F: FnOnce(&Mat) -> R,
    {
        use x11rb_async::protocol::xproto::ConnectionExt;

        let root = x11_connection.screen().root;

        let reply = x11_connection
            .async_connection()
            .get_image(
                ImageFormat::Z_PIXMAP,
                root,
                rect.origin.x.into(),
                rect.origin.y.into(),
                rect.size.width.into(),
                rect.size.height.into(),
                u32::MAX,
            )
            .await?
            .reply()
            .await?;

        let mat = Mat::new_rows_cols_with_bytes::<opencv::core::Vec4b>(
            rect.size.height.into(),
            rect.size.width.into(),
            &reply.data,
        )?;

        if use_color {
            let mut target_mat = self.rgb_mat_buffer.lock();

            #[allow(clippy::redundant_closure_call)]
            (|| {
                opencv::opencv_has_inherent_feature_algorithm_hint! {
                    {
                        cvt_color(&mat, &mut *target_mat, COLOR_BGRA2BGR, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                    } else {
                        cvt_color(&mat, &mut *target_mat, COLOR_BGRA2BGR, 0)
                    }
                }
            })()?;

            Ok(f(&*target_mat))
        } else {
            let mut target_mat = self.greyscale_mat_buffer.lock();

            #[allow(clippy::redundant_closure_call)]
            (|| {
                opencv::opencv_has_inherent_feature_algorithm_hint! {
                    {
                        cvt_color(&mat, &mut *target_mat, COLOR_BGRA2GRAY, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                    } else {
                        cvt_color(&mat, &mut *target_mat, COLOR_BGRA2GRAY, 0)
                    }
                }
            })()?;

            Ok(f(&*target_mat))
        }
    }
}

#[derive(Debug)]
pub struct ScreenshotImpl {
    runtime: Arc<Runtime>,
    display_map: AsyncResource<HashMap<u32, Arc<Display>>>,
}

impl ScreenshotImpl {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Arc<Self>> {
        let screenshot_impl = Arc::new(Self {
            runtime: runtime.clone(),
            display_map: AsyncResource::new(runtime.cancellation_token()),
        });

        async fn wait_and_update(
            displays: &Displays,
            screenshot_impl: &ScreenshotImpl,
        ) -> Result<()> {
            let displays_info = displays.wait_get_info().await?;

            screenshot_impl.update(displays_info).await?;

            Ok(())
        }

        let local_screenshot_impl = screenshot_impl.clone();
        runtime.task_tracker().spawn(async move {
            if let Err(err) = wait_and_update(&displays, &local_screenshot_impl).await {
                error!("error getting displays info: {err}");
            }

            loop {
                if displays.changed().await.is_err() {
                    break;
                }

                if let Err(err) = wait_and_update(&displays, &local_screenshot_impl).await {
                    error!("error getting displays info: {err}");
                }
            }
        });

        Ok(screenshot_impl)
    }

    async fn update(&self, displays_info: Arc<DisplayInfoVec>) -> Result<()> {
        let mut new_display_map = HashMap::with_capacity(displays_info.len());
        for display_info in displays_info.iter() {
            new_display_map.insert(
                display_info.id,
                Arc::new(Display::new(self.runtime.clone(), display_info).await?),
            );
        }

        self.display_map.set(new_display_map);

        Ok(())
    }

    // TMP
    pub async fn capture_mat(&self, display_id: u32, use_color: bool) -> Result<()> {
        let display_map = self.display_map.wait_get().await?;
        let display = display_map
            .get(&display_id)
            .ok_or_else(|| eyre!("unknown display id: {display_id}"))?;

        display.capture_mat(use_color).await
    }
}

impl ScreenshotImplTrait for ScreenshotImpl {
    async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        let x11_connection = self.runtime.platform().x11_connection();

        // For now we use non-SHM for rects.
        // We might want to do some benchmark on start later.
        capture_get_image(x11_connection, rect).await
    }

    async fn capture_display(&self, display_id: u32) -> Result<Image> {
        let display_map = self.display_map.wait_get().await?;
        let display = display_map
            .get(&display_id)
            .ok_or_else(|| eyre!("unknown display id: {display_id}"))?;

        display.capture().await
    }

    async fn capture_pixel(&self, position: Point) -> Result<Color> {
        let result = self
            .capture_rect(rect(point(position.x, position.y), size(1, 1)))
            .await?;

        Ok((*result
            .as_rgba8()
            .expect("image should be RGBA")
            .get_pixel(0, 0))
        .into())
    }
}

fn image_from_bgr_data(data: &[u8], width: usize, height: usize) -> Result<Image> {
    let needed = width
        .checked_mul(height)
        .and_then(|pixel_count| pixel_count.checked_mul(BYTES_PER_PIXEL))
        .ok_or_else(|| eyre!("image dimensions overflow: {width}x{height}"))?;

    if data.len() < needed {
        return Err(eyre!(
            "X11 image data too small: expected {needed} bytes, got {}",
            data.len()
        ));
    }

    let width = u32::try_from(width)?;
    let height = u32::try_from(height)?;

    let mut image = RgbaImage::new(width, height);
    let image_data: &mut [u8] = image.as_mut();

    image_data.copy_from_slice(&data[..needed]);

    // Convert from BGR to RGB
    image_data.par_chunks_exact_mut(4).for_each(|c| {
        c.swap(0, 2); // Swap R and B
        c[3] = 255; // Set alpha to 255 (fully opaque)
    });

    Ok(DynamicImage::ImageRgba8(image).into())
}

async fn capture_get_image(x11_connection: Arc<X11Connection>, rect: Rect) -> Result<Image> {
    use x11rb_async::protocol::xproto::ConnectionExt;

    let root = x11_connection.screen().root;

    let reply = x11_connection
        .async_connection()
        .get_image(
            ImageFormat::Z_PIXMAP,
            root,
            rect.origin.x.into(),
            rect.origin.y.into(),
            rect.size.width.into(),
            rect.size.height.into(),
            u32::MAX,
        )
        .await?
        .reply()
        .await?;

    image_from_bgr_data(
        &reply.data,
        rect.size.width.try_into()?,
        rect.size.height.try_into()?,
    )
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use tokio::{fs, join};

    use crate::{
        core::{
            displays::Displays,
            screenshot::platform::{ScreenshotImplTrait, x11::ScreenshotImpl},
        },
        runtime::Runtime,
    };

    #[test]
    #[ignore]
    fn test_screenshot() {
        Runtime::test(async |runtime| {
            let displays =
                Displays::new(runtime.cancellation_token(), runtime.task_tracker()).unwrap();

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
            let imp2 = ScreenshotImpl::new(runtime, displays.clone())
                .await
                .unwrap();

            /*

            let displays_info = displays.wait_get_info().await.unwrap();

            for display in displays_info.iter() {
                let start = Instant::now();

                let image = imp2.capture_display(display.id).await.unwrap();

                println!("elapsed: {}", (Instant::now() - start).as_secs_f32());

                fs::create_dir_all("/tmp/test").await.unwrap();
                image
                    .save(format!("/tmp/test/test{}.bmp", display.id))
                    .unwrap();
            }
            */

            let displays_info = displays.wait_get_info().await.unwrap();
            let mut iter = displays_info.iter();
            let a = iter.next().unwrap().id;

            imp2.capture_mat(a, true).await.unwrap();

            /*
            let b = iter.next().unwrap().id;
            let c = iter.next().unwrap().id;

            let start = Instant::now();

            let (image1, image2, image3) = join!(
                imp2.capture_display(a),
                imp2.capture_display(b),
                imp2.capture_display(c)
            );

            println!("elapsed: {}", (Instant::now() - start).as_secs_f32());

            let image1 = image1.unwrap();
            let image2 = image2.unwrap();
            let image3 = image3.unwrap();

            fs::create_dir_all("/tmp/test").await.unwrap();
            image1.save("/tmp/test/test1.bmp").unwrap();
            image2.save("/tmp/test/test2.bmp").unwrap();
            image3.save("/tmp/test/test3.bmp").unwrap();
            */

            /*
            let start = Instant::now();

            let image = imp2
                .capture_rect(rect(point(1800, 0), size(300, 100)))
                .await
                .unwrap();
            println!("elapsed: {}", (Instant::now() - start).as_secs_f32());


            image.save("/tmp/test/test.bmp").unwrap();
            */
        });
    }
}
