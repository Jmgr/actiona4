#[cfg(not(windows))]
use winit::dpi::PhysicalPosition;

#[derive(Clone, Copy, Debug)]
pub enum AppEvent {
    /// Pointer moved in the selection window or from the X11 grab thread.
    #[cfg(not(windows))]
    CursorMoved(PhysicalPosition<f64>),
    /// Left button released at window-local position (from the X11 grab thread).
    #[cfg(not(windows))]
    Click(PhysicalPosition<f64>),
}
