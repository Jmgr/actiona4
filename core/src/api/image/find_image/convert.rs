use color_eyre::Result;
use opencv::{
    core::{Mat, ToInputArray, ToOutputArray},
    imgproc::cvt_color,
};
use tracing::instrument;

/// Convert a Mat to a different color space using OpenCV's cvtColor.
///
/// The feature-flagged macro enables algorithm hints on newer OpenCV builds.
#[instrument(skip(source))]
pub fn convert_colors(source: &impl ToInputArray, conversion_code: i32) -> Result<Mat> {
    let mut result = Mat::default();
    convert_colors_into(source, &mut result, conversion_code)?;
    Ok(result)
}

/// Convert a Mat to a different color space, writing into a pre-allocated destination Mat.
///
/// This avoids allocating a new Mat on each call, which is useful in hot paths
/// where the destination buffer can be reused.
#[instrument(skip(source, destination))]
pub fn convert_colors_into(
    source: &impl ToInputArray,
    destination: &mut impl ToOutputArray,
    conversion_code: i32,
) -> Result<()> {
    // Note: using an anonymous function because this is a requirement to compile on Windows when using this macro.
    #[allow(clippy::redundant_closure_call)]
    (|| {
        opencv::opencv_has_inherent_feature_algorithm_hint! {
            {
                cvt_color(source, destination, conversion_code, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
            } else {
                cvt_color(source, destination, conversion_code, 0)
            }
        }
    })()?;

    Ok(())
}
