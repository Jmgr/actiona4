use std::{collections::HashMap, fmt::Debug, sync::Arc};

use color_eyre::{Result, eyre::eyre};
use screenshot::Capture;
use tracing::error;

use crate::{
    api::{
        displays::Displays,
        image::{Image, find_image::Source},
        rect::Rect,
    },
    runtime::{Runtime, async_resource::AsyncResource, events::DisplayInfo},
};

mod convert;
pub mod overlay;

#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub use screenshot::blacken_non_display_areas;

/// Trait for platform-specific display capture implementations.
pub trait DisplayCapture: Debug + Send + Sync + 'static {
    /// Create a new display capture instance for the given display.
    fn new(
        runtime: Arc<Runtime>,
        capture_screen: screenshot::Screen,
        display_info: &DisplayInfo,
    ) -> impl Future<Output = Result<Self>> + Send
    where
        Self: Sized;

    /// Get the display's rectangle.
    fn rect(&self) -> Rect;

    /// Capture the display and return its raw BGRA pixel buffer.
    fn capture_raw(&self) -> impl Future<Output = Result<Capture>> + Send;
}

/// Generic screenshot implementation that works with any `DisplayCapture` backend.
#[derive(Debug)]
pub struct ScreenImplBase<D: DisplayCapture> {
    runtime: Arc<Runtime>,
    displays: Displays,
    capture_screen: screenshot::Screen,
    display_map: AsyncResource<HashMap<u32, Arc<D>>>,
}

impl<D: DisplayCapture> ScreenImplBase<D> {
    pub async fn from_capture_screen(
        runtime: Arc<Runtime>,
        displays: Displays,
        capture_screen: screenshot::Screen,
    ) -> Result<Arc<Self>> {
        let screen_impl = Arc::new(Self {
            runtime: runtime.clone(),
            displays: displays.clone(),
            capture_screen,
            display_map: AsyncResource::new(runtime.cancellation_token()),
        });

        let local_screen_impl = screen_impl.clone();
        runtime.task_tracker().spawn(async move {
            loop {
                if displays.changed().await.is_err() {
                    break;
                }

                if let Err(err) = local_screen_impl.wait_and_update(&displays).await {
                    error!("error getting displays info: {err}");
                }
            }
        });

        Ok(screen_impl)
    }

    pub const fn capture_screen(&self) -> &screenshot::Screen {
        &self.capture_screen
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
                Arc::new(
                    D::new(
                        self.runtime.clone(),
                        self.capture_screen.clone(),
                        display_info,
                    )
                    .await?,
                ),
            );
        }

        self.display_map.set(new_display_map);

        Ok(())
    }

    async fn get_display(&self, display_id: u32) -> Result<Arc<D>> {
        if self.display_map.try_get().is_none()
            && let Err(err) = self.wait_and_update(&self.displays).await
        {
            error!("error getting displays info: {err}");
        }
        let display_map = self.display_map.wait_get().await?;
        display_map
            .get(&display_id)
            .cloned()
            .ok_or_else(|| eyre!("unknown display id: {display_id}"))
    }

    /// Computes the bounding rectangle of all connected displays (the entire desktop).
    pub async fn desktop_rect(&self) -> Result<Rect> {
        let displays_info = self.displays.wait_get_info().await?;
        let mut iter = displays_info.iter();
        let first = iter.next().ok_or_else(|| eyre!("no displays found"))?;
        Ok(iter.fold(first.rect, |acc, info| acc.union(info.rect)))
    }

    /// Returns the bounding rectangle of each connected display.
    pub async fn display_rects(&self) -> Result<Vec<Rect>> {
        let displays_info = self.displays.wait_get_info().await?;
        Ok(displays_info.iter().map(|d| d.rect).collect())
    }

    /// Capture a display and return an Image.
    pub async fn capture_display(&self, display_id: u32) -> Result<Image> {
        let display = self.get_display(display_id).await?;
        let capture = display.capture_raw().await?;
        Image::from_capture(capture)
    }

    /// Capture a display directly to a Source for find_image_on_screen.
    /// This avoids the intermediate RGBA conversion.
    /// Returns the Source and the display's rectangle (for coordinate offset).
    pub async fn capture_display_to_source(&self, display_id: u32) -> Result<(Arc<Source>, Rect)> {
        let display = self.get_display(display_id).await?;
        let rect = display.rect();
        let capture = display.capture_raw().await?;
        let source = Source::from_bgra(&capture.bgra, capture.width, capture.height)?;
        Ok((source, rect))
    }
}
