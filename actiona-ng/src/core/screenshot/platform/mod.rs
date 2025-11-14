use color_eyre::Result;

use crate::core::{color::Color, image::Image, point::Point, rect::Rect};

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub trait ScreenshotImplTrait {
    async fn capture_rect(&mut self, rect: Rect) -> Result<Image>;
    async fn _capture_display(&mut self, display_id: u32) -> Result<Image>;
    async fn _capture_pixel(&mut self, position: Point) -> Result<Color>;
}
