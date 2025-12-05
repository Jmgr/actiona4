use color_eyre::Result;

use crate::core::{color::Color, image::Image, point::Point, rect::Rect};

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub trait ScreenshotImplTrait {
    async fn capture_rect(&self, rect: Rect) -> Result<Image>;
    async fn capture_display(&self, display_id: u32) -> Result<Image>;
    async fn capture_pixel(&self, position: Point) -> Result<Color>;
}
