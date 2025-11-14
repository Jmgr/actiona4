use std::{
    f64::consts::TAU,
    fmt::Display,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use color_eyre::{Report, Result};
use derive_more::{Add, AddAssign, Constructor, Neg, Sub, SubAssign};
use serde::{Deserialize, Serialize};
use tween::TweenValue;

use crate::{
    core::size::Size,
    runtime::shared_rng::SharedRng,
    types::{
        display::DisplayFields,
        si32::{Si32, TryDiv, TryDivAssign},
    },
};

pub mod js;

#[derive(
    Add,
    AddAssign,
    Clone,
    Constructor,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    Neg,
    PartialEq,
    Serialize,
    Sub,
    SubAssign,
)]
pub struct Point {
    pub x: Si32,
    pub y: Si32,
}

#[must_use]
pub fn point<X: Into<Si32>, Y: Into<Si32>>(x: X, y: Y) -> Point {
    Point::new(x.into(), y.into())
}

pub fn try_point<X, Y>(x: X, y: Y) -> Result<Point>
where
    X: TryInto<Si32>,
    Y: TryInto<Si32>,
    color_eyre::Report: From<X::Error> + From<Y::Error>,
{
    Ok(Point::new(x.try_into()?, y.try_into()?))
}

impl Mul<i32> for Point {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<i32> for Point {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl TryDiv<i32> for Point {
    type Output = Self;
    type Error = Report;

    fn try_div(self, rhs: i32) -> std::result::Result<Self::Output, Self::Error> {
        Ok(Self::new(self.x.try_div(rhs)?, self.y.try_div(rhs)?))
    }
}

impl TryDivAssign<i32> for Point {
    type Error = Report;

    fn try_div_assign(&mut self, rhs: i32) -> std::result::Result<(), Self::Error> {
        self.x.try_div_assign(rhs)?;
        self.y.try_div_assign(rhs)?;
        Ok(())
    }
}

impl Add<Size> for Point {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        point(self.x + rhs.width, self.y + rhs.height)
    }
}

impl AddAssign<Size> for Point {
    fn add_assign(&mut self, rhs: Size) {
        *self = point(self.x + rhs.width, self.y + rhs.height);
    }
}

impl Sub<Size> for Point {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        point(self.x - rhs.width, self.y - rhs.height)
    }
}

impl SubAssign<Size> for Point {
    fn sub_assign(&mut self, rhs: Size) {
        *self = point(self.x - rhs.width, self.y - rhs.height);
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("x", self.x)
            .display("y", self.y)
            .finish(f)
    }
}

impl TweenValue for Point {
    // We can't return an error here so this just returns 0 on NaN
    fn scale(self, scale: f32) -> Self {
        let (x, y) = self.as_f64();
        let scale: f64 = scale.into();

        try_point(x * scale, y * scale).unwrap_or_default()
    }
}

impl Point {
    pub const ZERO: Self = Self::new(Si32::ZERO, Si32::ZERO);

    #[must_use]
    pub fn dot_product(self, other: Self) -> Si32 {
        self.x * other.x + self.y * other.y
    }

    #[must_use]
    pub fn cross_product(self, other: Self) -> Si32 {
        self.x * other.y - self.y * other.x
    }

    #[must_use]
    pub fn normalize(self) -> (f64, f64) {
        let (x, y) = self.as_f64();

        let len = self.length();
        if len > 0. {
            ((x / len), (y / len))
        } else {
            (0., 0.)
        }
    }

    #[must_use]
    pub fn length(&self) -> f64 {
        let (x, y) = self.as_f64();

        x.hypot(y)
    }

    #[must_use]
    pub fn length_squared(&self) -> Si32 {
        self.x * self.x + self.y * self.y
    }

    pub fn random_in_circle(center: Self, radius: f64, rng: SharedRng) -> Result<Self> {
        let (center_x, center_y) = center.as_f64();
        let theta = rng.random_range(0.0..TAU);
        let r = radius * rng.random::<f64>().sqrt();
        let x = r.mul_add(theta.cos(), center_x);
        let y = r.mul_add(theta.sin(), center_y);

        try_point(x, y)
    }

    #[must_use]
    pub fn distance_to(&self, other: Self) -> f64 {
        let (x, y) = self.as_f64();
        let (other_x, other_y) = other.as_f64();

        (x - other_x).hypot(y - other_y)
    }

    #[must_use]
    pub fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    #[must_use]
    pub fn distance(a: Self, b: Self) -> f64 {
        a.distance_to(b)
    }

    pub fn scaled(&self, factor: f64) -> Result<Self> {
        let (x, y) = self.as_f64();

        Ok(Self {
            x: (x * factor).try_into()?,
            y: (y * factor).try_into()?,
        })
    }

