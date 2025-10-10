use std::fmt::Display;

use derive_more::{Add, Constructor, Mul, Sub};
use eyre::{Error, OptionExt, Result, eyre};
use num_traits::{Float, ToPrimitive};
use serde::{Deserialize, Serialize};

use crate::types::DisplayFields;

pub mod js;

#[derive(
    Add, Clone, Copy, Debug, Default, Deserialize, Eq, Mul, PartialEq, Serialize, Sub, Constructor,
)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

pub const fn size(width: u32, height: u32) -> Size {
    Size::new(width, height)
}

pub fn try_size<W, H>(width: W, height: H) -> Result<Size>
where
    W: ToPrimitive + Display,
    H: ToPrimitive + Display,
{
    Ok(Size::new(
        width
            .to_u32()
            .ok_or_else(|| eyre!("{width} cannot be converted into an unsigned integer"))?,
        height
            .to_u32()
            .ok_or_else(|| eyre!("{height} cannot be converted into an unsigned integer"))?,
    ))
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
