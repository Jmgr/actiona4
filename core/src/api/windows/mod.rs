use std::sync::Arc;

use thiserror::Error;

use self::platform::WindowsHandler;
use crate::{
    api::{point::Point, rect::Rect, size::Size},
    runtime::Runtime,
};

pub mod js;
pub mod platform;

pub use platform::{Error, WindowId};

#[derive(Debug, Error)]
pub enum WindowsError {
    #[error(transparent)]
    Platform(#[from] platform::Error),
}

pub type Result<T> = std::result::Result<T, WindowsError>;

#[derive(Debug)]
pub struct Windows {
    #[cfg(unix)]
    handler: platform::x11::X11WindowHandler,

    #[cfg(windows)]
    handler: platform::win::WindowsWindowHandler,
}

impl Windows {
    #[must_use]
    pub fn new(runtime: Arc<Runtime>) -> Self {
        #[cfg(unix)]
        {
            Self {
                handler: platform::x11::X11WindowHandler::new(runtime),
            }
        }

        #[cfg(windows)]
        {
            let _ = runtime;
            Self {
                handler: platform::win::WindowsWindowHandler::default(),
            }
        }
    }

    pub fn all(&self) -> Result<Vec<WindowId>> {
        Ok(self.handler.all()?)
    }

    pub fn active_window(&self) -> Result<WindowId> {
        Ok(self.handler.active_window()?)
    }

    pub fn is_visible(&self, id: WindowId) -> Result<bool> {
        Ok(self.handler.is_visible(id)?)
    }

    pub fn title(&self, id: WindowId) -> Result<String> {
        Ok(self.handler.title(id)?)
    }

    pub fn classname(&self, id: WindowId) -> Result<String> {
        Ok(self.handler.classname(id)?)
    }

    pub fn close(&self, id: WindowId) -> Result<()> {
        Ok(self.handler.close(id)?)
    }

    pub fn process_id(&self, id: WindowId) -> Result<u32> {
        Ok(self.handler.process_id(id)?)
    }

    pub fn rect(&self, id: WindowId) -> Result<Rect> {
        Ok(self.handler.rect(id)?)
    }

    pub fn set_active(&self, id: WindowId) -> Result<()> {
        Ok(self.handler.set_active(id)?)
    }

    pub fn minimize(&self, id: WindowId) -> Result<()> {
        Ok(self.handler.minimize(id)?)
    }

    pub fn maximize(&self, id: WindowId) -> Result<()> {
        Ok(self.handler.maximize(id)?)
    }

    pub fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        Ok(self.handler.set_position(id, position)?)
    }

    pub fn position(&self, id: WindowId) -> Result<Point> {
        Ok(self.handler.position(id)?)
    }

    pub fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        Ok(self.handler.set_size(id, size)?)
    }

    pub fn size(&self, id: WindowId) -> Result<Size> {
        Ok(self.handler.size(id)?)
    }

    pub fn is_active(&self, id: WindowId) -> Result<bool> {
        Ok(self.handler.is_active(id)?)
    }
}
