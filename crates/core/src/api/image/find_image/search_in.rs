use crate::api::{rect::Rect, windows::WindowId};

/// Specifies the screen area to search within for find-image operations.
///
/// Created via the `SearchIn` JS class factory methods.
#[derive(Clone, Debug)]
pub enum SearchIn {
    /// The entire desktop: the bounding rectangle of all connected displays.
    Desktop,
    /// A specific display, identified by its numeric ID.
    Display(u32),
    /// An explicit screen rectangle.
    Rect(Rect),
    /// A specific window, identified by its [`WindowId`].
    Window(WindowId),
}
