use std::{collections::HashMap, fmt::Debug, sync::Arc};

use color_eyre::{Result, eyre::eyre};
use tracing::error;

use crate::{
    api::{
        displays::Displays,
        image::{Image, find_image::Source},
        rect::Rect,
    },
    runtime::{Runtime, async_resource::AsyncResource, events::DisplayInfo},
    types::su32::{Su32, su32},
};

mod convert;
pub mod overlay;

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
pub struct ScreenImplBase<D: DisplayCapture> {
    runtime: Arc<Runtime>,
    displays: Displays,
    display_map: AsyncResource<HashMap<u32, Arc<D>>>,
}

impl<D: DisplayCapture> ScreenImplBase<D> {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Arc<Self>> {
        let screen_impl = Arc::new(Self {
            runtime: runtime.clone(),
            displays: displays.clone(),
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

    /// Returns the bounding rectangle of each connected display.
    pub async fn display_rects(&self) -> Result<Vec<Rect>> {
        let displays_info = self.displays.wait_get_info().await?;
        Ok(displays_info.iter().map(|d| d.rect).collect())
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

/// Fills all pixels that lie outside every display rectangle with black.
///
/// `desktop_rect` is the bounding box of all displays; the image origin maps
/// to `desktop_rect.top_left` in screen coordinates. Pixels outside every
/// display are zeroed in-place (no extra allocation).
pub fn blacken_non_display_areas(image: &mut Image, desktop_rect: Rect, display_rects: &[Rect]) {
    let width = desktop_rect.size.width;

    // Compute each display's image-coordinate coverage via intersection with the desktop rect,
    // pre-sorted by x0 so per-row scanning needs no allocation.
    let mut bands: Vec<(Su32, Su32, Su32, Su32)> = display_rects
        .iter()
        .filter_map(|&display_rect| {
            let overlap = display_rect.intersection(desktop_rect)?;
            let offset = overlap.top_left - desktop_rect.top_left;
            let img_x0: Su32 = offset.x.into();
            let img_y0: Su32 = offset.y.into();
            let img_x1 = img_x0 + overlap.size.width;
            let img_y1 = img_y0 + overlap.size.height;
            Some((img_y0, img_y1, img_x0, img_x1))
        })
        .collect();
    bands.sort_unstable_by_key(|&(_, _, x0, _)| x0);

    let pixels: &mut [u8] = image;

    for (y_idx, row) in pixels.chunks_exact_mut(usize::from(width) * 4).enumerate() {
        let y = su32(y_idx);

        // Zero out gaps between (and around) the covered x-ranges for this row.
        let mut cursor = Su32::ZERO;
        for &(y0, y1, x0, x1) in &bands {
            if y < y0 || y >= y1 {
                continue;
            }
            if cursor < x0 {
                row[usize::from(cursor) * 4..usize::from(x0) * 4].fill(0);
            }
            cursor = cursor.max(x1);
        }
        if cursor < width {
            row[usize::from(cursor) * 4..].fill(0);
        }
    }
}
