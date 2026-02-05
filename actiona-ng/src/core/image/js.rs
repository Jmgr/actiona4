use std::{io, sync::Arc};

use image::{ImageReader, ImageResult};
use itertools::Itertools;
use macros::{FromJsObject, FromSerde, IntoSerde};
use rquickjs::{
    Class, Ctx, Exception, JsLifetime, Result, TypedArray,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::{Opt, This},
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::{
    IntoJsResult,
    core::{
        color::js::{JsColor, JsColorLike},
        image::{
            BlurOptions, DrawImageOptions, DrawTextOptions, DrawingOptions, FlipDirection,
            Interpolation, ResizeFilter, ResizeOptions, RotationOptions, TextHorizontalAlign,
            TextVerticalAlign,
            find_image::{Source, Template},
        },
        js::classes::{HostClass, ValueClass, register_enum},
        point::js::{JsPoint, JsPointLike},
        rect::js::{JsRect, JsRectLike},
    },
    error::CommonError,
};

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "FlipDirection")]
pub enum JsFlipDirection {
    Horizontal,
    Vertical,
}

impl From<JsFlipDirection> for FlipDirection {
    fn from(value: JsFlipDirection) -> Self {
        match value {
            JsFlipDirection::Horizontal => Self::Horizontal,
            JsFlipDirection::Vertical => Self::Vertical,
        }
    }
}

/// Resize filters
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "ResizeFilter")]
pub enum JsResizeFilter {
    Nearest,
    Linear,
    Cubic,
    Gaussian,
    Lanczos3,
}

impl From<JsResizeFilter> for ResizeFilter {
    fn from(value: JsResizeFilter) -> Self {
        match value {
            JsResizeFilter::Nearest => Self::Nearest,
            JsResizeFilter::Linear => Self::Linear,
            JsResizeFilter::Cubic => Self::Cubic,
            JsResizeFilter::Gaussian => Self::Gaussian,
            JsResizeFilter::Lanczos3 => Self::Lanczos3,
        }
    }
}

/// Interpolation algorithms used for rotations
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "Interpolation")]
pub enum JsInterpolation {
    Nearest,
    Bilinear,
    Bicubic,
}

impl From<JsInterpolation> for Interpolation {
    fn from(value: JsInterpolation) -> Self {
        match value {
            JsInterpolation::Nearest => Self::Nearest,
            JsInterpolation::Bilinear => Self::Bilinear,
            JsInterpolation::Bicubic => Self::Bicubic,
        }
    }
}

/// Resize options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsResizeOptions {
    /// Should the aspect ratio be kept?
    /// @default `false`
    pub keep_aspect_ratio: bool,

    /// What filter to use
    /// @default `ResizeFilter.Cubic`
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

impl From<JsResizeOptions> for ResizeOptions {
    fn from(value: JsResizeOptions) -> Self {
        Self {
            keep_aspect_ratio: value.keep_aspect_ratio,
            filter: value.filter.into(),
        }
    }
}

/// Blur options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsBlurOptions {
    /// Perform a fast, lower quality blur
    /// @default `false`
    pub fast: bool,

    /// Standard deviation of the (approximated) Gaussian
    /// @default `2`
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

impl From<JsBlurOptions> for BlurOptions {
    fn from(value: JsBlurOptions) -> Self {
        Self {
            fast: value.fast,
            sigma: value.sigma,
        }
    }
}

/// Draw image options
/// @options
#[derive(Clone, Copy, Debug, Default, FromJsObject)]
pub struct JsDrawImageOptions {
    /// Source rectangle.
    /// `undefined` means the whole image.
    /// @default `undefined`
    pub source_rect: Option<JsRect>,
}

impl From<JsDrawImageOptions> for DrawImageOptions {
    fn from(value: JsDrawImageOptions) -> Self {
        Self {
            source_rect: value.source_rect.map(Into::into),
        }
    }
}

/// Rotation options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsRotationOptions {
    /// Interpolation algorithm (used if the rotation angle is different from 90, 180, and 270 degrees and no center position has been set)
    /// @default `Interpolation.Bilinear`
    pub interpolation: JsInterpolation,

    /// Rotation center
    /// @default image center
    pub center: Option<JsPoint>,

    /// Default color, used if the rotation triggers more pixels to be displayed
    /// @default `Color.Black`
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

