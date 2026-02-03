use std::sync::Arc;

use color_eyre::Result;

mod platform;

pub mod js;

use platform::ScreenshotImplTrait;
#[cfg(windows)]
use platform::win::ScreenshotImpl;
#[cfg(unix)]
use platform::x11::ScreenshotImpl;

use super::{displays::Displays, image::Image, rect::Rect};
use crate::{
    core::{color::Color, point::Point},
    runtime::Runtime,
};

#[derive(Debug)]
pub struct Screenshot {
    implementation: Arc<ScreenshotImpl>,
}

impl Screenshot {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Self> {
        Ok(Self {
            implementation: ScreenshotImpl::new(runtime, displays).await?,
        })
    }

    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        self.implementation.capture_rect(rect).await
    }

    pub async fn capture_display(&self, display_id: u32) -> Result<Image> {
        self.implementation.capture_display(display_id).await
    }

    pub async fn capture_pixel(&self, position: Point) -> Result<Color> {
        self.implementation.capture_pixel(position).await
    }
}
