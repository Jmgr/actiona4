use color_eyre::Result;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use types::size::Size;

pub struct Screenshot {
    pub size: Size,
    pub rgba: Vec<u8>,
}

pub async fn capture_screenshot() -> Result<Screenshot> {
    let screen = ::screenshot::Screen::new(TaskTracker::new(), CancellationToken::new()).await?;
    let capture = screen.capture_full_screen().await?;
    Ok(Screenshot {
        size: capture.size,
        rgba: capture.into_rgba(),
    })
}
