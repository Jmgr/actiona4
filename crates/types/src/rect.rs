use std::fmt::Display;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use super::point::{Point, point};
use crate::{
    display::DisplayFields,
    si32::TryDiv,
    size::{Size, size},
    su32::Su32,
};

#[derive(Clone, Constructor, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Rect {
    pub top_left: Point,
    pub size: Size,
}

#[must_use]
pub const fn rect(origin: Point, size: Size) -> Rect {
    Rect::new(origin, size)
}

impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("x", self.top_left.x)
            .display("y", self.top_left.y)
            .display("width", self.size.width)
            .display("height", self.size.height)
            .finish(f)
    }
}

impl Rect {
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.top_left.x
            && point.x < self.top_left.x + self.size.width
            && point.y >= self.top_left.y
            && point.y < self.top_left.y + self.size.height
    }

    #[must_use]
    pub fn intersects(&self, other: Self) -> bool {
        !(self.top_left.x + self.size.width <= other.top_left.x
            || other.top_left.x + other.size.width <= self.top_left.x
            || self.top_left.y + self.size.height <= other.top_left.y
            || other.top_left.y + other.size.height <= self.top_left.y)
    }

    #[must_use]
    pub fn intersection(&self, other: Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        let x1 = self.top_left.x.max(other.top_left.x);
        let y1 = self.top_left.y.max(other.top_left.y);
        let x2 = (self.top_left.x + self.size.width).min(other.top_left.x + other.size.width);
        let y2 = (self.top_left.y + self.size.height).min(other.top_left.y + other.size.height);

        Some(Self {
            top_left: point(x1, y1),
            size: size(x2 - x1, y2 - y1),
        })
    }

    #[must_use]
    pub fn union(&self, other: Self) -> Self {
        let x1 = self.top_left.x.min(other.top_left.x);
        let y1 = self.top_left.y.min(other.top_left.y);
        let x2 = (self.top_left.x + self.size.width).max(other.top_left.x + other.size.width);
        let y2 = (self.top_left.y + self.size.height).max(other.top_left.y + other.size.height);

        Self {
            top_left: point(x1, y1),
            size: size(x2 - x1, y2 - y1),
        }
    }

    #[must_use]
    pub fn clamped(&self) -> (u32, u32, u32, u32) {
        let clamped_x: Su32 = self.top_left.x.into_inner().max(0).into();
        let clamped_y: Su32 = self.top_left.y.into_inner().max(0).into();

        let adjusted_width = if self.top_left.x < 0 {
            self.size.width - self.top_left.x.unsigned_abs()
        } else {
            self.size.width
        };

        let adjusted_height = if self.top_left.y < 0 {
            self.size.height - self.top_left.y.unsigned_abs()
        } else {
            self.size.height
        };

        (
            clamped_x.into(),
            clamped_y.into(),
            adjusted_width.into(),
            adjusted_height.into(),
        )
    }

    #[must_use]
    pub fn center(&self) -> Point {
        self.top_left
            + self
                .size
                .try_div(2)
                .expect("dividing a Size by 2 should never fail")
    }

    #[must_use]
    pub const fn top_left(&self) -> Point {
        self.top_left
    }

    #[must_use]
    pub fn bottom_right(&self) -> Point {
        self.top_left + (self.size - size(1, 1))
    }

    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    #[must_use]
    pub fn surface(&self) -> u32 {
        (self.size.width * self.size.height).into()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn r(x: i32, y: i32, w: u32, h: u32) -> Rect {
        rect(point(x, y), size(w, h))
    }

    #[rstest]
    #[case::equal_same(r(0, 0, 10, 10), r(0, 0, 10, 10), true)]
    #[case::different_origin(r(1, 0, 10, 10), r(0, 0, 10, 10), false)]
    #[case::different_size(r(0, 0, 9, 10), r(0, 0, 10, 10), false)]
    fn equals_works(#[case] a: Rect, #[case] b: Rect, #[case] want: bool) {
        assert_eq!(a.equals(b), want);
        assert_eq!(a == b, want);
    }

    #[rstest]
    // inside (inclusive on top/left)
    #[case::inside_top_left(r(0, 0, 10, 10), point(0, 0), true)]
    // far edge is exclusive, so last included is (w-1,h-1)
    #[case::inside_last_inclusive(r(0, 0, 3, 2), point(2, 1), true)]
    // outside: right/bottom edges are excluded
    #[case::on_right_edge_excluded(r(0, 0, 3, 2), point(3, 1), false)]
    #[case::on_bottom_edge_excluded(r(0, 0, 3, 2), point(2, 2), false)]
    // outside: left/top
    #[case::left_of(r(5, 5, 3, 3), point(4, 6), false)]
    #[case::above(r(5, 5, 3, 3), point(6, 4), false)]
    fn contains_behaviour(#[case] rectv: Rect, #[case] p: Point, #[case] want: bool) {
        assert_eq!(rectv.contains(p), want);
    }

    #[rstest]
    // overlap
    #[case::overlap(r(0, 0, 10, 10), r(5, 5, 10, 10), true)]
    // touching edge -> NOT intersecting (edges are exclusive)
    #[case::touch_right_edge(r(0, 0, 10, 10), r(10, 0, 5, 5), false)]
    #[case::touch_bottom_edge(r(0, 0, 10, 10), r(0, 10, 5, 5), false)]
    // disjoint
    #[case::disjoint(r(0, 0, 3, 3), r(10, 10, 2, 2), false)]
    fn intersects_cases(#[case] a: Rect, #[case] b: Rect, #[case] want: bool) {
        assert_eq!(a.intersects(b), want);
        assert_eq!(b.intersects(a), want);
    }

    #[test]
    fn intersection_area_and_coords() {
        let a = r(0, 0, 10, 10);
        let b = r(5, 2, 10, 5);
        let inter = a.intersection(b).expect("should intersect");
        // x: [0,10) ∩ [5,15) = [5,10) => 5
        // y: [0,10) ∩ [2, 7) = [2, 7) => 5
        assert_eq!(inter, r(5, 2, 5, 5));
    }

    #[test]
    fn intersection_none_when_touching() {
        let a = r(0, 0, 10, 10);
        let b = r(10, 0, 3, 3); // touches on the right edge only
        assert!(a.intersection(b).is_none());
    }

    #[test]
    fn union_bounds_both() {
        let a = r(2, 3, 5, 7); // right=7, bottom=10
        let b = r(-4, -1, 10, 2); // right=6, bottom=1
        let u = a.union(b);
        // min x=-4, min y=-1, max right=max(7,6)=7, max bottom=max(10,1)=10
        // width=7-(-4)=11, height=10-(-1)=11
        assert_eq!(u, r(-4, -1, 11, 11));
    }

    #[rstest]
    #[case::pos_origin(r(2,3,10,10), (2,3,10,10))]
    #[case::neg_origin_partial(r(-5,-3,10,10), (0,0,5,7))]
    #[case::neg_origin_exceeds_size(r(-50,-60,10,10), (0,0,0,0))]
    fn clamped_rules(#[case] input: Rect, #[case] want: (u32, u32, u32, u32)) {
        assert_eq!(input.clamped(), want);
    }

    #[test]
    fn center_even_sizes() {
        let a = r(10, 10, 8, 4);
        // center = origin + (width/2, height/2) = (10,10) + (4,2)
        assert_eq!(a.center(), point(14, 12));
    }

    #[test]
    fn top_left_and_bottom_right() {
        let a = r(-3, 4, 3, 2);
        assert_eq!(a.top_left(), point(-3, 4));
        // bottom_right = origin + (w-1, h-1) = (-3,4) + (2,1) = (-1,5)
        assert_eq!(a.bottom_right(), point(-1, 5));
    }

    #[test]
    fn size_and_surface() {
        let a = r(0, 0, 6, 7);
        assert_eq!(a.size(), size(6, 7));
        assert_eq!(a.surface(), 42);
    }
}
