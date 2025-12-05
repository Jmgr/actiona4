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
use crate::runtime::Runtime;

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

    pub async fn capture_rect(&mut self, rect: Rect) -> Result<Image> {
        self.implementation.capture_rect(rect).await
    }
}
