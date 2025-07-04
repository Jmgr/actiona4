use std::io;

use convert_case::{Case, Casing};
use image::{
    ColorType, DynamicImage, GenericImage, GenericImageView, ImageReader, ImageResult, RgbaImage,
    imageops::FilterType, metadata::Orientation,
};
use imageproc::{
    drawing::{
        draw_cross, draw_cross_mut, draw_filled_circle, draw_filled_circle_mut,
        draw_filled_ellipse, draw_filled_ellipse_mut, draw_filled_rect, draw_filled_rect_mut,
        draw_hollow_circle, draw_hollow_circle_mut, draw_hollow_ellipse, draw_hollow_ellipse_mut,
        draw_hollow_rect, draw_hollow_rect_mut, draw_line_segment, draw_line_segment_mut,
    },
    geometric_transformations::{self, rotate, rotate_about_center},
};
use macros::{ExposeEnum, FromJsObject};
use rquickjs::{
    Class, Ctx, Exception, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::*,
};
use strum::Display;

use crate::{
    IntoJS,
    core::{
        ValueClass,
        color::js::{JsColor, JsColorParam},
        point::{
            self,
            js::{JsPoint, JsPointParam},
            point,
        },
        rect::{
            js::{JsRect, JsRectParam},
            rect,
        },
    },
};

#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "FlipDirection")]
pub enum JsFlipDirection {
    Horizontal,
    Vertical,
}

/// Resize filters
#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "ResizeFilter")]
pub enum JsResizeFilter {
    Nearest,
    Linear,
    Cubic,
    Gaussian,
    Lanczos3,
}

impl From<JsResizeFilter> for FilterType {
    fn from(value: JsResizeFilter) -> Self {
        use JsResizeFilter::*;

        match value {
            Nearest => FilterType::Nearest,
            Linear => FilterType::Triangle,
            Cubic => FilterType::CatmullRom,
            Gaussian => FilterType::Gaussian,
            Lanczos3 => FilterType::Lanczos3,
        }
    }
}

/// Interpolation algorithms used for rotations
#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "Interpolation")]
pub enum JsInterpolation {
    Nearest,
    Bilinear,
    Bicubic,
}

impl From<JsInterpolation> for geometric_transformations::Interpolation {
    fn from(value: JsInterpolation) -> Self {
        use JsInterpolation::*;

        match value {
            Nearest => geometric_transformations::Interpolation::Nearest,
            Bilinear => geometric_transformations::Interpolation::Bilinear,
            Bicubic => geometric_transformations::Interpolation::Bicubic,
        }
    }
}

/// Resize options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsResizeOptions {
    /// Should the aspect ratio be kept?
    /// @default false
    pub keep_aspect_ratio: bool,

    /// What filter to use
    /// @default ResizeFilter.CUBIC
    pub filter: JsResizeFilter,
}

impl Default for JsResizeOptions {
    fn default() -> Self {
        Self {
            keep_aspect_ratio: false,
            filter: JsResizeFilter::Cubic,
        }
    }
}

/// Blur options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsBlurOptions {
    /// Perform a fast, lower quality blur
    /// @default false
    pub fast: bool,

    /// Standard deviation of the (approximated) Gaussian
    /// @default 2
    pub sigma: f32,
}

impl Default for JsBlurOptions {
    fn default() -> Self {
        Self {
            fast: false,
            sigma: 2.,
        }
    }
}

/// Draw image options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject, Default)]
pub struct JsDrawImageOptions {
    /// Source rectangle
    /// @default Whole image
    pub source_rect: Option<JsRect>,
}

/// Rotation options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsRotationOptions {
    /// Interpolation algorithm (used if the rotation angle is different from 90, 180, and 270 degrees and no center position has been set)
    /// @default Interpolation.BILINEAR
    pub interpolation: JsInterpolation,

    /// Rotation center
    /// @default image center
    pub center: Option<JsPoint>,

    /// Default color, used if the rotation triggers more pixels to be displayed
    /// @default Color.BLACK
    pub default_color: JsColor,
}

