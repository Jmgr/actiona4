use std::{
    f64::consts::TAU,
    fmt::Display,
    ops::{Div, DivAssign, Mul, MulAssign},
};

use derive_more::{Add, AddAssign, Constructor, Neg, Sub, SubAssign};
use eyre::{Result, eyre};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use tween::TweenValue;

use crate::{core::ToIntClamped, runtime::shared_rng::SharedRng, types::DisplayFields};

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
    pub x: i32,
    pub y: i32,
}

#[must_use]
pub const fn point(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

pub fn try_point<X, Y>(x: X, y: Y) -> Result<Point>
where
    X: ToPrimitive + Display,
    Y: ToPrimitive + Display,
{
    Ok(Point::new(
        x.to_i32()
            .ok_or_else(|| eyre!("{x} cannot be converted into an integer"))?,
        y.to_i32()
            .ok_or_else(|| eyre!("{y} cannot be converted into an integer"))?,
    ))
}

impl Mul<i32> for Point {
    type Output = Self;
    #[inline]
    fn mul(self, k: i32) -> Self {
        Self::new(self.x * k, self.y * k)
    }
}
impl MulAssign<i32> for Point {
    #[inline]
    fn mul_assign(&mut self, k: i32) {
        self.x *= k;
        self.y *= k;
    }
}
impl Div<i32> for Point {
    type Output = Self;
    #[inline]
    fn div(self, k: i32) -> Self {
        Self::new(self.x / k, self.y / k)
    }
}
impl DivAssign<i32> for Point {
    #[inline]
    fn div_assign(&mut self, k: i32) {
        self.x /= k;
        self.y /= k;
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

        point(
            (x * scale).to_i32_clamped().unwrap_or_default(),
            (y * scale).to_i32_clamped().unwrap_or_default(),
        )
    }
}

impl Point {
    pub const ZERO: Self = Self::new(0, 0);

    #[must_use]
    pub const fn dot_product(self, other: Self) -> i32 {
        self.x
            .saturating_mul(other.x)
            .saturating_add(self.y.saturating_mul(other.y))
    }

    #[must_use]
    pub const fn cross_product(self, other: Self) -> i32 {
        self.x
            .saturating_mul(other.y)
            .saturating_sub(self.y.saturating_mul(other.x))
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
    pub const fn length_squared(&self) -> i32 {
        self.x
            .saturating_mul(self.x)
            .saturating_add(self.y.saturating_mul(self.y))
    }

    #[must_use]
    pub fn random_in_circle(center: Self, radius: f64, rng: SharedRng) -> Result<Self> {
        let (center_x, center_y) = center.as_f64();
        let theta = rng.random_range(0.0..TAU);
        let r = radius * rng.random::<f64>().sqrt();
        let x = r.mul_add(theta.cos(), center_x).to_i32_clamped()?;
        let y = r.mul_add(theta.sin(), center_y).to_i32_clamped()?;

        Ok(Self { x, y })
    }

    #[must_use]
    pub fn distance_to(&self, other: Self) -> f64 {
        let (x, y) = self.as_f64();
        let (other_x, other_y) = other.as_f64();

        (x - other_x).hypot(y - other_y)
    }

    #[must_use]
    pub const fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    #[must_use]
    pub fn distance(a: Self, b: Self) -> f64 {
        a.distance_to(b)
    }

    #[must_use]
    pub fn scaled(&self, factor: f64) -> Result<Self> {
        let (x, y) = self.as_f64();

        Ok(Self {
            x: (x * factor).to_i32_clamped()?,
            y: (y * factor).to_i32_clamped()?,
        })
    }

    #[must_use]
    pub fn clamped(&self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }

    pub(crate) fn as_i64(&self) -> (i64, i64) {
        (self.x.into(), self.y.into())
    }

    pub(crate) fn as_f64(&self) -> (f64, f64) {
        (self.x.into(), self.y.into())
    }
}