impl From<JsRotationOptions> for RotationOptions {
    fn from(value: JsRotationOptions) -> Self {
        Self {
            interpolation: value.interpolation.into(),
            center: value.center.map(Into::into),
            default_color: value.default_color.into(),
        }
    }
}

/// Drawing options
/// @options
#[derive(Clone, Copy, Debug, Default, FromJsObject)]
pub struct JsDrawingOptions {
    /// Draw a hollow shape instead of a filled one
    /// @default `false`
    pub hollow: bool,
}

impl From<JsDrawingOptions> for DrawingOptions {
    fn from(value: JsDrawingOptions) -> Self {
        Self {
            hollow: value.hollow,
        }
    }
}

/// Horizontal alignment for text drawing.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "TextHorizontalAlign")]
pub enum JsTextHorizontalAlign {
    Left,
    Center,
    Right,
}

impl From<JsTextHorizontalAlign> for TextHorizontalAlign {
    fn from(value: JsTextHorizontalAlign) -> Self {
        match value {
            JsTextHorizontalAlign::Left => Self::Left,
            JsTextHorizontalAlign::Center => Self::Center,
            JsTextHorizontalAlign::Right => Self::Right,
        }
    }
}

/// Vertical alignment for text drawing.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "TextVerticalAlign")]
pub enum JsTextVerticalAlign {
    Top,
    Middle,
    Bottom,
}

impl From<JsTextVerticalAlign> for TextVerticalAlign {
    fn from(value: JsTextVerticalAlign) -> Self {
        match value {
            JsTextVerticalAlign::Top => Self::Top,
            JsTextVerticalAlign::Middle => Self::Middle,
            JsTextVerticalAlign::Bottom => Self::Bottom,
        }
    }
}

/// Text drawing options.
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct JsDrawTextOptions {
    /// Font size in pixels.
    /// @default `16`
    pub font_size: f32,

    /// Multiplier applied to the default line height when rendering multi-line text.
    /// @default `1`
    pub line_spacing: f32,

    /// Horizontal alignment relative to the provided position.
    /// @default `TextHorizontalAlign.Left`
    pub horizontal_align: JsTextHorizontalAlign,

    /// Vertical alignment relative to the provided position.
    /// @default `TextVerticalAlign.Top`
    pub vertical_align: JsTextVerticalAlign,
}

impl Default for JsDrawTextOptions {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_spacing: 1.0,
            horizontal_align: JsTextHorizontalAlign::Left,
            vertical_align: JsTextVerticalAlign::Top,
        }
    }
}

impl From<JsDrawTextOptions> for DrawTextOptions {
    fn from(value: JsDrawTextOptions) -> Self {
        Self {
            font_size: value.font_size,
            line_spacing: value.line_spacing,
            horizontal_align: value.horizontal_align.into(),
            vertical_align: value.vertical_align.into(),
        }
    }
}

pub type JsFindImageOptions = super::find_image::FindImageTemplateOptions;

/// A match returned by a find_image call.
///
/// @prop position: Point // the position on the source image where the target image was found
/// @prop rect: Rect // the rectangle on the source image where the target image was found
/// @prop score: number // the score for this match, goes from 0 (worst) to 1 (best)
#[derive(Clone, Copy, Debug, JsLifetime, PartialEq)]
#[rquickjs::class(rename = "Match")]
pub struct JsMatch {
    inner: super::find_image::Match,
}

impl HostClass<'_> for JsMatch {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMatch {
    /// @skip
    #[qjs(get)]
    #[must_use]
    pub fn position(&self) -> JsPoint {
        self.inner.position.into()
    }

    /// @skip
    #[qjs(get)]
    #[must_use]
    pub fn rect(&self) -> JsRect {
        self.inner.rect.into()
    }

    /// @skip
    #[qjs(get)]
    #[must_use]
    pub fn score(&self) -> f64 {
        self.inner.score
    }

    /// Returns true if a Match equals another.
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Returns a string representation of this Match.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        format!(
            "({}, {}, {})",
            self.inner.position, self.inner.rect, self.inner.score
        )
    }

    /// Clones this Match.
    #[qjs(rename = "clone")]
    #[must_use]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    /// @skip
    #[qjs(skip)]
    #[must_use]
    pub const fn inner(&self) -> super::find_image::Match {
        self.inner
    }
}

