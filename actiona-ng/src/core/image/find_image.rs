use std::{cmp::Ordering, collections::HashMap, sync::Arc, time::Instant};

use color_eyre::{Result, eyre::eyre};
use derive_more::Constructor;
use image::{GrayImage, RgbImage, RgbaImage};
use macros::FromJsObject;
use opencv::{
    core::{
        BORDER_DEFAULT, CV_32FC1, Mat, MatExprTraitConst, MatTraitConst, Point as CvPoint,
        Rect, Size as CvSize, Vector, merge, min_max_loc, no_array, split,
    },
    imgcodecs::imwrite,
    imgproc::{
        self, COLOR_RGB2BGR, COLOR_RGBA2BGRA, TM_CCOEFF_NORMED, cvt_color,
        match_template as cv_match_template, pyr_down,
    },
};
use opencv::prelude::MatTraitConstManual;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
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

    max_results: Option<u32>,
}

impl Default for JsFindImageOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            use_transparency: true,
            match_threshold: 0.8,
            search_one: false,
            non_maximum_suppression_radius: Some(10),
            max_results: None,
        }
    }
}

#[derive(Clone, Constructor, Debug, PartialEq)]
pub struct FindImageMatch {
    point: Point,
    score: f32,
}

#[derive(Clone, Constructor, Debug, PartialEq)]
pub struct FindImagesImage {
    image: Image,
    label: Option<String>,
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
    fn find_image(
        &self,
        image: &Image,
        options: JsFindImageOptions,
    ) -> Result<Vec<FindImageMatch>> {
        if image.width() > self.width() || image.height() > self.height() {
            return Err(eyre!(
                "searched image size ({}, {}) larger than source size ({}, {})",
                image.width(),
                image.height(),
                self.width(),
                self.height(),
            ));
        }

        let source = if options.use_colors {
            self.rgb_to_mat()?
        } else {
            self.greyscale_to_mat()?
        };

        let template =
            image_to_template(&image, options.use_colors, options.use_transparency, None)?;

        let (_, result) = match_template(source.as_ref(), &template)?;

        let matches = compute_results(
            &result,
            options.match_threshold,
            options.max_results,
            options.non_maximum_suppression_radius,
        )?;

        Ok(matches)
    }

    #[instrument(skip_all)]
    fn find_images(
        &self,
        images: &[FindImagesImage],
        options: JsFindImageOptions,
    ) -> Result<HashMap<Option<String>, Vec<FindImageMatch>>> {
        for image in images {
            if image.image.width() > self.width() || image.image.height() > self.height() {
                return Err(eyre!(
                    "searched image size ({}, {}) larger than source size ({}, {})",
                    image.image.width(),
                    image.image.height(),
                    self.width(),
                    self.height(),
                ));
            }
        }

        let source = if options.use_colors {
            self.rgb_to_mat()?
        } else {
            self.greyscale_to_mat()?
        };

        Ok(images
            .par_iter()
            .map(|image| {
                image_to_template(
                    &image.image,
                    options.use_colors,
                    options.use_transparency,
                    image.label.clone(),
                )
            })
            .collect::<Result<Vec<_>>>()?
            .into_par_iter()
            .map(|template| match_template(source.as_ref(), &template))
            .collect::<Result<Vec<_>>>()?
            .into_par_iter()
            .map(|(label, result)| {
                let matches = compute_results(
                    &result,
                    options.match_threshold,
                    options.max_results,
                    options.non_maximum_suppression_radius,
                )?;

                Ok((label, matches))
            })
            .collect::<Result<HashMap<Option<String>, Vec<FindImageMatch>>>>()?)
    }
}

#[instrument(skip_all)]
fn match_template(source: &Mat, template: &Template) -> Result<(Option<String>, Mat)> {
    let template_mat = &template.mat;
    let rows = source.rows() - template_mat.rows() + 1;
    let cols = source.cols() - template_mat.cols() + 1;
    let tile_target_count = match_template_tile_target_count();
    let use_tiled = tile_target_count > 1;

    let result = if use_tiled {
        match_template_tiled(source, template, rows, cols, tile_target_count)?
    } else {
        match_template_direct(source, template, rows, cols)?
    };

    Ok((template.label.clone(), result))
}

fn match_template_direct(source: &Mat, template: &Template, rows: i32, cols: i32) -> Result<Mat> {
    let template_mat = &template.mat;
    let mut result = Mat::zeros(rows, cols, CV_32FC1)?.to_mat()?;

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

    Ok(result)
}

fn match_template_tile_target_count() -> i32 {
    let target = sysinfo::System::physical_core_count()
        .and_then(|count| count.checked_sub(1))
        .unwrap_or(1);
    target.min(i32::MAX as usize).max(1) as i32
}

