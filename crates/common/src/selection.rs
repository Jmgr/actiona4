use serde::Serialize;
use types::{point::Point, size::Size};

#[derive(Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize)]
pub struct PositionSelection {
    #[serde(flatten)]
    pub point: Point,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

#[derive(Serialize)]
pub struct RectSelection {
    #[serde(flatten)]
    pub top_left: Point,
    #[serde(flatten)]
    pub size: Size,
}
