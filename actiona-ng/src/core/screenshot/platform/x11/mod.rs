use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};

use self::capture::{ShmCapture, get_image};
use super::{DisplayCapture, ScreenshotImplBase};
use crate::{
    core::{
        color::Color,
        image::{Image, find_image::Source},
        point::{Point, point},
        rect::{Rect, rect},
        size::size,
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
    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        let x11_connection = self.runtime.platform().x11_connection();
        let data = get_image(x11_connection, rect).await?;
        Image::from_bgra(&data, rect.size.width.into(), rect.size.height.into())
    }

    pub async fn capture_rect_to_source(&self, rect: Rect) -> Result<Arc<Source>> {
        let x11_connection = self.runtime.platform().x11_connection();
        let data = get_image(x11_connection, rect).await?;
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
            let (_source, _rect) = impl_.capture_display_to_source(display_id).await.unwrap();
        });
    }
}
