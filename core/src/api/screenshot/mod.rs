use std::sync::Arc;

use color_eyre::Result;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

pub mod js;

mod platform;

#[cfg(windows)]
use platform::win::ScreenshotImpl;
#[cfg(unix)]
use platform::x11::ScreenshotImpl;

use super::{
    displays::{Displays, display_selector::DisplaySelector},
    image::{
        Image,
        find_image::{
            FindImageProgress, FindImageStage, FindImageTemplateOptions, Match, SearchIn, Source,
            Template,
        },
    },
    rect::Rect,
    windows::{WindowId, Windows},
};
use crate::{
    api::{color::Color, point::Point},
    runtime::Runtime,
};

#[derive(Clone, Debug)]
pub struct Screenshot {
    implementation: Arc<ScreenshotImpl>,
    windows: Windows,
}

impl Screenshot {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays, windows: Windows) -> Result<Self> {
        Ok(Self {
            implementation: ScreenshotImpl::new(runtime, displays).await?,
            windows,
        })
    }

    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        self.implementation.capture_rect(rect).await
    }

    /// Captures the entire virtual desktop (bounding box of all displays).
    pub async fn capture_desktop(&self) -> Result<Image> {
        self.implementation.capture_desktop().await
    }

    /// Captures the display identified by the given selector.
    ///
    /// For `DisplaySelector::Desktop` this is equivalent to `capture_desktop`.
    pub async fn capture_display(&self, selector: &DisplaySelector) -> Result<Image> {
        match selector {
            DisplaySelector::Desktop => self.capture_desktop().await,
            other => {
                let display_id = self.implementation.resolve_display_selector(other).await?;
                self.implementation.capture_display(display_id).await
            }
        }
    }

    /// Captures the bounding rectangle of the given window.
    pub async fn capture_window(&self, id: WindowId) -> Result<Image> {
        let rect = self.windows.rect(id)?;
        self.implementation.capture_rect(rect).await
    }

    pub async fn capture_pixel(&self, position: Point) -> Result<Color> {
        self.implementation.capture_pixel(position).await
    }

    /// Finds the best match of an image within the given search area.
    pub async fn find_image(
        &self,
        template: &Arc<Template>,
        search_in: &SearchIn,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Option<Match>> {
        progress.send_replace(FindImageProgress::new(FindImageStage::Capturing, 0));
        let (source, area_rect) = self.capture_search_in_to_source(search_in).await?;
        let origin = area_rect.top_left;
        let matches = source.find_template(template, options, cancellation_token, progress)?;
        Ok(matches.map(|m| m.offset(origin)))
    }

    /// Finds all matches of an image within the given search area.
    pub async fn find_image_all(
        &self,
        template: &Arc<Template>,
        search_in: &SearchIn,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        progress.send_replace(FindImageProgress::new(FindImageStage::Capturing, 0));
        let (source, area_rect) = self.capture_search_in_to_source(search_in).await?;
        let origin = area_rect.top_left;
        let matches = source.find_template_all(template, options, cancellation_token, progress)?;
        Ok(matches.into_iter().map(|m| m.offset(origin)).collect())
    }

    async fn capture_search_in_to_source(
        &self,
        search_in: &SearchIn,
    ) -> Result<(Arc<Source>, Rect)> {
        match search_in {
            SearchIn::Desktop => self.implementation.capture_desktop_to_source().await,
            SearchIn::Display(selector) => {
                let display_id = self
                    .implementation
                    .resolve_display_selector(selector)
                    .await?;
                self.implementation
                    .capture_display_to_source(display_id)
                    .await
            }
            SearchIn::Rect(rect) => {
                let source = self.implementation.capture_rect_to_source(*rect).await?;
                Ok((source, *rect))
            }
            SearchIn::Window(id) => {
                let rect = self.windows.rect(*id)?;
                let source = self.implementation.capture_rect_to_source(rect).await?;
                Ok((source, rect))
            }
        }
    }
}
