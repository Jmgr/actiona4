use std::fmt::Display;

use derive_more::{Add, Constructor, Mul, Sub};
use eyre::{Result, eyre};
use num_traits::ToPrimitive;
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

#[must_use]
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
    #[must_use]
    pub fn length(&self) -> f64 {
        f64::from(self.width).hypot(f64::from(self.height))
    }

    #[must_use]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len > 0. {
            Self {
                width: (f64::from(self.width) / len)
                    .round()
                    .to_u32()
                    .unwrap_or_default(),
                height: (f64::from(self.height) / len)
                    .round()
                    .to_u32()
                    .unwrap_or_default(),
            }
        } else {
            Self {
                width: 0,
                height: 0,
            }
        }
    }

    #[must_use]
    pub fn distance_to(&self, other: Self) -> f64 {
        f64::from(self.width - other.width).hypot(f64::from(self.height - other.height))
    }

    #[must_use]
    pub const fn is_origin(&self) -> bool {
        self.width == 0 && self.height == 0
    }

    #[must_use]
    pub fn distance(a: Self, b: Self) -> f64 {
        a.distance_to(b)
    }

    #[must_use]
    pub fn scaled(&self, factor: f64) -> Self {
        Self {
            width: (f64::from(self.width) * factor)
                .round()
                .to_u32()
                .unwrap_or_default(),
            height: (f64::from(self.height) * factor)
                .round()
                .to_u32()
                .unwrap_or_default(),
        }
    }

    pub(crate) fn as_i64(&self) -> (i64, i64) {
        (self.width.into(), self.height.into())
    }

    pub(crate) fn as_f64(&self) -> (f64, f64) {
        (self.width.into(), self.height.into())
    }
}
