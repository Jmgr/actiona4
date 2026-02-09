use std::sync::Arc;

use color_eyre::Result;

use self::capture::capture_rect as capture_rect_raw;
use super::{DisplayCapture, ScreenshotImplBase};
use crate::{
    api::{
        color::Color,
        image::{Image, find_image::Source},
        point::{Point, point},
        rect::{Rect, rect},
        size::size,
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
    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        let data = capture_rect_raw(rect)?;
        Image::from_bgra(&data, rect.size.width.into(), rect.size.height.into())
    }

    pub async fn capture_rect_to_source(&self, rect: Rect) -> Result<Arc<Source>> {
        let data = capture_rect_raw(rect)?;
        Source::from_bgra(&data, rect.size.width.into(), rect.size.height.into())
    }

    pub async fn capture_pixel(&self, position: Point) -> Result<Color> {
        let image = self
            .capture_rect(rect(point(position.x, position.y), size(1, 1)))
            .await?;
        Ok((*image.as_rgba8().get_pixel(0, 0)).into())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{
        api::{
            displays::Displays,
            point::point,
            rect::rect,
            screenshot::platform::{ScreenshotImplBase, win::WindowsDisplay},
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
            let (_source, _rect) = impl_.capture_display_to_source(display_id).await.unwrap();
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

            image
                .save("C:/Users/jmgr/Pictures/test_win.bmp")
                .await
                .unwrap();
        });
    }
}
