use std::sync::Arc;

use color_eyre::Result;
use tracing::{error, warn};

use self::capture::{ShmCapture, get_image};
use super::{DisplayCapture, ScreenshotImplBase, blacken_non_display_areas};
use crate::{
    api::{
        color::Color,
        displays::Displays,
        image::{Image, find_image::Source},
        point::{Point, point},
        rect::{Rect, rect},
        screenshot::display_selector::DisplaySelector,
        size::size,
    },
    platform::x11::X11Connection,
    runtime::{Runtime, async_resource::AsyncResource, events::DisplayInfo},
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
            return Err(color_eyre::eyre::eyre!(
                "unsupported X11 depth: {}",
                root_depth
            ));
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

/// X11 screenshot implementation with pre-allocated SHM for desktop capture.
#[derive(Debug)]
pub struct ScreenshotImpl {
    base: Arc<ScreenshotImplBase<X11Display>>,
    /// Pre-allocated SHM segment for full-desktop capture.
    /// Stores the rect it was sized for alongside the capture object.
    desktop_shm: AsyncResource<(Rect, ShmCapture)>,
    runtime: Arc<Runtime>,
}

impl ScreenshotImpl {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Arc<Self>> {
        let base = ScreenshotImplBase::<X11Display>::new(runtime.clone(), displays.clone()).await?;

        let desktop_shm = AsyncResource::new(runtime.cancellation_token());

        let impl_ = Arc::new(Self {
            base,
            desktop_shm,
            runtime: runtime.clone(),
        });

        // Initialize the desktop SHM immediately and watch for display changes.
        {
            let local_impl = impl_.clone();
            if let Err(err) = local_impl.update_desktop_shm().await {
                error!("Failed to initialize desktop SHM: {err}");
            }

            let local_impl = impl_.clone();
            runtime.task_tracker().spawn(async move {
                loop {
                    if displays.changed().await.is_err() {
                        break;
                    }
                    if let Err(err) = local_impl.update_desktop_shm().await {
                        error!("Failed to update desktop SHM after display change: {err}");
                    }
                }
            });
        }

        Ok(impl_)
    }

    /// Allocates (or re-allocates) the desktop SHM segment to match the current desktop rect.
    async fn update_desktop_shm(&self) -> Result<()> {
        if !self.runtime.platform().has_shm() {
            return Ok(());
        }
        let x11_connection = self.runtime.platform().x11_connection();
        let rect = self.base.desktop_rect().await?;
        let size = ShmCapture::buffer_size_for_rect(rect);
        let shm = ShmCapture::new(&x11_connection, size).await?;
        self.desktop_shm.set((rect, shm));
        Ok(())
    }

    pub async fn resolve_display_selector(&self, selector: &DisplaySelector) -> Result<u32> {
        self.base.resolve_display_selector(selector).await
    }

    pub async fn capture_display(&self, display_id: u32) -> Result<Image> {
        self.base.capture_display(display_id).await
    }

    pub async fn capture_display_to_source(&self, display_id: u32) -> Result<(Arc<Source>, Rect)> {
        self.base.capture_display_to_source(display_id).await
    }

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

    async fn capture_desktop_impl(&self) -> Result<(Image, Rect)> {
        let rect = self.base.desktop_rect().await?;
        let display_rects = self.base.display_rects().await?;

        let mut image = if let Some(shm_data) = self.desktop_shm.try_get() {
            let (shm_rect, shm) = &*shm_data;
            // The pre-allocated SHM is usable as long as the buffer is large enough.
            if ShmCapture::buffer_size_for_rect(rect) <= ShmCapture::buffer_size_for_rect(*shm_rect)
            {
                let x11_connection = self.runtime.platform().x11_connection();
                let data = shm.capture(&x11_connection, rect).await?;
                Image::from_bgra(data, rect.size.width.into(), rect.size.height.into())?
            } else {
                // The desktop rect grew after the last SHM allocation.  This
                // should only happen during the brief window between a display
                // change event and the background task completing a re-allocation.
                warn!(
                    "Desktop SHM too small for current rect {:?} (allocated for {:?}), \
                     falling back to XGetImage",
                    rect, shm_rect
                );
                self.capture_rect(rect).await?
            }
        } else {
            // SHM not yet allocated — fall back to XGetImage.
            self.capture_rect(rect).await?
        };

        blacken_non_display_areas(&mut image, rect, &display_rects);
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

#[cfg(test)]
mod tests {
    use crate::{
        api::{
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
