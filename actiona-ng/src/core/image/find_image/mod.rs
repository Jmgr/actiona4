use std::{borrow::Cow, sync::Arc};

use color_eyre::Result;
use derive_more::Constructor;
use image::DynamicImage;
use macros::FromJsObject;
use opencv::{
    core::{CV_8UC3, Mat, MatTraitConst, Scalar, Vector, extract_channel, split},
    imgproc::{COLOR_BGR2Lab, COLOR_RGB2BGR, COLOR_RGBA2BGR},
};
use tracing::instrument;

use crate::core::{
    image::{
        Image,
        find_image::{
            convert::convert_colors,
            matching::match_template,
            pyramids::{prepare_matching_inputs, resize_result_to_size},
            results::{compute_results, filter_results_by_color},
        },
    },
    point::Point,
    rect::Rect,
};

mod common;
mod convert;
mod matching;
mod pyramids;
mod results;

/// Warms up OpenCV's Lab color space processing code.
pub fn warm_up() -> Result<()> {
    let dummy = Mat::new_rows_cols_with_default(1, 1, CV_8UC3, Scalar::all(0.0))?;
    let _ = convert_colors(&dummy, COLOR_BGR2Lab)?;
    Ok(())
}

#[derive(Debug)]
pub struct LabImage {
    pub lightness: Mat,
    pub a: Mat,
    pub b: Mat,
}

#[derive(Debug)]
pub struct Source {
    pub image: LabImage,
}

#[derive(Debug)]
pub struct Template {
    pub image: LabImage,
    pub mask: Option<Mat>,
}

#[derive(Debug, Clone, Copy, Constructor, PartialEq)]
pub struct Match {
    pub position: Point,
    pub rect: Rect,
    pub score: f64,
}

/// Converts a `DynamicImage` to BGR format, optionally extracting an alpha mask.
fn image_to_bgr(image: &DynamicImage, extract_mask: bool) -> Result<(Mat, Option<Mat>)> {
    match image {
        DynamicImage::ImageRgba8(image) => {
            let (_width, height) = image.dimensions();
            let mat_boxed = Mat::from_slice(image.as_raw())?;
            let mat = mat_boxed.reshape(4, height.try_into()?)?;

            let mask = if extract_mask {
                let mut alpha = Mat::default();
                extract_channel(&mat, &mut alpha, 3)?;
                Some(alpha)
            } else {
                None
            };

            Ok((convert_colors(&mat, COLOR_RGBA2BGR)?, mask))
        }
        image => {
            let image = image.to_rgb8();
            let (_width, height) = image.dimensions();
            let mat_boxed = Mat::from_slice(image.as_raw())?;
            let mat = mat_boxed.reshape(3, height.try_into()?)?;
            Ok((convert_colors(&mat, COLOR_RGB2BGR)?, None))
        }
    }
}

/// Converts a BGR `Mat` to a `LabImage` by converting to Lab color space and splitting channels.
fn bgr_to_lab(bgr: &Mat) -> Result<LabImage> {
    let lab = convert_colors(bgr, COLOR_BGR2Lab)?;

    let mut channels = Vector::new();
    split(&lab, &mut channels)?;

    Ok(LabImage {
        lightness: channels.get(0)?,
        a: channels.get(1)?,
        b: channels.get(2)?,
    })
}

impl Image {
    #[instrument(skip_all)]
    pub fn to_source(&self) -> Result<Arc<Source>> {
        if let Some(source) = self.source.get() {
            return Ok(source);
        }

        let (bgr, _) = image_to_bgr(self.to_inner(), false)?;
        let source = Arc::new(Source {
            image: bgr_to_lab(&bgr)?,
        });

        self.source.set(source.clone());

        Ok(source)
    }

    #[instrument(skip_all)]
    pub fn to_template(&self) -> Result<Arc<Template>> {
        if let Some(template) = self.template.get() {
            return Ok(template);
        }

        let (bgr, mask) = image_to_bgr(self.to_inner(), true)?;
        let template = Arc::new(Template {
            image: bgr_to_lab(&bgr)?,
            mask,
        });

        self.template.set(template.clone());

        Ok(template)
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
    pub fn find_template(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
    ) -> Result<Vec<Match>> {
        self.find_template_impl(template, options, false)
    }

    /// Find the best match of a template in this source image.
    #[instrument(skip_all)]
    pub fn find_template_one(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
    ) -> Result<Option<Match>> {
        let matches = self.find_template_impl(template, options, true)?;
        Ok(matches.into_iter().next())
    }

    #[instrument(skip_all)]
    fn find_template_impl(
        &self,
        template: &Template,
        options: FindImageTemplateOptions,
        search_one: bool,
    ) -> Result<Vec<Match>> {
        let source_lightness = Cow::Borrowed(&self.image.lightness);
        let template_lightness = Cow::Borrowed(&template.image.lightness);

        // Only use the mask if transparency is enabled
        let mask_to_use = if options.use_transparency {
            template.mask.as_ref().map(Cow::Borrowed)
        } else {
            None
        };

        // Reduce the size if needed
        let (source_lightness, template_lightness, template_mask) = prepare_matching_inputs(
            source_lightness,
            template_lightness,
            mask_to_use,
            options.downscale,
        )?;

        // Apply template matching
        let mut result = match_template(
            source_lightness.as_ref(),
            template_lightness.as_ref(),
            template_mask.as_deref(),
        )?;

        // Resize the result if needed
        if options.downscale > 0 {
            result = resize_result_to_size(&result, source_lightness.size()?)?;
        }

        let template_size = template.image.lightness.size()?;
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

        let matches = compute_results(
            &result,
            template_size,
            options.match_threshold,
            search_one,
            options.non_maximum_suppression_radius,
        )?;

        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan};

    use crate::{
        core::image::{Image, find_image::FindImageTemplateOptions},
        runtime::Runtime,
    };

    #[test]
    fn test_find_image() {
        let _ = fmt()
            .with_env_filter(EnvFilter::new("info"))
            .with_span_events(FmtSpan::CLOSE)
            .try_init();

        Runtime::test(async |_runtime| {
            let source = include_bytes!("../../../../../tests/input.png");
            let source = Image::from_bytes(source).unwrap();
            let source = source.to_source().unwrap();

            let template = include_bytes!("../../../../../tests/pear.png");
            let template = Image::from_bytes(template).unwrap();
            let template = template.to_template().unwrap();

            let result = source
                .find_template(
                    &template,
                    FindImageTemplateOptions {
                        use_colors: true,
                        use_transparency: true,
                        match_threshold: 0.8,
                        non_maximum_suppression_radius: Some(10),
                        downscale: 0,
                    },
                )
                .unwrap();

            assert_eq!(result.len(), 2);
            assert_eq!(result[0].position, crate::core::point::point(1636, 233));
            assert_eq!(result[1].position, crate::core::point::point(237, 231));
        });
    }
}
