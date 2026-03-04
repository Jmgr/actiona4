use std::fmt;

use crate::{api::point::Point, types::display::DisplayFields};

/// A display selector resolved at capture time.
///
/// Created via the `Display` JS class factory methods and passed to
/// `Screenshot` capture and find-image operations.
#[derive(Clone, Debug)]
pub enum DisplaySelector {
    /// The entire desktop.
    Desktop,
    /// The primary (main) display.
    Primary,
    /// The display with the largest area.
    Largest,
    /// The display with the smallest area.
    Smallest,
    /// The display furthest to the left (minimum left edge).
    Leftmost,
    /// The display furthest to the right (maximum right edge).
    Rightmost,
    /// The display furthest to the top (minimum top edge).
    Topmost,
    /// The display furthest to the bottom (maximum bottom edge).
    Bottommost,
    /// The display whose center is closest to the center of the desktop.
    Center,
    /// A specific display looked up by its unique numeric ID.
    ById(u32),
    /// The display that contains the given point.
    FromPoint(Point),
}

impl fmt::Display for DisplaySelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Desktop => write!(f, "(desktop)"),
            Self::Primary => write!(f, "(primary)"),
            Self::Largest => write!(f, "(largest)"),
            Self::Smallest => write!(f, "(smallest)"),
            Self::Leftmost => write!(f, "(leftmost)"),
            Self::Rightmost => write!(f, "(rightmost)"),
            Self::Topmost => write!(f, "(topmost)"),
            Self::Bottommost => write!(f, "(bottommost)"),
            Self::Center => write!(f, "(center)"),
            Self::ById(id) => DisplayFields::default().display("id", id).finish(f),
            Self::FromPoint(point) => DisplayFields::default().display("point", point).finish(f),
        }
    }
}
