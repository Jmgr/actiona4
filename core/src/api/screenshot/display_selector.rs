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
            Self::ById(id) => DisplayFields::default().display("id", id).finish(f),
            Self::FromPoint(point) => DisplayFields::default().display("point", point).finish(f),
        }
    }
}
