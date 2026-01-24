use std::{cmp::Ordering, collections::HashSet, sync::Arc};

use color_eyre::{Result, eyre::eyre};
use derive_more::{Constructor, Display};
use image::{GrayImage, RgbImage, RgbaImage};
use macros::FromJsObject;
use opencv::imgcodecs::imwrite;
use opencv::prelude::MatTraitConstManual;
use opencv::{
    core::{
        CV_32FC1, Mat, MatExprTraitConst, MatTraitConst, Point as CvPoint, Rect, ToInputArray,
        Vector, merge, min_max_loc, no_array, split,
    },
    imgproc::{
        COLOR_RGB2BGR, COLOR_RGBA2BGRA, TM_CCOEFF_NORMED, cvt_color,
        match_template as cv_match_template,
    },
};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::instrument;

use crate::core::{
    image::Image,
    point::{Point, js::JsPoint, point},
};

// Below this output-map size, tiling overhead tends to dominate. For hinted searches we often
// evaluate small windows, so we use a single cv::matchTemplate call instead of splitting tiles.
const SMALL_MATCH_OUTPUT_ELEMENTS: i64 = 256 * 256;

#[derive(Debug, Default, Clone, Copy)]
pub enum MatchThreshold {
    Excellent,
    #[default]
    Good,
    Average,
    Custom(f32),
}

impl From<MatchThreshold> for f32 {
    fn from(value: MatchThreshold) -> Self {
        match value {
            MatchThreshold::Excellent => 0.9,
            MatchThreshold::Good => 0.8,
            MatchThreshold::Average => 0.6,
            MatchThreshold::Custom(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, Display, PartialEq, PartialOrd)]

pub enum MatchScore {
    #[display("Excellent ({_0})")]
    Excellent(f32),
    #[display("Good ({_0})")]
    Good(f32),
    #[display("Average ({_0})")]
    Average(f32),
    #[display("Poor ({_0})")]
    Poor(f32),
}

impl From<f64> for MatchScore {
    fn from(value: f64) -> Self {
        match value {
            value if value >= 0.9 => MatchScore::Excellent(value as f32),
            value if value >= 0.8 => MatchScore::Good(value as f32),
            value if value >= 0.6 => MatchScore::Average(value as f32),
            _ => MatchScore::Poor(value as f32),
        }
    }
}

impl From<f32> for MatchScore {
    fn from(value: f32) -> Self {
        Self::from(value as f64)
    }
}

/// Find image template options
/// @options
#[derive(Clone, Debug, FromJsObject, PartialEq)]
pub struct FindImageTemplateOptions {
    pub use_colors: bool,
    pub use_transparency: bool,
    pub match_threshold: f32,
    pub max_results: Option<u32>,

    /// Optional pixel radius (in result-map coordinates) to try around each hint.
    /// If omitted, a heuristic radius is used.
    pub hint_search_radius: Option<i32>,

    /// Radius to consider proximity (in pixels)
    pub non_maximum_suppression_radius: Option<i32>,

    pub position_hints: Option<Vec<JsPoint>>,
}

impl Default for FindImageTemplateOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            use_transparency: true,
            match_threshold: 0.8,
            max_results: None,
            hint_search_radius: None,
            non_maximum_suppression_radius: Some(10),
            position_hints: None,
        }
    }
}

#[derive(Clone, Constructor, Debug, PartialEq)]
pub struct FindImageMatch {
    point: Point,
    score: MatchScore,
}

#[derive(Clone, Constructor, Debug, PartialEq)]
pub struct FindImageTemplate {
    image: Image,
    options: FindImageTemplateOptions,
}

