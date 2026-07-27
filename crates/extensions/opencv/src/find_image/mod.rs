use std::{borrow::Cow, sync::Arc};

use color_eyre::{
    Result,
    eyre::{Error, ensure, eyre},
};
use extension::protocols::opencv::{
    FindImageProgress, FindImageStage, FindImageTemplateOptions, Match,
};
use opencv::{
    core::{CV_8UC3, Mat, MatTraitConst, Scalar, Vec4b, Vector, extract_channel, split},
    imgproc::{COLOR_BGR2Lab, COLOR_BGRA2BGR, COLOR_RGBA2BGR},
};
use satint::{SaturatingInto, Su32, su32};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use types::Size;

use crate::find_image::{
    convert::convert_colors,
    matching::match_template,
    pyramids::{prepare_matching_inputs, resize_result_to_size},
    results::{compute_results, filter_results_by_color},
};

mod common;
pub mod convert;
mod matching;
mod pyramids;
mod results;

/// The error returned when a find request is cancelled part-way through.
///
/// The host detects cancellation itself (it owns the token that triggered the
/// `cancel` call), so this only needs to unwind the extension's own work.
pub fn cancelled() -> Error {
    eyre!("Cancelled")
}

/// Warms up OpenCV's Lab color space processing code.
pub fn warm_up() -> Result<()> {
    let dummy = Mat::new_rows_cols_with_default(1, 1, CV_8UC3, Scalar::all(0.0))?;
    let _ = convert_colors(&dummy, COLOR_BGR2Lab)?;
    Ok(())
}

