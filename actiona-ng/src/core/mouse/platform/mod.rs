#[cfg(unix)]
pub(crate) mod x11;

#[cfg(windows)]
pub(crate) mod win;

pub use super::{JsButton, Result};

pub trait MouseImplTrait {
    fn is_button_pressed(&mut self, button: JsButton) -> Result<bool>;
}