impl<'js> Trace<'js> for JsMatch {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl From<super::find_image::Match> for JsMatch {
    fn from(value: super::find_image::Match) -> Self {
        Self { inner: value }
    }
}

#[derive(Clone, Debug, JsLifetime, PartialEq)]
#[rquickjs::class(rename = "Image")]
pub struct JsImage {
    inner: super::Image,
}

impl JsImage {
    /// @skip
    #[must_use]
    pub const fn to_inner(&self) -> &super::Image {
        &self.inner
    }

    /// @skip
    #[must_use]
    pub fn into_inner(self) -> super::Image {
        self.inner
    }
}

impl<'js> ValueClass<'js> for JsImage {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_enum::<JsFlipDirection>(ctx)?;
        register_enum::<JsResizeFilter>(ctx)?;
        register_enum::<JsInterpolation>(ctx)?;
        register_enum::<JsTextHorizontalAlign>(ctx)?;
        register_enum::<JsTextVerticalAlign>(ctx)?;

        Ok(())
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsImage {
    /// @skip
    #[qjs(skip)]
    #[must_use]
    pub const fn new(inner: super::Image) -> Self {
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
    #[must_use]
    pub fn new_js(width: u32, height: u32) -> Self {
        Self {
            inner: super::Image::new(width, height),
        }
    }

    #[qjs(static)]
    pub fn from_bytes(ctx: Ctx<'_>, bytes: TypedArray<'_, u8>) -> Result<Self> {
        let bytes = bytes
            .as_bytes()
            .ok_or(CommonError::DetachedArrayBuffer)
            .into_js_result(&ctx)?;

        Ok(Self {
            inner: super::Image::from_bytes(bytes).into_js_result(&ctx)?,
        })
    }

    // TODO: make this async
    pub fn save(&self, ctx: Ctx<'_>, path: String) -> Result<()> {
        self.inner.save(path).into_js_result(&ctx)
    }

    // TODO: make this async
    #[qjs(static)]
    pub fn load(ctx: Ctx<'_>, path: String) -> Result<Self> {
        let image = ImageReader::open(path)
            .into_js_result(&ctx)?
            .with_guessed_format()
            .into_js_result(&ctx)?
            .decode()
            .into_js_result(&ctx)?;

        Ok(super::Image::from_dynamic_image(image).into())
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        format!("({}, {})", self.width(), self.height())
    }

    #[qjs(rename = "clone")]
    #[must_use]
    pub fn clone_js(&self) -> Self {
        self.clone()
    }

    /// Invert the colors of this image.
    pub fn invert_colors<'js>(&mut self, this: This<Class<'js, Self>>) -> Class<'js, Self> {
        self.inner.invert_mut();

        this.0
    }

    /// Invert the colors of this image and returns a new image.
    #[must_use]
    pub fn inverted_colors(&self) -> Self {
        self.inner.inverted().into()
    }

    /// Blur the image.
    pub fn blur<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        options: Opt<JsBlurOptions>,
    ) -> Class<'js, Self> {
        self.inner.blur_mut(options.unwrap_or_default().into());

