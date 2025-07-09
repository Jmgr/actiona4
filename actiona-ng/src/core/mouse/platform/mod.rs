#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub use super::{JsButton, Result};

pub trait MouseImplTrait {
    async fn is_button_pressed(&mut self, button: JsButton) -> Result<bool>;
}
