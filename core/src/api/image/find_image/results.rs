use std::cmp::Ordering;

use color_eyre::Result;
use opencv::core::{
    Mat, MatTraitConst, MatTraitConstManual, MatTraitManual, NORM_L2, Point, Rect, Size,
    ToInputArray, count_non_zero, norm2, norm2_def,
};
use tracing::instrument;

use crate::{
    api::{
        image::find_image::{LabAMat, LabBMat, MaskMat, Match},
        rect,
    },
    types::si32::Si32,
};

/// Convert a match-score matrix into match locations.
///
/// This scans the matchTemplate output, applies a threshold, optionally
/// keeps only the best match, and can run non-maximum suppression to
/// drop overlapping detections.
#[instrument(skip_all)]
pub fn compute_results(
    match_template_result: &Mat,
    template_size: Size,
    match_threshold: f32,
    search_one: bool,
    non_maximum_suppression_radius: Option<i32>,
) -> Result<Vec<Match>> {
    // Collect all finite matches above the threshold.
    // For search_one we still use this path to avoid duplicating the scan logic,
    // then just keep the best result. (OpenCV min_max_loc can return NaN/Inf for
    // masked/flat regions, so we skip non-finite values while scanning.)
    let mut match_points = Vec::new();
    let rows = match_template_result.rows();
    let cols = match_template_result.cols();
    let mut push_match = |row: i32, col: i32, match_score: f32| {
        if !match_score.is_finite() || match_score < match_threshold {
            return;
        }
        let position = Point::new(col, row);
        let rect: rect::Rect = Rect::from_point_size(position, template_size).into();
        match_points.push(Match::new(rect.center(), rect, match_score.into()));
    };

    if match_template_result.is_continuous() {
        // Fast path: scan the raw (row-major) float buffer without per-pixel FFI calls.
        let values = match_template_result.data_typed::<f32>()?;
        for (idx, &match_score) in values.iter().enumerate() {
            let idx: i32 = Si32::from(idx).into();
            let row = idx / cols;
            let col = idx - row * cols;
            push_match(row, col, match_score);
        }
    } else {
        // Fallback: some Mats are not contiguous; scan row slices.
        for row in 0..rows {
            let row_values = match_template_result.at_row::<f32>(row)?;
            for (col, &match_score) in row_values.iter().enumerate() {
                let col: i32 = Si32::from(col).into();
                push_match(row, col, match_score);
            }
        }
    }

    // Sort by score descending so NMS (and search_one) keeps the best first.
    match_points.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

    if search_one {
        match_points.truncate(1);
        return Ok(match_points);
    }

    let matches = if let Some(non_maximum_suppression_radius) = non_maximum_suppression_radius {
        non_maximum_suppression(&match_points, non_maximum_suppression_radius)
    } else {
        match_points
    };

    Ok(matches)
}

/// Simple non-maximum suppression using a square radius in pixel space.
///
/// Assumes input is sorted by score descending; first match wins and
/// suppresses any later match within `radius` in both x and y.
#[instrument(skip_all)]
fn non_maximum_suppression(input: &[Match], radius: i32) -> Vec<Match> {
    let mut filtered: Vec<Match> = Vec::new();

    'candidates: for candidate in input {
        for existing in &filtered {
            if (candidate.position.x - existing.position.x).abs() < radius
                && (candidate.position.y - existing.position.y).abs() < radius
            {
                continue 'candidates;
            }
        }
        filtered.push(*candidate);
    }

    filtered
}

/// Apply a chroma (Lab a/b) RMS filter to reduce false positives.
///
/// For each candidate score above threshold, compute the RMS distance
/// between template and source in the a/b channels, optionally masked.
/// Scores whose chroma deviation exceeds the threshold are zeroed out.
#[instrument(skip_all)]
pub fn filter_results_by_color(
    result: &mut Mat,
    source_a: &LabAMat,
    source_b: &LabBMat,
    template_a: &LabAMat,
    template_b: &LabBMat,
    template_mask: Option<&MaskMat>,
    template_size: Size,
    match_threshold: f32,
) -> Result<()> {
    const CHROMA_RMS_THRESHOLD: f64 = 8.0;

    let valid_pixel_count = match template_mask {
        Some(mask) => f64::from(count_non_zero(&mask.0)?),
        None => f64::from(template_size.width * template_size.height),
    }
    .max(1.0);
    let normalization = valid_pixel_count.sqrt();

    let rows = result.rows();
    for row in 0..rows {
        let row_values = result.at_row_mut::<f32>(row)?;
        for (col_idx, value) in row_values.iter_mut().enumerate() {
            if !value.is_finite() {
                *value = 0.0;
                continue;
            }
            if *value < match_threshold {
                continue;
            }
            let col: i32 = Si32::from(col_idx).into();
            let roi = Rect::new(col, row, template_size.width, template_size.height);
            let source_a_roi = source_a.0.roi(roi)?;
            let source_b_roi = source_b.0.roi(roi)?;

            let rms_a = channel_rms(&source_a_roi, &template_a.0, template_mask, normalization)?;
            let rms_b = channel_rms(&source_b_roi, &template_b.0, template_mask, normalization)?;
            let combined_rms = rms_a.hypot(rms_b);

            if combined_rms > CHROMA_RMS_THRESHOLD {
                *value = 0.0;
            }
        }
    }

    Ok(())
}

/// Compute RMS distance between two channels, optionally with a mask.
fn channel_rms(
    source: &impl ToInputArray,
    template: &impl ToInputArray,
    mask: Option<&MaskMat>,
    normalization: f64,
) -> Result<f64> {
    let norm = if let Some(mask) = mask {
        norm2(source, template, NORM_L2, &mask.0)?
    } else {
        norm2_def(source, template)?
    };

    Ok(norm / normalization)
}

#[cfg(test)]
mod tests {
    use color_eyre::Result;
    use opencv::{
        core::{Mat, Size},
        prelude::MatTraitConst,
    };

    use crate::api::image::find_image::results::compute_results;

    fn f32_mat(values: &[f32], rows: i32) -> Result<Mat> {
        let mat_boxed = Mat::from_slice(values)?;
        let mat_reshaped = mat_boxed.reshape(1, rows)?;
        Ok(mat_reshaped.try_clone()?)
    }

    #[test]
    fn compute_results_ignores_non_finite_scores_find_all() {
        let result = f32_mat(&[f32::NAN, f32::INFINITY, 0.9, 0.2], 1).unwrap();
        let matches = compute_results(&result, Size::new(1, 1), 0.8, false, None).unwrap();

        assert_eq!(matches.len(), 1);
        assert!(matches[0].score.is_finite());
        assert!((matches[0].score - 0.9).abs() < 1e-6);
    }

    #[test]
    fn compute_results_ignores_non_finite_scores_find_one() {
        let result = f32_mat(&[f32::INFINITY], 1).unwrap();
        let matches = compute_results(&result, Size::new(1, 1), 0.0, true, None).unwrap();

        assert!(matches.is_empty());
    }

    #[test]
    fn compute_results_find_one_picks_best_finite_score() {
        let result = f32_mat(&[f32::INFINITY, f32::NAN, 0.4, 0.9, 0.8], 1).unwrap();
        let matches = compute_results(&result, Size::new(1, 1), 0.8, true, None).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].position.x, 3);
        assert_eq!(matches[0].position.y, 0);
        assert!((matches[0].score - 0.9).abs() < 1e-6);
    }
}