impl Default for JsRotationOptions {
    fn default() -> Self {
        Self {
            interpolation: JsInterpolation::Bilinear,
            center: None,
            default_color: JsColor::new(0, 0, 0, 255),
        }
    }
}

/// Drawing options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject, Default)]
pub struct JsDrawingOptions {
    /// Draw a hollow shape instead of a filled one
    /// @default false
    pub hollow: bool,
}

pub type FindImageOptions = super::JsFindImageOptions;

#[derive(Clone, Debug, JsLifetime, PartialEq)]
#[rquickjs::class(rename = "Image")]
pub struct JsImage {
    inner: super::Image,
}

impl JsImage {
    pub fn inner(&self) -> &super::Image {
        &self.inner
    }
}

impl<'js> ValueClass<'js> for JsImage {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        JsFlipDirection::register(ctx)?;
        JsResizeFilter::register(ctx)?;
        JsInterpolation::register(ctx)?;

        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsImage {
    /// @skip
    #[qjs(skip)]
    pub fn new(inner: super::Image) -> Self {
        Self { inner }
    }

    /// Creates a new empty image.
    ///
    /// Example
    /// ```js
    /// let image = new Image(100, 100);
    /// ```
    ///
    /// @constructor
    #[qjs(constructor)]
    pub fn new_js(width: u32, height: u32) -> Self {
        Self {
            inner: super::Image::new(width, height),
        }
    }

    pub fn save(&self, ctx: Ctx<'_>, path: String) -> Result<()> {
        self.inner.save(path).into_js(&ctx)
    }

    #[qjs(static)]
    pub fn load(ctx: Ctx<'_>, path: String) -> Result<Self> {
        let image = ImageReader::open(path)
            .into_js(&ctx)?
            .with_guessed_format()
            .into_js(&ctx)?
            .decode()
            .into_js(&ctx)?;

        Ok(super::Image(image).into())
    }

    #[qjs(get)]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    #[qjs(get)]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string_js(&self) -> String {
        format!("({}, {})", self.width(), self.height())
    }

    #[qjs(rename = "clone")]
    pub fn clone_js(&self) -> Self {
        self.clone()
    }

