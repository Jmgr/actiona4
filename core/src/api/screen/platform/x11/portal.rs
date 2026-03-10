use color_eyre::{Result, eyre::eyre};

use crate::api::image::Image;

/// Asks the user to interactively select a screen area via the XDG Desktop
/// Portal screenshot interface.
///
/// Returns `Ok(None)` when the user cancels the dialog.
pub async fn ask_screenshot() -> Result<Option<Image>> {
    use ashpd::{
        Error,
        desktop::{ResponseError, screenshot::Screenshot},
    };

    let result = Screenshot::request().interactive(true).send().await;

    let screenshot = match result {
        Ok(request) => match request.response() {
            Ok(r) => r,
            Err(Error::Response(ResponseError::Cancelled)) => return Ok(None),
            Err(e) => return Err(eyre!("portal screenshot failed: {e}")),
        },
        Err(e) => return Err(eyre!("portal request failed: {e}")),
    };

    let image = Image::load(screenshot.uri().as_str()).await?;
    Ok(Some(image))
}
