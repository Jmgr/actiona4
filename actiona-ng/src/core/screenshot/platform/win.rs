use std::{ffi::c_void, sync::Arc};

use eyre::Result;
use image::{DynamicImage, RgbaImage};
use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, RGBQUAD, ReleaseDC, SRCCOPY,
    SelectObject,
};

use super::ScreenshotImplTrait;
use crate::{
    core::{color::Color, displays::Displays, image::Image, point::Point, rect::Rect},
    runtime::Runtime,
};

#[derive(Debug)]
pub struct ScreenshotImpl {}

impl ScreenshotImpl {
    pub async fn new(_runtime: Arc<Runtime>, _displays: Arc<Displays>) -> Result<Self> {
        Ok(ScreenshotImpl {})
    }
}

#[allow(unsafe_code)]
impl ScreenshotImplTrait for ScreenshotImpl {
    async fn capture_rect(&mut self, rect: Rect) -> Result<Image> {
        let hdc_screen = unsafe { GetDC(None) };
        let hdc_mem = unsafe { CreateCompatibleDC(Some(hdc_screen)) };

        let hbm =
            unsafe { CreateCompatibleBitmap(hdc_screen, rect.width as i32, rect.height as i32) };
        unsafe { SelectObject(hdc_mem, hbm.into()) };

        unsafe {
            BitBlt(
                hdc_mem,
                rect.x,
                rect.y,
                rect.width as i32,
                rect.height as i32,
                Some(hdc_screen),
                0,
                0,
                SRCCOPY,
            )
            .unwrap();
        }

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: rect.width as i32,
                biHeight: -(rect.height as i32), // Top-down bitmap
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };

        let mut image = RgbaImage::new(rect.width, rect.height);
        let image_data: &mut [u8] = image.as_mut();
        let image_data_ptr = image_data.as_mut_ptr() as *mut c_void;

        unsafe {
            GetDIBits(
                hdc_mem,
                hbm,
                0,
                rect.height,
                Some(image_data_ptr as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            )
        };

        // Swap from BGR to RGB
        image_data
            .par_chunks_exact_mut(4)
            .for_each(|c| c.swap(0, 2));

        unsafe { ReleaseDC(None, hdc_screen) };
        unsafe {
            let _ = DeleteDC(hdc_mem);
        };
        unsafe {
            let _ = DeleteObject(hbm.into());
        };

        Ok(DynamicImage::ImageRgba8(image).into())
    }

    async fn _capture_display(&mut self, display_id: u32) -> Result<Image> {
        let _ = display_id;
        todo!();
    }

    async fn _capture_pixel(&mut self, position: Point) -> Result<Color> {
        let _ = position;
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Instant};

    use crate::{
        core::{
            displays::Displays,
            rect::rect,
            screenshot::platform::{ScreenshotImplTrait, win::ScreenshotImpl},
        },
        runtime::Runtime,
    };

    #[test]
    fn test_screenshot() {
        Runtime::test(async |runtime| {
            let displays = Displays::new(runtime.clone()).unwrap();

            /*
            X11:
            SHM

            elapsed: 0.021464119
            elapsed: 0.010506714
            elapsed: 0.007166257

            getimage

            elapsed: 0.07405177
            elapsed: 0.014961153
            elapsed: 0.00688114

            Windows:
            blitblt

            elapsed: 0.11794569 --- 0.1954363

             */

            /*
            let displays_info = {
                let displays_info = displays.displays_info().lock().unwrap();
                displays_info.clone()
            };
            */

            let displays = Arc::new(displays);

            let mut imp = ScreenshotImpl::new(runtime, displays).await.unwrap();

            let start = Instant::now();

            let image = imp.capture_rect(rect(0, 0, 0, 0)).await.unwrap();

            println!("elapsed: {}", (Instant::now() - start).as_secs_f32());

            image.save("C:/Users/jmgr/Pictures/test_win.bmp").unwrap();
        });
    }
}
