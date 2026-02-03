use std::borrow::Cow;

use color_eyre::Result;
use opencv::{
    core::{Mat, Size},
    imgproc::{self, INTER_NEAREST},
};
use tracing::instrument;

/// Downscale inputs by the requested pyramid depth to speed up matching.
///
/// Each level uses OpenCV's pyrDown (Gaussian blur + 2x downscale) so the
/// template and source remain aligned across resolutions.
///
/// When `downscale == 0`, returns borrowed references to the inputs
/// without any processing.
#[instrument(skip_all)]
pub fn prepare_matching_inputs<'a>(
    source_lightness: Cow<'a, Mat>,
    template_lightness: Cow<'a, Mat>,
    template_mask: Option<Cow<'a, Mat>>,
    downscale: u64,
) -> Result<(Cow<'a, Mat>, Cow<'a, Mat>, Option<Cow<'a, Mat>>)> {
    if downscale == 0 {
        return Ok((source_lightness, template_lightness, template_mask));
    }

    let mut current_source = source_lightness.into_owned();
    let mut current_template = template_lightness.into_owned();
    let mut current_mask = template_mask.map(Cow::into_owned);

    for _ in 0..downscale {
        current_source = downscale_for_pyramid(&current_source)?;
        current_template = downscale_for_pyramid(&current_template)?;
        if let Some(mask_mat) = current_mask.take() {
            current_mask = Some(downscale_for_pyramid(&mask_mat)?);
        }
    }

    Ok((
        Cow::Owned(current_source),
        Cow::Owned(current_template),
        current_mask.map(Cow::Owned),
    ))
}

/// One pyramid step (blur + downscale) used by OpenCV's image pyramids.
#[instrument(skip_all)]
fn downscale_for_pyramid(input: &Mat) -> Result<Mat> {
    let mut output = Mat::default();

    imgproc::pyr_down_def(input, &mut output)?;

    Ok(output)
}

/// Upscale a match result map back to the target resolution.
///
/// Nearest-neighbor keeps the discrete score grid aligned to the original
/// pixel lattice.
#[instrument(skip_all)]
pub fn resize_result_to_size(result: &Mat, target_size: Size) -> Result<Mat> {
    let mut resized = Mat::default();

    imgproc::resize(result, &mut resized, target_size, 0.0, 0.0, INTER_NEAREST)?;

    Ok(resized)
}
