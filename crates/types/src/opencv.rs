use opencv::core::{Point as CvPoint, Rect as CvRect};

use crate::{
    point::{Point, point},
    rect::Rect,
    size::size,
};

impl From<CvRect> for Rect {
    fn from(value: CvRect) -> Self {
        Self::new(point(value.x, value.y), size(value.width, value.height))
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