    /// Invert the colors of this image.
    pub fn invert<'js>(&mut self, this: This<Class<'js, Self>>) -> Class<'js, Self> {
        self.inner.invert();

        this.0
    }

    /// Invert the colors of this image and returns a new image.
    pub fn inverted(&self) -> Self {
        let mut cloned = self.clone();
        cloned.inner.invert();
        cloned
    }

    /// Blur the image.
    pub fn blur<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        options: Opt<JsBlurOptions>,
    ) -> Class<'js, Self> {
        *self.inner = self.blur_impl(options);

        this.0
    }

    /// Blur the image and returns a new image.
    pub fn blurred(&self, options: Opt<JsBlurOptions>) -> Self {
        super::Image(self.blur_impl(options)).into()
    }

    #[qjs(skip)]
    fn blur_impl(&self, options: Opt<JsBlurOptions>) -> DynamicImage {
        let options = options.unwrap_or_default();

        if options.fast {
            self.inner.fast_blur(options.sigma)
        } else {
            self.inner.blur(options.sigma)
        }
    }

    /// Rotate the image.
    pub fn rotate<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        angle: f32,
        options: Opt<JsRotationOptions>,
    ) -> Class<'js, Self> {
        let options = options.unwrap_or_default();

        if let Some(center) = options.center {
            let rgba = self.ensure_rgba();
            *self.inner = DynamicImage::ImageRgba8(rotate(
                rgba,
                (center.get_x() as f32, center.get_y() as f32),
                angle.to_radians(),
                options.interpolation.into(),
                options.default_color.into(),
            ))
        } else {
            match angle {
                0. => {}
                90. => self.inner.apply_orientation(Orientation::Rotate90),
                180. => self.inner.apply_orientation(Orientation::Rotate180),
                270. => self.inner.apply_orientation(Orientation::Rotate270),
                angle => {
                    let rgba = self.ensure_rgba();
                    *self.inner = DynamicImage::ImageRgba8(rotate_about_center(
                        rgba,
                        angle.to_radians(),
                        options.interpolation.into(),
                        options.default_color.into(),
                    ))
                }
            }
        }

        this.0
    }

    /// Rotate the image and returns a new image.
    pub fn rotated(&self, angle: f32, options: Opt<JsRotationOptions>) -> Self {
        let options = options.unwrap_or_default();

        let result = if let Some(center) = options.center {
            DynamicImage::ImageRgba8(rotate(
                &self.inner.to_rgba8(),
                (center.get_x() as f32, center.get_y() as f32),
                angle.to_radians(),
                options.interpolation.into(),
                options.default_color.into(),
            ))
        } else {
            match angle {
                0. => self.inner.clone().into(),
                90. => self.inner.rotate90(),
                180. => self.inner.rotate180(),
                270. => self.inner.rotate270(),
                angle => DynamicImage::ImageRgba8(rotate_about_center(
                    &self.inner.to_rgba8(),
                    angle.to_radians(),
                    options.interpolation.into(),
                    options.default_color.into(),
                )),
            }
        };

        super::Image(result).into()
    }

    /// Flip the image.
    pub fn flip<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        flip_direction: JsFlipDirection,
    ) -> Class<'js, Self> {
        use JsFlipDirection::*;
        let orientation = match flip_direction {
            Horizontal => Orientation::FlipHorizontal,
            Vertical => Orientation::FlipVertical,
        };
        self.inner.apply_orientation(orientation);

        this.0
    }

    /// Flip the image and returns a new image.
    pub fn flipped(&self, flip_direction: JsFlipDirection) -> Self {
        use JsFlipDirection::*;
        super::Image(match flip_direction {
            Horizontal => self.inner.fliph(),
            Vertical => self.inner.flipv(),
        })
        .into()
    }

    /// Hue rotate the image.
    pub fn hue_rotate<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        value: i32,
    ) -> Class<'js, Self> {
        *self.inner = self.inner.huerotate(value);

        this.0
    }

    /// Hue rotate the image and returns a new image.
    pub fn hue_rotated(&self, value: i32) -> Self {
        super::Image(self.inner.huerotate(value)).into()
    }

    /// Transform this image into a grayscale.
    pub fn grayscale<'js>(&mut self, this: This<Class<'js, Self>>) -> Class<'js, Self> {
        *self.inner = self.inner.grayscale();

        this.0
    }

    /// Returns a grayscale version of this image.
    pub fn grayscaled(&self) -> Self {
        super::Image(self.inner.grayscale()).into()
    }

    /// Crops this image.
    pub fn crop<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        rect: JsRectParam,
    ) -> Class<'js, Self> {
        let (x, y, width, height) = rect.0.clamped();
        *self.inner = self.inner.crop_imm(x, y, width, height);

        this.0
    }

    /// Returns a cropped version of this image.
    pub fn cropped(&self, rect: JsRectParam) -> Self {
        let (x, y, width, height) = rect.0.clamped();
        super::Image(self.inner.crop_imm(x, y, width, height)).into()
    }

    /// Resizes this image.
    pub fn resize<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        width: u32,
        height: u32,
        options: Opt<JsResizeOptions>,
    ) -> Class<'js, Self> {
        *self.inner = self.resize_impl(width, height, options);

        this.0
    }

    /// Returns a resized version of this image.
    pub fn resized(&self, width: u32, height: u32, options: Opt<JsResizeOptions>) -> Self {
        super::Image(self.resize_impl(width, height, options)).into()
    }

    #[qjs(skip)]
    fn resize_impl(&self, width: u32, height: u32, options: Opt<JsResizeOptions>) -> DynamicImage {
        let options = options.unwrap_or_default();

        if options.keep_aspect_ratio {
            self.inner.resize(width, height, options.filter.into())
        } else {
            self.inner
                .resize_exact(width, height, options.filter.into())
        }
    }

    /// Brightens or darkens the pixels of this image.
    pub fn adjust_brightness<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        value: i32,
    ) -> Class<'js, Self> {
        *self.inner = self.inner.brighten(value);

        this.0
    }

    /// Returns a brightened or darkened version of this image.
    pub fn adjusted_brightness(&self, value: i32) -> Self {
        super::Image(self.inner.brighten(value)).into()
    }

    /// Adjusts the contrast of this image.
    pub fn adjust_contrast<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        value: f32,
    ) -> Class<'js, Self> {
        *self.inner = self.inner.adjust_contrast(value);

        this.0
    }

    /// Returns a new image with an adjusted contrast.
    pub fn adjusted_contrast(&self, value: f32) -> Self {
        super::Image(self.inner.adjust_contrast(value)).into()
    }

    /// Fill this image with a color.
    pub fn fill<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        color: JsColorParam,
    ) -> Class<'js, Self> {
        let rgba = self.ensure_rgba();

        for pixel in rgba.pixels_mut() {
            *pixel = color.0.into();
        }

        this.0
    }

    /// Fill this image with a color.
    pub fn filled(&self, color: JsColorParam) -> Self {
        let mut rgba = self.inner.0.to_rgba8();

        for pixel in rgba.pixels_mut() {
            *pixel = color.0.into();
        }

        super::Image(DynamicImage::ImageRgba8(rgba)).into()
    }

    #[qjs(skip)]
    fn check_position(&self, ctx: &Ctx<'_>, position: point::Point) -> Result<(u32, u32)> {
        if position.x < 0
            || position.x >= self.width() as i32
            || position.y < 0
            || position.y >= self.height() as i32
        {
            return Err(Exception::throw_message(
                ctx,
                &format!("Invalid position: {}", position),
            ));
        }

        Ok((position.x as u32, position.y as u32))
    }

    /// Returns the value of a pixel.
    pub fn get_pixel(&mut self, ctx: Ctx<'_>, position: JsPointParam) -> Result<JsColor> {
        let (x, y) = self.check_position(&ctx, position.0)?;

        let rgba = self.ensure_rgba();

        Ok((*rgba.get_pixel(x, y)).into())
    }

    /// Returns the value of a pixel.
    pub fn set_pixel<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        position: JsPointParam,
        color: JsColorParam,
    ) -> Result<Class<'js, Self>> {
        let (x, y) = self.check_position(&ctx, position.0)?;

        let rgba = self.ensure_rgba();

        rgba.put_pixel(x, y, color.0.into());

        Ok(this.0)
    }

    /// Creates a new image from a part of this image.
    pub fn copy_region(&self, ctx: Ctx<'_>, rect: JsRectParam) -> Result<Self> {
        let (x, y) = self.check_position(&ctx, point(rect.0.x, rect.0.y))?;

        Ok(super::Image(DynamicImage::ImageRgba8(
            self.inner
                .view(x, y, rect.0.width, rect.0.height)
                .to_image(),
        ))
        .into())
    }

    /// Returns a Rect representing this image.
    pub fn rect(&self) -> JsRect {
        rect(0, 0, self.width(), self.height()).into()
    }

    #[qjs(skip)]
    fn ensure_rgba(&mut self) -> &mut RgbaImage {
        if self.inner.color() != ColorType::Rgba8 {
            *self.inner = DynamicImage::ImageRgba8(self.inner.0.to_rgba8());
        }

        self.inner.as_mut_rgba8().expect("image should be the RGBA")
    }

    /// Draw a cross on this image.
    pub fn draw_cross<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        position: JsPointParam,
        color: JsColorParam,
    ) -> Class<'js, Self> {
        draw_cross_mut(&mut self.inner.0, *color.0, position.0.x, position.0.y);

        this.0
    }

    /// Draw a cross on a copy of this image.
    pub fn with_cross(&self, position: JsPointParam, color: JsColorParam) -> Self {
        super::Image(DynamicImage::ImageRgba8(draw_cross(
            &self.inner.0,
            *color.0,
            position.0.x,
            position.0.y,
        )))
        .into()
    }

    /// Draw a line on this image.
    pub fn draw_line<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        start: JsPointParam,
        end: JsPointParam,
        color: JsColorParam,
    ) -> Class<'js, Self> {
        draw_line_segment_mut(
            &mut self.inner.0,
            (start.0.x as f32, start.0.y as f32),
            (end.0.x as f32, end.0.y as f32),
            *color.0,
        );

        this.0
    }

    /// Draw a line on a copy of this image.
    pub fn with_line(&self, start: JsPointParam, end: JsPointParam, color: JsColorParam) -> Self {
        super::Image(DynamicImage::ImageRgba8(draw_line_segment(
            &self.inner.0,
            (start.0.x as f32, start.0.y as f32),
            (end.0.x as f32, end.0.y as f32),
            *color.0,
        )))
        .into()
    }

    /// Draw a circle on this image.
    pub fn draw_circle<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        center: JsPointParam,
        radius: i32,
        color: JsColorParam,
        options: Opt<JsDrawingOptions>,
    ) -> Class<'js, Self> {
        let options = options.unwrap_or_default();

        if options.hollow {
            draw_hollow_circle_mut(
                &mut self.inner.0,
                (center.0.x, center.0.y),
                radius,
                *color.0,
            );
        } else {
            draw_filled_circle_mut(
                &mut self.inner.0,
                (center.0.x, center.0.y),
                radius,
                *color.0,
            );
        }

        this.0
    }

    /// Draw a circle on a copy of this image.
    pub fn with_circle(
        &self,
        center: JsPointParam,
        radius: i32,
        color: JsColorParam,
        options: Opt<JsDrawingOptions>,
    ) -> Self {
        let options = options.unwrap_or_default();

        let image = if options.hollow {
            draw_hollow_circle(&self.inner.0, (center.0.x, center.0.y), radius, *color.0)
        } else {
            draw_filled_circle(&self.inner.0, (center.0.x, center.0.y), radius, *color.0)
        };

        super::Image(DynamicImage::ImageRgba8(image)).into()
    }

    /// Draw an ellipse on this image.
    pub fn draw_ellipse<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        center: JsPointParam,
        width_radius: i32,
        height_radius: i32,
        color: JsColorParam,
        options: Opt<JsDrawingOptions>,
    ) -> Class<'js, Self> {
        let options = options.unwrap_or_default();

        if options.hollow {
            draw_hollow_ellipse_mut(
                &mut self.inner.0,
                (center.0.x, center.0.y),
                width_radius,
                height_radius,
                *color.0,
            );
        } else {
            draw_filled_ellipse_mut(
                &mut self.inner.0,
                (center.0.x, center.0.y),
                width_radius,
                height_radius,
                *color.0,
            );
        }

        this.0
    }

    /// Draw an ellipse on a copy of this image.
    pub fn with_ellipse(
        &self,
        center: JsPointParam,
        width_radius: i32,
        height_radius: i32,
        color: JsColorParam,
        options: Opt<JsDrawingOptions>,
    ) -> Self {
        let options = options.unwrap_or_default();

        let image = if options.hollow {
            draw_hollow_ellipse(
                &self.inner.0,
                (center.0.x, center.0.y),
                width_radius,
                height_radius,
                *color.0,
            )
        } else {
            draw_filled_ellipse(
                &self.inner.0,
                (center.0.x, center.0.y),
                width_radius,
                height_radius,
                *color.0,
            )
        };

        super::Image(DynamicImage::ImageRgba8(image)).into()
    }

    /// Draw a rectangle on this image.
    pub fn draw_rectangle<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        rect: JsRectParam,
        color: JsColorParam,
        options: Opt<JsDrawingOptions>,
    ) -> Class<'js, Self> {
        let options = options.unwrap_or_default();

        if options.hollow {
            draw_hollow_rect_mut(&mut self.inner.0, rect.0.into(), *color.0);
        } else {
            draw_filled_rect_mut(&mut self.inner.0, rect.0.into(), *color.0);
        }

        this.0
    }

    /// Draw a rectangle on a copy of this image.
    pub fn with_rectangle(
        &self,
        rect: JsRectParam,
        color: JsColorParam,
        options: Opt<JsDrawingOptions>,
    ) -> Self {
        let options = options.unwrap_or_default();

        let image = if options.hollow {
            draw_hollow_rect(&self.inner.0, rect.0.into(), *color.0)
        } else {
            draw_filled_rect(&self.inner.0, rect.0.into(), *color.0)
        };

        super::Image(DynamicImage::ImageRgba8(image)).into()
    }

    /// Draw another image on this image.
    pub fn draw_image<'js>(
        &mut self,
        ctx: Ctx<'js>,
        this: This<Class<'js, Self>>,
        position: JsPointParam,
        image: &JsImage,
        options: Opt<JsDrawImageOptions>,
    ) -> Result<Class<'js, Self>> {
        let options = options.unwrap_or_default();

        let (x, y) = self.check_position(&ctx, position.0)?;

        self.ensure_rgba();

        if let Some(rect) = options.source_rect {
            let (x, y) = self.check_position(&ctx, point(rect.get_x(), rect.get_y()))?;
            let view = image.inner.view(x, y, rect.get_width(), rect.get_height());
            self.inner.copy_from(&view.to_image(), x, y).into_js(&ctx)?;
        } else {
            self.inner.copy_from(&image.inner.0, x, y).into_js(&ctx)?;
        }

        Ok(this.0)
    }

    /// Draw another image on a copy of this image.
    pub fn with_image(
        &self,
        ctx: Ctx<'_>,
        position: JsPointParam,
        image: &JsImage,
        options: Opt<JsDrawImageOptions>,
    ) -> Result<Self> {
        let options = options.unwrap_or_default();

        let (x, y) = self.check_position(&ctx, position.0)?;

        let mut target_image = self.inner.clone();

        if let Some(rect) = options.source_rect {
            let (x, y) = self.check_position(&ctx, point(rect.get_x(), rect.get_y()))?;
            let view = image.inner.view(x, y, rect.get_width(), rect.get_height());
            target_image
                .copy_from(&view.to_image(), x, y)
                .into_js(&ctx)?;
        } else {
            target_image.copy_from(&image.inner.0, x, y).into_js(&ctx)?;
        }

        Ok(target_image.into())
    }

    ///TODO
    pub fn find_image(
        &mut self,
        _ctx: Ctx<'_>,
        _image: &JsImage,
        options: Opt<FindImageOptions>,
    ) -> Result<()> {
        let _options = options.unwrap_or_default();

        /*
        //self.ensure_rgba();

        // TODO

        let source = self.inner.to_rgb8();
        let template = image.inner.to_rgba8();

        let source = dynamic_image_to_mat(&DynamicImage::ImageRgb8(source)).unwrap(); // TODO
        let template = dynamic_image_to_mat(&DynamicImage::ImageRgba8(template)).unwrap();

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

        // Create a result matrix for the matching result
        let result_rows = source.rows() - template_bgr.rows() + 1;
        let result_cols = source.cols() - template_bgr.cols() + 1;
        let mut result = Mat::zeros(result_rows, result_cols, CV_32FC1)
            .unwrap()
            .to_mat()
            .unwrap();

        let start = Instant::now();

        // Perform template matching on the color image with mask
        opencv::imgproc::match_template(
            &source,
            &template_bgr,
            &mut result,
            opencv::imgproc::TM_CCORR_NORMED, // Use TM_CCOEFF_NORMED for better matching
            &template_alpha,                  // Use the alpha channel as the mask
        )
        .unwrap();

        let duration = start.elapsed();
        println!("template matching took {:?}", duration);

        // Set a threshold for good matches
        let match_threshold = 0.99;

        // Collect all matches above the threshold
        let mut match_points: Vec<(point::Point, f32)> = Vec::new();
        for row in 0..result.rows() {
            for col in 0..result.cols() {
                let match_score = *result.at_2d::<f32>(row, col).unwrap();
                if match_score >= match_threshold {
                    match_points.push((point::point(col, row), match_score));
                }
            }
        }

        // Sort matches by score (in descending order)
        match_points.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply non-maximum suppression to remove overlapping matches
        let mut filtered_matches: Vec<(point::Point, f32)> = Vec::new();
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
        for (pt, score) in &filtered_matches {
            let rect = rect::Rect::new(
                pt.x,
                pt.y,
                template_bgr.cols() as u32,
                template_bgr.rows() as u32,
            );
            draw_hollow_rect_mut(
                &mut self.inner.0,
                rect.into(),
                Color::new(255, 0, 0, 255).into(),
            );
            println!(
                "Match found at ({}, {}) with score: {:.3}",
                pt.x, pt.y, score
            );
        }
        */

        Ok(())
    }

    pub fn test(&self, options: JsRotationOptions) -> Result<()> {
        println!("{options:?}");

        Ok(())
    }
}

