use std::sync::Arc;

use color_eyre::{Result, eyre::WrapErr};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub mod js;

mod platform;

#[cfg(windows)]
use platform::win::ScreenImpl;
#[cfg(unix)]
use platform::x11::ScreenImpl;

use super::{
    displays::Displays,
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
    api::{color::Color, point::Point, screen::platform::overlay::ask_rect},
    runtime::Runtime,
};

/// Controls which interactive screenshot method is used.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AskScreenshotMethod {
    /// Use the platform-default interactive screenshot picker.
    #[default]
    Auto,
    /// Use the platform native picker (XDG Desktop Portal on Linux, Snipping
    /// Tool on Windows). Fails if the native picker is unavailable.
    Native,
    /// Use the bundled overlay selector only.
    Overlay,
}

/// Options for [`Screen::ask_screenshot`].
#[derive(Clone, Debug, Default)]
pub struct AskScreenshotOptions {
    /// Controls which capture method to use.
    pub method: AskScreenshotMethod,
}

#[derive(Clone, Debug)]
pub struct Screen {
    implementation: Arc<ScreenImpl>,
    windows: Windows,
    runtime: Arc<Runtime>,
}

impl Screen {
    pub async fn new(runtime: Arc<Runtime>, displays: Displays, windows: Windows) -> Result<Self> {
        Ok(Self {
            implementation: ScreenImpl::new(runtime.clone(), displays).await?,
            windows,
            runtime: runtime.clone(),
        })
    }

    pub async fn capture_rect(&self, rect: Rect) -> Result<Image> {
        self.runtime.require_not_wayland()?;
        self.implementation.capture_rect(rect).await
    }

    /// Captures the entire virtual desktop (bounding box of all displays).
    pub async fn capture_desktop(&self) -> Result<Image> {
        self.runtime.require_not_wayland()?;
        self.implementation.capture_desktop().await
    }

    /// Captures the display with the given numeric ID.
    pub async fn capture_display(&self, display_id: u32) -> Result<Image> {
        self.runtime.require_not_wayland()?;
        self.implementation.capture_display(display_id).await
    }

    /// Captures the bounding rectangle of the given window.
    pub async fn capture_window(&self, id: WindowId) -> Result<Image> {
        self.runtime.require_not_wayland()?;
        let rect = self.windows.rect(id)?;
        self.implementation.capture_rect(rect).await
    }

    pub async fn capture_pixel(&self, position: Point) -> Result<Color> {
        self.runtime.require_not_wayland()?;
        self.implementation.capture_pixel(position).await
    }

    /// Finds the best match of an image within the given search area.
    pub async fn find_on_screen(
        &self,
        template: &Arc<Template>,
        search_in: &SearchIn,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<Option<Match>> {
        self.runtime.require_not_wayland()?;
        let _ = progress.send(FindImageProgress::new(FindImageStage::Capturing, 0));
        let (source, area_rect) = self.capture_search_in_to_source(search_in).await?;
        let origin = area_rect.top_left;
        let template = template.clone();
        let matches = self
            .runtime
            .task_tracker()
            .spawn_blocking(move || {
                source.find_template(&template, options, cancellation_token, progress)
            })
            .await??;
        Ok(matches.map(|m| m.offset(origin)))
    }

    /// Finds all matches of an image within the given search area.
    pub async fn find_all_on_screen(
        &self,
        template: &Arc<Template>,
        search_in: &SearchIn,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        self.runtime.require_not_wayland()?;
        let _ = progress.send(FindImageProgress::new(FindImageStage::Capturing, 0));
        let (source, area_rect) = self.capture_search_in_to_source(search_in).await?;
        let origin = area_rect.top_left;
        let template = template.clone();
        let matches = self
            .runtime
            .task_tracker()
            .spawn_blocking(move || {
                source.find_template_all(&template, options, cancellation_token, progress)
            })
            .await??;
        Ok(matches.into_iter().map(|m| m.offset(origin)).collect())
    }

    /// Asks the user to interactively select a screen area and returns a
    /// screenshot of that area, or `None` if the user cancels.
    pub async fn ask_screenshot(&self, options: AskScreenshotOptions) -> Result<Option<Image>> {
        use AskScreenshotMethod::*;

        match options.method {
            Native => self.ask_native_screenshot().await,
            Overlay => match self.ask_overlay_rect().await? {
                Some(rect) => self.implementation.capture_rect(rect).await.map(Some),
                None => Ok(None),
            },
            Auto => {
                let native_result = self.ask_native_screenshot().await;
                if native_result.is_ok() {
                    return native_result;
                }
                let native_error =
                    native_result.expect_err("native_result error should be present");

                let rect = self.ask_overlay_rect().await.wrap_err_with(|| {
                    format!(
                        "native screenshot failed and overlay fallback was unavailable: {}",
                        native_error
                    )
                })?;
                match rect {
                    Some(rect) => self.implementation.capture_rect(rect).await.map(Some),
                    None => Ok(None),
                }
            }
        }
    }

    async fn ask_native_screenshot(&self) -> Result<Option<Image>> {
        #[cfg(unix)]
        {
            use crate::api::screen::platform::x11::portal::ask_screenshot as ask_portal_screenshot;

            ask_portal_screenshot().await
        }
        #[cfg(windows)]
        {
            use crate::api::screen::platform::win::ask_screenshot::ask_screenshot as ask_system_screenshot;

            ask_system_screenshot(self.runtime.cancellation_token()).await
        }
    }

    async fn ask_overlay_rect(&self) -> Result<Option<Rect>> {
        self.runtime.require_not_wayland()?;

        ask_rect(&self.runtime, self.runtime.cancellation_token()).await
    }

    async fn capture_search_in_to_source(
        &self,
        search_in: &SearchIn,
    ) -> Result<(Arc<Source>, Rect)> {
        match search_in {
            SearchIn::Desktop => self.implementation.capture_desktop_to_source().await,
            SearchIn::Display(id) => self.implementation.capture_display_to_source(*id).await,
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
