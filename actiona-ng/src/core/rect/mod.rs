use std::fmt::Display;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use super::point::{Point, point};
use crate::{
    core::size::{Size, size},
    types::DisplayFields,
};

pub mod js;

#[derive(Clone, Constructor, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

pub fn rect(origin: Point, size: Size) -> Rect {
    Rect::new(origin, size)
}

impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("x", self.origin.x)
            .display("y", self.origin.y)
            .display("width", self.size.width)
            .display("height", self.size.height)
            .finish(f)
    }
}

impl Rect {
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    pub const fn contains(&self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x < self.origin.x + self.size.width as i32
            && point.y >= self.origin.y
            && point.y < self.origin.y + self.size.height as i32
    }

    pub const fn intersects(&self, other: Self) -> bool {
        !(self.origin.x + self.size.width as i32 <= other.origin.x
            || other.origin.x + other.size.width as i32 <= self.origin.x
            || self.origin.y + self.size.height as i32 <= other.origin.y
            || other.origin.y + other.size.height as i32 <= self.origin.y)
    }

    pub fn intersection(&self, other: Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        let x1 = self.origin.x.max(other.origin.x);
        let y1 = self.origin.y.max(other.origin.y);
        let x2 =
            (self.origin.x + self.size.width as i32).min(other.origin.x + other.size.width as i32);
        let y2 = (self.origin.y + self.size.height as i32)
            .min(other.origin.y + other.size.height as i32);

        Some(Self {
            origin: point(x1, y1),
            size: size((x2 - x1) as u32, (y2 - y1) as u32),
        })
    }

    pub fn union(&self, other: Self) -> Self {
        let x1 = self.origin.x.min(other.origin.x);
        let y1 = self.origin.y.min(other.origin.y);
        let x2 =
            (self.origin.x + self.size.width as i32).max(other.origin.x + other.size.width as i32);
        let y2 = (self.origin.y + self.size.height as i32)
            .max(other.origin.y + other.size.height as i32);

        Self {
            origin: point(x1, y1),
            size: size((x2 - x1) as u32, (y2 - y1) as u32),
        }
    }

    pub fn clamped(&self) -> (u32, u32, u32, u32) {
        let clamped_x = self.origin.x.max(0) as u32;
        let clamped_y = self.origin.y.max(0) as u32;

        let adjusted_width = if self.origin.x < 0 {
            self.size.width.saturating_sub(self.origin.x.unsigned_abs())
        } else {
            self.size.width
        };

        let adjusted_height = if self.origin.y < 0 {
            self.size
                .height
                .saturating_sub(self.origin.y.unsigned_abs())
        } else {
            self.size.height
        };

        (clamped_x, clamped_y, adjusted_width, adjusted_height)
    }

    pub fn center(&self) -> Point {
        point(
            self.origin.x + self.size.width as i32 / 2,
            self.origin.y + self.size.height as i32 / 2,
        )
    }

    pub fn top_left(&self) -> Point {
        point(self.origin.x, self.origin.y)
    }

    pub fn bottom_right(&self) -> Point {
        point(
            self.origin.x + self.size.width as i32,
            self.origin.y + self.size.height as i32,
        )
    }

    pub fn size(&self) -> Size {
        size(self.size.width, self.size.height)
    }

    pub const fn surface(&self) -> u32 {
        self.size.width * self.size.height
    }
}

impl From<Rect> for imageproc::rect::Rect {
    fn from(value: Rect) -> Self {
        Self::at(value.origin.x, value.origin.y).of_size(value.size.width, value.size.height)
    }
}
