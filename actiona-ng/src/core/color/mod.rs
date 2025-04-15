//! Represents a color.

use std::ops::{Deref, DerefMut};

use image::Rgba;

pub mod js;

/// Color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(Rgba<u8>);

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
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
