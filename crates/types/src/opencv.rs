use opencv::core::{Point as CvPoint, Rect as CvRect, Size as CvSize};
use satint::SaturatingInto;

use crate::{
    point::{Point, point},
    rect::Rect,
    size::Size,
};

impl From<CvRect> for Rect {
    fn from(value: CvRect) -> Self {
        Self::new(
            point(value.x, value.y),
            Size::new(
                value.width.saturating_into(),
                value.height.saturating_into(),
            ),
        )
    }
}

impl From<Rect> for CvRect {
    fn from(value: Rect) -> Self {
        Self::new(
            value.top_left.x.into(),
            value.top_left.y.into(),
            value.size.width.to_signed().into(),
            value.size.height.to_signed().into(),
        )
    }
}

impl From<CvSize> for Size {
    fn from(value: CvSize) -> Self {
        Self::new(
            value.width.saturating_into(),
            value.height.saturating_into(),
        )
    }
}

impl From<CvPoint> for Point {
    fn from(value: CvPoint) -> Self {
        Self::new(value.x.into(), value.y.into())
    }
}

impl From<Point> for CvPoint {
    fn from(value: Point) -> Self {
        Self::new(value.x.into(), value.y.into())
    }
}
