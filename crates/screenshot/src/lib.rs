//! Cross-platform screen capture primitives.
//!
//! This crate provides the low-level platform-specific code for grabbing
//! pixels from a display.

mod blacken;
mod capture;
mod platform;

pub use blacken::blacken_non_display_areas;
pub use capture::{Capture, bgra_to_rgba_in_place};
pub use platform::Screen;
#[cfg(unix)]
pub use platform::x11::ShmSegment;
