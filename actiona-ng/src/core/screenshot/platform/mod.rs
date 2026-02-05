use std::{collections::HashMap, fmt::Debug, sync::Arc};

use color_eyre::{Result, eyre::eyre};
use tracing::error;

use crate::{
    core::{
        color::Color,
        displays::Displays,
        image::{Image, find_image::Source},
        point::{Point, point},
        rect::{Rect, rect},
        size::size,
    },
    runtime::{Runtime, async_resource::AsyncResource, events::DisplayInfo},
};

mod convert;

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

/// Trait for platform-specific display capture implementations.
pub trait DisplayCapture: Debug + Send + Sync + 'static {
    /// Create a new display capture instance for the given display.
    fn new(
        runtime: Arc<Runtime>,
        display_info: &DisplayInfo,
    ) -> impl Future<Output = Result<Self>> + Send
    where
        Self: Sized;

    /// Get the display's rectangle.
    fn rect(&self) -> Rect;

    /// Capture the display and return raw BGRA pixel data.
    fn capture_raw(&self) -> impl Future<Output = Result<Vec<u8>>> + Send;
}

/// Generic screenshot implementation that works with any `DisplayCapture` backend.
#[derive(Debug)]
pub struct ScreenshotImplBase<D: DisplayCapture> {
    runtime: Arc<Runtime>,
    display_map: AsyncResource<HashMap<u32, Arc<D>>>,
}

impl<D: DisplayCapture> ScreenshotImplBase<D> {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Arc<Self>> {
        let screenshot_impl = Arc::new(Self {
            runtime: runtime.clone(),
            display_map: AsyncResource::new(runtime.cancellation_token()),
        });

        let local_screenshot_impl = screenshot_impl.clone();
        runtime.task_tracker().spawn(async move {
            if let Err(err) = local_screenshot_impl.wait_and_update(&displays).await {
                error!("error getting displays info: {err}");
            }

            loop {
                if displays.changed().await.is_err() {
                    break;
                }

                if let Err(err) = local_screenshot_impl.wait_and_update(&displays).await {
                    error!("error getting displays info: {err}");
                }
            }
        });

        Ok(screenshot_impl)
    }

    async fn wait_and_update(&self, displays: &Displays) -> Result<()> {
        let displays_info = displays.wait_get_info().await?;
        self.update_displays(&displays_info).await?;
        Ok(())
    }

    async fn update_displays(&self, displays_info: &[DisplayInfo]) -> Result<()> {
        let mut new_display_map = HashMap::with_capacity(displays_info.len());
        for display_info in displays_info {
            new_display_map.insert(
                display_info.id,
                Arc::new(D::new(self.runtime.clone(), display_info).await?),
            );
        }

        self.display_map.set(new_display_map);

        Ok(())
    }

    async fn get_display(&self, display_id: u32) -> Result<Arc<D>> {
        let display_map = self.display_map.wait_get().await?;
        display_map
            .get(&display_id)
            .cloned()
            .ok_or_else(|| eyre!("unknown display id: {display_id}"))
    }

    /// Get the runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Capture a display and return an Image.
    pub async fn capture_display(&self, display_id: u32) -> Result<Image> {
        let display = self.get_display(display_id).await?;
        let rect = display.rect();
        let data = display.capture_raw().await?;
        Image::from_bgra(&data, rect.size.width.into(), rect.size.height.into())
    }

    /// Capture a display directly to a Source for find_image_on_screen.
    /// This avoids the intermediate RGBA conversion.
    pub async fn capture_display_to_source(&self, display_id: u32) -> Result<Arc<Source>> {
        let display = self.get_display(display_id).await?;
        let rect = display.rect();
        let data = display.capture_raw().await?;
        Source::from_bgra(&data, rect.size.width.into(), rect.size.height.into())
    }

    /// Capture a single pixel.
    pub async fn capture_pixel(&self, position: Point) -> Result<Color>
    where
        Self: ScreenshotImplTrait,
    {
        let result = self
            .capture_rect(rect(point(position.x, position.y), size(1, 1)))
            .await?;

        Ok((*result.as_rgba8().get_pixel(0, 0)).into())
    }
}

pub trait ScreenshotImplTrait {
    fn capture_rect(&self, rect: Rect) -> impl Future<Output = Result<Image>> + Send;
    fn capture_display(&self, display_id: u32) -> impl Future<Output = Result<Image>> + Send;
    fn capture_pixel(&self, position: Point) -> impl Future<Output = Result<Color>> + Send;
}
