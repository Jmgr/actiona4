use std::{borrow::Cow, sync::Arc};

use color_eyre::{
    Result,
    eyre::{Error, eyre},
};
use derive_more::Constructor;
use macros::FromJsObject;
use opencv::{
    core::{CV_8UC3, Mat, MatTraitConst, Scalar, Vector, extract_channel, split},
    imgproc::{COLOR_BGR2Lab, COLOR_BGRA2BGR, COLOR_RGBA2BGR},
};
use strum::EnumIs;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{
    core::{
        image::{
            Image,
            find_image::{
                convert::convert_colors,
                matching::match_template,
                pyramids::{prepare_matching_inputs, resize_result_to_size},
                results::{compute_results, filter_results_by_color},
            },
        },
        js::task::IsDone,
        point::Point,
        rect::Rect,
    },
    error::CommonError,
};

mod common;
pub mod convert;
mod matching;
mod pyramids;
mod results;

/// Warms up OpenCV's Lab color space processing code.
pub fn warm_up() -> Result<()> {
    let dummy = Mat::new_rows_cols_with_default(1, 1, CV_8UC3, Scalar::all(0.0))?;
    let _ = convert_colors(&dummy, COLOR_BGR2Lab)?;
    Ok(())
}

#[derive(Debug, Clone)]
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

        Ok(LabImage {
            lightness: LabLightnessMat(channels.get(0)?),
            a: LabAMat(channels.get(1)?),
            b: LabBMat(channels.get(2)?),
        })
    }
}

impl BgrMat {
    pub fn from_bgra(data: &[u8], width: u32, height: u32) -> Result<Self> {
        const BYTES_PER_PIXEL: usize = 4;

        let needed = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixel_count| pixel_count.checked_mul(BYTES_PER_PIXEL))
            .ok_or_else(|| eyre!("image dimensions overflow: {width}x{height}"))?;

        if data.len() < needed {
            return Err(eyre!(
                "image data too small: expected {needed} bytes, got {}",
                data.len()
            ));
        }

        // Create a Mat view over the BGRA data
        let bgra_mat = Mat::new_rows_cols_with_bytes::<opencv::core::Vec4b>(
            height.try_into()?,
            width.try_into()?,
            &data[..needed],
        )?;

        // Convert BGRA to BGR
        let bgr = convert_colors(&bgra_mat, COLOR_BGRA2BGR)?;

        Ok(Self(bgr))
    }
}

#[derive(Debug, Clone)]
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

impl TryFrom<&Image> for Arc<Source> {
    type Error = Error;

    fn try_from(value: &Image) -> Result<Self, Self::Error> {
        if let Some(source) = value.source.get() {
            return Ok(source);
        }

        let (bgr, _) = value.to_bgr(false)?;
        let source = Arc::new(Source {
            image: LabImage::try_from(&bgr)?,
        });

        value.source.set(source.clone());

        Ok(source)
    }
}

impl Source {
    pub fn from_bgra(data: &[u8], width: u32, height: u32) -> Result<Arc<Self>> {
        let bgr = BgrMat::from_bgra(data, width, height)?;
        let lab = LabImage::try_from(&bgr)?;

        Ok(Arc::new(Source { image: lab }))
    }
}

#[derive(Debug)]
pub struct Template {
    pub image: LabImage,
    pub mask: Option<MaskMat>,
}

impl TryFrom<&Image> for Arc<Template> {
    type Error = Error;

    fn try_from(value: &Image) -> Result<Self, Self::Error> {
        if let Some(template) = value.template.get() {
            return Ok(template);
        }

        let (bgr, mask) = value.to_bgr(true)?;
        let template = Arc::new(Template {
            image: LabImage::try_from(&bgr)?,
            mask,
        });

        value.template.set(template.clone());

        Ok(template)
    }
}

#[derive(Clone, Constructor, Copy, Debug, PartialEq)]
pub struct Match {
    pub position: Point,
    pub rect: Rect,
    pub score: f64,
}

impl Match {
    /// Returns a new Match with position and rect offset by the given origin point.
    #[must_use]
    pub fn offset(self, origin: Point) -> Self {
        Self {
            position: self.position + origin,
            rect: Rect {
                top_left: self.rect.top_left + origin,
                ..self.rect
            },
            score: self.score,
        }
    }
}

impl Image {
    /// Converts an Image to the BGR format, optionally extracting an alpha mask.
    pub fn to_bgr(&self, extract_mask: bool) -> Result<(BgrMat, Option<MaskMat>)> {
        let image = self.as_rgba8();
        let height = image.height();
        let mat_boxed = Mat::from_slice(image.as_raw())?;
        let mat = mat_boxed.reshape(4, height.try_into()?)?;

        let mask = if extract_mask {
            let mut alpha = Mat::default();
            extract_channel(&mat, &mut alpha, 3)?;
            Some(MaskMat(alpha))
        } else {
            None
        };

        Ok((BgrMat(convert_colors(&mat, COLOR_RGBA2BGR)?), mask))
    }
}

