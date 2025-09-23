use std::{
    borrow::Cow,
    io::Cursor,
    ops::{Deref, DerefMut},
    time::Instant,
};

use eyre::Result;
use image::{ColorType, DynamicImage, GrayImage, ImageReader, RgbImage, RgbaImage};
use imageproc::drawing::draw_hollow_rect_mut;
use macros::FromJsObject;
use opencv::{
    core::{AlgorithmHint, Mat, MatExprTraitConst, MatTraitConst, CV_32FC1},
    imgproc::{cvt_color, COLOR_RGB2BGR, COLOR_RGBA2BGRA},
};

use crate::core::{
    color::Color,
    point::{Point, point},
    rect::Rect,
};

pub mod js;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Image(DynamicImage);

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self(DynamicImage::new(width, height, ColorType::Rgba8))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;
        Ok(Self(image))
    }

    pub fn into_inner(self) -> DynamicImage {
        self.0
    }
}

/// Find image options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsFindImageOptions {
    grayscale: bool,
    use_alpha: bool,
}

impl Default for JsFindImageOptions {
    fn default() -> Self {
        Self {
            grayscale: false,
            use_alpha: true,
        }
    }
}

impl Image {
    pub fn to_rgb8(&'_ self) -> Cow<'_, RgbImage> {
        if let DynamicImage::ImageRgb8(image) = &self.0 {
            Cow::Borrowed(image)
        } else {
            Cow::Owned(self.0.to_rgb8())
        }
    }

    pub fn to_rgba8(&'_ self) -> Cow<'_, RgbaImage> {
        if let DynamicImage::ImageRgba8(image) = &self.0 {
            Cow::Borrowed(image)
        } else {
            Cow::Owned(self.0.to_rgba8())
        }
    }

    pub fn to_luma8(&'_ self) -> Cow<'_, GrayImage> {
        if let DynamicImage::ImageLuma8(image) = &self.0 {
            Cow::Borrowed(image)
        } else {
            Cow::Owned(self.0.to_luma8())
        }
    }

    pub fn gray_image_to_mat(image: &GrayImage) -> Result<Mat> {
        let (_width, height) = image.dimensions();
        let data = image.as_raw();
        let mat_boxed = Mat::from_slice(data)?;
        let mat = mat_boxed.reshape(1, height as i32)?;
        Ok(mat.try_clone()?)
    }

    pub fn rgb_image_to_mat(image: &RgbImage) -> Result<Mat> {
        let (_width, height) = image.dimensions();
        let data = image.as_raw();
        let mat_boxed = Mat::from_slice(data)?;
        let mat = mat_boxed.reshape(3, height as i32)?;
        let mut mat_bgr = Mat::default();
        cvt_color(&mat, &mut mat_bgr, COLOR_RGB2BGR, 0, AlgorithmHint::ALGO_HINT_DEFAULT)?;
        Ok(mat_bgr)
    }

    pub fn rgba_image_to_mat(image: &RgbaImage) -> Result<Mat> {
        let (_width, height) = image.dimensions();
        let data = image.as_raw();
        let mat_boxed = Mat::from_slice(data)?;
        let mat = mat_boxed.reshape(4, height as i32)?;
        let mut mat_bgr = Mat::default();
        cvt_color(&mat, &mut mat_bgr, COLOR_RGBA2BGRA, 0, AlgorithmHint::ALGO_HINT_DEFAULT)?;
        Ok(mat_bgr)
    }

    pub fn find_image(&self, image: &Self, options: JsFindImageOptions) -> Result<()> {
        let (source, template, _mask) = if options.grayscale {
            let source = Self::gray_image_to_mat(&self.to_luma8())?;

            let (template, mask) = if options.use_alpha {
                let template = Self::rgba_image_to_mat(&image.to_rgba8())?;

                // Split template channels to extract the alpha channel
                let mut rgba_channels = opencv::core::Vector::<Mat>::new();
                opencv::core::split(&template, &mut rgba_channels).unwrap();

                let template_alpha = rgba_channels.get(3).unwrap(); // Alpha channel

                let template = Self::gray_image_to_mat(&image.to_luma8())?;

                (template, Some(template_alpha))
            } else {
                (Self::gray_image_to_mat(&image.to_luma8())?, None)
            };

            (source, template, mask)
        } else {
            let source = Self::rgb_image_to_mat(&self.to_rgb8())?;

            let (template, mask) = if options.use_alpha {
                let template = Self::rgba_image_to_mat(&image.to_rgba8())?;

                // Split template channels to extract the alpha channel
                let mut rgba_channels = opencv::core::Vector::<Mat>::new();
                opencv::core::split(&template, &mut rgba_channels).unwrap();

                let template_alpha = rgba_channels.get(3).unwrap(); // Alpha channel

                // Remove the alpha channel from the template to get BGR
                let mut template_bgr = Mat::default();
                let mut bgr_channels = opencv::core::Vector::<Mat>::new();

                // Add the individual channels to the OpenCV Vector
                bgr_channels.push(rgba_channels.get(0).unwrap());
                bgr_channels.push(rgba_channels.get(1).unwrap());
                bgr_channels.push(rgba_channels.get(2).unwrap());

                // Merge the BGR channels into a single BGR image
                opencv::core::merge(&bgr_channels, &mut template_bgr).unwrap();

                (template_bgr, Some(template_alpha))
            } else {
                (Self::rgb_image_to_mat(&image.to_rgb8())?, None)
            };
            (source, template, mask)
        };

        // Create a result matrix for the matching result
        let result_rows = source.rows() - template.rows() + 1;
        let result_cols = source.cols() - template.cols() + 1;
        let result = Mat::zeros(result_rows, result_cols, CV_32FC1)
            .unwrap()
            .to_mat()
            .unwrap();

        let start = Instant::now();

        /*
        if let Some(mask) = mask {
            // Perform template matching on the color image with mask
            opencv::imgproc::match_template(
                &source,
                &template,
                &mut result,
                opencv::imgproc::TM_CCOEFF_NORMED, // Use TM_CCOEFF_NORMED for better matching
                &mask,                             // Use the alpha channel as the mask
            )
            .unwrap();
        } else {
            opencv::imgproc::match_template(
                &source,
                &template,
                &mut result,
                opencv::imgproc::TM_CCOEFF_NORMED, // Use TM_CCOEFF_NORMED for better matching
                &no_array(),
            )
            .unwrap();
        }
        */

        let duration = start.elapsed();
        println!("template matching took {duration:?}");

        // Set a threshold for good matches
        let match_threshold = 0.8;

        // Collect all matches above the threshold
        let mut match_points: Vec<(Point, f32)> = Vec::new();
        for row in 0..result.rows() {
            for col in 0..result.cols() {
                let match_score = *result.at_2d::<f32>(row, col).unwrap();
                if match_score >= match_threshold {
                    match_points.push((point(col, row), match_score));
                }
            }
        }

        // Sort matches by score (in descending order)
        match_points.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply non-maximum suppression to remove overlapping matches
        let mut filtered_matches: Vec<(Point, f32)> = Vec::new();
        let suppression_radius = 10; // Define a radius to consider proximity (in pixels)

        for (pt, score) in match_points {
            let mut should_keep = true;
            for (existing_pt, _) in &filtered_matches {
                let dist_x = (pt.x - existing_pt.x).abs();
                let dist_y = (pt.y - existing_pt.y).abs();
                if dist_x < suppression_radius && dist_y < suppression_radius {
                    should_keep = false; // Suppress this match
                    break;
                }
            }

            if should_keep {
                filtered_matches.push((pt, score));
            }
        }

        // Draw rectangles around the filtered matches
        let mut result = self.clone();
        for (pt, score) in &filtered_matches {
            let rect = Rect::new(pt.x, pt.y, template.cols() as u32, template.rows() as u32);
            draw_hollow_rect_mut(
                &mut result.0,
                rect.into(),
                Color::new(255, 0, 0, 255).into(),
            );
            println!(
                "Match found at ({}, {}) with score: {:.3}",
                pt.x, pt.y, score
            );
        }
        result
            .save("/media/jmgr/Main/rust/test_ai_actiona/output.png")
            .unwrap();

        Ok(())
    }
}

impl From<Image> for DynamicImage {
    fn from(value: Image) -> Self {
        value.0
    }
}

impl From<DynamicImage> for Image {
    fn from(value: DynamicImage) -> Self {
        Self(value)
    }
}

impl Deref for Image {
    type Target = DynamicImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use image::ImageReader;
    use tracing_test::traced_test;

    use crate::{
        core::image::{Image, JsFindImageOptions},
        runtime::Runtime,
    };

    #[test]
    #[traced_test]
    fn test_find_image() {
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
            println!("template: {:?}", template.color());
            let source = Image(source);
            let template = Image(template);
            source
                .find_image(
                    &template,
                    JsFindImageOptions {
                        grayscale: true,
                        use_alpha: true,
                    },
                )
                .unwrap();
        });
    }
}
