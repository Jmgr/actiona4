use std::ops::{Add, Sub};

use tween::TweenValue;
use types::point::{Point, try_point};

#[derive(Clone, Copy, Debug)]
pub struct TweenPoint(Point);

impl From<Point> for TweenPoint {
    fn from(value: Point) -> Self {
        Self(value)
    }
}

impl From<TweenPoint> for Point {
    fn from(value: TweenPoint) -> Self {
        value.0
    }
}

impl Add for TweenPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for TweenPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl TweenValue for TweenPoint {
    // We can't return an error here so this just returns 0 on NaN
    fn scale(self, scale: f32) -> Self {
        let (x, y) = self.0.as_f64();
        let scale: f64 = scale.into();

        try_point(x * scale, y * scale).unwrap_or_default().into()
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use rstest::rstest;
    use types::point::point;

    use super::*;

    // ---------- TweenValue::scale (f32) -------------------------------------
    // On NaN -> returns Point::default() (ZERO); otherwise scales like scaled().

    #[rstest]
    #[case::ok(point(2, -3), 2.0f32, point(4, -6))]
    #[case::ok_round(point(1, 1), 1.6f32, point(1, 1))]
    #[case::nan(point(9, 9), f32::NAN, Point::ZERO)]
    fn tween_scale(#[case] p: Point, #[case] s: f32, #[case] want: Point) {
        let got = TweenValue::scale(TweenPoint::from(p), s);
        assert_eq!(want, got.0);
    }
}
