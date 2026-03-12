use winit::dpi::PhysicalPosition;

#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    /// Pointer moved in the selection window or from the X11 grab thread.
    CursorMoved(PhysicalPosition<f64>),
    /// Left button released at window-local position (from the X11 grab thread).
    Click(PhysicalPosition<f64>),
}
