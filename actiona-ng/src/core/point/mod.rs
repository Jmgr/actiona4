use std::{
    f32::consts::TAU,
    fmt::Display,
    ops::{Div, DivAssign, Mul, MulAssign},
};

use derive_more::{Add, AddAssign, Constructor, Neg, Sub, SubAssign};
use eyre::{Error, OptionExt, Result, bail, eyre};
use num_traits::{Float, PrimInt, ToPrimitive};
use serde::{Deserialize, Serialize};
use tween::TweenValue;

use crate::{runtime::shared_rng::SharedRng, types::DisplayFields};

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
    fn scale(self, scale: f32) -> Self {
        point(
            (self.x as f32 * scale).round() as i32,
            (self.y as f32 * scale).round() as i32,
        )
    }
}

impl Point {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn dot_product(self, other: Self) -> i32 {
        self.x * other.x + self.y * other.y
    }

    pub const fn cross_product(self, other: Self) -> i32 {
        self.x * other.y - self.y * other.x
    }

    pub fn normalize(self) -> (f32, f32) {
        let len = self.length();
        if len > 0. {
            ((self.x as f32 / len), (self.y as f32 / len))
        } else {
            (0., 0.)
        }
    }

    pub fn length(&self) -> f32 {
        (self.x as f32).hypot(self.y as f32)
    }

    pub fn length_squared(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }

    pub fn random_in_circle(center: Self, radius: f32, rng: SharedRng) -> Self {
        let theta = rng.random_range(0.0..TAU);
        let r = radius * rng.random::<f32>().sqrt();
        let x = r.mul_add(theta.cos(), center.x as f32).round() as i32;
        let y = r.mul_add(theta.sin(), center.y as f32).round() as i32;

        Self { x, y }
    }

    pub fn distance_to(&self, other: Self) -> f32 {
        ((self.x - other.x) as f32).hypot((self.y - other.y) as f32)
    }

    pub const fn is_origin(&self) -> bool {
        self.x == 0 && self.y == 0
    }

    pub fn distance(a: Self, b: Self) -> f32 {
        a.distance_to(b)
    }

    pub fn scaled(&self, factor: f32) -> Self {
        Self {
            x: (self.x as f32 * factor).round() as i32,
            y: (self.y as f32 * factor).round() as i32,
        }
    }

    pub fn clamped(&self, min: Self, max: Self) -> Self {
        Self {
            x: self.x.clamp(min.x, max.x),
            y: self.y.clamp(min.y, max.y),
        }
    }
}
