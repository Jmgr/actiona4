use std::{fmt::Display, sync::Arc};

use tokio_util::sync::CancellationToken;

use self::platform::WindowsHandler;
use crate::{
    api::{point::Point, rect::Rect, size::Size},
    runtime::Runtime,
    types::display::DisplayFields,
};

pub mod js;
pub mod platform;

pub use platform::WindowId;

pub type Result<T> = color_eyre::Result<T>;

#[derive(Clone, Debug)]
pub struct Windows {
    #[cfg(unix)]
    handler: Arc<platform::x11::X11WindowHandler>,

    #[cfg(windows)]
    handler: Arc<platform::win::WindowsWindowHandler>,

    runtime: Arc<Runtime>,
}

impl Display for Windows {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default().finish(f)
    }
}

impl Windows {
    #[must_use]
    pub fn new(runtime: Arc<Runtime>) -> Self {
        #[cfg(unix)]
        {
            Self {
                handler: Arc::new(platform::x11::X11WindowHandler::new(runtime.clone())),
                runtime,
            }
        }

        #[cfg(windows)]
        {
            Self {
                handler: Arc::new(platform::win::WindowsWindowHandler::default()),
                runtime,
            }
        }
    }

    pub fn all(&self) -> Result<Vec<WindowId>> {
        self.runtime.require_not_wayland()?;
        self.handler.all()
    }

    pub fn active_window(&self) -> Result<Option<WindowId>> {
        self.runtime.require_not_wayland()?;
        self.handler.active_window()
    }

    pub fn is_visible(&self, id: WindowId) -> Result<bool> {
        self.runtime.require_not_wayland()?;
        self.handler.is_visible(id)
    }

    pub fn title(&self, id: WindowId) -> Result<String> {
        self.runtime.require_not_wayland()?;
        self.handler.title(id)
    }

    pub fn classname(&self, id: WindowId) -> Result<String> {
        self.runtime.require_not_wayland()?;
        self.handler.classname(id)
    }

    pub fn close(&self, id: WindowId) -> Result<()> {
        self.runtime.require_not_wayland()?;
        self.handler.close(id)
    }

    pub fn process_id(&self, id: WindowId) -> Result<u32> {
        self.runtime.require_not_wayland()?;
        self.handler.process_id(id)
    }

    pub fn rect(&self, id: WindowId) -> Result<Rect> {
        self.runtime.require_not_wayland()?;
        self.handler.rect(id)
    }

    pub fn set_active(&self, id: WindowId) -> Result<()> {
        self.runtime.require_not_wayland()?;
        self.handler.set_active(id)
    }

    pub fn minimize(&self, id: WindowId) -> Result<()> {
        self.runtime.require_not_wayland()?;
        self.handler.minimize(id)
    }

    pub fn maximize(&self, id: WindowId) -> Result<()> {
        self.runtime.require_not_wayland()?;
        self.handler.maximize(id)
    }

    pub fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        self.runtime.require_not_wayland()?;
        self.handler.set_position(id, position)
    }

    pub fn position(&self, id: WindowId) -> Result<Point> {
        self.runtime.require_not_wayland()?;
        self.handler.position(id)
    }

    pub fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        self.runtime.require_not_wayland()?;
        self.handler.set_size(id, size)
    }

    pub fn size(&self, id: WindowId) -> Result<Size> {
        self.runtime.require_not_wayland()?;
        self.handler.size(id)
    }

    pub fn is_active(&self, id: WindowId) -> Result<bool> {
        self.runtime.require_not_wayland()?;
        self.handler.is_active(id)
    }

    #[must_use]
    pub fn runtime(&self) -> Arc<Runtime> {
        self.runtime.clone()
    }

    pub async fn wait_for_closed(
        &self,
        id: WindowId,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        self.runtime.require_not_wayland()?;
        self.handler
            .wait_for_closed(id, self.runtime.clone(), cancellation_token)
            .await
    }
}
