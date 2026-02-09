use std::borrow::Cow;

use color_eyre::Result;
use opencv::{
    core::{Mat, Size},
    imgproc::{self, INTER_NEAREST},
};
use tracing::instrument;

use crate::api::image::find_image::{LabLightnessMat, MaskMat};

/// Downscale inputs by the requested pyramid depth to speed up matching.
///
/// Each level uses OpenCV's pyrDown (Gaussian blur + 2x downscale) so the
/// template and source remain aligned across resolutions.
///
/// When `downscale == 0`, returns borrowed references to the inputs
/// without any processing.
#[allow(clippy::type_complexity)]
#[instrument(skip_all)]
pub fn prepare_matching_inputs<'a>(
    source_lightness: Cow<'a, LabLightnessMat>,
    template_lightness: Cow<'a, LabLightnessMat>,
    template_mask: Option<Cow<'a, MaskMat>>,
    downscale: u64,
) -> Result<(
    Cow<'a, LabLightnessMat>,
    Cow<'a, LabLightnessMat>,
    Option<Cow<'a, MaskMat>>,
)> {
    if downscale == 0 {
        return Ok((source_lightness, template_lightness, template_mask));
    }

    let mut current_source = source_lightness.into_owned();
    let mut current_template = template_lightness.into_owned();
    let mut current_mask = template_mask.map(Cow::into_owned);

    for _ in 0..downscale {
        current_source = LabLightnessMat(downscale_for_pyramid(&current_source.0)?);
        current_template = LabLightnessMat(downscale_for_pyramid(&current_template.0)?);
        if let Some(mask_mat) = current_mask.take() {
            current_mask = Some(MaskMat(downscale_for_pyramid(&mask_mat.0)?));
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
