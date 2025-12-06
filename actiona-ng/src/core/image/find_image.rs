use std::{cmp::Ordering, sync::Arc, time::Instant};

use color_eyre::{Result, eyre::eyre};
use derive_more::Constructor;
use image::{GrayImage, RgbImage, RgbaImage};
use macros::FromJsObject;
use opencv::{
    core::{
        BORDER_DEFAULT, CV_32FC1, Mat, MatExprTraitConst, MatTraitConst, Point as CvPoint,
        Size as CvSize, Vector, merge, min_max_loc, no_array, split,
    },
    imgcodecs::imwrite,
    imgproc::{
        COLOR_RGB2BGR, COLOR_RGBA2BGRA, TM_CCOEFF_NORMED, cvt_color, match_template, pyr_down,
    },
};
use tracing::instrument;

use crate::core::{
    image::Image,
    point::{Point, point},
};

/// Find image options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsFindImageOptions {
    use_colors: bool,
    use_transparency: bool,
    match_threshold: f32,
    search_one: bool,

    /// Radius to consider proximity (in pixels)
    non_maximum_suppression_radius: Option<i32>,

    /// Number of pyramid levels (0 = disabled)
    max_pyramid_levels: Option<u8>,

    max_results: Option<u8>,
}

impl Default for JsFindImageOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            use_transparency: true,
            match_threshold: 0.8,
            search_one: false,
            non_maximum_suppression_radius: Some(10),
            max_pyramid_levels: None,
            max_results: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct FindImageMatch {
    point: Point,
    score: f32,
    label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct SearchedImage {
    image: Image,
    label: Option<String>,
}

struct PyramidLevel {
    source: Mat,
    template: Mat,
    mask: Option<Mat>,
    inv_scale: f32, // 1.0 at full res, 2.0 after one pyr_down, 4.0 after two, etc.
}

#[instrument(skip_all)]
fn build_pyramid_levels(
    base_source: &Mat,
    base_template: &Mat,
    base_mask: Option<&Mat>,
    max_levels: u8,
) -> Result<Vec<PyramidLevel>> {
    let mut levels = Vec::new();

    // level 0 = full resolution
    levels.push(PyramidLevel {
        source: base_source.clone(),
        template: base_template.clone(),
        mask: base_mask.cloned(),
        inv_scale: 1.0,
    });

    let mut current_source = base_source.clone();
    let mut current_template = base_template.clone();
    let mut current_mask = base_mask.cloned();
    let mut inv_scale = 1.0_f32;

    for level in 0..max_levels {
        let mut src_down = Mat::default();
        let mut tpl_down = Mat::default();

        // downscale by factor 2
        pyr_down(
            &current_source,
            &mut src_down,
            CvSize::default(),
            BORDER_DEFAULT,
        )?;
        pyr_down(
            &current_template,
            &mut tpl_down,
            CvSize::default(),
            BORDER_DEFAULT,
        )?;

        // stop if template becomes too small
        if tpl_down.rows() < 1 || tpl_down.cols() < 1 {
            break;
        }

        let mask_down = if let Some(ref m) = current_mask {
            let mut m_down = Mat::default();
            pyr_down(m, &mut m_down, CvSize::default(), BORDER_DEFAULT)?;
            Some(m_down)
        } else {
            None
        };

        imwrite(&format!("source{level}.png"), &src_down, &Vector::new()).unwrap();
        imwrite(&format!("template{level}.png"), &tpl_down, &Vector::new()).unwrap();
        if let Some(m_down) = &mask_down {
            imwrite(&format!("mask{level}.png"), &m_down, &Vector::new()).unwrap();
        }

        inv_scale *= 2.0;

        levels.push(PyramidLevel {
            source: src_down.clone(),
            template: tpl_down.clone(),
            mask: mask_down.clone(),
            inv_scale,
        });

        current_source = src_down;
        current_template = tpl_down;
        current_mask = mask_down;
    }

    Ok(levels)
}

impl Image {
    #[instrument(skip_all)]
    fn rgb_to_mat(&self) -> Result<Arc<Mat>> {
        if let Some(mat) = self.rgb_mat.get() {
            return Ok(mat);
        }

        let mat = Arc::new(rgb_to_mat(self.to_rgb8().as_ref())?);
        self.rgb_mat.set(mat.clone());
        Ok(mat)
    }

    #[instrument(skip_all)]
    fn rgba_to_mat(&self) -> Result<Arc<Mat>> {
        if let Some(mat) = self.rgba_mat.get() {
            return Ok(mat);
        }

        let mat = Arc::new(rgba_to_mat(self.to_rgba8().as_ref())?);
        self.rgba_mat.set(mat.clone());
        Ok(mat)
    }

    #[instrument(skip_all)]
    fn greyscale_to_mat(&self) -> Result<Arc<Mat>> {
        if let Some(mat) = self.greyscale_mat.get() {
            return Ok(mat);
        }

        let mat = Arc::new(greyscale_to_mat(self.to_luma8().as_ref())?);
        self.greyscale_mat.set(mat.clone());
        Ok(mat)
    }

    #[instrument(skip_all)]
    fn prepare_template_matching_mats(
        source: &Self,
        template: &Self,
        use_colors: bool,
        use_transparency: bool,
    ) -> Result<(Arc<Mat>, Arc<Mat>, Option<Mat>)> {
        if !use_colors {
            let source = source.greyscale_to_mat()?;

            if !use_transparency {
                return Ok((source, template.greyscale_to_mat()?, None));
            }

            let template_rgba = template.rgba_to_mat()?;

            // Split template channels to extract the alpha channel
            let mut rgba_channels = Vector::<Mat>::new();
            split(template_rgba.as_ref(), &mut rgba_channels)?;

            let template_alpha = rgba_channels.get(3)?; // Alpha channel

            let template = template.greyscale_to_mat()?;

            return Ok((source, template, Some(template_alpha)));
        }

        let source = source.rgb_to_mat()?;

        if !use_transparency {
            return Ok((source, template.rgb_to_mat()?, None));
        }

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

        Ok((source, Arc::new(template_bgr), Some(template_alpha)))
    }

    #[instrument(skip_all)]
    fn find_image_single_scale(
        &self,
        searched_image: &SearchedImage,
        options: JsFindImageOptions,
    ) -> Result<Vec<FindImageMatch>> {
        let searched_image = &searched_image.image;

        if searched_image.width() > self.width() || searched_image.height() > self.height() {
            return Err(eyre!(
                "searched image size ({}, {}) larger than source size ({}, {})",
                searched_image.width(),
                searched_image.height(),
                self.width(),
                self.height(),
            ));
        }

        let (source, template, mask) = Self::prepare_template_matching_mats(
            self,
            searched_image,
            options.use_colors,
            options.use_transparency,
        )?;

        // Create a result matrix for the matching result
        let result_rows = source.rows() - template.rows() + 1;
        let result_cols = source.cols() - template.cols() + 1;
        let mut result = Mat::zeros(result_rows, result_cols, CV_32FC1)?.to_mat()?;

        let start = Instant::now();

        if let Some(mask) = mask {
            // Perform template matching on the color image with mask
            match_template(
                source.as_ref(),
                template.as_ref(),
                &mut result,
                TM_CCOEFF_NORMED,
                &mask, // Use the alpha channel as the mask
            )?;
        } else {
            match_template(
                source.as_ref(),
                template.as_ref(),
                &mut result,
                TM_CCOEFF_NORMED,
                &no_array(),
            )?;
        }

        let duration = start.elapsed();
        println!("template matching took {duration:?}");

        // Set a threshold for good matches
        let match_threshold = options.match_threshold;

        if options.search_one {
            let mut max_val = 0.;
            let mut max_loc = CvPoint::default();

            min_max_loc(
                &result,
                None,
                Some(&mut max_val),
                None,
                Some(&mut max_loc),
                &no_array(),
            )?;

            if max_val >= match_threshold.into() {
                #[allow(clippy::as_conversions)]
                return Ok(vec![FindImageMatch::new(
                    max_loc.into(),
                    max_val as f32,
                    None,
                )]);
            } else {
                return Ok(vec![]);
            }
        }

        // Collect all matches above the threshold
        let mut match_points = Vec::new();
        for row in 0..result.rows() {
            for col in 0..result.cols() {
                let match_score = *result.at_2d::<f32>(row, col)?;
                if match_score >= match_threshold {
                    match_points.push(FindImageMatch::new(point(col, row), match_score, None));
                }
            }
        }

        // Sort matches by score (in descending order)
        match_points.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

        let mut matches =
            if let Some(non_maximum_suppression_radius) = options.non_maximum_suppression_radius {
                non_maximum_suppression(&match_points, non_maximum_suppression_radius)
            } else {
                match_points
            };

        if let Some(max_results) = options.max_results {
            matches.truncate(max_results.into());
        }

        Ok(matches)
    }

    #[instrument(skip_all)]
    fn find_image_pyramid(
        &self,
        searched_image: &SearchedImage,
        options: JsFindImageOptions,
        max_pyramid_levels: u8,
    ) -> Result<Vec<FindImageMatch>> {
        let searched_image = &searched_image.image;

        if searched_image.width() > self.width() || searched_image.height() > self.height() {
            return Err(eyre!(
                "searched image size ({}, {}) larger than source size ({}, {})",
                searched_image.width(),
                searched_image.height(),
                self.width(),
                self.height(),
            ));
        }

        let (base_source, base_template, base_mask) = Self::prepare_template_matching_mats(
            self,
            searched_image,
            options.use_colors,
            options.use_transparency,
        )?;

        // Build pyramid levels
        let levels = build_pyramid_levels(
            base_source.as_ref(),
            base_template.as_ref(),
            base_mask.as_ref(),
            max_pyramid_levels,
        )?;

        let start = Instant::now();

        let match_threshold = options.match_threshold;

        // --- 1. COARSE SEARCH ---
        // Use the smallest pyramid level
        let coarse_lvl = levels.last().unwrap();
        let src = &coarse_lvl.source;
        let tpl = &coarse_lvl.template;

        if tpl.rows() > src.rows() || tpl.cols() > src.cols() {
            return Ok(vec![]);
        }

        let result_rows = src.rows() - tpl.rows() + 1;
        let result_cols = src.cols() - tpl.cols() + 1;
        let mut coarse_result = Mat::zeros(result_rows, result_cols, CV_32FC1)?.to_mat()?;

        if let Some(ref mask) = coarse_lvl.mask {
            match_template(src, tpl, &mut coarse_result, TM_CCOEFF_NORMED, mask)?;
        } else {
            match_template(src, tpl, &mut coarse_result, TM_CCOEFF_NORMED, &no_array())?;
        }

        // --- 2. GATHER & CLUSTER CANDIDATES ---
        let mut coarse_matches = Vec::new();

        // Fast Path: If looking for only one, just take the absolute max on the coarse image
        if options.search_one {
            let mut max_val = 0.;
            let mut max_loc = CvPoint::default();
            min_max_loc(
                &coarse_result,
                None,
                Some(&mut max_val),
                None,
                Some(&mut max_loc),
                &no_array(),
            )?;

            // Loose threshold for coarse search
            if max_val as f32 >= match_threshold * 0.85 {
                coarse_matches.push(FindImageMatch::new(
                    point(max_loc.x, max_loc.y),
                    max_val as f32,
                    None,
                ));
            }
        } else {
            // Multi-search path
            for row in 0..coarse_result.rows() {
                for col in 0..coarse_result.cols() {
                    let score = *coarse_result.at_2d::<f32>(row, col)?;
                    // Relaxed threshold for coarse check
                    if score >= match_threshold * 0.9 {
                        coarse_matches.push(FindImageMatch::new(point(col, row), score, None));
                    }
                }
            }

            // OPTIMIZATION: Sort & NMS on the *Coarse* matches immediately.
            // A radius of 2 pixels in a 4x downscale = 8 pixels in real life.
            // This condenses 100 neighboring pixels into 1 candidate.
            coarse_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
            coarse_matches = non_maximum_suppression(&coarse_matches, 3); // 3px radius on small image
        }

        if coarse_matches.is_empty() {
            return Ok(vec![]);
        }

        // --- 3. REFINE MATCHES (Full Resolution & Color) ---
        // Now we only run the expensive color check on the few remaining distinct points.

        let lvl0 = &levels[0];
        let inv_scale = coarse_lvl.inv_scale;
        let search_margin = (inv_scale + 2.0).ceil() as i32; // Search radius around predicted point

        let mut refined_matches = Vec::new();

        for coarse_match in coarse_matches {
            let rough_x = (f32::try_from(coarse_match.point.x)? * inv_scale).round() as i32;
            let rough_y = (f32::try_from(coarse_match.point.y)? * inv_scale).round() as i32;

            // Define search ROI on Level 0
            let start_x = (rough_x - search_margin).max(0);
            let start_y = (rough_y - search_margin).max(0);

            let max_valid_x = lvl0.source.cols() - lvl0.template.cols();
            let max_valid_y = lvl0.source.rows() - lvl0.template.rows();

            let roi_start_x = start_x.min(max_valid_x);
            let roi_start_y = start_y.min(max_valid_y);

            let search_w = (rough_x + search_margin).min(max_valid_x) - roi_start_x + 1;
            let search_h = (rough_y + search_margin).min(max_valid_y) - roi_start_y + 1;

            if search_w <= 0 || search_h <= 0 {
                continue;
            }

            let crop_w = lvl0.template.cols() + search_w - 1;
            let crop_h = lvl0.template.rows() + search_h - 1;

            let roi = opencv::core::Rect::new(roi_start_x, roi_start_y, crop_w, crop_h);

            // This is the expensive part, but now we run it rarely
            let source_roi = Mat::roi(&lvl0.source, roi)?;

            let mut roi_result = Mat::default();
            if let Some(ref mask) = lvl0.mask {
                match_template(
                    &source_roi,
                    &lvl0.template,
                    &mut roi_result,
                    TM_CCOEFF_NORMED,
                    mask,
                )?;
            } else {
                match_template(
                    &source_roi,
                    &lvl0.template,
                    &mut roi_result,
                    TM_CCOEFF_NORMED,
                    &no_array(),
                )?;
            }

            let mut best_val = 0.0;
            let mut best_loc = CvPoint::default();
            min_max_loc(
                &roi_result,
                None,
                Some(&mut best_val),
                None,
                Some(&mut best_loc),
                &no_array(),
            )?;

            // Final strict threshold check on color data
            if (best_val as f32) >= match_threshold {
                let final_x = roi_start_x + best_loc.x;
                let final_y = roi_start_y + best_loc.y;

                refined_matches.push(FindImageMatch::new(
                    point(final_x, final_y),
                    best_val as f32,
                    None,
                ));
            }

            // Optimization: If we found a good match and only want one, stop here.
            if options.search_one && !refined_matches.is_empty() {
                break;
            }
        }

        let duration = start.elapsed();
        println!("template matching took {duration:?}");

        // Final Sort
        refined_matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

        // One last NMS (optional, but good if the refinement shifted points closer together)
        if !options.search_one {
            if let Some(radius) = options.non_maximum_suppression_radius {
                refined_matches = non_maximum_suppression(&refined_matches, radius);
            }
        }

        if let Some(max_results) = options.max_results {
            refined_matches.truncate(max_results.into());
        }

        Ok(refined_matches)
    }

    #[instrument(skip_all)]
    pub fn find_image(
        &self,
        searched_image: &SearchedImage,
        options: JsFindImageOptions,
    ) -> Result<Vec<FindImageMatch>> {
        if let Some(max_pyramid_levels) = options.max_pyramid_levels {
            self.find_image_pyramid(searched_image, options, max_pyramid_levels)
        } else {
            self.find_image_single_scale(searched_image, options)
        }
    }
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
            filtered_matches.push(FindImageMatch::new(*point, *score, None));
        }
    }

    filtered_matches
}

