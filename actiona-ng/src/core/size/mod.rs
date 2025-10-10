use std::fmt::Display;

use derive_more::{Add, Mul, Sub};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::types::DisplayFields;

pub mod js;

#[derive(Add, Clone, Copy, Debug, Default, Deserialize, Eq, Mul, PartialEq, Serialize, Sub)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub fn size<Width: ToPrimitive, Height: ToPrimitive>(width: Width, height: Height) -> Size {
    Size {
        width: width.to_u32().unwrap_or(0),
        height: height.to_u32().unwrap_or(0),
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("width", self.width)
            .display("height", self.height)
            .finish(f)
    }
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn length(&self) -> f32 {
        (self.width as f32).hypot(self.height as f32)
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0. {
            Self {
                width: (self.width as f32 / len).round() as u32,
                height: (self.height as f32 / len).round() as u32,
            }
        } else {
            Self {
                width: 0,
                height: 0,
            }
        }
    }

    pub fn distance_to(&self, other: Self) -> f32 {
        ((self.width - other.width) as f32).hypot((self.height - other.height) as f32)
    }

    pub const fn is_origin(&self) -> bool {
        self.width == 0 && self.height == 0
    }

    pub fn distance(a: Self, b: Self) -> f32 {
        a.distance_to(b)
    }

    pub fn scale(&self, factor: f32) -> Self {
        Self {
            width: (self.width as f32 * factor).round() as u32,
            height: (self.height as f32 * factor).round() as u32,
        }
    }
}
