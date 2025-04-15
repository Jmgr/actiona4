use std::{f32::consts::TAU, fmt::Display};

use derive_more::{Add, Mul, Sub};
use num_traits::ToPrimitive;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tween::TweenValue;

pub mod js;

#[derive(Add, Clone, Copy, Debug, Deserialize, Mul, PartialEq, Eq, Serialize, Sub)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn point<X: ToPrimitive, Y: ToPrimitive>(x: X, y: Y) -> Point {
    Point {
        x: x.to_i32().unwrap_or(0),
        y: y.to_i32().unwrap_or(0),
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
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x as f32).hypot(self.y as f32)
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0. {
            Self {
                x: (self.x as f32 / len).round() as i32,
                y: (self.y as f32 / len).round() as i32,
            }
        } else {
            Self { x: 0, y: 0 }
        }
    }

    pub fn random_in_circle(center: Self, radius: f32) -> Self {
        let mut rng = rand::rng();

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

    pub fn scale(&self, factor: f32) -> Self {
        Self {
            x: (self.x as f32 * factor).round() as i32,
            y: (self.y as f32 * factor).round() as i32,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.x, self.y)
    }
}