fn greyscale_to_mat(image: &GrayImage) -> Result<Mat> {
    let (_width, height) = image.dimensions();
    let data = image.as_raw();
    let mat_boxed = Mat::from_slice(data)?;
    let mat = mat_boxed.reshape(1, height.try_into()?)?;
    Ok(mat.try_clone()?)
}

fn rgb_to_mat(image: &RgbImage) -> Result<Mat> {
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

    Ok(mat_bgr)
}

fn rgba_to_mat(image: &RgbaImage) -> Result<Mat> {
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

    Ok(mat_bgr)
}

#[cfg(test)]
mod tests {
    use image::ImageReader;
    use imageproc::drawing::draw_hollow_rect_mut;
    use tracing::info;
    use tracing_subscriber::{
        EnvFilter, fmt, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
    };

    use crate::{
        core::{
            color::Color,
            image::{
                Image,
                find_image::{FindImageMatch, JsFindImageOptions, SearchedImage},
            },
            rect::Rect,
            size::size,
        },
        runtime::Runtime,
    };

    #[test]
    //#[traced_test]
    #[ignore]
    fn test_find_image() {
        let _ = fmt()
            .with_env_filter(EnvFilter::new("info"))
            //.with_span_events(FmtSpan::CLOSE)
            .try_init();

        Runtime::test(async |_runtime| {
            let source = ImageReader::open("/media/jmgr/Main/rust/test_ai_actiona/input.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let template = ImageReader::open("/media/jmgr/Main/rust/test_ai_actiona/pear.png")
                .unwrap()
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let mut source = Image::from_dynamic_image(source);
            let template = SearchedImage::new(Image::from_dynamic_image(template), None);
            let result = source
                .find_image(
                    &template,
                    JsFindImageOptions {
                        use_colors: true,
                        use_transparency: true,
                        match_threshold: 0.3,
                        non_maximum_suppression_radius: Some(10),
                        search_one: false,
                        max_pyramid_levels: Some(1),
                        max_results: None,
                    },
                )
                .unwrap();
            for FindImageMatch { point, score, .. } in result {
                let rect = Rect::new(point, size(template.image.width(), template.image.height()));
                draw_hollow_rect_mut(
                    &mut source.inner,
                    rect.try_into().unwrap(),
                    Color::new(255, 0, 0, 255).into(),
                );
                println!("Match found at {} with score: {:.3}", point, score);
            }
            source
                .save("/media/jmgr/Main/rust/test_ai_actiona/output.png")
                .unwrap();
        });
    }
}
