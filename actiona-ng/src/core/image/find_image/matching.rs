use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use color_eyre::{Result, eyre::ensure};
use itertools::Itertools;
use opencv::{
    core::{CV_32FC1, Mat, Rect, no_array},
    imgproc::{TM_CCOEFF_NORMED, match_template as cv_match_template},
    prelude::{MatTraitConst, MatTraitConstManual, MatTraitManual},
};
use rayon::prelude::*;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{
    core::image::find_image::{
        FindImageProgress, FindImageStage, LabLightnessMat, MaskMat, common::ideal_thread_count,
    },
    error::CommonError,
};

/// Run a single tile's template match against a vertical slice of the source.
fn match_tile(
    source_lightness: &LabLightnessMat,
    template_lightness: &LabLightnessMat,
    template_mask: Option<&MaskMat>,
    roi: Rect,
) -> Result<Mat> {
    let source_roi = source_lightness.0.roi(roi)?;
    let mut tile_result = Mat::default();

    if let Some(mask) = template_mask {
        cv_match_template(
            &source_roi,
            &template_lightness.0,
            &mut tile_result,
            TM_CCOEFF_NORMED,
            &mask.0,
        )?;
    } else {
        cv_match_template(
            &source_roi,
            &template_lightness.0,
            &mut tile_result,
            TM_CCOEFF_NORMED,
            &no_array(),
        )?;
    }
    Ok(tile_result)
}

/// Run template matching in parallel by splitting the source into row tiles.
///
/// Each tile includes enough extra rows to compute matches that overlap the
/// tile boundary (template height - 1). Results are stitched back together
/// into a full response map that matches OpenCV's output layout.
#[instrument(skip_all)]
#[allow(unsafe_code)]
pub fn match_template(
    source_lightness: &LabLightnessMat,
    template_lightness: &LabLightnessMat,
    template_mask: Option<&MaskMat>,
    cancellation_token: CancellationToken,
    progress: watch::Sender<FindImageProgress>,
) -> Result<Mat> {
    if cancellation_token.is_cancelled() {
        return Err(CommonError::Cancelled.into());
    }

    ensure!(
        source_lightness.0.rows() >= template_lightness.0.rows()
            && source_lightness.0.cols() >= template_lightness.0.cols(),
        "template must fit inside source image"
    );

    let source_size = source_lightness.0.size()?;
    let template_size = template_lightness.0.size()?;
    let result_rows = source_size.height - template_size.height + 1;
    let result_cols = source_size.width - template_size.width + 1;
    let tile_count = ideal_thread_count().clamp(1, result_rows.max(1) as usize);

    // Build tile ranges and compute the source ROI each tile needs.
    let tile_ranges = build_tiles(result_rows, tile_count)
        .into_iter()
        .map(|(start_row, end_row)| {
            let tile_result_rows = end_row - start_row;
            let roi_height =
                (tile_result_rows + template_size.height - 1).min(source_size.height - start_row);
            let roi = Rect::new(0, start_row, source_size.width, roi_height);
            (start_row, roi)
        })
        .collect_vec();

    let total_tiles = tile_ranges.len();
    let completed_tiles = Arc::new(AtomicUsize::new(0));

    // Run template matching on each tile in parallel.
    let mut tile_results = tile_ranges
        .into_par_iter()
        .map(|(start_row, roi)| {
            if cancellation_token.is_cancelled() {
                return Err(crate::error::CommonError::Cancelled.into());
            }

            let tile_result =
                match_tile(&source_lightness, &template_lightness, template_mask, roi)?;

            // Update progress: matching phase is 20-70%, so 50% of total range
            let completed = completed_tiles.fetch_add(1, Ordering::Relaxed) + 1;
            let percent = 20 + ((completed * 50) / total_tiles);

            progress.send_replace(FindImageProgress::new(
                FindImageStage::Matching,
                percent.min(70) as u8,
            ));

            Ok::<_, color_eyre::eyre::Error>((start_row, tile_result))
        })
        .collect::<Result<Vec<_>>>()?;

    tile_results.sort_by_key(|(start_row, ..)| *start_row);

    // Stitch the per-tile results into a single matrix.
    let mut result = unsafe { Mat::new_rows_cols(result_rows, result_cols, CV_32FC1)? };

    for (start_row, tile_result) in &tile_results {
        for offset in 0..tile_result.rows() {
            let dest_row = result.at_row_mut::<f32>(start_row + offset)?;
            let src_row = tile_result.at_row::<f32>(offset)?;
            dest_row.copy_from_slice(src_row);
        }
    }

    Ok(result)
}

/// Partition a row count into contiguous, near-equal ranges.
///
/// This keeps tiles roughly balanced while ensuring coverage of all rows.
#[instrument(skip_all)]
fn build_tiles(total_rows: i32, desired_tiles: usize) -> Vec<(i32, i32)> {
    let total_rows = total_rows.max(1);
    let tile_count = desired_tiles.clamp(1, total_rows as usize);
    let base = total_rows / tile_count as i32;
    let remainder = total_rows % tile_count as i32;
    let mut tiles = Vec::with_capacity(tile_count);
    let mut start = 0;

    for idx in 0..tile_count {
        let mut height = base;
        if idx < remainder as usize {
            height += 1;
        }
        let end = (start + height).min(total_rows);
        if start >= end {
            break;
        }
        tiles.push((start, end));
        start = end;
    }

    if tiles.is_empty() {
        tiles.push((0, total_rows));
    }

    tiles
}
