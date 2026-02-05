use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};

use self::capture::{ShmCapture, get_image};
use super::{DisplayCapture, ScreenshotImplBase, ScreenshotImplTrait};
use crate::{
    core::{
        color::Color,
        image::{Image, find_image::Source},
        point::Point,
        rect::Rect,
    },
    platform::x11::X11Connection,
    runtime::{Runtime, events::DisplayInfo},
};

mod capture;

#[derive(Debug)]
pub struct X11Display {
    rect: Rect,
    x11_connection: Arc<X11Connection>,
    shm: Option<ShmCapture>,
}

impl DisplayCapture for X11Display {
    async fn new(runtime: Arc<Runtime>, display_info: &DisplayInfo) -> Result<Self> {
        let rect = display_info.rect;
        let x11_connection = runtime.platform().x11_connection();

        let root_depth = x11_connection.screen().root_depth;
        if root_depth != 24 {
            return Err(eyre!("unsupported X11 depth: {}", root_depth));
        }

        let shm = if runtime.platform().has_shm() {
            let size = ShmCapture::buffer_size_for_rect(rect);
            Some(ShmCapture::new(&x11_connection, size).await?)
        } else {
            None
        };

        Ok(Self {
            rect,
            x11_connection,
            shm,
        })
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    async fn capture_raw(&self) -> Result<Vec<u8>> {
        if let Some(shm) = &self.shm {
            let data = shm.capture(&self.x11_connection, self.rect).await?;
            Ok(data.to_vec())
        } else {
            get_image(self.x11_connection.clone(), self.rect).await
        }
    }
}

/// X11 screenshot implementation.
pub type ScreenshotImpl = ScreenshotImplBase<X11Display>;

impl ScreenshotImpl {
    /// Capture a rect directly to a Source for find_image.
    pub async fn capture_rect_to_source(&self, rect: Rect) -> Result<Arc<Source>> {
        let x11_connection = self.runtime().platform().x11_connection();
        let data = get_image(x11_connection, rect).await?;
        Source::from_bgra(&data, rect.size.width.into(), rect.size.height.into())
    }
}

impl ScreenshotImplTrait for ScreenshotImpl {
    async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        let x11_connection = self.runtime().platform().x11_connection();
        let data = get_image(x11_connection, rect).await?;
        Image::from_bgra(&data, rect.size.width.into(), rect.size.height.into())
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
    use crate::{
        core::{
            displays::Displays,
            screenshot::platform::{ScreenshotImplBase, x11::X11Display},
        },
        runtime::Runtime,
    };

    #[test]
    #[ignore]
    fn test_screenshot() {
        Runtime::test(async |runtime| {
            let displays =
                Displays::new(runtime.cancellation_token(), runtime.task_tracker()).unwrap();

            let impl_ = ScreenshotImplBase::<X11Display>::new(runtime, displays.clone())
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
}
