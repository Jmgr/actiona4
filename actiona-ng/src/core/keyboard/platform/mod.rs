#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub use super::{Key, Result};

pub trait KeyboardImplTrait {
    fn is_key_pressed(&self, key: Key) -> Result<bool>;
}