impl From<JsImage> for super::Image {
    fn from(value: JsImage) -> Self {
        value.inner
    }
}

impl From<super::Image> for JsImage {
    fn from(value: super::Image) -> Self {
        Self { inner: value }
    }
}

impl<T> IntoJS<T> for io::Result<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<T> IntoJS<T> for ImageResult<T> {
    fn into_js(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<'js> Trace<'js> for JsImage {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, ImageReader};
    use opencv::{
        Result,
        core::{
            CV_32FC1, Mat, MatExprTraitConst, MatTraitConst, NORM_MINMAX, Point, min_max_loc,
            no_array, normalize,
        },
        imgproc,
    };
    use tracing_test::traced_test;

    use crate::{eval, runtime::Runtime};

    /// Convert `image::DynamicImage` to `opencv::core::Mat` in BGR format
    fn dynamic_image_to_mat(img: &DynamicImage) -> Result<Mat> {
        let rgb = img.to_rgb8();
        let (_width, height) = rgb.dimensions();

        let mat = Mat::from_slice(&rgb)?;
        let mat = mat.reshape(3, height as i32)?; // 3 channels (RGB)

        let mut mat_bgr = Mat::default();
        opencv::imgproc::cvt_color(&mat, &mut mat_bgr, imgproc::COLOR_RGB2BGR, 0)?;
        Ok(mat_bgr)
    }

    #[test]
    #[traced_test]
    fn test_() {
        Runtime::test_with_js(async |script_engine| {
            // Load images using image crate
            let source_img = ImageReader::open("/media/jmgr/Main/rust/test_ai_actiona/input.png")
                .unwrap()
                .decode()
                .unwrap();
            let template_img = ImageReader::open("/media/jmgr/Main/rust/test_ai_actiona/pear.png")
                .unwrap()
                .decode()
                .unwrap();

            let source = dynamic_image_to_mat(&source_img).unwrap();
            let template = dynamic_image_to_mat(&template_img).unwrap();

            // Create the result matrix
            let result_cols = source.cols() - template.cols() + 1;
            let result_rows = source.rows() - template.rows() + 1;
            let mut result = Mat::zeros(result_rows, result_cols, CV_32FC1)
                .unwrap()
                .to_mat()
                .unwrap();

            imgproc::match_template(
                &source,
                &template,
                &mut result,
                imgproc::TM_CCOEFF_NORMED,
                &no_array(),
            )
            .unwrap();

            // Normalize result
            let mut out_result = result.clone();
            normalize(
                &result,
                &mut out_result,
                0.0,
                1.0,
                NORM_MINMAX,
                -1,
                &no_array(),
            )
            .unwrap();

            // Find the best match location
            let mut min_val = 0.0;
            let mut max_val = 0.0;
            let mut min_loc = Point::default();
            let mut max_loc = Point::default();
            min_max_loc(
                &out_result,
                Some(&mut min_val),
                Some(&mut max_val),
                Some(&mut min_loc),
                Some(&mut max_loc),
                &no_array(),
            )
            .unwrap();

            println!("{min_val} {max_val} {min_loc:?} {max_loc:?}");
        });
    }

    #[test]
    #[traced_test]
    fn test_options() {
        Runtime::test_with_js(async |script_engine| {
            eval::<()>(
                &js_context,
                "let image = new Image(100, 100); image.test({
                    default_color: Color.WHITE,
                    center: new Point(42, 42),
                })",
            )
            .unwrap();
        });
    }
}
