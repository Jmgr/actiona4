use std::{collections::HashMap, fmt::Debug, sync::Arc};

use color_eyre::{Result, eyre::eyre};
use tracing::error;

use super::display_selector::{DisplayName, DisplaySelector};
use crate::{
    api::{
        color::Color,
        displays::Displays,
        image::{DrawImageOptions, Image, find_image::Source},
        point::point,
        rect::Rect,
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
    displays: Displays,
    display_map: AsyncResource<HashMap<u32, Arc<D>>>,
}

impl<D: DisplayCapture> ScreenshotImplBase<D> {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Arc<Self>> {
        let screenshot_impl = Arc::new(Self {
            runtime: runtime.clone(),
            displays: displays.clone(),
            display_map: AsyncResource::new(runtime.cancellation_token()),
        });

        let local_screenshot_impl = screenshot_impl.clone();
        runtime.task_tracker().spawn(async move {
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

    /// Resolves a `DisplaySelector` to a display ID.
    ///
    /// Returns an error for `DisplaySelector::Desktop` since it does not map
    /// to a single display; use `desktop_rect` for that case instead.
    pub async fn resolve_display_selector(&self, selector: &DisplaySelector) -> Result<u32> {
        match selector {
            DisplaySelector::Desktop => {
                Err(eyre!("Desktop selector does not map to a single display"))
            }
            DisplaySelector::ById(id) => Ok(*id),
            DisplaySelector::Primary => {
                let info = self.displays.primary().await?;
                Ok(info.id)
            }
            DisplaySelector::Largest => {
                let info = self
                    .displays
                    .largest()
                    .await?
                    .ok_or_else(|| eyre!("no displays found"))?;
                Ok(info.id)
            }
            DisplaySelector::Smallest => {
                let info = self
                    .displays
                    .smallest()
                    .await?
                    .ok_or_else(|| eyre!("no displays found"))?;
                Ok(info.id)
            }
            DisplaySelector::ByName(name) => {
                let displays_info = self.displays.wait_get_info().await?;
                let mut matching: Vec<_> = displays_info
                    .iter()
                    .filter(|d| name.matches(&d.friendly_name))
                    .collect();
                match matching.len() {
                    0 => Err(match name {
                        DisplayName::Literal(s) => eyre!("display not found: {s}"),
                        DisplayName::Wildcard(w) => {
                            eyre!("no display found matching: {}", w.to_string_js())
                        }
                    }),
                    1 => Ok(matching.remove(0).id),
                    n => Err(match name {
                        DisplayName::Literal(s) => eyre!(
                            "{n} displays match the name \"{s}\"; use Display.fromId() to select by ID"
                        ),
                        DisplayName::Wildcard(w) => eyre!(
                            "{n} displays match the pattern \"{}\"; use a more specific pattern",
                            w.to_string_js()
                        ),
                    }),
                }
            }
            DisplaySelector::FromPoint(point) => {
                let info = self
                    .displays
                    .from_point(*point)
                    .await?
                    .ok_or_else(|| eyre!("no display found at point {point}"))?;
                Ok(info.id)
            }
        }
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
    /// Returns the Source and the display's rectangle (for coordinate offset).
    pub async fn capture_display_to_source(&self, display_id: u32) -> Result<(Arc<Source>, Rect)> {
        let display = self.get_display(display_id).await?;
        let rect = display.rect();
        let data = display.capture_raw().await?;
        let source = Source::from_bgra(&data, rect.size.width.into(), rect.size.height.into())?;
        Ok((source, rect))
    }
}