        this.0
    }

    /// Blur the image and returns a new image.
    #[must_use]
    pub fn blurred(&self, options: Opt<JsBlurOptions>) -> Self {
        self.inner
            .blurred(options.unwrap_or_default().into())
            .into()
    }

    /// Rotate the image.
    pub fn rotate<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        angle: f32,
        options: Opt<JsRotationOptions>,
    ) -> Class<'js, Self> {
        self.inner
            .rotate_mut(angle, options.unwrap_or_default().into());

        this.0
    }

    /// Rotate the image and returns a new image.
    #[must_use]
    pub fn rotated(&self, angle: f32, options: Opt<JsRotationOptions>) -> Self {
        self.inner
            .rotated(angle, options.unwrap_or_default().into())
            .into()
    }

    /// Flip the image.
    pub fn flip<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        flip_direction: JsFlipDirection,
    ) -> Class<'js, Self> {
        self.inner.flip_mut(flip_direction.into());

        this.0
    }

    /// Flip the image and returns a new image.
    #[must_use]
    pub fn flipped(&self, flip_direction: JsFlipDirection) -> Self {
        self.inner.flipped(flip_direction.into()).into()
    }

    /// Hue rotate the image.
    pub fn hue_rotate<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        value: i32,
    ) -> Class<'js, Self> {
        self.inner.hue_rotate_mut(value);

        this.0
    }

    /// Hue rotate the image and returns a new image.
    #[must_use]
    pub fn with_hue_rotation(&self, value: i32) -> Self {
        self.inner.hue_rotated(value).into()
    }

    /// Transform this image into a grayscale.
    pub fn grayscale<'js>(&mut self, this: This<Class<'js, Self>>) -> Class<'js, Self> {
        self.inner.grayscale_mut();

        this.0
    }

    /// Returns a grayscale version of this image.
    #[must_use]
    pub fn with_grayscale(&self) -> Self {
        self.inner.grayscaled().into()
    }

    /// Crops this image.
    pub fn crop<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        rect: JsRectLike,
    ) -> Class<'js, Self> {
        self.inner.crop_mut(rect.0);

        this.0
    }

    /// Returns a cropped version of this image.
    #[must_use]
    pub fn cropped(&self, rect: JsRectLike) -> Self {
        self.inner.cropped(rect.0).into()
    }

    /// Resizes this image.
    pub fn resize<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        width: u32,
        height: u32,
        options: Opt<JsResizeOptions>,
    ) -> Class<'js, Self> {
        self.inner
            .resize_mut(width, height, options.unwrap_or_default().into());

        this.0
    }

    /// Returns a resized version of this image.
    #[must_use]
    pub fn resized(&self, width: u32, height: u32, options: Opt<JsResizeOptions>) -> Self {
        self.inner
            .resized(width, height, options.unwrap_or_default().into())
            .into()
    }

    /// Brightens or darkens the pixels of this image.
    pub fn adjust_brightness<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        value: i32,
    ) -> Class<'js, Self> {
        self.inner.adjust_brightness_mut(value);

        this.0
    }

    /// Returns a brightened or darkened version of this image.
    #[must_use]
    pub fn with_adjusted_brightness(&self, value: i32) -> Self {
        self.inner.adjusted_brightness(value).into()
    }

    /// Adjusts the contrast of this image.
    pub fn adjust_contrast<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        value: f32,
    ) -> Class<'js, Self> {
        self.inner.adjust_contrast_mut(value);

        this.0
    }

    /// Returns a new image with an adjusted contrast.
    #[must_use]
    pub fn with_adjusted_contrast(&self, value: f32) -> Self {
        self.inner.adjusted_contrast(value).into()
    }

    /// Fill this image with a color.
    pub fn fill<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        color: JsColorLike,
    ) -> Class<'js, Self> {
        self.inner.fill_mut(color.0);

        this.0
    }

    /// Fill this image with a color.
    #[must_use]
    pub fn filled(&self, color: JsColorLike) -> Self {
        self.inner.filled(color.0).into()
    }

    /// Returns the value of a pixel.
    pub fn get_pixel(&mut self, ctx: Ctx<'_>, position: JsPointLike) -> Result<JsColor> {
        self.inner
            .get_pixel_color(position.0)
            .into_js_result(&ctx)
            .map(Into::into)
    }

    /// Returns the value of a pixel.
    pub fn set_pixel<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        ctx: Ctx<'js>,
        position: JsPointLike,
        color: JsColorLike,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .set_pixel_color(position.0, color.0)
            .into_js_result(&ctx)?;

        Ok(this.0)
    }

    /// Creates a new image from a part of this image.
    pub fn copy_region(&self, ctx: Ctx<'_>, rect: JsRectLike) -> Result<Self> {
        self.inner
            .copy_region(rect.0)
            .into_js_result(&ctx)
            .map(Into::into)
    }

    /// Returns a Rect representing this image.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn rect(&self) -> JsRect {
        self.inner.bounds_rect().into()
    }

    /// Draw a cross on this image.
    pub fn draw_cross<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        position: JsPointLike,
        color: JsColorLike,
    ) -> Class<'js, Self> {
        self.inner.draw_cross_mut(position.0, color.0);

        this.0
    }

    /// Draw a cross on a copy of this image.
    #[must_use]
    pub fn with_cross(&self, position: JsPointLike, color: JsColorLike) -> Self {
        self.inner.with_cross(position.0, color.0).into()
    }

    /// Draw a line on this image.
    pub fn draw_line<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        start: JsPointLike,
        end: JsPointLike,
        color: JsColorLike,
    ) -> Class<'js, Self> {
        self.inner.draw_line_mut(start.0, end.0, color.0);

        this.0
    }

    /// Draw a line on a copy of this image.
    #[must_use]
    pub fn with_line(&self, start: JsPointLike, end: JsPointLike, color: JsColorLike) -> Self {
        self.inner.with_line(start.0, end.0, color.0).into()
    }

    /// Draw a circle on this image.
    pub fn draw_circle<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        center: JsPointLike,
        radius: i32,
        color: JsColorLike,
        options: Opt<JsDrawingOptions>,
    ) -> Class<'js, Self> {
        self.inner.draw_circle_mut(
            center.0,
            radius,
            color.0,
            options.unwrap_or_default().into(),
        );

        this.0
    }

    /// Draw a circle on a copy of this image.
    #[must_use]
    pub fn with_circle(
        &self,
        center: JsPointLike,
        radius: i32,
        color: JsColorLike,
        options: Opt<JsDrawingOptions>,
    ) -> Self {
        self.inner
            .with_circle(
                center.0,
                radius,
                color.0,
                options.unwrap_or_default().into(),
            )
            .into()
    }

    /// Draw an ellipse on this image.
    pub fn draw_ellipse<'js>(
        &mut self,
        this: This<Class<'js, Self>>,
        center: JsPointLike,
        width_radius: i32,
        height_radius: i32,
        color: JsColorLike,
        options: Opt<JsDrawingOptions>,
    ) -> Class<'js, Self> {
        self.inner.draw_ellipse_mut(
            center.0,
            width_radius,
            height_radius,
            color.0,
            options.unwrap_or_default().into(),
        );

        this.0
    }

    /// Draw an ellipse on a copy of this image.
    #[must_use]
    pub fn with_ellipse(
        &self,
        center: JsPointLike,
        width_radius: i32,
        height_radius: i32,
        color: JsColorLike,
        options: Opt<JsDrawingOptions>,
    ) -> Self {
        self.inner
            .with_ellipse(
                center.0,
                width_radius,
                height_radius,
                color.0,
                options.unwrap_or_default().into(),
            )
            .into()
    }

    /// Draw a rectangle on this image.
    pub fn draw_rectangle<'js>(
        &mut self,
        ctx: Ctx<'js>,
        this: This<Class<'js, Self>>,
        rect: JsRectLike,
        color: JsColorLike,
        options: Opt<JsDrawingOptions>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .draw_rectangle_mut(rect.0, color.0, options.unwrap_or_default().into())
            .into_js_result(&ctx)?;

        Ok(this.0)
    }

    /// Draw a rectangle on a copy of this image.
    pub fn with_rectangle<'js>(
        &self,
        ctx: Ctx<'js>,
        rect: JsRectLike,
        color: JsColorLike,
        options: Opt<JsDrawingOptions>,
    ) -> Result<Self> {
        self.inner
            .with_rectangle(rect.0, color.0, options.unwrap_or_default().into())
            .into_js_result(&ctx)
            .map(Into::into)
    }

    /// Draw text on this image using the provided font.
    pub fn draw_text<'js>(
        &mut self,
        ctx: Ctx<'js>,
        this: This<Class<'js, Self>>,
        position: JsPointLike,
        text: String,
        font_path: String,
        color: JsColorLike,
        options: Opt<JsDrawTextOptions>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .draw_text_mut(
                position.0,
                &text,
                &font_path,
                color.0,
                options.unwrap_or_default().into(),
            )
            .into_js_result(&ctx)?;

        Ok(this.0)
    }

    /// Draw text on a copy of this image.
    pub fn with_text(
        &self,
        ctx: Ctx<'_>,
        position: JsPointLike,
        text: String,
        font_path: String,
        color: JsColorLike,
        options: Opt<JsDrawTextOptions>,
    ) -> Result<Self> {
        self.inner
            .with_text(
                position.0,
                &text,
                &font_path,
                color.0,
                options.unwrap_or_default().into(),
            )
            .into_js_result(&ctx)
            .map(Into::into)
    }

    /// Draw another image on this image.
    pub fn draw_image<'js>(
        &mut self,
        ctx: Ctx<'js>,
        this: This<Class<'js, Self>>,
        position: JsPointLike,
        image: &JsImage,
        options: Opt<JsDrawImageOptions>,
    ) -> Result<Class<'js, Self>> {
        self.inner
            .draw_image_mut(position.0, &image.inner, options.unwrap_or_default().into())
            .into_js_result(&ctx)?;

        Ok(this.0)
    }

    /// Draw another image on a copy of this image.
    pub fn with_image(
        &self,
        ctx: Ctx<'_>,
        position: JsPointLike,
        image: &JsImage,
        options: Opt<JsDrawImageOptions>,
    ) -> Result<Self> {
        self.inner
            .with_image(position.0, &image.inner, options.unwrap_or_default().into())
            .into_js_result(&ctx)
            .map(Into::into)
    }

    /// Find an image inside this image.
    pub fn find_image(
        &self,
        ctx: Ctx<'_>,
        image: &JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Option<JsMatch>> {
        let options = options.0.unwrap_or_default();
        let source = Arc::<Source>::try_from(&self.inner).into_js_result(&ctx)?;
        let template = Arc::<Template>::try_from(&image.inner).into_js_result(&ctx)?;

        let result = source
            .find_template_one(&template, options)
            .into_js_result(&ctx)?
            .map(JsMatch::from);

        Ok(result)
    }

    /// Find any occurence of an image inside this image.
    pub fn find_image_all(
        &self,
        ctx: Ctx<'_>,
        image: &JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Vec<JsMatch>> {
        let options = options.0.unwrap_or_default();
        let source = Arc::<Source>::try_from(&self.inner).into_js_result(&ctx)?;
        let template = Arc::<Template>::try_from(&image.inner).into_js_result(&ctx)?;

        let result = source
            .find_template(&template, options)
            .into_js_result(&ctx)?
            .into_iter()
            .map(JsMatch::from)
            .collect_vec();

        Ok(result)
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

impl<T> IntoJsResult<T> for io::Result<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<T> IntoJsResult<T> for ImageResult<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<'js> Trace<'js> for JsImage {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serial_test::serial;

    use crate::{core::image::js::JsImage, runtime::Runtime};

    fn sanitize(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() || c.is_ascii_digit() {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_end_matches("_")
            .to_string()
    }

    #[rstest]
    #[serial]
    #[case::invert_colors("invertColors()")]
    #[case::blur_default("blur()")]
    #[case::blur_sigma(r#"blur({sigma: 4})"#)]
    #[case::flip_horizontal(r#"flip(FlipDirection.Horizontal)"#)]
    #[case::flip_vertical(r#"flip(FlipDirection.Vertical)"#)]
    #[case::hue_rotate("hueRotate(90)")]
    #[case::grayscale("grayscale()")]
    #[case::crop_rect(r#"crop(new Rect(5, 5, 50, 40))"#)]
    #[case::resize_default(r#"resize(30, 40)"#)]
    #[case::resize_keep_aspect(r#"resize(30, 40, {keepAspectRatio: true})"#)]
    #[case::adjust_brightness("adjustBrightness(20)")]
    #[case::adjust_contrast("adjustContrast(-15)")]
    #[case::fill_color("fill(Color.Green)")]
    #[case::set_pixel_point(r#"setPixel(new Point(0, 0), Color.Blue)"#)]
    #[case::set_pixel_object(r#"setPixel({x: 1, y: 2}, Color.Red)"#)]
    #[case::set_pixel_numbers(r#"setPixel(3, 4, Color.Green)"#)]
    #[case::draw_cross(r#"drawCross(new Point(10, 10), Color.Red)"#)]
    #[case::draw_line(r#"drawLine(new Point(0, 0), new Point(20, 10), Color.Blue)"#)]
    #[case::draw_circle_hollow(r#"drawCircle(new Point(25, 25), 10, Color.Green, {hollow: true})"#)]
    #[case::draw_ellipse(r#"drawEllipse(new Point(25, 20), 12, 8, Color.Black)"#)]
    #[case::draw_rectangle_hollow(
        r#"drawRectangle(new Rect(5, 5, 30, 15), Color.Black, {hollow: true})"#
    )]
    #[case::draw_text(
        r#"drawText(new Point(5, 25), "Test", fontPath, Color.White, {fontSize: 12, horizontalAlign: TextHorizontalAlign.Left})"#
    )]
    #[case::draw_image(
        r#"drawImage(new Point(10, 10), overlay, {sourceRect: new Rect(0, 0, 16, 16)})"#
    )]
    #[case::rotate_0("rotate(0)")]
    #[case::rotate_90("rotate(90)")]
    #[case::rotate_45("rotate(45)")]
    #[case::rotate_0_center(r#"rotate(0, {center: new Point(0, 0)})"#)]
    #[case::rotate_90_center(r#"rotate(90, {center: new Point(0, 0)})"#)]
    #[case::rotate_45_center(r#"rotate(45, {center: new Point(0, 0)})"#)]
    #[case::rotate_0_center_default_color(
        r#"rotate(0, {center: new Point(0, 0), defaultColor: Color.Red})"#
    )]
    #[case::rotate_90_center_default_color(
        r#"rotate(90, {center: new Point(0, 0), defaultColor: Color.Red})"#
    )]
    #[case::rotate_45_center_default_color(
        r#"rotate(45, {center: new Point(0, 0), defaultColor: Color.Red})"#
    )]
    #[case::rotate_0_center_default_color_nearest(
        r#"rotate(0, {center: new Point(0, 0), defaultColor: Color.Red, interpolation: Interpolation.Nearest})"#
    )]
    #[case::rotate_90_center_default_color_nearest(
        r#"rotate(90, {center: new Point(0, 0), defaultColor: Color.Red, interpolation: Interpolation.Nearest})"#
    )]
    #[case::rotate_45_center_default_color_nearest(
        r#"rotate(45, {center: new Point(0, 0), defaultColor: Color.Red, interpolation: Interpolation.Nearest})"#
    )]
    fn test_generate(#[case] operation: String) {
        Runtime::test_with_script_engine(async move |script_engine| {
            // Load the input image
            script_engine
                .eval_async::<()>(
                    r#"var input = await Image.load("../tests/pear.png");
                    var overlay = await Image.load("../tests/fire.png");
                    var fontPath = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";"#,
                )
                .await
                .unwrap();

            // Run the operation
            script_engine
                .eval::<()>(&format!("input.{operation}"))
                .await
                .unwrap();

            // Save the result
            let file_name = sanitize(&operation);
            script_engine
                .eval_async::<()>(&format!(r#"await input.save("test-data/{file_name}.png")"#))
                .await
                .unwrap();
        })
    }

    #[rstest]
    #[case("rotate(90)", "rotate.png")]
    fn test_all(#[case] operation: String, #[case] output_filename: String) {
        Runtime::test_with_script_engine(async move |script_engine| {
            script_engine
                .eval_async::<()>(r#"var input = await Image.load("../tests/pear.png")"#)
                .await
                .unwrap();

            let output = script_engine
                .eval::<JsImage>(&format!("input.{operation}"))
                .await
                .unwrap();

            script_engine
                .with2(|ctx| output.save(ctx.clone(), format!("../tests/{output_filename}")))
                .await
                .unwrap();
        })
    }
}
