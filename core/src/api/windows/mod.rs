use std::{fmt::Display, sync::Arc};

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
                handler: Arc::new(platform::x11::X11WindowHandler::new(runtime)),
            }
        }

        #[cfg(windows)]
        {
            let _ = runtime;
            Self {
                handler: Arc::new(platform::win::WindowsWindowHandler::default()),
            }
        }
    }

    pub fn all(&self) -> Result<Vec<WindowId>> {
        self.handler.all()
    }

    pub fn active_window(&self) -> Result<WindowId> {
        self.handler.active_window()
    }

    pub fn is_visible(&self, id: WindowId) -> Result<bool> {
        self.handler.is_visible(id)
    }

    pub fn title(&self, id: WindowId) -> Result<String> {
        self.handler.title(id)
    }

    pub fn classname(&self, id: WindowId) -> Result<String> {
        self.handler.classname(id)
    }

    pub fn close(&self, id: WindowId) -> Result<()> {
        self.handler.close(id)
    }

    pub fn process_id(&self, id: WindowId) -> Result<u32> {
        self.handler.process_id(id)
    }

    pub fn rect(&self, id: WindowId) -> Result<Rect> {
        self.handler.rect(id)
    }

    pub fn set_active(&self, id: WindowId) -> Result<()> {
        self.handler.set_active(id)
    }

    pub fn minimize(&self, id: WindowId) -> Result<()> {
        self.handler.minimize(id)
    }

    pub fn maximize(&self, id: WindowId) -> Result<()> {
        self.handler.maximize(id)
    }

    pub fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        self.handler.set_position(id, position)
    }

    pub fn position(&self, id: WindowId) -> Result<Point> {
        self.handler.position(id)
    }

    pub fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        self.handler.set_size(id, size)
    }

    pub fn size(&self, id: WindowId) -> Result<Size> {
        self.handler.size(id)
    }

    pub fn is_active(&self, id: WindowId) -> Result<bool> {
        self.handler.is_active(id)
    }
}
