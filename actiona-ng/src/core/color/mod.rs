//! Represents a color.

use std::fmt::Display;

use derive_more::{Deref, DerefMut, From, Into};
use image::Rgba;

use crate::types::DisplayFields;

pub mod js;

/// Color.
#[derive(Clone, Copy, Debug, Deref, DerefMut, Eq, From, Into, PartialEq)]
pub struct Color(Rgba<u8>);

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("r", self.0[0])
            .display("g", self.0[1])
            .display("b", self.0[2])
            .display("a", self.0[3])
            .finish(f)
    }
}

impl Color {
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Rgba([r, g, b, a]))
    }
}
