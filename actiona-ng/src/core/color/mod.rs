//! Represents a color.

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use image::Rgba;

use crate::types::DisplayFields;

pub mod js;

/// Color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Rgba([r, g, b, a]))
    }
}

impl From<Color> for Rgba<u8> {
    fn from(value: Color) -> Self {
        value.0
    }
}

impl From<Rgba<u8>> for Color {
    fn from(value: Rgba<u8>) -> Self {
        Self(value)
    }
}

impl Deref for Color {
    type Target = Rgba<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
