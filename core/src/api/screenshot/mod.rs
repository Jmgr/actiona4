use std::sync::Arc;

use color_eyre::Result;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

mod platform;

pub mod js;

#[cfg(windows)]
use platform::win::ScreenshotImpl;
#[cfg(unix)]
use platform::x11::ScreenshotImpl;

use super::{
    displays::Displays,
    image::{
        Image,
        find_image::{
            FindImageProgress, FindImageStage, FindImageTemplateOptions, Match, Template,
        },
    },
    rect::Rect,
};
use crate::{
    api::{color::Color, point::Point},
    runtime::Runtime,
};

#[derive(Clone, Debug)]
pub struct Screenshot {
    implementation: Arc<ScreenshotImpl>,
}

impl Screenshot {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> Result<Self> {
        Ok(Self {
            implementation: ScreenshotImpl::new(runtime, displays).await?,
        })
    }

    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        self.implementation.capture_rect(rect).await
    }

    pub async fn capture_display(&self, display_id: u32) -> Result<Image> {
        self.implementation.capture_display(display_id).await
    }

    pub async fn capture_pixel(&self, position: Point) -> Result<Color> {
        self.implementation.capture_pixel(position).await
    }

    pub async fn find_image_on_rect(
        &self,
        rect: Rect,
        template: &Arc<Template>,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Option<Match>> {
        progress.send_replace(FindImageProgress::new(FindImageStage::Capturing, 0));
        let source = self.implementation.capture_rect_to_source(rect).await?;
        let origin = rect.top_left;
        let matches = source.find_template(template, options, cancellation_token, progress)?;
        Ok(matches.map(|m| m.offset(origin)))
    }

    pub async fn find_image_on_rect_all(
        &self,
        rect: Rect,
        template: &Arc<Template>,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        progress.send_replace(FindImageProgress::new(FindImageStage::Capturing, 0));
        let source = self.implementation.capture_rect_to_source(rect).await?;
        let origin = rect.top_left;
        let matches = source.find_template_all(template, options, cancellation_token, progress)?;
        Ok(matches.into_iter().map(|m| m.offset(origin)).collect())
    }

    pub async fn find_image_on_display(
        &self,
        display_id: u32,
        template: &Arc<Template>,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Option<Match>> {
        progress.send_replace(FindImageProgress::new(FindImageStage::Capturing, 0));
        let (source, display_rect) = self
            .implementation
            .capture_display_to_source(display_id)
            .await?;
        let origin = display_rect.top_left;
        let matches = source.find_template(template, options, cancellation_token, progress)?;
        Ok(matches.map(|m| m.offset(origin)))
    }

    pub async fn find_image_on_display_all(
        &self,
        display_id: u32,
        template: &Arc<Template>,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        progress.send_replace(FindImageProgress::new(FindImageStage::Capturing, 0));
        let (source, display_rect) = self
            .implementation
            .capture_display_to_source(display_id)
            .await?;
        let origin = display_rect.top_left;
        let matches = source.find_template_all(template, options, cancellation_token, progress)?;
        Ok(matches.into_iter().map(|m| m.offset(origin)).collect())
    }
}
