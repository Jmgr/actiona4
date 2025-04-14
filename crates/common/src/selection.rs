use serde::Serialize;

#[derive(Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Serialize)]
pub struct PositionSelection {
    pub x: i32,
    pub y: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

#[derive(Serialize)]
pub struct RectSelection {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
