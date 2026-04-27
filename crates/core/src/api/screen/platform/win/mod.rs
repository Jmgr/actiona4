use std::sync::Arc;

use color_eyre::Result;
use screenshot::Capture;

use super::{DisplayCapture, ScreenImplBase, blacken_non_display_areas};
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

pub mod ask_screenshot;

#[derive(Debug)]
pub struct WindowsDisplay {
    rect: Rect,
    capture_screen: screenshot::Screen,
}

impl DisplayCapture for WindowsDisplay {
    async fn new(
        _runtime: Arc<Runtime>,
        capture_screen: screenshot::Screen,
        display_info: &DisplayInfo,
    ) -> Result<Self> {
        Ok(Self {
            rect: display_info.rect,
            capture_screen,
        })
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    async fn capture_raw(&self) -> Result<Capture> {
        self.capture_screen.capture_rect(self.rect).await
    }
}

/// Windows screen implementation.
pub type ScreenImpl = ScreenImplBase<WindowsDisplay>;

impl ScreenImpl {
    pub async fn new(
        runtime: Arc<Runtime>,
        displays: crate::api::displays::Displays,
    ) -> Result<Arc<Self>> {
        let capture_screen =
            screenshot::Screen::new(runtime.task_tracker(), runtime.cancellation_token()).await?;
        ScreenImplBase::<WindowsDisplay>::from_capture_screen(runtime, displays, capture_screen)
            .await
    }

    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        let capture = self.capture_screen().capture_rect(rect).await?;
        Image::from_capture(capture)
    }

    pub async fn capture_rect_to_source(&self, rect: Rect) -> Result<Arc<Source>> {
        let capture = self.capture_screen().capture_rect(rect).await?;
        Source::from_bgra(&capture.bgra, capture.width, capture.height)
    }

    pub async fn capture_pixel(&self, position: Point) -> Result<Color> {
        let image = self
            .capture_rect(rect(point(position.x, position.y), size(1, 1)))
            .await?;
        Ok((*image.as_rgba8().get_pixel(0, 0)).into())
    }

    async fn capture_desktop_impl(&self) -> Result<(Image, Rect)> {
        let rect = self.desktop_rect().await?;
        let display_rects = self.display_rects().await?;
        let mut image = self.capture_rect(rect).await?;
        blacken_non_display_areas(image.as_mut(), rect, &display_rects);
        Ok((image, rect))
    }

    pub async fn capture_desktop(&self) -> Result<Image> {
        let (image, _rect) = self.capture_desktop_impl().await?;
        Ok(image)
    }

    pub async fn capture_desktop_to_source(&self) -> Result<(Arc<Source>, Rect)> {
        let (image, rect) = self.capture_desktop_impl().await?;
        let source = Arc::<Source>::try_from(&image)?;
        Ok((source, rect))
    }
}
