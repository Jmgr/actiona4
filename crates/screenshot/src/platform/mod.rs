#[cfg(unix)]
pub mod x11;
#[cfg(unix)]
pub use x11::Screen;

#[cfg(windows)]
pub mod win;
#[cfg(windows)]
pub use win::Screen;
