#[cfg(unix)]
pub(crate) mod x11;

#[cfg(windows)]
pub(crate) mod win;

pub use super::Result;