#[derive(Clone, Copy, Debug, Default, EnumIs, Eq, PartialEq)]
pub enum FindImageStage {
    Capturing,
    #[default]
    Preparing,
    Downscaling,
    Matching,
    Filtering,
    ComputingResults,
    Finished,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FindImageProgress {
    pub stage: FindImageStage,
    pub percent: u8,
}

impl FindImageProgress {
    #[must_use]
    pub const fn new(stage: FindImageStage, percent: u8) -> Self {
        Self { stage, percent }
    }
}

impl IsDone for FindImageProgress {
    fn is_done(&self) -> bool {
        self.stage.is_finished()
    }
}

/// Find image template options
/// @options
#[derive(Clone, Debug, FromJsObject, PartialEq)]
pub struct FindImageTemplateOptions {
    /// Use color matching.
    /// @default `true`
    pub use_colors: bool,

    /// Use template transparency.
    /// @default `true`
    pub use_transparency: bool,

    /// Matching threshold.
    /// Values are between 0 (worst) to 1 (best).
    /// @default `0.8`
    pub match_threshold: f32,

    /// Radius to consider proximity (in pixels).
    /// @default `10`
    pub non_maximum_suppression_radius: Option<i32>,

    /// How many times should the source image and the template be downscaled?
    /// @default `0`
    pub downscale: u64,
}

impl Default for FindImageTemplateOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            use_transparency: true,
            match_threshold: 0.8,
            non_maximum_suppression_radius: Some(10),
            downscale: 0,
        }
    }
}

impl Source {
    /// Find all occurrences of a template in this source image.
    #[instrument(skip_all)]
    pub fn find_template_all(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        self.find_template_impl(template, options, false, cancellation_token, progress)
    }

    /// Find the best match of a template in this source image.
    #[instrument(skip_all)]
    pub fn find_template(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
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
        cancellation_token: CancellationToken,
        progress: watch::Sender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        // Check cancellation at the start
        if cancellation_token.is_cancelled() {
            return Err(CommonError::Cancelled.into());
        }

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
            return Err(CommonError::Cancelled.into());
        }

        progress.send_replace(FindImageProgress::new(FindImageStage::Downscaling, 10));

        // Reduce the size if needed
        let (source_lightness, template_lightness, template_mask) = prepare_matching_inputs(
            source_lightness,
            template_lightness,
            mask_to_use,
            options.downscale,
        )?;

        // Check before expensive template matching
        if cancellation_token.is_cancelled() {
            return Err(CommonError::Cancelled.into());
        }

        progress.send_replace(FindImageProgress::new(FindImageStage::Matching, 20));

        // Apply template matching
        let mut result = match_template(
            source_lightness.as_ref(),
            template_lightness.as_ref(),
            template_mask.as_deref(),
            cancellation_token.clone(),
            progress.clone(),
        )?;

        // Resize the result if needed
        if options.downscale > 0 {
            result = resize_result_to_size(&result, source_lightness.0.size()?)?;
        }

        // Check before color filtering
        if cancellation_token.is_cancelled() {
            return Err(CommonError::Cancelled.into());
        }

        progress.send_replace(FindImageProgress::new(FindImageStage::Filtering, 70));

        let template_size = template.image.lightness.0.size()?;
        if options.use_colors {
            filter_results_by_color(
                &mut result,
                &self.image.a,
                &self.image.b,
                &template.image.a,
                &template.image.b,
                template_mask.as_deref(),
                template_size,
                options.match_threshold,
            )?;
        }

        // Final check before computing results
        if cancellation_token.is_cancelled() {
            return Err(CommonError::Cancelled.into());
        }

        progress.send_replace(FindImageProgress::new(FindImageStage::ComputingResults, 90));

        let matches = compute_results(
            &result,
            template_size,
            options.match_threshold,
            search_one,
            options.non_maximum_suppression_radius,
        )?;

        progress.send_replace(FindImageProgress::new(FindImageStage::Finished, 100));

        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan};

    use tokio::sync::watch;

    use crate::{
        core::image::{
            Image,
            find_image::{FindImageProgress, FindImageTemplateOptions, Source, Template},
        },
        runtime::Runtime,
    };

    #[test]
    fn test_find_image() {
        let _ = fmt()
            .with_env_filter(EnvFilter::new("info"))
            .with_span_events(FmtSpan::CLOSE)
            .try_init();

        Runtime::test(async |runtime| {
            let source = include_bytes!("../../../../../tests/input.png");
            let source = Image::from_bytes(source).unwrap();
            let source = Arc::<Source>::try_from(&source).unwrap();

            let template = include_bytes!("../../../../../tests/pear.png");
            let template = Image::from_bytes(template).unwrap();
            let template = Arc::<Template>::try_from(&template).unwrap();

            let cancellation_token = runtime.cancellation_token();
            let (progress_sender, _) = watch::channel(FindImageProgress::default());

            let result = source
                .find_template_all(
                    &template,
                    FindImageTemplateOptions {
                        use_colors: true,
                        use_transparency: true,
                        match_threshold: 0.8,
                        non_maximum_suppression_radius: Some(10),
                        downscale: 0,
                    },
                    cancellation_token,
                    progress_sender,
                )
                .unwrap();

            assert_eq!(result.len(), 2);
            assert_eq!(result[0].position, crate::core::point::point(1636, 233));
            assert_eq!(result[1].position, crate::core::point::point(237, 231));
        });
    }
}
