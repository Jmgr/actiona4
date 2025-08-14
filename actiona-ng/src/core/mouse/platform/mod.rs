#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub use super::{Button, Result};

pub trait MouseImplTrait {
    async fn is_button_pressed(&self, button: Button) -> Result<bool>;
}
