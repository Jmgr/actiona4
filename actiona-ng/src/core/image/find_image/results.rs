use std::cmp::Ordering;

use color_eyre::Result;
use opencv::core::{
    Mat, MatTraitConst, MatTraitConstManual, MatTraitManual, NORM_L2, Point, Rect, Size,
    ToInputArray, count_non_zero, min_max_loc, no_array, norm2, norm2_def,
};
use tracing::instrument;

use crate::core::image::find_image::Match;

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
    if search_one {
        // Fast path for "best match only": use OpenCV's min/max finder.
        let mut max_val = 0.;
        let mut max_loc = Point::default();

        min_max_loc(
            &match_template_result,
            None,
            Some(&mut max_val),
            None,
            Some(&mut max_loc),
            &no_array(),
        )?;

        if max_val >= match_threshold.into() {
            #[allow(clippy::as_conversions)]
            return Ok(vec![Match::new(
                max_loc.into(),
                Rect::from_point_size(max_loc, template_size).into(),
                max_val,
            )]);
        }

        return Ok(vec![]);
    }

    // Collect all matches above the threshold.
    let mut match_points = Vec::new();
    let rows = match_template_result.rows();
    let cols = match_template_result.cols();
    let mut push_match = |row: i32, col: i32, match_score: f32| {
        if match_score < match_threshold {
            return;
        }
        let position = Point::new(col, row);
        match_points.push(Match::new(
            position.into(),
            Rect::from_point_size(position, template_size).into(),
            match_score.into(),
        ));
    };

    if match_template_result.is_continuous() {
        // Fast path: scan the raw (row-major) float buffer without per-pixel FFI calls.
        let values = match_template_result.data_typed::<f32>()?;
        for (idx, &match_score) in values.iter().enumerate() {
            #[allow(clippy::as_conversions)]
            let idx = idx as i32;
            let row = idx / cols;
            let col = idx - row * cols;
            push_match(row, col, match_score);
        }
    } else {
        // Fallback: some Mats are not contiguous; scan row slices.
        for row in 0..rows {
            let row_values = match_template_result.at_row::<f32>(row)?;
            for (col, &match_score) in row_values.iter().enumerate() {
                #[allow(clippy::as_conversions)]
                let col = col as i32;
                push_match(row, col, match_score);
            }
        }
    }

    // Sort matches by score (in descending order) so NMS keeps best-first.
    match_points.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

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
        filtered.push(candidate.clone());
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
    source_a: &Mat,
    source_b: &Mat,
    template_a: &Mat,
    template_b: &Mat,
    template_mask: Option<&Mat>,
    template_size: Size,
    match_threshold: f32,
) -> Result<()> {
    const CHROMA_RMS_THRESHOLD: f64 = 8.0;

    let valid_pixel_count = match template_mask {
        Some(mask) => count_non_zero(mask)? as f64,
        None => (template_size.width * template_size.height) as f64,
    }
    .max(1.0);
    let normalization = valid_pixel_count.sqrt();

    let rows = result.rows();
    for row in 0..rows {
        let row_values = result.at_row_mut::<f32>(row)?;
        for (col_idx, value) in row_values.iter_mut().enumerate() {
            if *value < match_threshold {
                continue;
            }
            #[allow(clippy::as_conversions)]
            let col = col_idx as i32;
            let roi = Rect::new(col, row, template_size.width, template_size.height);
            let source_a_roi = source_a.roi(roi)?;
            let source_b_roi = source_b.roi(roi)?;

            let rms_a = channel_rms(&source_a_roi, &template_a, template_mask, normalization)?;
            let rms_b = channel_rms(&source_b_roi, &template_b, template_mask, normalization)?;
            let combined_rms = (rms_a * rms_a + rms_b * rms_b).sqrt();

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
    mask: Option<&Mat>,
    normalization: f64,
) -> Result<f64> {
    let norm = if let Some(mask) = mask {
        norm2(source, template, NORM_L2, mask)?
    } else {
        norm2_def(source, template)?
    };

    Ok(norm / normalization)
}
