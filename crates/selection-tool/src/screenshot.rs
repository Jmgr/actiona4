use color_eyre::Result;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub async fn capture_screenshot() -> Result<Screenshot> {
    let screen = ::screenshot::Screen::new(TaskTracker::new(), CancellationToken::new()).await?;
    let capture = screen.capture_full_screen().await?;
    Ok(Screenshot {
        width: capture.width,
        height: capture.height,
        rgba: capture.into_rgba(),
    })
}

#[allow(clippy::as_conversions)]
pub fn screenshot_color_at(
    screenshot: &Screenshot,
    x_position: i32,
    y_position: i32,
) -> Option<[u8; 3]> {
    if x_position < 0
        || y_position < 0
        || x_position as u32 >= screenshot.width
        || y_position as u32 >= screenshot.height
    {
        return None;
    }

    let pixel_index = ((y_position as u32 * screenshot.width + x_position as u32) * 4) as usize;
    Some([
        screenshot.rgba[pixel_index],
        screenshot.rgba[pixel_index + 1],
        screenshot.rgba[pixel_index + 2],
    ])
}
