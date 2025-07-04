#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub mod win;

pub use super::Result;
