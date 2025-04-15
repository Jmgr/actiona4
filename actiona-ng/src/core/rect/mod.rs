use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use super::point::{Point, point};

pub mod js;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn rect<X: ToPrimitive, Y: ToPrimitive, Width: ToPrimitive, Height: ToPrimitive>(
    x: X,
    y: Y,
    width: Width,
    height: Height,
) -> Rect {
    Rect::new(
        x.to_i32().unwrap_or(0),
        y.to_i32().unwrap_or(0),
        width.to_u32().unwrap_or(0),
        height.to_u32().unwrap_or(0),
    )
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    pub const fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width as i32
            && point.y >= self.y
            && point.y < self.y + self.height as i32
    }

    pub const fn intersects(&self, other: Self) -> bool {
        !(self.x + self.width as i32 <= other.x
            || other.x + other.width as i32 <= self.x
            || self.y + self.height as i32 <= other.y
            || other.y + other.height as i32 <= self.y)
    }

    pub fn intersection(&self, other: Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width as i32).min(other.x + other.width as i32);
        let y2 = (self.y + self.height as i32).min(other.y + other.height as i32);

        Some(Self {
            x: x1,
            y: y1,
            width: (x2 - x1) as u32,
            height: (y2 - y1) as u32,
        })
    }

    pub fn union(&self, other: Self) -> Self {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.width as i32).max(other.x + other.width as i32);
        let y2 = (self.y + self.height as i32).max(other.y + other.height as i32);

        Self {
            x: x1,
            y: y1,
            width: (x2 - x1) as u32,
            height: (y2 - y1) as u32,
        }
    }

    pub fn clamped(&self) -> (u32, u32, u32, u32) {
        let clamped_x = self.x.max(0) as u32;
        let clamped_y = self.y.max(0) as u32;

        let adjusted_width = if self.x < 0 {
            self.width.saturating_sub(self.x.unsigned_abs())
        } else {
            self.width
        };

        let adjusted_height = if self.y < 0 {
            self.height.saturating_sub(self.y.unsigned_abs())
        } else {
            self.height
        };

        (clamped_x, clamped_y, adjusted_width, adjusted_height)
    }

    pub fn center(&self) -> Point {
        point(
            self.x + self.width as i32 / 2,
            self.y + self.height as i32 / 2,
        )
    }

    pub const fn surface(&self) -> u32 {
        self.width * self.height
    }
}

impl From<Rect> for imageproc::rect::Rect {
    fn from(value: Rect) -> Self {
        Self::at(value.x, value.y).of_size(value.width, value.height)
    }
}
