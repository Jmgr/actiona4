use std::{io, sync::Arc};

use image::ImageResult;
use itertools::Itertools;
use macros::{FromJsObject, FromSerde, IntoSerde};
use rquickjs::{
    Class, Ctx, Exception, JsLifetime, Promise, Result, TypedArray,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::{Opt, This},
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tokio::sync::watch;

use crate::{
    IntoJsResult,
    api::{
        color::js::{JsColor, JsColorLike},
        image::{
            BlurOptions, DrawImageOptions, DrawTextOptions, DrawingOptions, FlipDirection,
            Interpolation, ResizeFilter, ResizeOptions, RotationOptions, TextHorizontalAlign,
            TextVerticalAlign,
            find_image::{FindImageProgress, Source, Template},
        },
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, ValueClass, register_enum, register_host_class},
            task::{IsDone, progress_task_with_token},
        },
        point::js::{JsPoint, JsPointLike},
        rect::js::{JsRect, JsRectLike},
        size::js::JsSize,
    },
    error::CommonError,
    runtime::WithUserData,
    types::display::display_with_type,
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
/// Direction to flip an image.
///
/// ```ts
/// // Flip horizontally (mirror)
/// image.flip(FlipDirection.Horizontal);
///
/// // Flip vertically
/// image.flip(FlipDirection.Vertical);
/// ```
///
/// @expand
#[serde(rename = "FlipDirection")]
pub enum JsFlipDirection {
    /// `FlipDirection.Horizontal`
    Horizontal,
    /// `FlipDirection.Vertical`
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

/// Resize filter algorithms.
///
/// ```ts
/// // Use nearest-neighbor for pixel art (no smoothing)
/// image.resize(64, 64, { filter: ResizeFilter.Nearest });
///
/// // Use Lanczos3 for high-quality downscaling
/// image.resize(200, 150, { filter: ResizeFilter.Lanczos3 });
/// ```
///
/// @expand
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
    /// `ResizeFilter.Nearest`
    Nearest,
    /// `ResizeFilter.Linear`
    Linear,
    /// `ResizeFilter.Cubic`
    Cubic,
    /// `ResizeFilter.Gaussian`
    Gaussian,
    /// `ResizeFilter.Lanczos3`
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

/// Interpolation algorithms used for image rotations.
///
/// ```ts
/// // Fast but lower quality
/// image.rotate(45, { interpolation: Interpolation.Nearest });
///
/// // Smooth result (default)
/// image.rotate(45, { interpolation: Interpolation.Bilinear });
/// ```
///
/// @expand
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
    /// `Interpolation.Nearest`
    Nearest,
    /// `Interpolation.Bilinear`
    Bilinear,
    /// `Interpolation.Bicubic`
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

/// Options for resizing an image.
///
/// ```ts
/// // Resize while preserving aspect ratio
/// image.resize(200, 150, { keepAspectRatio: true });
///
/// // Resize with a specific filter
/// image.resize(200, 150, { filter: ResizeFilter.Lanczos3, keepAspectRatio: true });
/// ```
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

/// Options for blurring an image.
///
/// ```ts
/// // Fast blur
/// image.blur({ fast: true });
///
/// // Gaussian blur with custom sigma
/// image.blur({ sigma: 5.0 });
/// ```
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

/// Options for drawing an image onto another image.
///
/// ```ts
/// // Draw only a portion of the source image
/// canvas.drawImage(0, 0, sprite, {
///   sourceRect: new Rect(0, 0, 32, 32)
/// });
/// ```
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

/// Options for rotating an image.
///
/// ```ts
/// // Rotate around a custom center point
/// image.rotate(45, { center: new Point(10, 10) });
///
/// // You can also use a plain object for the center
/// image.rotate(45, { center: {x: 10, y: 10} });
///
/// // Rotate with a background color for exposed areas
/// image.rotate(30, { defaultColor: Color.White });
/// ```
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

/// Options for drawing shapes on an image.
///
/// ```ts
/// // Draw a hollow circle (outline only)
/// image.drawCircle(50, 50, 20, Color.Red, { hollow: true });
/// ```
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
///
/// ```ts
/// image.drawText(100, 50, "Centered", fontPath, Color.Black, {
///   horizontalAlign: TextHorizontalAlign.Center
/// });
/// ```
///
/// @expand
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
    /// `TextHorizontalAlign.Left`
    Left,
    /// `TextHorizontalAlign.Center`
    Center,
    /// `TextHorizontalAlign.Right`
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
///
/// ```ts
/// image.drawText(50, 100, "Middle", fontPath, Color.Black, {
///   verticalAlign: TextVerticalAlign.Middle
/// });
/// ```
///
/// @expand
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
    /// `TextVerticalAlign.Top`
    Top,
    /// `TextVerticalAlign.Middle`
    Middle,
    /// `TextVerticalAlign.Bottom`
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

/// Options for drawing text on an image.
///
/// ```ts
/// // Draw large, centered text
/// image.drawText(100, 50, "Hello", fontPath, Color.White, {
///   fontSize: 32,
///   horizontalAlign: TextHorizontalAlign.Center,
///   verticalAlign: TextVerticalAlign.Middle
/// });
/// ```
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

/// Options for finding an image within another image.
///
/// ```ts
/// // Find with stricter matching
/// const match = await source.findImage(template, { matchThreshold: 0.95 });
///
/// // Find with abort support
/// const controller = new AbortController();
/// const match = await source.findImage(template, { signal: controller.signal });
/// ```
/// @options
#[derive(Clone, Debug, FromJsObject)]
pub struct JsFindImageOptions {
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

