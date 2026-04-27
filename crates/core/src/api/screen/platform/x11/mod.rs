use std::sync::Arc;

use color_eyre::Result;
use screenshot::{Capture, ShmSegment};
use tracing::{error, warn};

use super::{DisplayCapture, ScreenImplBase, blacken_non_display_areas};
use crate::{
    api::{
        color::Color,
        displays::Displays,
        image::{Image, find_image::Source},
        point::{Point, point},
        rect::{Rect, rect},
        size::size,
    },
    runtime::{Runtime, async_resource::AsyncResource, events::DisplayInfo},
};

pub mod portal;

#[derive(Debug)]
pub struct X11Display {
    rect: Rect,
    capture_screen: screenshot::Screen,
    shm: Option<ShmSegment>,
}

impl DisplayCapture for X11Display {
    async fn new(
        runtime: Arc<Runtime>,
        capture_screen: screenshot::Screen,
        display_info: &DisplayInfo,
    ) -> Result<Self> {
        let rect = display_info.rect;

        if capture_screen.root_depth() != 24 {
            return Err(color_eyre::eyre::eyre!(
                "unsupported X11 depth: {}",
                capture_screen.root_depth()
            ));
        }

        let shm = if runtime.platform().has_shm() {
            let capacity = ShmSegment::capacity_for_rect(rect);
            Some(ShmSegment::new(&capture_screen, capacity).await?)
        } else {
            None
        };

        Ok(Self {
            rect,
            capture_screen,
            shm,
        })
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    async fn capture_raw(&self) -> Result<Capture> {
        if let Some(shm) = &self.shm {
            shm.capture_rect(&self.capture_screen, self.rect).await
        } else {
            self.capture_screen.capture_rect(self.rect).await
        }
    }
}

/// X11 screenshot implementation with pre-allocated SHM for desktop capture.
#[derive(Debug)]
pub struct ScreenImpl {
    base: Arc<ScreenImplBase<X11Display>>,
    /// Pre-allocated SHM segment for full-desktop capture.
    desktop_shm: AsyncResource<(Rect, ShmSegment)>,
    runtime: Arc<Runtime>,
}

impl ScreenImpl {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Arc<Self>> {
        let capture_screen =
            screenshot::Screen::new(runtime.task_tracker(), runtime.cancellation_token()).await?;
        let base = ScreenImplBase::<X11Display>::from_capture_screen(
            runtime.clone(),
            displays.clone(),
            capture_screen,
        )
        .await?;

        let desktop_shm = AsyncResource::new(runtime.cancellation_token());

        let impl_ = Arc::new(Self {
            base,
            desktop_shm,
            runtime: runtime.clone(),
        });

        // Watch for display changes and re-allocate desktop SHM once it has been created.
        {
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
        let rect = self.base.desktop_rect().await?;
        let capacity = ShmSegment::capacity_for_rect(rect);
        let shm = ShmSegment::new(self.base.capture_screen(), capacity).await?;
        self.desktop_shm.set((rect, shm));
        Ok(())
    }

    pub async fn capture_display(&self, display_id: u32) -> Result<Image> {
        self.base.capture_display(display_id).await
    }

    pub async fn capture_display_to_source(&self, display_id: u32) -> Result<(Arc<Source>, Rect)> {
        self.base.capture_display_to_source(display_id).await
    }

    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        let capture = self.base.capture_screen().capture_rect(rect).await?;
        Image::from_capture(capture)
    }

    pub async fn capture_rect_to_source(&self, rect: Rect) -> Result<Arc<Source>> {
        let capture = self.base.capture_screen().capture_rect(rect).await?;
        Source::from_bgra(&capture.bgra, capture.width, capture.height)
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

        if self.runtime.platform().has_shm()
            && self.desktop_shm.try_get().is_none()
            && let Err(err) = self.update_desktop_shm().await
        {
            error!("Failed to lazily initialize desktop SHM: {err}");
        }

        let mut image = if let Some(shm_data) = self.desktop_shm.try_get() {
            let (shm_rect, shm) = &*shm_data;
            // The pre-allocated SHM is usable as long as the buffer is large enough.
            if ShmSegment::capacity_for_rect(rect) <= ShmSegment::capacity_for_rect(*shm_rect) {
                let capture = shm.capture_rect(self.base.capture_screen(), rect).await?;
                Image::from_capture(capture)?
            } else {
                warn!(
                    "Desktop SHM too small for current rect {:?} (allocated for {:?}), \
                     falling back to XGetImage",
                    rect, shm_rect
                );
                self.capture_rect(rect).await?
            }
        } else {
            self.capture_rect(rect).await?
        };

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
