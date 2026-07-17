use tokio::sync::oneshot;
use types::{Point, Rect};
#[cfg(not(windows))]
use winit::dpi::PhysicalPosition;

use crate::screenshot::Screenshot;

pub enum AppEvent {
    SelectRect {
        screenshot: Screenshot,
        response: oneshot::Sender<Option<Rect>>,
    },
    SelectPosition {
        screenshot: Screenshot,
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