    /// Abort signal to cancel the search.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl Default for JsFindImageOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            use_transparency: true,
            match_threshold: 0.8,
            non_maximum_suppression_radius: Some(10),
            downscale: 0,
            signal: None,
        }
    }
}

impl JsFindImageOptions {
    pub(crate) fn into_inner(self) -> super::find_image::FindImageTemplateOptions {
        super::find_image::FindImageTemplateOptions {
            use_colors: self.use_colors,
            use_transparency: self.use_transparency,
            match_threshold: self.match_threshold,
            non_maximum_suppression_radius: self.non_maximum_suppression_radius,
            downscale: self.downscale,
        }
    }
}

/// A match returned by a findImage or findImageAll call.
///
/// ```ts
/// const source = await Image.load("screenshot.png");
/// const template = await Image.load("button.png");
/// const match = await source.findImage(template);
/// if (match) {
///   println(`Found at ${match.position} with score ${match.score}`);
///   println(`Bounding rect: ${match.rect}`);
/// }
/// ```
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
    pub const fn score(&self) -> f64 {
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
        display_with_type("Match", self.inner)
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

/// Stages of a find image operation.
///
/// ```ts
/// const task = source.findImage(template);
/// for await (const progress of task) {
///   if (progress.stage === FindImageStage.Matching) {
///     println(`Matching: ${formatPercent(progress.percent)}`);
///   }
/// }
/// ```
///
/// @expand
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
#[serde(rename = "FindImageStage")]
pub enum JsFindImageStage {
    /// `FindImageStage.Capturing`
    Capturing,
    /// `FindImageStage.Preparing`
    Preparing,
    /// `FindImageStage.Downscaling`
    Downscaling,
    /// `FindImageStage.Matching`
    Matching,
    /// `FindImageStage.Filtering`
    Filtering,
    /// `FindImageStage.ComputingResults`
    ComputingResults,
    /// `FindImageStage.Finished`
    Finished,
}

impl From<super::find_image::FindImageStage> for JsFindImageStage {
    fn from(value: super::find_image::FindImageStage) -> Self {
        use super::find_image::FindImageStage;
        match value {
            FindImageStage::Capturing => Self::Capturing,
            FindImageStage::Preparing => Self::Preparing,
            FindImageStage::Downscaling => Self::Downscaling,
            FindImageStage::Matching => Self::Matching,
            FindImageStage::Filtering => Self::Filtering,
            FindImageStage::ComputingResults => Self::ComputingResults,
            FindImageStage::Finished => Self::Finished,
        }
    }
}

/// Progress of a find image operation.
///
/// Received by iterating over the async iterator returned by `findImage` or `findImageAll`.
///
/// ```ts
/// const task = source.findImage(template);
/// for await (const progress of task) {
///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
///   if (progress.finished) break;
/// }
/// const result = await task;
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, JsLifetime, PartialEq)]
#[rquickjs::class(rename = "FindImageProgress")]
pub struct JsFindImageProgress {
    inner: super::find_image::FindImageProgress,
}

