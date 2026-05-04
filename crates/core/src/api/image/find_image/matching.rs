use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use color_eyre::{Result, eyre::ensure};
use itertools::Itertools;
use opencv::{
    core::{AccessFlag, CV_32FC1, Mat, Rect, UMat, UMatUsageFlags, no_array},
    imgproc::{TM_CCOEFF_NORMED, match_template as cv_match_template},
    prelude::{MatTraitConst, MatTraitConstManual, MatTraitManual, UMatTraitConst},
};
use rayon::prelude::*;
use satint::{SaturatingFrom, SaturatingInto, TryRem, su32};
use satint::{Su32, TryDiv};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use types::size::Size;

use crate::{
    api::image::find_image::{
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

/// Run template matching once over the full image using OpenCV's UMat path.
fn match_gpu(
    source_lightness: &LabLightnessMat,
    template_lightness: &LabLightnessMat,
    template_mask: Option<&MaskMat>,
) -> Result<Mat> {
    let source = source_lightness
        .0
        .get_umat(AccessFlag::ACCESS_READ, UMatUsageFlags::USAGE_DEFAULT)?;
    let template = template_lightness
        .0
        .get_umat(AccessFlag::ACCESS_READ, UMatUsageFlags::USAGE_DEFAULT)?;
    let mut result = UMat::new_def();

    if let Some(mask) = template_mask {
        let mask = mask
            .0
            .get_umat(AccessFlag::ACCESS_READ, UMatUsageFlags::USAGE_DEFAULT)?;
        cv_match_template(&source, &template, &mut result, TM_CCOEFF_NORMED, &mask)?;
    } else {
        cv_match_template(
            &source,
            &template,
            &mut result,
            TM_CCOEFF_NORMED,
            &no_array(),
        )?;
    }

    let mut downloaded = Mat::default();
    result.copy_to(&mut downloaded)?;
    Ok(downloaded)
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
    enable_gpu: bool,
    cancellation_token: CancellationToken,
    progress: mpsc::UnboundedSender<FindImageProgress>,
) -> Result<Mat> {
    if cancellation_token.is_cancelled() {
        return Err(CommonError::Cancelled.into());
    }

    ensure!(
        source_lightness.0.rows() >= template_lightness.0.rows()
            && source_lightness.0.cols() >= template_lightness.0.cols(),
        "template must fit inside source image"
    );

    if enable_gpu {
        return match_gpu(source_lightness, template_lightness, template_mask);
    }

    let source_size: Size = source_lightness.0.size()?.into();
    let template_size: Size = template_lightness.0.size()?.into();
    let result_rows = source_size.height - template_size.height + Su32::ONE;
    let result_cols = source_size.width - template_size.width + Su32::ONE;
    let tile_count = ideal_thread_count().clamp(Su32::ONE, result_rows.max(Su32::ONE));

    // Build tile ranges and compute the source ROI each tile needs.
    let tile_ranges = build_tiles(result_rows, tile_count)?
        .into_iter()
        .map(|(start_row, end_row)| {
            let tile_result_rows = end_row - start_row;
            let roi_height = (tile_result_rows + template_size.height - Su32::ONE)
                .min(source_size.height - start_row);
            let roi = Rect::new(
                0,
                start_row.to_signed().saturating_into(),
                source_size.width.to_signed().saturating_into(),
                roi_height.to_signed().saturating_into(),
            );
            (start_row, roi)
        })
        .collect_vec();

    let total_tiles: Su32 = tile_ranges.len().saturating_into();
    let completed_tiles = Arc::new(AtomicUsize::new(0));

    // Run template matching on each tile in parallel.
    let mut tile_results = tile_ranges
        .into_par_iter()
        .map(|(start_row, roi)| {
            if cancellation_token.is_cancelled() {
                return Err(CommonError::Cancelled.into());
            }

            let tile_result = match_tile(source_lightness, template_lightness, template_mask, roi)?;

            // Update progress: matching phase is 20-70%, so 50% of total range
            let completed: Su32 =
                (completed_tiles.fetch_add(1, Ordering::Relaxed) + 1).saturating_into();
            let percent = su32(20)
                + ((completed * su32(50))
                    .try_div(total_tiles)
                    .expect("total_tiles cannot be 0"));

            let _ = progress.send(FindImageProgress::new(
                FindImageStage::Matching,
                percent.min(su32(70)).saturating_into(),
            ));

            Ok::<_, color_eyre::eyre::Error>((start_row, tile_result))
        })
        .collect::<Result<Vec<_>>>()?;

    tile_results.sort_by_key(|(start_row, ..)| *start_row);

    // Stitch the per-tile results into a single matrix.
    let mut result = unsafe {
        Mat::new_rows_cols(
            result_rows.saturating_into(),
            result_cols.saturating_into(),
            CV_32FC1,
        )?
    };

    for (start_row, tile_result) in &tile_results {
        for offset in 0..tile_result.rows() {
            let src_row = tile_result.at_row::<f32>(offset)?;
            let offset = Su32::saturating_from(offset);
            let dest_row_index: i32 = (*start_row + offset).to_signed().into();
            let dest_row = result.at_row_mut::<f32>(dest_row_index)?;
            dest_row.copy_from_slice(src_row);
        }
    }

    Ok(result)
}

/// Partition a row count into contiguous, near-equal ranges.
///
/// This keeps tiles roughly balanced while ensuring coverage of all rows.
#[instrument(skip_all)]
fn build_tiles(total_rows: Su32, desired_tiles: Su32) -> Result<Vec<(Su32, Su32)>> {
    let total_rows = total_rows.max(Su32::ONE);
    let tile_count = desired_tiles.clamp(Su32::ONE, total_rows);
    let base: Su32 = total_rows.try_div(tile_count)?;
    let remainder: Su32 = total_rows.try_rem(tile_count)?;
    let mut tiles = Vec::with_capacity(tile_count.saturating_into());
    let mut start = Su32::ZERO;

    for idx in 0..tile_count.saturating_into() {
        let mut height = base;
        if idx < remainder {
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
        tiles.push((Su32::ZERO, total_rows));
    }

    Ok(tiles)
}