fn match_template_tiled(
    source: &Mat,
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
    let tiles_y = rows.min(tile_target_count).max(1);
    let tiles_x = ((tile_target_count + tiles_y - 1) / tiles_y).max(1);
    let tile_rows_step = ((rows + tiles_y - 1) / tiles_y).max(1);
    let tile_cols_step = ((cols + tiles_x - 1) / tiles_x).max(1);

    let mut tiles = Vec::new();
    let mut tile_origin_y = 0;
    while tile_origin_y < rows {
        let tile_rows = (rows - tile_origin_y).min(tile_rows_step);
        let mut tile_origin_x = 0;

        while tile_origin_x < cols {
            let tile_cols = (cols - tile_origin_x).min(tile_cols_step);
            tiles.push(TileRects {
                source_rect: Rect::new(
                    tile_origin_x,
                    tile_origin_y,
                    tile_cols + template_cols - 1,
                    tile_rows + template_rows - 1,
                ),
                result_rect: Rect::new(tile_origin_x, tile_origin_y, tile_cols, tile_rows),
            });
            tile_origin_x += tile_cols;
        }

        tile_origin_y += tile_rows;
    }

    let mask = template.mask.as_ref();
    let template_mat = template_mat.clone();
    let tile_results = tiles
        .into_par_iter()
        .map(|rects| -> Result<(Rect, Mat)> {
            let source_roi = source.roi(rects.source_rect)?;
            let mut tile_result =
                Mat::zeros(rects.result_rect.height, rects.result_rect.width, CV_32FC1)?
                    .to_mat()?;

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
            return Ok(vec![FindImageMatch::new(max_loc.into(), max_val as f32)]);
        } else {
            return Ok(vec![]);
        }
    }

    // Collect all matches above the threshold
    let mut match_points = Vec::new();
    let rows = match_template_result.rows();
    let cols = match_template_result.cols();
    if match_template_result.is_continuous() {
        let values = match_template_result.data_typed::<f32>()?;
        for (idx, &match_score) in values.iter().enumerate() {
            if match_score >= match_threshold {
                #[allow(clippy::as_conversions)]
                let idx = idx as i32;
                let row = idx / cols;
                let col = idx - row * cols;
                match_points.push(FindImageMatch::new(point(col, row), match_score));
            }
        }
    } else {
        for row in 0..rows {
            let row_values = match_template_result.at_row::<f32>(row)?;
            for (col, &match_score) in row_values.iter().enumerate() {
                if match_score >= match_threshold {
                    #[allow(clippy::as_conversions)]
                    let col = col as i32;
                    match_points.push(FindImageMatch::new(point(col, row), match_score));
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
    label: Option<String>,
}

#[instrument(skip_all)]
fn image_to_template(
    template: &Image,
    use_colors: bool,
    use_transparency: bool,
    label: Option<String>,
) -> Result<Template> {
    Ok(match (use_colors, use_transparency) {
        (false, false) => Template::new(template.greyscale_to_mat()?, None, label),
        (true, false) => Template::new(template.rgb_to_mat()?, None, label),
        (false, true) => {
            let template_rgba = template.rgba_to_mat()?;

            // Split template channels to extract the alpha channel
            let mut rgba_channels = Vector::<Mat>::new();
            split(template_rgba.as_ref(), &mut rgba_channels)?;

            let template_alpha = rgba_channels.get(3)?; // Alpha channel

            let template = template.greyscale_to_mat()?;

            Template::new(template, Some(template_alpha), label)
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

            Template::new(Arc::new(template_bgr), Some(template_alpha), label)
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
    use ab_glyph::{FontArc, PxScale};
    use image::ImageReader;
    use imageproc::drawing::{draw_hollow_rect_mut, draw_text, draw_text_mut};
    use opencv::core::set_num_threads;
    use tracing_subscriber::{
        EnvFilter,
        fmt::{fmt, format::FmtSpan},
        util::SubscriberInitExt,
    };

    use crate::{
        core::{
            color::Color,
            image::{
                Image,
                find_image::{
                    FindImageMatch, FindImagesImage, JsFindImageOptions, image_to_template,
                },
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

            let result = source
                .find_images(
                    &vec![
                        FindImagesImage::new(
                            Image::from_dynamic_image(pear),
                            Some("pear".to_string()),
                        ),
                        FindImagesImage::new(
                            Image::from_dynamic_image(fire),
                            Some("fire".to_string()),
                        ),
                    ],
                    JsFindImageOptions {
                        use_colors: true,
                        use_transparency: true,
                        match_threshold: 0.8,
                        non_maximum_suppression_radius: Some(10),
                        search_one: false,
                        max_results: None,
                    },
                )
                .unwrap();

            let font_data: &[u8] =
                include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
            let font = FontArc::try_from_slice(font_data).expect("error constructing FontArc");

            for (label, matches) in result {
                for FindImageMatch { point, score, .. } in matches {
                    let rect = Rect::new(point, size(template_w, template_h));
                    draw_text_mut(
                        &mut source.inner,
                        Color::new(255, 0, 0, 255).into(),
                        rect.origin.x.into(),
                        rect.origin.y.into(),
                        PxScale::from(24.0),
                        &font,
                        label.as_deref().unwrap_or("None"),
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
}