impl<'js> Trace<'js> for JsFindImageProgress {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl HostClass<'_> for JsFindImageProgress {}

impl IsDone for JsFindImageProgress {
    fn is_done(&self) -> bool {
        self.inner.stage.is_finished()
    }
}

impl From<super::find_image::FindImageProgress> for JsFindImageProgress {
    fn from(value: super::find_image::FindImageProgress) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsFindImageProgress {
    /// The current stage of the find image operation.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn stage(&self) -> JsFindImageStage {
        self.inner.stage.into()
    }

    /// Completion percentage (0-100).
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn percent(&self) -> u8 {
        self.inner.percent
    }

    /// Whether the operation has finished.
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn finished(&self) -> bool {
        self.inner.stage.is_finished()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("FindImageProgress", self.inner)
    }
}

/// An image that can be loaded, created, manipulated, and saved.
///
/// Provides methods for image processing (blur, rotate, resize, color adjustments),
/// drawing primitives (lines, circles, rectangles, text), and template matching (findImage).
///
/// Most mutating methods return `this` for chaining. Each also has an immutable variant
/// that returns a new `Image` (e.g., `blur()` vs `blurred()`).
///
/// ```ts
/// // Create, manipulate, and save
/// let image = new Image(200, 100);
/// image.fill(Color.White)
///      .drawCircle(100, 50, 30, Color.Red)
///      .drawText(10, 10, "Hello", "/path/to/font.ttf", Color.Black);
/// await image.save("output.png");
/// ```
///
/// ```ts
/// // Load, transform, and save
/// let photo = await Image.load("photo.png");
/// photo.resize(800, 600, { keepAspectRatio: true })
///      .adjustBrightness(10)
///      .adjustContrast(5);
/// await photo.save("photo_edited.png");
/// ```
///
/// ```ts
/// // Find an image within another
/// const screen = await Image.load("screenshot.png");
/// const button = await Image.load("button.png");
/// const match = await screen.findImage(button, { matchThreshold: 0.9 });
/// if (match) {
///   println(`Button found at ${match.position}`);
/// }
/// ```
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
        register_enum::<JsFindImageStage>(ctx)?;
        register_host_class::<JsFindImageProgress>(ctx)?;

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

    /// Creates a new image from raw encoded bytes (PNG, JPEG, etc.).
    ///
    /// ```ts
    /// const bytes = await file.readAll();
    /// const image = Image.fromBytes(bytes);
    /// ```
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

    /// Saves this image to a file. The format is inferred from the file extension.
    pub async fn save(&self, ctx: Ctx<'_>, path: String) -> Result<()> {
        self.inner.save(path).await.into_js_result(&ctx)
    }

    /// Loads an image from a file. The format is guessed from the file contents.
    #[qjs(static)]
    pub async fn load(ctx: Ctx<'_>, path: String) -> Result<Self> {
        let image = super::Image::load(path).await.into_js_result(&ctx)?;

        Ok(image.into())
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

    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn size(&self) -> JsSize {
        self.inner.size().into()
    }

    /// Returns true if this image equals another (same dimensions and pixel data).
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Returns a string representation of this image.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Image", &self.inner)
    }

    /// Clones this image.
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

    /// Returns a copy of this image filled with a color.
    #[must_use]
    pub fn filled(&self, color: JsColorLike) -> Self {
        self.inner.filled(color.0).into()
    }

    /// Returns the value of a pixel.
    pub fn get_pixel(&self, ctx: Ctx<'_>, position: JsPointLike) -> Result<JsColor> {
        self.inner
            .get_pixel_color(position.0)
            .into_js_result(&ctx)
            .map(Into::into)
    }

    /// Sets the color of a pixel.
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
    /// @readonly
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

