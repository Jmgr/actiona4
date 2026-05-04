use tokio::sync::oneshot;
use types::{point::Point, rect::Rect};
#[cfg(not(windows))]
use winit::dpi::PhysicalPosition;

pub enum AppEvent {
    SelectRect {
        screenshot: crate::screenshot::Screenshot,
        response: oneshot::Sender<Option<Rect>>,
    },
    SelectPosition {
        screenshot: crate::screenshot::Screenshot,
        response: oneshot::Sender<Option<Point>>,
    },
    Shutdown,
    /// Pointer moved in the selection window or from the X11 grab thread.
    #[cfg(not(windows))]
    CursorMoved(PhysicalPosition<f64>),
    /// Left button released at window-local position (from the X11 grab thread).
    #[cfg(not(windows))]
    Click(PhysicalPosition<f64>),
}