/// Disable OpenCV parallelism since we perform our own parallelism using rayon.
pub fn setup_threading() -> Result<()> {
    use opencv::core::set_num_threads;

    opencv::opencv_branch_34! {
        {
            set_num_threads(0)?;
        }
        else
        {
            set_num_threads(1)?;
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct LabLightnessMat(Mat);

#[derive(Debug)]
pub struct LabAMat(Mat);

#[derive(Debug)]
pub struct LabBMat(Mat);

#[derive(Debug)]
pub struct BgrMat(Mat);

impl TryFrom<&BgrMat> for LabImage {
    type Error = Error;

    fn try_from(value: &BgrMat) -> Result<Self, Self::Error> {
        let lab = convert_colors(&value.0, COLOR_BGR2Lab)?;

        let mut channels = Vector::new();
        split(&lab, &mut channels)?;

        Ok(Self {
            lightness: LabLightnessMat(channels.get(0)?),
            a: LabAMat(channels.get(1)?),
            b: LabBMat(channels.get(2)?),
        })
    }
}

impl BgrMat {
    pub fn from_bgra(data: &[u8], size: Size) -> Result<Self> {
        const BYTES_PER_PIXEL: Su32 = su32(4);

        let needed = size.width * size.height * BYTES_PER_PIXEL;
        let needed = needed.saturating_into();

        if data.len() < needed {
            return Err(eyre!(
                "image data too small: expected {needed} bytes, got {}",
                data.len()
            ));
        }

        // Create a Mat view over the BGRA data
        let bgra_mat = Mat::new_rows_cols_with_bytes::<Vec4b>(
            size.height.saturating_into(),
            size.width.saturating_into(),
            &data[..needed],
        )?;

        // Convert BGRA to BGR
        let bgr = convert_colors(&bgra_mat, COLOR_BGRA2BGR)?;

        Ok(Self(bgr))
    }

    /// Converts a packed RGBA8 buffer to BGR, optionally extracting the alpha
    /// channel as a transparency mask.
    pub fn from_rgba(
        data: &[u8],
        size: Size,
        extract_mask: bool,
    ) -> Result<(Self, Option<MaskMat>)> {
        const BYTES_PER_PIXEL: Su32 = su32(4);

        let needed: usize = (size.width * size.height * BYTES_PER_PIXEL).saturating_into();
        if data.len() < needed {
            return Err(eyre!(
                "image data too small: expected {needed} bytes, got {}",
                data.len()
            ));
        }

        let mat_boxed = Mat::from_slice(&data[..needed])?;
        let mat = mat_boxed.reshape(4, size.height.saturating_into())?;

        let mask = if extract_mask {
            let mut alpha = Mat::default();
            extract_channel(&mat, &mut alpha, 3)?;
            Some(MaskMat(alpha))
        } else {
            None
        };

        Ok((Self(convert_colors(&mat, COLOR_RGBA2BGR)?), mask))
    }
}

#[derive(Clone, Debug)]
pub struct MaskMat(Mat);

#[derive(Debug)]
pub struct LabImage {
    pub lightness: LabLightnessMat,
    pub a: LabAMat,
    pub b: LabBMat,
}

#[derive(Debug)]
pub struct Source {
    pub image: LabImage,
}

impl Source {
    pub fn from_bgra(data: &[u8], size: Size) -> Result<Arc<Self>> {
        let bgr = BgrMat::from_bgra(data, size)?;
        let lab = LabImage::try_from(&bgr)?;

        Ok(Arc::new(Self { image: lab }))
    }

    pub fn from_rgba(data: &[u8], size: Size) -> Result<Arc<Self>> {
        let (bgr, _) = BgrMat::from_rgba(data, size, false)?;
        let lab = LabImage::try_from(&bgr)?;

        Ok(Arc::new(Self { image: lab }))
    }
}

#[derive(Debug)]
pub struct Template {
    pub image: LabImage,
    pub mask: Option<MaskMat>,
}

impl Template {
    pub fn from_rgba(data: &[u8], size: Size) -> Result<Arc<Self>> {
        let (bgr, mask) = BgrMat::from_rgba(data, size, true)?;

        Ok(Arc::new(Self {
            image: LabImage::try_from(&bgr)?,
            mask,
        }))
    }
}

impl Source {
    /// Find all occurrences of a template in this source image.
    #[instrument(skip_all)]
    pub fn find_template_all(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
        cancellation_token: &CancellationToken,
        progress: &mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        self.find_template_impl(template, options, false, cancellation_token, progress)
    }

    /// Find the best match of a template in this source image.
    #[instrument(skip_all)]
    pub fn find_template(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
        cancellation_token: &CancellationToken,
        progress: &mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<Option<Match>> {
        let matches =
            self.find_template_impl(template, options, true, cancellation_token, progress)?;
        Ok(matches.into_iter().next())
    }

    #[instrument(skip_all)]
    fn find_template_impl(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
        search_one: bool,
        cancellation_token: &CancellationToken,
        progress: &mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        // Check cancellation at the start
        if cancellation_token.is_cancelled() {
            return Err(cancelled());
        }

        ensure!(
            options.match_threshold.is_finite() && (0.0..=1.0).contains(&options.match_threshold),
            "match threshold must be finite and between 0.0 and 1.0"
        );

        let source_size = self.image.lightness.0.size()?;
        let template_size = template.image.lightness.0.size()?;
        ensure!(
            source_size.height >= template_size.height && source_size.width >= template_size.width,
            "template must fit inside source image"
        );

        let source_lightness = Cow::Borrowed(&self.image.lightness);
        let template_lightness = Cow::Borrowed(&template.image.lightness);

        // Only use the mask if transparency is enabled
        let mask_to_use = if options.use_transparency {
            template.mask.as_ref().map(Cow::Borrowed)
        } else {
            None
        };

        // Check before pyramid downscaling
        if cancellation_token.is_cancelled() {
            return Err(cancelled());
        }

        let _ = progress.send(FindImageProgress::new(FindImageStage::Downscaling, 10));

        // Reduce the size if needed
        let (source_lightness, template_lightness, template_mask) = prepare_matching_inputs(
            source_lightness,
            template_lightness,
            mask_to_use,
            options.downscale,
        )?;

        // Check before expensive template matching
        if cancellation_token.is_cancelled() {
            return Err(cancelled());
        }

        let _ = progress.send(FindImageProgress::new(FindImageStage::Matching, 20));

        // Apply template matching
        let mut result = match_template(
            source_lightness.as_ref(),
            template_lightness.as_ref(),
            template_mask.as_deref(),
            options.enable_gpu,
            cancellation_token,
            progress,
        )?;

        // Resize the result if needed
        if options.downscale > 0 {
            result = resize_result_to_size(&result, source_lightness.0.size()?)?;
        }

        // Check before color filtering
        if cancellation_token.is_cancelled() {
            return Err(cancelled());
        }

        let _ = progress.send(FindImageProgress::new(FindImageStage::Filtering, 70));

        if options.use_colors {
            // Always use the original (full-resolution) mask here: the result map has
            // been upscaled back to source resolution, so the ROIs and template channels
            // are full-size and the mask must match.
            let original_mask = if options.use_transparency {
                template.mask.as_ref()
            } else {
                None
            };
            filter_results_by_color(
                &mut result,
                &self.image.a,
                &self.image.b,
                &template.image.a,
                &template.image.b,
                original_mask,
                template_size,
                options.match_threshold,
            )?;
        }

        // Final check before computing results
        if cancellation_token.is_cancelled() {
            return Err(cancelled());
        }

        let _ = progress.send(FindImageProgress::new(FindImageStage::ComputingResults, 90));

        let matches = compute_results(
            &result,
            template_size,
            options.match_threshold,
            search_one,
            options.non_maximum_suppression_radius,
        )?;

        let _ = progress.send(FindImageProgress::new(FindImageStage::Finished, 100));

        Ok(matches)
    }
}

#[cfg(test)]
// The test fixture is a real-world-sized screenshot; the lint's default 1 MB cap doesn't apply here.
#[allow(clippy::large_include_file)]
mod tests {
    use itertools::Itertools;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;
    use types::{Size, point, size};

    use crate::find_image::{FindImageProgress, FindImageTemplateOptions, Source, Template};

    /// Decode a PNG fixture into (RGBA8 bytes, size).
    fn rgba_fixture(bytes: &[u8]) -> (Vec<u8>, Size) {
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();
        let dimensions = size(image.width(), image.height());
        (image.into_raw(), dimensions)
    }

    #[test]
    fn find_image() {
        let (source_pixels, source_size) =
            rgba_fixture(include_bytes!("../../../../core/test-data/input.png"));
        let source = Source::from_rgba(&source_pixels, source_size).unwrap();

        let (template_pixels, template_size) = rgba_fixture(include_bytes!(
            "../../../../core/test-data/Crown_icon_transparent.png"
        ));
        let template = Template::from_rgba(&template_pixels, template_size).unwrap();

        let cancellation_token = CancellationToken::new();
        let (progress_sender, _) = mpsc::unbounded_channel::<FindImageProgress>();

        let result = source
            .find_template_all(
                &template,
                FindImageTemplateOptions::default(),
                &cancellation_token,
                &progress_sender,
            )
            .unwrap();

        assert_eq!(result.len(), 2);
        let match_positions = result.iter().map(|result| result.position).collect_vec();
        assert!(match_positions.contains(&point(287, 352)));
        assert!(match_positions.contains(&point(727, 932)));
    }

    #[test]
    fn find_image_errors_when_template_larger_than_source() {
        let source = Source::from_rgba(&vec![0; 8 * 8 * 4], size(8, 8)).unwrap();
        let template = Template::from_rgba(&vec![0; 9 * 9 * 4], size(9, 9)).unwrap();

        let (progress_sender, _) = mpsc::unbounded_channel::<FindImageProgress>();
        let cancellation_token = CancellationToken::new();
        let error = source
            .find_template_all(
                &template,
                FindImageTemplateOptions::default(),
                &cancellation_token,
                &progress_sender,
            )
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("template must fit inside source image")
        );
    }
}