    /// Finds the best match of an image inside this image.
    ///
    /// Returns a `ProgressTask` that can be awaited for the result and iterated
    /// for progress updates. Returns `undefined` if no match is found.
    ///
    /// ```ts
    /// const match = await source.findImage(template);
    /// if (match) {
    ///   println(`Found at ${match.position} with score ${match.score}`);
    /// }
    /// ```
    ///
    /// ```ts
    /// // Track progress while searching
    /// const task = source.findImage(template);
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const match = await task;
    /// ```
    /// @returns ProgressTask<Match | undefined, FindImageProgress>
    pub fn find_image<'js>(
        &self,
        ctx: Ctx<'js>,
        image: JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let source = Arc::<Source>::try_from(&self.inner).into_js_result(&ctx)?;
        let template = Arc::<Template>::try_from(&image.inner).into_js_result(&ctx)?;
        let (progress_sender, progress_receiver) = watch::channel(FindImageProgress::default());

        progress_task_with_token::<_, _, _, _, _, JsFindImageProgress>(
            ctx,
            signal,
            progress_receiver,
            async move |ctx, token| {
                let task_tracker = ctx.user_data().task_tracker();

                let result = task_tracker
                    .spawn_blocking(move || {
                        source.find_template(
                            &template,
                            options.into_inner(),
                            token,
                            progress_sender,
                        )
                    })
                    .await
                    .map_err(|e| Exception::throw_message(&ctx, &format!("Task join error: {e}")))?
                    .into_js_result(&ctx)?;

                Ok(result.map(JsMatch::from))
            },
        )
    }

    /// Finds all occurrences of an image inside this image.
    ///
    /// Returns a `ProgressTask` that can be awaited for an array of matches.
    ///
    /// ```ts
    /// const matches = await source.findImageAll(template, { matchThreshold: 0.85 });
    /// for (const match of matches) {
    ///   println(`Found at ${match.position}`);
    /// }
    /// ```
    ///
    /// ```ts
    /// // Track progress while searching
    /// const task = source.findImageAll(template);
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const matches = await task;
    /// ```
    /// @returns ProgressTask<Match[], FindImageProgress>
    pub fn find_image_all<'js>(
        &self,
        ctx: Ctx<'js>,
        image: JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let source = Arc::<Source>::try_from(&self.inner).into_js_result(&ctx)?;
        let template = Arc::<Template>::try_from(&image.inner).into_js_result(&ctx)?;
        let (progress_sender, progress_receiver) = watch::channel(FindImageProgress::default());

        progress_task_with_token::<_, _, _, _, _, JsFindImageProgress>(
            ctx,
            signal,
            progress_receiver,
            async move |ctx, token| {
                let task_tracker = ctx.user_data().task_tracker();

                let result = task_tracker
                    .spawn_blocking(move || {
                        source.find_template_all(
                            &template,
                            options.into_inner(),
                            token,
                            progress_sender,
                        )
                    })
                    .await
                    .map_err(|e| Exception::throw_message(&ctx, &format!("Task join error: {e}")))?
                    .into_js_result(&ctx)?
                    .into_iter()
                    .map(JsMatch::from)
                    .collect_vec();

                Ok(result)
            },
        )
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
    use std::path::{Path, PathBuf};

    use rstest::rstest;
    use serial_test::serial;

    use crate::runtime::Runtime;

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

    fn workspace_tests_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tests")
    }

    fn default_font_path() -> PathBuf {
        #[cfg(windows)]
        {
            PathBuf::from(r"C:\Windows\Fonts\arial.ttf")
        }

        #[cfg(not(windows))]
        {
            PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf")
        }
    }

    /// Set JS globals for image test paths to avoid backslash escaping issues.
    async fn set_image_test_globals(engine: &crate::scripting::Engine, globals: &[(&str, &Path)]) {
        engine
            .with(|ctx| {
                let g = ctx.globals();
                for &(name, path) in globals {
                    g.set(name, path.to_string_lossy().to_string())?;
                }
                Ok(())
            })
            .await
            .unwrap();
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
            let tests_dir = workspace_tests_dir();
            let input_path = tests_dir.join("pear.png");
            let overlay_path = tests_dir.join("fire.png");
            let font_path = default_font_path();
            let file_name = sanitize(&operation);
            let output_path = std::env::temp_dir().join(format!("actiona_ng_{file_name}.png"));

            set_image_test_globals(
                &script_engine,
                &[
                    ("inputPath", &input_path),
                    ("overlayPath", &overlay_path),
                    ("fontPath", &font_path),
                    ("outputPath", &output_path),
                ],
            )
            .await;

            // Load the input image
            script_engine
                .eval_async::<()>(
                    r#"var input = await Image.load(inputPath);
                    var overlay = await Image.load(overlayPath);"#,
                )
                .await
                .unwrap();

            // Run the operation
            script_engine
                .eval_async::<()>(&format!("input.{operation}"))
                .await
                .unwrap();

            // Save the result
            script_engine
                .eval_async::<()>(r#"await input.save(outputPath)"#)
                .await
                .unwrap();
        })
    }

    #[test]
    #[serial]
    fn test_find_image() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let tests_dir = workspace_tests_dir();
            let source_path = tests_dir.join("input.png");
            let template_path = tests_dir.join("pear.png");

            set_image_test_globals(
                &script_engine,
                &[
                    ("sourcePath", &source_path),
                    ("templatePath", &template_path),
                ],
            )
            .await;

            let result = script_engine
                .eval_async::<Vec<String>>(
                    r#"
                    const source = await Image.load(sourcePath);
                    const template = await Image.load(templatePath);

                    const task = source.findImage(template);

                    // Collect progress updates from the async iterator.
                    // watch channels coalesce values, so we may not see every
                    // intermediate update — but we will always see the latest.
                    const stages = [];
                    for await (const progress of task) {
                        stages.push(progress.stage);
                    }

                    const result = await task;

                    if (!result) {
                        throw new Error("Expected a match but got undefined");
                    }

                    // Verify that the progress iterator produced updates and
                    // ended with the "Finished" stage.
                    if (stages.length === 0) {
                        throw new Error("Expected at least one progress update");
                    }
                    if (stages[stages.length - 1] !== FindImageStage.Finished) {
                        throw new Error(`Expected last stage to be "Finished", got "${stages[stages.length - 1]}"`);
                    }

                    // Return the stages for assertion on the Rust side
                    stages
                    "#,
                )
                .await
                .unwrap();

            // We should have received at least one progress update
            assert!(!result.is_empty());
            assert_eq!(result.last().unwrap(), "Finished");

            // All stages should be valid
            let valid_stages = [
                "Preparing",
                "Downscaling",
                "Matching",
                "Filtering",
                "ComputingResults",
                "Finished",
            ];
            for stage in &result {
                assert!(
                    valid_stages.contains(&stage.as_str()),
                    "unexpected stage: {stage}"
                );
            }
        })
    }

    #[test]
    #[serial]
    fn test_find_image_all() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let tests_dir = workspace_tests_dir();
            let source_path = tests_dir.join("input.png");
            let template_path = tests_dir.join("pear.png");

            set_image_test_globals(
                &script_engine,
                &[
                    ("sourcePath", &source_path),
                    ("templatePath", &template_path),
                ],
            )
            .await;

            script_engine
                .eval_async::<()>(
                    r#"
                    const source = await Image.load(sourcePath);
                    const template = await Image.load(templatePath);

                    const task = source.findImageAll(template);

                    let lastStage = "";
                    let lastPercent = 0;
                    for await (const progress of task) {
                        lastStage = progress.stage;
                        lastPercent = progress.percent;
                    }

                    const results = await task;

                    if (results.length !== 2) {
                        throw new Error(`Expected 2 matches, got ${results.length}`);
                    }
                    if (lastStage !== FindImageStage.Finished) {
                        throw new Error(`Expected last stage "Finished", got "${lastStage}"`);
                    }
                    if (lastPercent !== 100) {
                        throw new Error(`Expected last percent 100, got ${lastPercent}`);
                    }
                    "#,
                )
                .await
                .unwrap();
        })
    }
}
