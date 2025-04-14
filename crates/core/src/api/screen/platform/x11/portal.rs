use color_eyre::{Result, eyre::eyre};
use url::Url;

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

    let uri = screenshot.uri().as_str();
    let path = Url::parse(uri)
        .map_err(|e| eyre!("portal returned invalid URI `{uri}`: {e}"))?
        .to_file_path()
        .map_err(|_| eyre!("portal returned non-file URI: {uri}"))?;
    let image = Image::load(path).await?;
    Ok(Some(image))
}