    #[must_use]
    pub fn clamped(&self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }

    pub(crate) fn as_f64(&self) -> (f64, f64) {
        (self.x.into(), self.y.into())
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{
        runtime::shared_rng::SharedRng,
        types::si32::{Si32, TryDiv, TryDivAssign, si32},
    };

    // ---------- constructors -------------------------------------------------

    #[rstest]
    #[case::zero(point(0, 0), (0, 0))]
    #[case::pos(point(3, 5), (3, 5))]
    #[case::neg(point(-7, -2), (-7, -2))]
    fn ctor_point_and_accessors(#[case] p: Point, #[case] want: (i32, i32)) {
        assert_eq!(want.0, p.x.into_inner());
        assert_eq!(want.1, p.y.into_inner());
    }

    #[rstest]
    #[case::ok_small(3.0f64, 5.0f64, (3, 5))]
    #[case::ok_clamp_hi((i32::MAX as f64) + 10.0, 1.0f64, (i32::MAX, 1))]
    #[case::ok_clamp_lo(-1e10f64, -2.5f64, (i32::MIN, -2))]
    fn ctor_try_point_ok<TX, TY>(#[case] x: TX, #[case] y: TY, #[case] want: (i32, i32))
    where
        TX: TryInto<Si32>,
        TY: TryInto<Si32>,
        color_eyre::Report: From<TX::Error> + From<TY::Error>,
    {
        let p = try_point(x, y).unwrap();
        assert_eq!(want.0, p.x.into_inner());
        assert_eq!(want.1, p.y.into_inner());
    }

    // ---------- Mul / MulAssign by i32 (saturating via Si32) ----------------

    #[rstest]
    #[case::basic(point(2, -3), 3, point(6, -9))]
    #[case::saturate_hi(point(i32::MAX, 1), 2, point(i32::MAX, 2))]
    #[case::saturate_lo(point(i32::MIN, -1), 2, point(i32::MIN, -2))]
    fn mul_i32(#[case] p: Point, #[case] k: i32, #[case] want: Point) {
        assert_eq!(want, p * k);
    }

    #[rstest]
    #[case::basic(point(2, -3), 3, point(6, -9))]
    #[case::saturate_hi(point(i32::MAX, 1), 2, point(i32::MAX, 2))]
    fn mul_assign_i32(#[case] start: Point, #[case] k: i32, #[case] want: Point) {
        let mut p = start;
        p *= k;
        assert_eq!(want, p);
    }

    // ---------- TryDiv / TryDivAssign by i32 (ok + err) ---------------------

    #[rstest]
    #[case::ok_even(point(6, -4), 2, point(3, -2))]
    #[case::ok_trunc_toward_zero(point(7, -7), 3, point(2, -2))]
    fn try_div_ok(#[case] p: Point, #[case] d: i32, #[case] want: Point) {
        let got = p.try_div(d);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::div_by_zero(point(1, 2), 0)]
    #[case::min_overflow(point(i32::MIN, 0), -1)]
    #[case::min_overflow_y(point(0, i32::MIN), -1)]
    fn try_div_err(#[case] p: Point, #[case] d: i32) {
        let got = p.try_div(d);
        assert!(got.is_err());
    }

    #[rstest]
    #[case::ok_even(point(6, -4), 2, point(3, -2))]
    fn try_div_assign_ok(#[case] start: Point, #[case] d: i32, #[case] want: Point) {
        let mut p = start;
        let r = p.try_div_assign(d);
        assert!(r.is_ok());
        assert_eq!(want, p);
    }

    #[rstest]
    #[case::div_by_zero(point(1, 2), 0)]
    #[case::min_overflow(point(i32::MIN, 0), -1)]
    fn try_div_assign_err(#[case] start: Point, #[case] d: i32) {
        let mut p = start;
        let r = p.try_div_assign(d);
        assert!(r.is_err());
    }

    // ---------- vector math --------------------------------------------------

    #[rstest]
    #[case::dot_orthogonal(point(1, 0), point(0, 1), 0)]
    #[case::dot_basic(point(2, 3), point(-1, 4), 10)] // 2*(-1)+3*4=10
    fn dot_product(#[case] a: Point, #[case] b: Point, #[case] want: i32) {
        assert_eq!(si32(want), a.dot_product(b));
    }

    #[rstest]
    #[case::cross_parallel(point(1, 0), point(2, 0), 0)]
    #[case::cross_basic(point(2, 3), point(-1, 4), 11)] // 2*4 - 3*(-1)=11
    fn cross_product(#[case] a: Point, #[case] b: Point, #[case] want: i32) {
        assert_eq!(si32(want), a.cross_product(b));
    }

    #[rstest]
    #[case::norm_unit_x(point(5, 0), (1.0, 0.0))]
    #[case::norm_unit_y(point(0, -9), (0.0, -1.0))]
    #[case::norm_zero(point(0, 0), (0.0, 0.0))]
    fn normalize_(#[case] p: Point, #[case] want: (f64, f64)) {
        let got = p.normalize();
        assert!((got.0 - want.0).abs() < 1e-12);
        assert!((got.1 - want.1).abs() < 1e-12);
    }

    #[rstest]
    #[case::len_zero(point(0, 0), 0.0)]
    #[case::len_3_4_5(point(3, 4), 5.0)]
    #[case::len_diag(point(-3, -4), 5.0)]
    fn length_(#[case] p: Point, #[case] want: f64) {
        let got = p.length();
        assert!((got - want).abs() < 1e-12);
    }

    #[rstest]
    #[case::sq_zero(point(0, 0), 0)]
    #[case::sq_3_4(point(3, 4), 25)] // 9+16
    fn length_squared_(#[case] p: Point, #[case] want: i32) {
        assert_eq!(si32(want), p.length_squared());
    }

    #[rstest]
    #[case::dist_same(point(1, 2), point(1, 2), 0.0)]
    #[case::dist_simple(point(0, 0), point(3, 4), 5.0)]
    fn distance_to_and_static(#[case] a: Point, #[case] b: Point, #[case] want: f64) {
        let d1 = a.distance_to(b);
        let d2 = Point::distance(a, b);
        assert!((d1 - want).abs() < 1e-12);
        assert!((d2 - want).abs() < 1e-12);
    }

    #[rstest]
    #[case::origin_true(point(0, 0), true)]
    #[case::origin_false(point(0, 1), false)]
    fn is_origin_(#[case] p: Point, #[case] want: bool) {
        assert_eq!(want, p.is_origin());
    }

    #[rstest]
    #[case::clamped_inside(point(5, 5), point(0, 0), point(10, 10), point(5, 5))]
    #[case::clamped_low(point(-1,7), point(0,0), point(10,10), point(0,7))]
    #[case::clamped_high(point(9, 11), point(0, 0), point(10, 10), point(9, 10))]
    fn clamped_(#[case] p: Point, #[case] min: Point, #[case] max: Point, #[case] want: Point) {
        assert_eq!(want, p.clamped(min, max));
    }

    #[rstest]
    #[case::scale_int(point(3, -4), 2.0, point(6, -8))]
    #[case::scale_round(point(1, 1), 1.6, point(1, 1))]
    #[case::scale_clamp_hi(point(i32::MAX, 1), 2.0, point(i32::MAX, 2))]
    #[case::scale_clamp_lo(point(i32::MIN, -1), 2.0, point(i32::MIN, -2))]
    fn scaled_ok(#[case] p: Point, #[case] factor: f64, #[case] want: Point) {
        let got = p.scaled(factor);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::nan(point(1, 1), f64::NAN)]
    #[case::pos_inf(point(1, 1), f64::INFINITY)]
    #[case::neg_inf(point(1, 1), f64::NEG_INFINITY)]
    fn scaled_err(#[case] p: Point, #[case] factor: f64) {
        let got = p.scaled(factor);
        assert!(got.is_err());
    }

    // ---------- TweenValue::scale (f32) -------------------------------------
    // On NaN -> returns Point::default() (ZERO); otherwise scales like scaled().

    #[rstest]
    #[case::ok(point(2, -3), 2.0f32, point(4, -6))]
    #[case::ok_round(point(1, 1), 1.6f32, point(1, 1))]
    #[case::nan(point(9, 9), f32::NAN, Point::ZERO)]
    fn tween_scale(#[case] p: Point, #[case] s: f32, #[case] want: Point) {
        let got = TweenValue::scale(p, s);
        assert_eq!(want, got);
    }

    // ---------- random_in_circle --------------------------------------------
    // Basic property test: results lie within ~radius (allowing <= 1.0 slack for rounding to i32).

    #[test]
    fn random_in_circle_within_radius() {
        let center = point(100, -50);
        let radius = 100.0;
        let rng = SharedRng::default();

        for _ in 0..1000 {
            let p = Point::random_in_circle(center, radius, rng.clone()).unwrap();
            let d = center.distance_to(p);
            assert!(d <= radius + 1.0, "d={} > radius", d); // rounding slack
        }
    }

    // ---------- Display ------------------------------------------------------
    // We don't assert the exact formatting (since DisplayFields may evolve),
    // but we ensure it contains x and y values.

    #[test]
    fn display_contains_coords() {
        let p = point(12, -34);
        let s = p.to_string();
        assert!(s.contains("12"));
        assert!(s.contains("-34"));
    }
}
