#[cfg(windows)]
mod win;
#[cfg(unix)]
mod x11;

#[cfg(windows)]
pub use win::*;
#[cfg(unix)]
pub use x11::*;