impl Image {
    #[instrument(skip_all)]
    fn rgb_to_mat(&self) -> Result<Arc<Mat>> {
        if let Some(mat) = self.rgb_mat.get() {
            return Ok(mat);
        }

        let image = self.to_rgb8();
        let image = image.as_ref();
        let (_width, height) = image.dimensions();
        let data = image.as_raw();
        let mat_boxed = Mat::from_slice(data)?;
        let mat = mat_boxed.reshape(3, height.try_into()?)?;
        let mut mat_bgr = Mat::default();

        #[allow(clippy::redundant_closure_call)]
        (|| {
            opencv::opencv_has_inherent_feature_algorithm_hint! {
                {
                    cvt_color(&mat, &mut mat_bgr, COLOR_RGB2BGR, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                } else {
                    cvt_color(&mat, &mut mat_bgr, COLOR_RGB2BGR, 0)
                }
            }
        })()?;

        let mat = Arc::new(mat_bgr);
        self.rgb_mat.set(mat.clone());
        Ok(mat)
    }

    #[instrument(skip_all)]
    fn rgba_to_mat(&self) -> Result<Arc<Mat>> {
        if let Some(mat) = self.rgba_mat.get() {
            return Ok(mat);
        }

        let image = self.to_rgba8();
        let image = image.as_ref();
        let (_width, height) = image.dimensions();
        let data = image.as_raw();
        let mat_boxed = Mat::from_slice(data)?;
        let mat = mat_boxed.reshape(4, height.try_into()?)?;
        let mut mat_bgr = Mat::default();

        #[allow(clippy::redundant_closure_call)]
        (|| {
            opencv::opencv_has_inherent_feature_algorithm_hint! {
                {
                    cvt_color(&mat, &mut mat_bgr, COLOR_RGBA2BGRA, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT)
                } else {
                    cvt_color(&mat, &mut mat_bgr, COLOR_RGBA2BGRA, 0)
                }
            }
        })()?;

        let mat = Arc::new(mat_bgr);
        self.rgba_mat.set(mat.clone());
        Ok(mat)
    }

    #[instrument(skip_all)]
    fn greyscale_to_mat(&self) -> Result<Arc<Mat>> {
        if let Some(mat) = self.greyscale_mat.get() {
            return Ok(mat);
        }

        let image = self.to_luma8();
        let image = image.as_ref();
        let (_width, height) = image.dimensions();
        let data = image.as_raw();
        let mat_boxed = Mat::from_slice(data)?;
        let mat = mat_boxed.reshape(1, height.try_into()?)?;

        let mat = Arc::new(mat.try_clone()?);
        self.greyscale_mat.set(mat.clone());
        Ok(mat)
    }

    #[instrument(skip_all)]
    fn find_image(&self, template: &FindImageTemplate) -> Result<Vec<FindImageMatch>> {
        if template.image.width() > self.width() || template.image.height() > self.height() {
            return Err(eyre!(
                "searched image size ({}, {}) larger than source size ({}, {})",
                template.image.width(),
                template.image.height(),
                self.width(),
                self.height(),
            ));
        }

        let source = if template.options.use_colors {
            self.rgb_to_mat()?
        } else {
            self.greyscale_to_mat()?
        };

        let cv_template = image_to_template(
            &template.image,
            template.options.use_colors,
            template.options.use_transparency,
        )?;

        let template_mat = &cv_template.mat;
        let rows = source.rows() - template_mat.rows() + 1;
        let cols = source.cols() - template_mat.cols() + 1;
        if rows <= 0 || cols <= 0 {
            return Ok(vec![]);
        }

        let max_results = template.options.max_results;
        let threshold = template.options.match_threshold;
        let nms_radius = template.options.non_maximum_suppression_radius;

        // Build an ordered list of result-map regions to search:
        // - optional small windows around the last-known positions (hints)
        // - finally everything we haven't searched yet (correctness fallback)
        //
        // Note: for `max_results == None` we must scan the full map anyway, so hints are ignored.
        let hint_regions = if max_results.is_some() {
            if let Some(hints) = template
                .options
                .position_hints
                .as_deref()
                .filter(|h| !h.is_empty())
            {
                let hints = hints.iter().map(|p| p.inner()).collect::<Vec<_>>();
                build_hint_regions(
                    rows,
                    cols,
                    &hints,
                    template.options.hint_search_radius,
                    template_mat.rows(),
                    template_mat.cols(),
                )?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        let mut regions = hint_regions.clone();
        if max_results.is_some() && !hint_regions.is_empty() {
            // Avoid re-running matchTemplate on hint windows when we fall back to a full search.
            regions.extend(split_remaining_regions(rows, cols, &hint_regions));
        } else {
            regions.push(Region {
                origin_x: 0,
                origin_y: 0,
                width: cols,
                height: rows,
            });
        }

        println!("REGIONS: {:?}", regions);

        let mut collected = Vec::new();
        for region in regions {
            let region_result = match_template_in_region(source.as_ref(), &cv_template, region)?;
            let mut matches = compute_results(&region_result, threshold, max_results, nms_radius)?;
            offset_matches(&mut matches, region.origin_x, region.origin_y);

            // With hints present, we allow an early exit as soon as we have enough matches.
            // This trades strict "global best K" semantics for speed when the caller provides
            // a position hint.
            if let Some(k) = max_results {
                if k == 1 {
                    if matches.is_empty() {
                        continue;
                    }
                    return Ok(matches);
                }

                collected.extend(matches);
                collected.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
                if let Some(radius) = nms_radius {
                    collected = non_maximum_suppression(&collected, radius);
                }
                collected.truncate(k.try_into()?);
                if collected.len() == k as usize {
                    return Ok(collected);
                }
            } else {
                // Full-map search (no max_results): this is always the last region.
                return Ok(matches);
            }
        }

        Ok(collected)
    }
}

#[instrument(skip_all)]
fn match_template(source: &Mat, template: &Template) -> Result<Mat> {
    let template_mat = &template.mat;
    let rows = source.rows() - template_mat.rows() + 1;
    let cols = source.cols() - template_mat.cols() + 1;
    let tile_target_count = ideal_thread_count();
    match_template_tiled(source, template, rows, cols, tile_target_count)
}

fn ideal_thread_count() -> i32 {
    let target = sysinfo::System::physical_core_count()
        .and_then(|count| count.checked_sub(1))
        .unwrap_or(1);
    target.min(i32::MAX as usize).max(1) as i32
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Region {
    origin_x: i32,
    origin_y: i32,
    width: i32,
    height: i32,
}

fn build_hint_regions(
    rows: i32,
    cols: i32,
    hints: &[Point],
    hint_search_radius: Option<i32>,
    template_rows: i32,
    template_cols: i32,
) -> Result<Vec<Region>> {
    // Radii are in result-map coordinates (top-left match positions). A (w×h) result window
    // corresponds to a source ROI of (w + template_w - 1, h + template_h - 1).
    let max_radius = rows.max(cols);
    let base = (template_rows.max(template_cols) / 4).clamp(16, 64);
    let effective_max_radius = hint_search_radius
        .map(|r| r.max(0).min(max_radius))
        .unwrap_or(base.saturating_mul(8).min(max_radius));

    let mut radii = vec![0, base.min(effective_max_radius)];
    while *radii.last().unwrap() < effective_max_radius {
        let next = radii.last().unwrap().saturating_mul(2);
        if next == *radii.last().unwrap() {
            break;
        }
        radii.push(next.min(effective_max_radius));
    }
    radii.dedup();

    let mut regions = Vec::new();
    for radius in radii {
        for &hint in hints {
            let hx: i32 = hint.x.into();
            let hy: i32 = hint.y.into();

            let start_x = (hx - radius).clamp(0, cols - 1);
            let start_y = (hy - radius).clamp(0, rows - 1);
            let end_x = (hx + radius).clamp(0, cols - 1);
            let end_y = (hy + radius).clamp(0, rows - 1);

            regions.push(Region {
                origin_x: start_x,
                origin_y: start_y,
                width: end_x - start_x + 1,
                height: end_y - start_y + 1,
            });
        }
    }

    // Preserve the intended search order (smallest radius first); only remove exact duplicates.
    let mut seen = HashSet::with_capacity(regions.len());
    let mut ordered = Vec::with_capacity(regions.len());
    for region in regions {
        if seen.insert(region) {
            ordered.push(region);
        }
    }
    Ok(ordered)
}

fn split_remaining_regions(full_rows: i32, full_cols: i32, searched: &[Region]) -> Vec<Region> {
    // We search in "result-map coordinates": each (x, y) corresponds to a top-left match position.
    // `searched` are rectangles in that result-map space; this returns a set of disjoint rectangles
    // that cover everything *except* the already-searched area.
    if full_rows <= 0 || full_cols <= 0 {
        return Vec::new();
    }
    if searched.is_empty() {
        return vec![Region {
            origin_x: 0,
            origin_y: 0,
            width: full_cols,
            height: full_rows,
        }];
    }

    let mut y_breaks = Vec::with_capacity(2 + searched.len() * 2);
    y_breaks.push(0);
    y_breaks.push(full_rows);
    for r in searched {
        let y0 = r.origin_y.clamp(0, full_rows);
        let y1 = (r.origin_y + r.height).clamp(0, full_rows);
        y_breaks.push(y0);
        y_breaks.push(y1);
    }
    y_breaks.sort_unstable();
    y_breaks.dedup();

    let mut remaining = Vec::new();
    for band in y_breaks.windows(2) {
        let band_y0 = band[0];
        let band_y1 = band[1];
        if band_y1 <= band_y0 {
            continue;
        }

        // Collect covered X-intervals for rectangles that overlap this horizontal band.
        let mut covered = Vec::<(i32, i32)>::new();
        for r in searched {
            let ry0 = r.origin_y;
            let ry1 = r.origin_y + r.height;
            if ry0 >= band_y1 || ry1 <= band_y0 {
                continue;
            }
            let x0 = r.origin_x.clamp(0, full_cols);
            let x1 = (r.origin_x + r.width).clamp(0, full_cols);
            if x1 > x0 {
                covered.push((x0, x1));
            }
        }

        if covered.is_empty() {
            remaining.push(Region {
                origin_x: 0,
                origin_y: band_y0,
                width: full_cols,
                height: band_y1 - band_y0,
            });
            continue;
        }

        // Merge overlaps so we can compute the complement efficiently.
        covered.sort_unstable_by_key(|(x0, _x1)| *x0);
        let mut merged = Vec::<(i32, i32)>::with_capacity(covered.len());
        for (x0, x1) in covered {
            match merged.last_mut() {
                Some((last0, last1)) if x0 <= *last1 => {
                    let _ = last0;
                    *last1 = (*last1).max(x1);
                }
                _ => merged.push((x0, x1)),
            }
        }

        // Emit complement intervals in [0, full_cols).
        let mut cursor = 0;
        for (x0, x1) in merged {
            if cursor < x0 {
                remaining.push(Region {
                    origin_x: cursor,
                    origin_y: band_y0,
                    width: x0 - cursor,
                    height: band_y1 - band_y0,
                });
            }
            cursor = cursor.max(x1);
            if cursor >= full_cols {
                break;
            }
        }
        if cursor < full_cols {
            remaining.push(Region {
                origin_x: cursor,
                origin_y: band_y0,
                width: full_cols - cursor,
                height: band_y1 - band_y0,
            });
        }
    }

    // Reduce region count by merging vertically-adjacent rectangles with the same X span.
    remaining.sort_by_key(|r| (r.origin_x, r.width, r.origin_y, r.height));
    let mut merged: Vec<Region> = Vec::with_capacity(remaining.len());
    for r in remaining {
        if let Some(last) = merged.last_mut() {
            let last_end_y = last.origin_y + last.height;
            if last.origin_x == r.origin_x && last.width == r.width && last_end_y == r.origin_y {
                last.height += r.height;
                continue;
            }
        }
        merged.push(r);
    }
    merged
}

fn match_template_in_region(source: &Mat, template: &Template, region: Region) -> Result<Mat> {
    let template_mat = &template.mat;
    let template_rows = template_mat.rows();
    let template_cols = template_mat.cols();

    let source_rect = Rect::new(
        region.origin_x,
        region.origin_y,
        region.width + template_cols - 1,
        region.height + template_rows - 1,
    );
    let source_roi = source.roi(source_rect)?;

    let tile_target_count = ideal_thread_count();
    match_template_tiled(
        &source_roi,
        template,
        region.height,
        region.width,
        tile_target_count,
    )
}

fn offset_matches(matches: &mut [FindImageMatch], dx: i32, dy: i32) {
    for m in matches {
        let x: i32 = m.point.x.into();
        let y: i32 = m.point.y.into();
        m.point = point(x + dx, y + dy);
    }
}

fn match_template_tiled(
    source: &(impl MatTraitConst + ToInputArray + Sync),
    template: &Template,
    rows: i32,
    cols: i32,
    tile_target_count: i32,
) -> Result<Mat> {
    #[derive(Clone, Copy)]
    struct TileRects {
        source_rect: Rect,
        result_rect: Rect,
    }

    let template_mat = &template.mat;
    let template_rows = template_mat.rows();
    let template_cols = template_mat.cols();
    let mut result = Mat::zeros(rows, cols, CV_32FC1)?.to_mat()?;

    // Fast path for small outputs: avoid tile enumeration, ROI slicing, and rayon overhead.
    #[allow(clippy::as_conversions)]
    let output_elems = rows as i64 * cols as i64;
    if tile_target_count <= 1 || output_elems <= SMALL_MATCH_OUTPUT_ELEMENTS {
        if let Some(mask) = &template.mask {
            cv_match_template(
                source,
                template_mat.as_ref(),
                &mut result,
                TM_CCOEFF_NORMED,
                mask,
            )?;
        } else {
            cv_match_template(
                source,
                template_mat.as_ref(),
                &mut result,
                TM_CCOEFF_NORMED,
                &no_array(),
            )?;
        }
        return Ok(result);
    }
    // Build a tile grid with approximately `tile_target_count` tiles, split across Y and X.
    // Each tile represents a sub-rectangle of the *result map* (not the source image).
    let tiles_y = rows.min(tile_target_count).max(1);
    let tiles_x = ((tile_target_count + tiles_y - 1) / tiles_y).max(1);
    let tile_rows_step = ((rows + tiles_y - 1) / tiles_y).max(1);
    let tile_cols_step = ((cols + tiles_x - 1) / tiles_x).max(1);

    #[allow(clippy::as_conversions)]
    let tiles = (0..rows)
        .step_by(tile_rows_step as usize)
        .flat_map(|tile_origin_y| {
            let tile_rows = (rows - tile_origin_y).min(tile_rows_step);
            (0..cols)
                .step_by(tile_cols_step as usize)
                .map(move |tile_origin_x| {
                    let tile_cols = (cols - tile_origin_x).min(tile_cols_step);
                    // For this (tile_rows × tile_cols) rectangle in the result map, we need a source ROI
                    // large enough to run matchTemplate and produce exactly that output size.
                    TileRects {
                        source_rect: Rect::new(
                            tile_origin_x,
                            tile_origin_y,
                            tile_cols + template_cols - 1,
                            tile_rows + template_rows - 1,
                        ),
                        result_rect: Rect::new(tile_origin_x, tile_origin_y, tile_cols, tile_rows),
                    }
                })
        })
        .collect::<Vec<_>>();

    let mask = template.mask.as_ref();
    let template_mat = template_mat.clone();
    let tile_results = tiles
        .into_par_iter()
        .map(|rects| -> Result<(Rect, Mat)> {
            let source_roi = source.roi(rects.source_rect)?;
            let mut tile_result =
                Mat::zeros(rects.result_rect.height, rects.result_rect.width, CV_32FC1)?
                    .to_mat()?;

            // TMP
            let mut rng = rand::rng();
            imwrite(
                &format!("../tests/region{}.png", rng.random_range(0..10000)),
                &source_roi,
                &Vector::new(),
            )?;

            if let Some(mask) = mask {
                cv_match_template(
                    &source_roi,
                    template_mat.as_ref(),
                    &mut tile_result,
                    TM_CCOEFF_NORMED,
                    mask,
                )?;
            } else {
                cv_match_template(
                    &source_roi,
                    template_mat.as_ref(),
                    &mut tile_result,
                    TM_CCOEFF_NORMED,
                    &no_array(),
                )?;
            }

            Ok((rects.result_rect, tile_result))
        })
        .collect::<Result<Vec<_>>>()?;

    for (result_rect, tile_result) in tile_results {
        let mut result_roi = Mat::roi_mut(&mut result, result_rect)?;
        tile_result.copy_to(&mut result_roi)?;
    }

    Ok(result)
}

#[instrument(skip_all)]
fn compute_results(
    match_template_result: &Mat,
    match_threshold: f32,
    maximum_results: Option<u32>,
    non_maximum_suppression_radius: Option<i32>,
) -> Result<Vec<FindImageMatch>> {
    if maximum_results == Some(1) {
        let mut max_val = 0.;
        let mut max_loc = CvPoint::default();

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
            return Ok(vec![FindImageMatch::new(max_loc.into(), max_val.into())]);
        } else {
            return Ok(vec![]);
        }
    }

    // Collect all matches above the threshold
    let mut match_points = Vec::new();
    let rows = match_template_result.rows();
    let cols = match_template_result.cols();
    if match_template_result.is_continuous() {
        // Fast path: scan the raw (row-major) float buffer without per-pixel FFI calls.
        let values = match_template_result.data_typed::<f32>()?;
        for (idx, &match_score) in values.iter().enumerate() {
            if match_score >= match_threshold {
                #[allow(clippy::as_conversions)]
                let idx = idx as i32;
                let row = idx / cols;
                let col = idx - row * cols;
                match_points.push(FindImageMatch::new(point(col, row), match_score.into()));
            }
        }
    } else {
        // Fallback: some Mats are not contiguous; scan row slices.
        for row in 0..rows {
            let row_values = match_template_result.at_row::<f32>(row)?;
            for (col, &match_score) in row_values.iter().enumerate() {
                if match_score >= match_threshold {
                    #[allow(clippy::as_conversions)]
                    let col = col as i32;
                    match_points.push(FindImageMatch::new(point(col, row), match_score.into()));
                }
            }
        }
    }

    // Sort matches by score (in descending order)
    match_points.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

    let mut matches = if let Some(non_maximum_suppression_radius) = non_maximum_suppression_radius {
        non_maximum_suppression(&match_points, non_maximum_suppression_radius)
    } else {
        match_points
    };

    if let Some(maximum_results) = maximum_results {
        matches.truncate(maximum_results.try_into()?);
    }

    Ok(matches)
}

#[derive(Debug, Constructor)]
pub struct Template {
    mat: Arc<Mat>,
    mask: Option<Mat>,
}

#[instrument(skip_all)]
fn image_to_template(
    template: &Image,
    use_colors: bool,
    use_transparency: bool,
) -> Result<Template> {
    Ok(match (use_colors, use_transparency) {
        (false, false) => Template::new(template.greyscale_to_mat()?, None),
        (true, false) => Template::new(template.rgb_to_mat()?, None),
        (false, true) => {
            let template_rgba = template.rgba_to_mat()?;

            // Split template channels to extract the alpha channel
            let mut rgba_channels = Vector::<Mat>::new();
            split(template_rgba.as_ref(), &mut rgba_channels)?;

            let template_alpha = rgba_channels.get(3)?; // Alpha channel

            let template = template.greyscale_to_mat()?;

            Template::new(template, Some(template_alpha))
        }
        (true, true) => {
            let template = template.rgba_to_mat()?;

            // Split template channels to extract the alpha channel
            let mut rgba_channels = Vector::<Mat>::new();
            split(template.as_ref(), &mut rgba_channels)?;

            let template_alpha = rgba_channels.get(3)?; // Alpha channel

            // Remove the alpha channel from the template to get BGR
            let mut template_bgr = Mat::default();
            let mut bgr_channels = Vector::<Mat>::new();

            // Add the individual channels to the OpenCV Vector
            bgr_channels.push(rgba_channels.get(0)?);
            bgr_channels.push(rgba_channels.get(1)?);
            bgr_channels.push(rgba_channels.get(2)?);

            // Merge the BGR channels into a single BGR image
            merge(&bgr_channels, &mut template_bgr)?;

            Template::new(Arc::new(template_bgr), Some(template_alpha))
        }
    })
}

#[instrument(skip_all)]
fn non_maximum_suppression(input: &[FindImageMatch], radius: i32) -> Vec<FindImageMatch> {
    // Apply non-maximum suppression to remove overlapping matches
    let mut filtered_matches = Vec::new();

    for FindImageMatch { point, score, .. } in input {
        let mut should_keep = true;

        for FindImageMatch {
            point: existing_pt, ..
        } in &filtered_matches
        {
            let dist_x = (point.x - existing_pt.x).abs();
            let dist_y = (point.y - existing_pt.y).abs();
            if dist_x < radius && dist_y < radius {
                should_keep = false; // Suppress this match
                break;
            }
        }

        if should_keep {
            filtered_matches.push(FindImageMatch::new(*point, *score));
        }
    }

    filtered_matches
}

#[cfg(test)]
mod tests {
    use ab_glyph::{FontArc, PxScale};
    use image::ImageReader;
    use imageproc::drawing::{draw_hollow_rect_mut, draw_text_mut};
    use opencv::core::set_num_threads;
    use std::time::{Duration, Instant};
    use tracing_subscriber::{
        EnvFilter,
        fmt::{fmt, format::FmtSpan},
    };

    use super::{Region, split_remaining_regions};
    use crate::{
        core::{
            color::Color,
            image::{
                Image,
                find_image::{FindImageMatch, FindImageTemplate, FindImageTemplateOptions},
            },
            point::{js::JsPoint, point},
            rect::Rect,
            size::size,
        },
        runtime::Runtime,
    };

    fn area(r: Region) -> i64 {
        i64::from(r.width) * i64::from(r.height)
    }

    fn intersects(a: Region, b: Region) -> bool {
        let ax0 = a.origin_x;
        let ay0 = a.origin_y;
        let ax1 = a.origin_x + a.width;
        let ay1 = a.origin_y + a.height;

        let bx0 = b.origin_x;
        let by0 = b.origin_y;
        let bx1 = b.origin_x + b.width;
        let by1 = b.origin_y + b.height;

        ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
    }

    #[test]
    fn remaining_regions_empty_searched_is_full() {
        let remaining = split_remaining_regions(10, 20, &[]);
        assert_eq!(
            remaining,
            vec![Region {
                origin_x: 0,
                origin_y: 0,
                width: 20,
                height: 10,
            }]
        );
    }

    #[test]
    fn remaining_regions_excludes_searched_and_covers_rest() {
        let full_rows = 10;
        let full_cols = 10;
        let searched = vec![
            Region {
                origin_x: 2,
                origin_y: 2,
                width: 4,
                height: 4,
            },
            // Overlapping rectangle; should still be excluded.
            Region {
                origin_x: 4,
                origin_y: 4,
                width: 4,
                height: 4,
            },
        ];

        let remaining = split_remaining_regions(full_rows, full_cols, &searched);

        // Remaining must not overlap any searched region.
        for &s in &searched {
            for &r in &remaining {
                assert!(!intersects(s, r), "searched={s:?} overlaps remaining={r:?}");
            }
        }

        // Remaining regions should be disjoint.
        for (i, &a) in remaining.iter().enumerate() {
            for &b in remaining.iter().skip(i + 1) {
                assert!(
                    !intersects(a, b),
                    "remaining regions overlap: {a:?} vs {b:?}"
                );
            }
        }

        // Area(searched union) + Area(remaining) == full area.
        // Compute searched union area by brute force on this tiny grid.
        let mut searched_cells = 0i64;
        for y in 0..full_rows {
            for x in 0..full_cols {
                let cell = Region {
                    origin_x: x,
                    origin_y: y,
                    width: 1,
                    height: 1,
                };
                if searched.iter().any(|&s| intersects(s, cell)) {
                    searched_cells += 1;
                }
            }
        }
        let remaining_area: i64 = remaining.into_iter().map(area).sum();
        assert_eq!(
            searched_cells + remaining_area,
            i64::from(full_rows * full_cols)
        );
    }

    #[test]
    //#[traced_test]
    #[ignore]
    fn test_find_image() {
        let _ = fmt()
            .with_env_filter(EnvFilter::new("info"))
            .with_span_events(FmtSpan::CLOSE)
            .try_init();

        set_num_threads(0).unwrap();

        Runtime::test(async |_runtime| {
            let source = ImageReader::open("../tests/input.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let pear = ImageReader::open("../tests/pear.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let fire = ImageReader::open("../tests/fire.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let template_w = pear.width();
            let template_h = pear.height();
            let mut source = Image::from_dynamic_image(source);

            let templates = [
                ("pear", Image::from_dynamic_image(pear)),
                ("fire", Image::from_dynamic_image(fire)),
            ];

            let result = [
                source
                    .find_image(&FindImageTemplate::new(
                        templates[0].1.clone(),
                        FindImageTemplateOptions {
                            use_colors: true,
                            use_transparency: true,
                            match_threshold: 0.8,
                            max_results: None,
                            hint_search_radius: None,
                            non_maximum_suppression_radius: Some(10),
                            position_hints: None,
                        },
                    ))
                    .unwrap(),
                source
                    .find_image(&FindImageTemplate::new(
                        templates[1].1.clone(),
                        FindImageTemplateOptions {
                            use_colors: true,
                            use_transparency: true,
                            match_threshold: 0.8,
                            max_results: None,
                            hint_search_radius: None,
                            non_maximum_suppression_radius: Some(10),
                            position_hints: None,
                        },
                    ))
                    .unwrap(),
            ];

            let font_data: &[u8] =
                include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
            let font = FontArc::try_from_slice(font_data).expect("error constructing FontArc");

            for (idx, matches) in result.into_iter().enumerate() {
                let label = templates[idx].0;
                for FindImageMatch { point, score, .. } in matches {
                    let rect = Rect::new(point, size(template_w, template_h));
                    draw_text_mut(
                        &mut source.inner,
                        Color::new(255, 0, 0, 255).into(),
                        rect.top_left.x.into(),
                        rect.top_left.y.into(),
                        PxScale::from(24.0),
                        &font,
                        label,
                    );
                    draw_hollow_rect_mut(
                        &mut source.inner,
                        rect.try_into().unwrap(),
                        Color::new(255, 0, 0, 255).into(),
                    );
                    println!(
                        "Match found for {label:?} at {} with score: {:.3}",
                        point, score
                    );
                }
            }

            source.save("../tests/output.png").unwrap();
        });
    }

    #[test]
    #[ignore]
    fn bench_find_images_without_hints() {
        let _ = fmt()
            .with_env_filter(EnvFilter::new("info"))
            .with_span_events(FmtSpan::CLOSE)
            .try_init();

        set_num_threads(0).unwrap();

        Runtime::test(async |_runtime| {
            let source = ImageReader::open("../tests/input.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let fire = ImageReader::open("../tests/fire.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();

            let source = Image::from_dynamic_image(source);
            let fire = Image::from_dynamic_image(fire);

            let warmup = 2;
            let iters = 10;
            let mut total = Duration::default();
            let mut last = None;

            for i in 0..(warmup + iters) {
                let start = Instant::now();
                let result = source
                    .find_image(&FindImageTemplate::new(
                        fire.clone(),
                        FindImageTemplateOptions {
                            use_colors: true,
                            use_transparency: true,
                            match_threshold: 0.9,
                            max_results: Some(1),
                            hint_search_radius: None,
                            non_maximum_suppression_radius: None,
                            position_hints: None,
                        },
                    ))
                    .unwrap();
                let elapsed = start.elapsed();
                if i >= warmup {
                    total += elapsed;
                }
                last = Some(result);
            }

            let avg = total / iters;
            println!("without hints avg={avg:?}, last={last:?}");
        });
    }

    #[test]
    #[ignore]
    fn bench_find_images_with_hints() {
        let _ = fmt()
            .with_env_filter(EnvFilter::new("info"))
            .with_span_events(FmtSpan::CLOSE)
            .try_init();

        set_num_threads(0).unwrap();

        Runtime::test(async |_runtime| {
            let source = ImageReader::open("../tests/input.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let fire = ImageReader::open("../tests/fire.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();

            let source = Image::from_dynamic_image(source);
            let fire = Image::from_dynamic_image(fire);

            let position_hints = Some(vec![JsPoint::from(point(1638, 932))]);

            let warmup = 2;
            let iters = 10;
            let mut total = Duration::default();
            let mut last = None;

            for i in 0..(warmup + iters) {
                let start = Instant::now();
                let result = source
                    .find_image(&FindImageTemplate::new(
                        fire.clone(),
                        FindImageTemplateOptions {
                            use_colors: true,
                            use_transparency: true,
                            match_threshold: 0.9,
                            max_results: Some(1),
                            hint_search_radius: None,
                            non_maximum_suppression_radius: None,
                            position_hints: position_hints.clone(),
                        },
                    ))
                    .unwrap();
                let elapsed = start.elapsed();
                if i >= warmup {
                    total += elapsed;
                }
                last = Some(result);
            }

            let avg = total / iters;
            println!("with hints avg={avg:?}, last={last:?}");
        });
    }
}
