use std::sync::Arc;

use color_eyre::Result;

use self::capture::capture_rect as capture_rect_raw;
use super::{
    DisplayCapture, ScreenshotImplBase, ScreenshotImplTrait,
    convert::{bgra_to_rgba_image, bgra_to_source},
};
use crate::{
    core::{
        color::Color,
        image::{Image, find_image::Source},
        point::Point,
        rect::Rect,
    },
    runtime::{Runtime, events::DisplayInfo},
};

mod capture;

#[derive(Debug)]
pub struct WindowsDisplay {
    rect: Rect,
}

impl DisplayCapture for WindowsDisplay {
    async fn new(_runtime: Arc<Runtime>, display_info: &DisplayInfo) -> Result<Self> {
        Ok(Self {
            rect: display_info.rect,
        })
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    async fn capture_raw(&self) -> Result<Vec<u8>> {
        capture_rect_raw(self.rect)
    }
}

/// Windows screenshot implementation.
pub type ScreenshotImpl = ScreenshotImplBase<WindowsDisplay>;

impl ScreenshotImpl {
    /// Capture a rect directly to a Source for find_image.
    pub async fn capture_rect_to_source(&self, rect: Rect) -> Result<Arc<Source>> {
        let data = capture_rect_raw(rect)?;
        bgra_to_source(&data, rect.size.width.into(), rect.size.height.into())
    }
}

impl ScreenshotImplTrait for ScreenshotImpl {
    async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        let data = capture_rect_raw(rect)?;
        bgra_to_rgba_image(&data, rect.size.width.into(), rect.size.height.into())
    }

    async fn capture_display(&self, display_id: u32) -> Result<Image> {
        ScreenshotImplBase::capture_display(self, display_id).await
    }

    async fn capture_pixel(&self, position: Point) -> Result<Color> {
        ScreenshotImplBase::capture_pixel(self, position).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{
        core::{
            displays::Displays,
            point::point,
            rect::rect,
            screenshot::platform::{ScreenshotImplBase, ScreenshotImplTrait, win::WindowsDisplay},
            size::size,
        },
        runtime::Runtime,
    };

    #[test]
    #[ignore]
    fn test_screenshot() {
        Runtime::test(async |runtime| {
            let displays =
                Displays::new(runtime.cancellation_token(), runtime.task_tracker()).unwrap();

            let impl_ = ScreenshotImplBase::<WindowsDisplay>::new(runtime, displays.clone())
                .await
                .unwrap();

            let displays_info = displays.wait_get_info().await.unwrap();
            let display_id = displays_info.first().unwrap().id;

            // Test capture to image
            let _image = impl_.capture_display(display_id).await.unwrap();

            // Test capture to source
            let _source = impl_.capture_display_to_source(display_id).await.unwrap();
        });
    }

    #[test]
    #[ignore]
    fn test_screenshot_rect() {
        Runtime::test(async |runtime| {
            let displays =
                Displays::new(runtime.cancellation_token(), runtime.task_tracker()).unwrap();

            let imp = ScreenshotImplBase::<WindowsDisplay>::new(runtime, displays)
                .await
                .unwrap();

            let start = Instant::now();

            let image = imp
                .capture_rect(rect(point(0, 0), size(1920, 1080)))
                .await
                .unwrap();

            println!("elapsed: {}", (Instant::now() - start).as_secs_f32());

            image.save("C:/Users/jmgr/Pictures/test_win.bmp").unwrap();
        });
    }
}
