use std::{fmt::Display, io::Cursor, ops::DerefMut, path::Path, sync::Arc};

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use arc_swap::ArcSwapOption;
use color_eyre::{Result, eyre::eyre};
use derive_more::Deref;
use image::{
    DynamicImage, GrayImage, ImageFormat, ImageReader, RgbImage, RgbaImage,
    imageops::{self, FilterType},
};
use imageproc::{
    drawing::{
        draw_cross, draw_cross_mut, draw_filled_circle, draw_filled_circle_mut,
        draw_filled_ellipse, draw_filled_ellipse_mut, draw_filled_rect, draw_filled_rect_mut,
        draw_hollow_circle, draw_hollow_circle_mut, draw_hollow_ellipse, draw_hollow_ellipse_mut,
        draw_hollow_rect, draw_hollow_rect_mut, draw_line_segment, draw_line_segment_mut,
        draw_text_mut, text_size,
    },
    geometric_transformations::{self, rotate, rotate_about_center},
    rect::Rect as ImgRect,
};
use tokio::fs;

pub mod find_image;
pub mod js;

use crate::{
    api::{
        color::Color,
        image::find_image::{Source, Template},
        point::{Point, point},
        rect::{Rect, rect},
        size::size,
    },
    types::{display::DisplayFields, si32::Si32, su32::su32},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlipDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizeFilter {
    Nearest,
    Linear,
    Cubic,
    Gaussian,
    Lanczos3,
}

impl From<ResizeFilter> for FilterType {
    fn from(value: ResizeFilter) -> Self {
        match value {
            ResizeFilter::Nearest => Self::Nearest,
            ResizeFilter::Linear => Self::Triangle,
            ResizeFilter::Cubic => Self::CatmullRom,
            ResizeFilter::Gaussian => Self::Gaussian,
            ResizeFilter::Lanczos3 => Self::Lanczos3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interpolation {
    Nearest,
    Bilinear,
    Bicubic,
}

impl From<Interpolation> for geometric_transformations::Interpolation {
    fn from(value: Interpolation) -> Self {
        match value {
            Interpolation::Nearest => Self::Nearest,
            Interpolation::Bilinear => Self::Bilinear,
            Interpolation::Bicubic => Self::Bicubic,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResizeOptions {
    pub keep_aspect_ratio: bool,
    pub filter: ResizeFilter,
}

impl Default for ResizeOptions {
    fn default() -> Self {
        Self {
            keep_aspect_ratio: false,
            filter: ResizeFilter::Cubic,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BlurOptions {
    pub fast: bool,
    pub sigma: f32,
}

impl Default for BlurOptions {
    fn default() -> Self {
        Self {
            fast: false,
            sigma: 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawImageOptions {
    pub source_rect: Option<Rect>,
}

#[derive(Clone, Copy, Debug)]
pub struct RotationOptions {
    pub interpolation: Interpolation,
    pub center: Option<Point>,
    pub default_color: Color,
}

impl Default for RotationOptions {
    fn default() -> Self {
        Self {
            interpolation: Interpolation::Bilinear,
            center: None,
            default_color: Color::new(0, 0, 0, 255),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DrawingOptions {
    pub hollow: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextHorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextVerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Debug)]
pub struct DrawTextOptions {
    pub font_size: f32,
    pub line_spacing: f32,
    pub horizontal_align: TextHorizontalAlign,
    pub vertical_align: TextVerticalAlign,
}

impl Default for DrawTextOptions {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_spacing: 1.0,
            horizontal_align: TextHorizontalAlign::Left,
            vertical_align: TextVerticalAlign::Top,
        }
    }
}

#[derive(Debug)]
struct Cache<T>(ArcSwapOption<T>);

impl<T> Default for Cache<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[allow(dead_code)]
impl<T> Cache<T> {
    fn reset(&self) {
        self.0.store(None);
    }

    fn get(&self) -> Option<Arc<T>> {
        self.0.load_full()
    }

    fn set(&self, value: Arc<T>) {
        self.0.store(Some(value));
    }
}

/// An image that is always stored as RGBA8.
///
/// All constructors convert to RGBA8 eagerly, so pixel-level operations
/// never need a format check or on-the-fly conversion.
#[derive(Debug, Default, Deref)]
pub struct Image {
    inner: RgbaImage,

    #[deref(ignore)]
    source: Cache<Source>,

    #[deref(ignore)]
    template: Cache<Template>,
}

impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("width", self.inner.width())
            .display("height", self.inner.height())
            .finish(f)
    }
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.source.reset();
        self.template.reset();

        &mut self.inner
    }
}

impl Clone for Image {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            source: Default::default(),
            template: Default::default(),
        }
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl From<DynamicImage> for Image {
    fn from(value: DynamicImage) -> Self {
        Self::from_dynamic_image(value)
    }
}

impl From<RgbaImage> for Image {
    fn from(value: RgbaImage) -> Self {
        Self::from_rgba8(value)
    }
}

impl Image {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self::from_rgba8(RgbaImage::new(width, height))
    }

    #[must_use]
    pub fn from_rgba8(image: RgbaImage) -> Self {
        Self {
            inner: image,
            source: Default::default(),
            template: Default::default(),
        }
    }

    #[must_use]
    pub fn from_dynamic_image(image: DynamicImage) -> Self {
        Self::from_rgba8(image.into_rgba8())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;
        Ok(Self::from_dynamic_image(image))
    }

    #[must_use]
    pub fn into_rgba8(self) -> RgbaImage {
        self.inner
    }

    #[must_use]
    pub const fn as_rgba8(&self) -> &RgbaImage {
        &self.inner
    }

    /// Returns a mutable reference to the inner image, resetting caches.
    ///
    /// Use this for in-place mutations instead of accessing `self.inner` directly,
    /// so that OpenCV caches are always invalidated.
    fn inner_mut(&mut self) -> &mut RgbaImage {
        self.source.reset();
        self.template.reset();
        &mut self.inner
    }

    /// Replaces the inner image, resetting caches.
    ///
    /// Use this instead of `self.inner = ...` so that OpenCV caches are always invalidated.
    fn set_inner(&mut self, image: RgbaImage) {
        self.source.reset();
        self.template.reset();
        self.inner = image;
    }

    /// Load an image from a file path.
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = fs::read(path).await?;

        Self::from_bytes(&bytes)
    }

    /// Save the image to a file path.
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let format = ImageFormat::from_path(path)?;
        let mut writer = Cursor::new(Vec::new());

        DynamicImage::ImageRgba8(self.inner.clone()).write_to(&mut writer, format)?;

        fs::write(path, writer.into_inner()).await?;

        Ok(())
    }
}

impl Image {
    #[must_use]
    pub fn to_rgb8(&self) -> RgbImage {
        DynamicImage::ImageRgba8(self.inner.clone()).into_rgb8()
    }

    #[must_use]
    pub fn to_luma8(&self) -> GrayImage {
        DynamicImage::ImageRgba8(self.inner.clone()).into_luma8()
    }

    /// Helper: wrap inner as DynamicImage, apply a closure, convert back.
    fn via_dynamic<F>(&self, f: F) -> Self
    where
        F: FnOnce(DynamicImage) -> DynamicImage,
    {
        let dyn_img = DynamicImage::ImageRgba8(self.inner.clone());
        Self::from_dynamic_image(f(dyn_img))
    }

    /// Helper: wrap inner as DynamicImage, apply a mutating closure, convert back in place.
    fn via_dynamic_mut<F>(&mut self, f: F)
    where
        F: FnOnce(DynamicImage) -> DynamicImage,
    {
        let dyn_img = DynamicImage::ImageRgba8(std::mem::take(&mut self.inner));
        *self = Self::from_dynamic_image(f(dyn_img));
    }
}

impl Image {
    #[must_use]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    pub fn invert_mut(&mut self) {
        imageops::invert(self.inner_mut());
    }

    #[must_use]
    pub fn inverted(&self) -> Self {
        let mut cloned = self.clone();
        cloned.invert_mut();
        cloned
    }

    pub fn blur_mut(&mut self, options: BlurOptions) {
        self.via_dynamic_mut(|img| {
            if options.fast {
                img.fast_blur(options.sigma)
            } else {
                img.blur(options.sigma)
            }
        });
    }

    #[must_use]
    pub fn blurred(&self, options: BlurOptions) -> Self {
        self.via_dynamic(|img| {
            if options.fast {
                img.fast_blur(options.sigma)
            } else {
                img.blur(options.sigma)
            }
        })
    }

    pub fn rotate_mut(&mut self, angle: f32, options: RotationOptions) {
        if let Some(center) = options.center {
            self.set_inner(rotate(
                &self.inner,
                (center.x.into(), center.y.into()),
                angle.to_radians(),
                options.interpolation.into(),
                options.default_color.into(),
            ));
        } else {
            match angle.rem_euclid(360.0) {
                0.0 => {}
                90.0 => self.set_inner(imageops::rotate90(&self.inner)),
                180.0 => self.set_inner(imageops::rotate180(&self.inner)),
                270.0 => self.set_inner(imageops::rotate270(&self.inner)),
                _ => {
                    self.set_inner(rotate_about_center(
                        &self.inner,
                        angle.to_radians(),
                        options.interpolation.into(),
                        options.default_color.into(),
                    ));
                }
            }
        }
    }

    #[must_use]
    pub fn rotated(&self, angle: f32, options: RotationOptions) -> Self {
        match options.center {
            Some(_) => {
                let mut clone = self.clone();
                clone.rotate_mut(angle, options);
                clone
            }
            None => {
                let result = match angle.rem_euclid(360.0) {
                    0.0 => self.inner.clone(),
                    90.0 => imageops::rotate90(&self.inner),
                    180.0 => imageops::rotate180(&self.inner),
                    270.0 => imageops::rotate270(&self.inner),
                    _ => rotate_about_center(
                        &self.inner,
                        angle.to_radians(),
                        options.interpolation.into(),
                        options.default_color.into(),
                    ),
                };

                Self::from_rgba8(result)
            }
        }
    }

    pub fn flip_mut(&mut self, direction: FlipDirection) {
        let inner = self.inner_mut();
        match direction {
            FlipDirection::Horizontal => imageops::flip_horizontal_in_place(inner),
            FlipDirection::Vertical => imageops::flip_vertical_in_place(inner),
        }
    }

    #[must_use]
    pub fn flipped(&self, direction: FlipDirection) -> Self {
        let image = match direction {
            FlipDirection::Horizontal => imageops::flip_horizontal(&self.inner),
            FlipDirection::Vertical => imageops::flip_vertical(&self.inner),
        };

        Self::from_rgba8(image)
    }

    pub fn hue_rotate_mut(&mut self, value: i32) {
        self.via_dynamic_mut(|img| img.huerotate(value));
    }

    #[must_use]
    pub fn hue_rotated(&self, value: i32) -> Self {
        self.via_dynamic(|img| img.huerotate(value))
    }

    pub fn grayscale_mut(&mut self) {
        self.via_dynamic_mut(|img| img.grayscale());
    }

    #[must_use]
    pub fn grayscaled(&self) -> Self {
        self.via_dynamic(|img| img.grayscale())
    }

    pub fn crop_mut(&mut self, rect: Rect) {
        let (x, y, width, height) = rect.clamped();
        let cropped = imageops::crop_imm(&self.inner, x, y, width, height).to_image();
        self.set_inner(cropped);
    }

    #[must_use]
    pub fn cropped(&self, rect: Rect) -> Self {
        let (x, y, width, height) = rect.clamped();
        Self::from_rgba8(imageops::crop_imm(&self.inner, x, y, width, height).to_image())
    }

    pub fn resize_mut(&mut self, width: u32, height: u32, options: ResizeOptions) {
        self.via_dynamic_mut(|img| {
            if options.keep_aspect_ratio {
                img.resize(width, height, options.filter.into())
            } else {
                img.resize_exact(width, height, options.filter.into())
            }
        });
    }

    #[must_use]
    pub fn resized(&self, width: u32, height: u32, options: ResizeOptions) -> Self {
        self.via_dynamic(|img| {
            if options.keep_aspect_ratio {
                img.resize(width, height, options.filter.into())
            } else {
                img.resize_exact(width, height, options.filter.into())
            }
        })
    }

    pub fn adjust_brightness_mut(&mut self, value: i32) {
        self.via_dynamic_mut(|img| img.brighten(value));
    }

    #[must_use]
    pub fn adjusted_brightness(&self, value: i32) -> Self {
        self.via_dynamic(|img| img.brighten(value))
    }

    pub fn adjust_contrast_mut(&mut self, value: f32) {
        self.via_dynamic_mut(|img| img.adjust_contrast(value));
    }

    #[must_use]
    pub fn adjusted_contrast(&self, value: f32) -> Self {
        self.via_dynamic(|img| img.adjust_contrast(value))
    }

    pub fn fill_mut(&mut self, color: Color) {
        let fill_color: image::Rgba<u8> = color.into();
        for pixel in self.inner_mut().pixels_mut() {
            *pixel = fill_color;
        }
    }

    #[must_use]
    pub fn filled(&self, color: Color) -> Self {
        let mut rgba = self.inner.clone();
        let fill_color: image::Rgba<u8> = color.into();
        for pixel in rgba.pixels_mut() {
            *pixel = fill_color;
        }

        Self::from_rgba8(rgba)
    }

    pub fn get_pixel_color(&self, position: Point) -> Result<Color> {
        let (x, y) = self.check_position(position)?;
        Ok(Color::from(*self.inner.get_pixel(x, y)))
    }

    pub fn set_pixel_color(&mut self, position: Point, color: Color) -> Result<()> {
        let (x, y) = self.check_position(position)?;
        self.inner_mut().put_pixel(x, y, color.into());
        Ok(())
    }

    pub fn copy_region(&self, rect: Rect) -> Result<Self> {
        let (x, y, width, height) = self.check_rect(&rect)?;
        Ok(Self::from_rgba8(
            imageops::crop_imm(&self.inner, x, y, width, height).to_image(),
        ))
    }

    #[must_use]
    pub fn bounds_rect(&self) -> Rect {
        rect(point(0, 0), size(self.width(), self.height()))
    }

    pub fn draw_cross_mut(&mut self, position: Point, color: Color) {
        let rgba: image::Rgba<u8> = color.into();
        draw_cross_mut(self.inner_mut(), rgba, position.x.into(), position.y.into());
    }

    #[must_use]
    pub fn with_cross(&self, position: Point, color: Color) -> Self {
        let rgba: image::Rgba<u8> = color.into();
        Self::from_rgba8(draw_cross(
            &self.inner,
            rgba,
            position.x.into(),
            position.y.into(),
        ))
    }

    pub fn draw_line_mut(&mut self, start: Point, end: Point, color: Color) {
        let rgba: image::Rgba<u8> = color.into();
        draw_line_segment_mut(
            self.inner_mut(),
            (start.x.into(), start.y.into()),
            (end.x.into(), end.y.into()),
            rgba,
        );
    }

    #[must_use]
    pub fn with_line(&self, start: Point, end: Point, color: Color) -> Self {
        let rgba: image::Rgba<u8> = color.into();
        Self::from_rgba8(draw_line_segment(
            &self.inner,
            (start.x.into(), start.y.into()),
            (end.x.into(), end.y.into()),
            rgba,
        ))
    }

    pub fn draw_circle_mut(
        &mut self,
        center: Point,
        radius: i32,
        color: Color,
        options: DrawingOptions,
    ) {
        let rgba: image::Rgba<u8> = color.into();
        let inner = self.inner_mut();
        if options.hollow {
            draw_hollow_circle_mut(inner, (center.x.into(), center.y.into()), radius, rgba);
        } else {
            draw_filled_circle_mut(inner, (center.x.into(), center.y.into()), radius, rgba);
        }
    }

    #[must_use]
    pub fn with_circle(
        &self,
        center: Point,
        radius: i32,
        color: Color,
        options: DrawingOptions,
    ) -> Self {
        let rgba: image::Rgba<u8> = color.into();
        let image = if options.hollow {
            draw_hollow_circle(
                &self.inner,
                (center.x.into(), center.y.into()),
                radius,
                rgba,
            )
        } else {
            draw_filled_circle(
                &self.inner,
                (center.x.into(), center.y.into()),
                radius,
                rgba,
            )
        };

        Self::from_rgba8(image)
    }

    pub fn draw_ellipse_mut(
        &mut self,
        center: Point,
        width_radius: i32,
        height_radius: i32,
        color: Color,
        options: DrawingOptions,
    ) {
        let rgba: image::Rgba<u8> = color.into();
        let center_tuple = (center.x.into(), center.y.into());
        let inner = self.inner_mut();
        if options.hollow {
            draw_hollow_ellipse_mut(inner, center_tuple, width_radius, height_radius, rgba);
        } else {
            draw_filled_ellipse_mut(inner, center_tuple, width_radius, height_radius, rgba);
        }
    }

    #[must_use]
    pub fn with_ellipse(
        &self,
        center: Point,
        width_radius: i32,
        height_radius: i32,
        color: Color,
        options: DrawingOptions,
    ) -> Self {
        let rgba: image::Rgba<u8> = color.into();
        let image = if options.hollow {
            draw_hollow_ellipse(
                &self.inner,
                (center.x.into(), center.y.into()),
                width_radius,
                height_radius,
                rgba,
            )
        } else {
            draw_filled_ellipse(
                &self.inner,
                (center.x.into(), center.y.into()),
                width_radius,
                height_radius,
                rgba,
            )
        };

        Self::from_rgba8(image)
    }

    pub fn draw_rectangle_mut(
        &mut self,
        rect: Rect,
        color: Color,
        options: DrawingOptions,
    ) -> Result<()> {
        let img_rect: ImgRect = rect.try_into().map_err(|err| eyre!("{err}"))?;
        let rgba: image::Rgba<u8> = color.into();
        let inner = self.inner_mut();

        if options.hollow {
            draw_hollow_rect_mut(inner, img_rect, rgba);
        } else {
            draw_filled_rect_mut(inner, img_rect, rgba);
        }

        Ok(())
    }

    pub fn with_rectangle(
        &self,
        rect: Rect,
        color: Color,
        options: DrawingOptions,
    ) -> Result<Self> {
        let img_rect: ImgRect = rect.try_into().map_err(|err| eyre!("{err}"))?;
        let rgba: image::Rgba<u8> = color.into();
        let image = if options.hollow {
            draw_hollow_rect(&self.inner, img_rect, rgba)
        } else {
            draw_filled_rect(&self.inner, img_rect, rgba)
        };

        Ok(Self::from_rgba8(image))
    }

    pub fn draw_image_mut(
        &mut self,
        position: Point,
        image: &Self,
        options: DrawImageOptions,
    ) -> Result<()> {
        Self::draw_image_into(self, position, image, options)
    }

    pub fn with_image(
        &self,
        position: Point,
        image: &Self,
        options: DrawImageOptions,
    ) -> Result<Self> {
        let mut clone = self.clone();
        Self::draw_image_into(&mut clone, position, image, options)?;
        Ok(clone)
    }

    pub fn draw_text_mut(
        &mut self,
        position: Point,
        text: &str,
        font_path: &str,
        color: Color,
        options: DrawTextOptions,
    ) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        let font = Self::load_font(font_path)?;
        Self::render_text(self, position, text, &font, color, options)
    }

    pub fn with_text(
        &self,
        position: Point,
        text: &str,
        font_path: &str,
        color: Color,
        options: DrawTextOptions,
    ) -> Result<Self> {
        let mut clone = self.clone();
        clone.draw_text_mut(position, text, font_path, color, options)?;
        Ok(clone)
    }

    fn check_position(&self, position: Point) -> Result<(u32, u32)> {
        let x = u32::try_from(position.x.into_inner())
            .map_err(|_| eyre!("Invalid position: {position}"))?;
        let y = u32::try_from(position.y.into_inner())
            .map_err(|_| eyre!("Invalid position: {position}"))?;

        if x >= self.width() || y >= self.height() {
            Err(eyre!("Invalid position: {position}"))
        } else {
            Ok((x, y))
        }
    }

    fn check_rect(&self, rect: &Rect) -> Result<(u32, u32, u32, u32)> {
        let (x, y) = self.check_position(rect.top_left)?;
        let rect_width: u32 = rect.size.width.into();
        let rect_height: u32 = rect.size.height.into();

        Self::ensure_region_fits(
            self.width(),
            self.height(),
            x,
            y,
            rect_width,
            rect_height,
            "rectangle",
        )?;

        Ok((x, y, rect_width, rect_height))
    }

    fn ensure_region_fits(
        width: u32,
        height: u32,
        start_x: u32,
        start_y: u32,
        region_width: u32,
        region_height: u32,
        label: &str,
    ) -> Result<()> {
        if region_width == 0 || region_height == 0 {
            return Err(eyre!("{label} must have a non-zero size"));
        }

        let end_x = start_x
            .checked_add(region_width)
            .ok_or_else(|| eyre!("{label} width is too large"))?;
        let end_y = start_y
            .checked_add(region_height)
            .ok_or_else(|| eyre!("{label} height is too large"))?;

        if end_x > width || end_y > height {
            return Err(eyre!("{label} extends beyond image bounds"));
        }

        Ok(())
    }

    fn draw_image_into(
        dest: &mut Self,
        position: Point,
        source: &Self,
        options: DrawImageOptions,
    ) -> Result<()> {
        let (dest_x, dest_y) = dest.check_position(position)?;

        let (src_x, src_y, width, height) = if let Some(rect) = options.source_rect {
            source.check_rect(&rect)?
        } else {
            (0, 0, source.width(), source.height())
        };

        Self::ensure_region_fits(
            dest.width(),
            dest.height(),
            dest_x,
            dest_y,
            width,
            height,
            "destination region",
        )?;

        let cropped = imageops::crop_imm(&source.inner, src_x, src_y, width, height).to_image();

        imageops::overlay(
            dest.inner_mut(),
            &cropped,
            i64::from(dest_x),
            i64::from(dest_y),
        );

        Ok(())
    }

    // TODO: expose a JsFont
    fn load_font(font_path: &str) -> Result<FontArc> {
        let data = std::fs::read(font_path)
            .map_err(|err| eyre!("Unable to read font \"{font_path}\": {err}"))?;

        FontArc::try_from_vec(data)
            .map_err(|err| eyre!("Unable to parse font \"{font_path}\": {err}"))
    }

    fn render_text(
        image: &mut Self,
        position: Point,
        text: &str,
        font: &FontArc,
        color: Color,
        options: DrawTextOptions,
    ) -> Result<()> {
        let scale = PxScale::from(options.font_size.max(1.0));
        let scaled_font = font.as_scaled(scale);

        let line_height = scaled_font.height().max(1.0);
        let spacing_factor = options.line_spacing.max(1.0);
        let line_step = (line_height * spacing_factor).max(1.0);
        let line_step_i32 = Self::clamp_f32_to_i32(line_step.round());

        let lines: Vec<&str> = text.split('\n').collect();
        let line_widths: Vec<u32> = lines
            .iter()
            .map(|line: &&str| text_size(scale, font, line).0)
            .collect();

        let extra_lines = u32::try_from(lines.len().saturating_sub(1)).unwrap_or(u32::MAX);
        let total_height = line_height + Self::u32_to_f32(extra_lines) * line_step;
        let total_height_i32 = Self::clamp_f32_to_i32(total_height.ceil());

        let vertical_offset = match options.vertical_align {
            TextVerticalAlign::Top => 0,
            TextVerticalAlign::Middle => -(total_height_i32 / 2),
            TextVerticalAlign::Bottom => -total_height_i32,
        };
        let mut current_y = position.y + vertical_offset;

        let color_pixel: image::Rgba<u8> = color.into();
        let canvas = image.inner_mut();

        for (line, &width) in lines.iter().zip(&line_widths) {
            let width_i32: i32 = Si32::from(su32(width)).into();
            let horizontal_offset = match options.horizontal_align {
                TextHorizontalAlign::Left => 0,
                TextHorizontalAlign::Center => -(width_i32 / 2),
                TextHorizontalAlign::Right => -width_i32,
            };
            let line_x = position.x + horizontal_offset;

            if !line.is_empty() {
                draw_text_mut(
                    canvas,
                    color_pixel,
                    line_x.into(),
                    current_y.into(),
                    scale,
                    font,
                    line,
                );
            }

            current_y += line_step_i32;
        }

        Ok(())
    }

    const fn clamp_f32_to_i32(value: f32) -> i32 {
        #[allow(clippy::as_conversions)]
        {
            if !value.is_finite() {
                0
            } else {
                value.clamp(Self::i32_to_f32(i32::MIN), Self::i32_to_f32(i32::MAX)) as i32
            }
        }
    }

    const fn i32_to_f32(value: i32) -> f32 {
        #[allow(clippy::as_conversions)]
        {
            value as f32
        }
    }

    const fn u32_to_f32(value: u32) -> f32 {
        #[allow(clippy::as_conversions)]
        {
            value as f32
        }
    }
}

#[cfg(test)]
impl Image {
    /// Returns `true` if the source (find-image) cache is populated.
    pub fn has_cached_source(&self) -> bool {
        self.source.get().is_some()
    }

    /// Returns `true` if the template (find-image) cache is populated.
    pub fn has_cached_template(&self) -> bool {
        self.template.get().is_some()
    }

    /// Populates both source and template caches so tests can verify they get reset.
    pub fn populate_caches(&self) {
        let _ = Arc::<find_image::Source>::try_from(self).unwrap();
        let _ = Arc::<find_image::Template>::try_from(self).unwrap();
        assert!(self.has_cached_source(), "source cache should be populated");
        assert!(
            self.has_cached_template(),
            "template cache should be populated"
        );
    }
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;

    #[test]
    fn new_image_is_rgba8_with_zero_pixels() {
        let img = Image::new(10, 20);
        assert_eq!(img.width(), 10);
        assert_eq!(img.height(), 20);
        // Every pixel should be transparent black
        for pixel in img.as_rgba8().pixels() {
            assert_eq!(*pixel, Rgba([0, 0, 0, 0]));
        }
    }

    #[test]
    fn from_dynamic_image_converts_rgb_to_rgba() {
        let rgb = image::RgbImage::from_pixel(4, 4, image::Rgb([100, 150, 200]));
        let dyn_img = DynamicImage::ImageRgb8(rgb);
        let img = Image::from_dynamic_image(dyn_img);
        assert_eq!(img.width(), 4);
        assert_eq!(img.height(), 4);
        let pixel = img.as_rgba8().get_pixel(0, 0);
        assert_eq!(pixel.0[0], 100);
        assert_eq!(pixel.0[1], 150);
        assert_eq!(pixel.0[2], 200);
        assert_eq!(pixel.0[3], 255);
    }

    #[test]
    fn from_dynamic_image_preserves_rgba() {
        let rgba = RgbaImage::from_pixel(3, 3, Rgba([10, 20, 30, 128]));
        let dyn_img = DynamicImage::ImageRgba8(rgba.clone());
        let img = Image::from_dynamic_image(dyn_img);
        assert_eq!(*img.as_rgba8(), rgba);
    }

    #[test]
    fn from_dynamic_image_converts_grayscale_to_rgba() {
        let gray = image::GrayImage::from_pixel(2, 2, image::Luma([42]));
        let dyn_img = DynamicImage::ImageLuma8(gray);
        let img = Image::from_dynamic_image(dyn_img);
        let pixel = img.as_rgba8().get_pixel(0, 0);
        assert_eq!(pixel.0[0], 42);
        assert_eq!(pixel.0[1], 42);
        assert_eq!(pixel.0[2], 42);
        assert_eq!(pixel.0[3], 255);
    }

    #[test]
    fn get_pixel_color_is_immutable() {
        let rgba = RgbaImage::from_pixel(5, 5, Rgba([10, 20, 30, 255]));
        let img = Image::from_rgba8(rgba);
        let color = img.get_pixel_color(point(2, 3)).unwrap();
        assert_eq!(color, Color::new(10, 20, 30, 255));
    }

    #[test]
    fn get_pixel_color_out_of_bounds() {
        let img = Image::new(5, 5);
        assert!(img.get_pixel_color(point(5, 0)).is_err());
        assert!(img.get_pixel_color(point(0, 5)).is_err());
        assert!(img.get_pixel_color(point(-1, 0)).is_err());
    }

    #[test]
    fn set_pixel_invalidates_caches() {
        let img_data = RgbaImage::from_pixel(5, 5, Rgba([10, 20, 30, 255]));
        let mut img = Image::from_rgba8(img_data);

        // Populate caches via the TryFrom impl
        let _source = Arc::<Source>::try_from(&img).unwrap();
        assert!(img.source.get().is_some());

        img.set_pixel_color(point(0, 0), Color::new(255, 0, 0, 255))
            .unwrap();
        assert!(img.source.get().is_none());
    }

    #[test]
    fn clone_does_not_share_caches() {
        let img_data = RgbaImage::from_pixel(5, 5, Rgba([10, 20, 30, 255]));
        let img = Image::from_rgba8(img_data);

        // Populate caches via the TryFrom impl
        let _source = Arc::<Source>::try_from(&img).unwrap();
        assert!(img.source.get().is_some());

        #[allow(clippy::redundant_clone)]
        let cloned = img.clone();
        assert!(cloned.source.get().is_none());
    }

    #[test]
    fn from_bytes_png() {
        // Create a tiny valid PNG in memory
        let rgba = RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        let mut bytes = Vec::new();
        DynamicImage::ImageRgba8(rgba)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();

        let img = Image::from_bytes(&bytes).unwrap();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        let pixel = img.as_rgba8().get_pixel(0, 0);
        assert_eq!(*pixel, Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn fill_and_read_back() {
        let mut img = Image::new(3, 3);
        let color = Color::new(100, 200, 50, 255);
        img.fill_mut(color);
        let read_back = img.get_pixel_color(point(1, 1)).unwrap();
        assert_eq!(read_back, color);
    }

    #[test]
    fn invert_round_trip() {
        let rgba = RgbaImage::from_pixel(2, 2, Rgba([100, 150, 200, 255]));
        let img = Image::from_rgba8(rgba);
        let inverted = img.inverted().inverted();
        assert_eq!(*img.as_rgba8(), *inverted.as_rgba8());
    }

    /// Creates a 10x10 opaque white image with both caches populated.
    fn cached_image() -> Image {
        let img = Image::from_rgba8(RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255])));
        img.populate_caches();
        img
    }

    fn assert_caches_cleared(img: &Image) {
        assert!(!img.has_cached_source(), "source cache should be cleared");
        assert!(
            !img.has_cached_template(),
            "template cache should be cleared"
        );
    }

    // -- Cache invalidation tests for every _mut method --

    #[test]
    fn cache_reset_by_invert_mut() {
        let mut img = cached_image();
        img.invert_mut();
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_blur_mut() {
        let mut img = cached_image();
        img.blur_mut(BlurOptions::default());
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_rotate_mut_90() {
        let mut img = cached_image();
        img.rotate_mut(90.0, RotationOptions::default());
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_rotate_mut_arbitrary() {
        let mut img = cached_image();
        img.rotate_mut(45.0, RotationOptions::default());
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_rotate_mut_with_center() {
        let mut img = cached_image();
        img.rotate_mut(
            45.0,
            RotationOptions {
                center: Some(point(5, 5)),
                ..Default::default()
            },
        );
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_flip_mut_horizontal() {
        let mut img = cached_image();
        img.flip_mut(FlipDirection::Horizontal);
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_flip_mut_vertical() {
        let mut img = cached_image();
        img.flip_mut(FlipDirection::Vertical);
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_hue_rotate_mut() {
        let mut img = cached_image();
        img.hue_rotate_mut(90);
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_grayscale_mut() {
        let mut img = cached_image();
        img.grayscale_mut();
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_crop_mut() {
        let mut img = cached_image();
        img.crop_mut(rect(point(0, 0), size(5, 5)));
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_resize_mut() {
        let mut img = cached_image();
        img.resize_mut(20, 20, ResizeOptions::default());
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_adjust_brightness_mut() {
        let mut img = cached_image();
        img.adjust_brightness_mut(10);
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_adjust_contrast_mut() {
        let mut img = cached_image();
        img.adjust_contrast_mut(10.0);
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_fill_mut() {
        let mut img = cached_image();
        img.fill_mut(Color::new(0, 0, 0, 255));
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_set_pixel_color() {
        let mut img = cached_image();
        img.set_pixel_color(point(0, 0), Color::new(0, 0, 0, 255))
            .unwrap();
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_cross_mut() {
        let mut img = cached_image();
        img.draw_cross_mut(point(5, 5), Color::new(255, 0, 0, 255));
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_line_mut() {
        let mut img = cached_image();
        img.draw_line_mut(point(0, 0), point(9, 9), Color::new(255, 0, 0, 255));
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_circle_mut() {
        let mut img = cached_image();
        img.draw_circle_mut(
            point(5, 5),
            3,
            Color::new(255, 0, 0, 255),
            DrawingOptions::default(),
        );
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_circle_mut_hollow() {
        let mut img = cached_image();
        img.draw_circle_mut(
            point(5, 5),
            3,
            Color::new(255, 0, 0, 255),
            DrawingOptions { hollow: true },
        );
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_ellipse_mut() {
        let mut img = cached_image();
        img.draw_ellipse_mut(
            point(5, 5),
            3,
            2,
            Color::new(255, 0, 0, 255),
            DrawingOptions::default(),
        );
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_ellipse_mut_hollow() {
        let mut img = cached_image();
        img.draw_ellipse_mut(
            point(5, 5),
            3,
            2,
            Color::new(255, 0, 0, 255),
            DrawingOptions { hollow: true },
        );
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_rectangle_mut() {
        let mut img = cached_image();
        img.draw_rectangle_mut(
            rect(point(1, 1), size(4, 4)),
            Color::new(255, 0, 0, 255),
            DrawingOptions::default(),
        )
        .unwrap();
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_rectangle_mut_hollow() {
        let mut img = cached_image();
        img.draw_rectangle_mut(
            rect(point(1, 1), size(4, 4)),
            Color::new(255, 0, 0, 255),
            DrawingOptions { hollow: true },
        )
        .unwrap();
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_draw_image_mut() {
        let mut img = cached_image();
        let overlay = Image::from_rgba8(RgbaImage::from_pixel(3, 3, Rgba([0, 0, 255, 255])));
        img.draw_image_mut(point(0, 0), &overlay, DrawImageOptions::default())
            .unwrap();
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_reset_by_deref_mut() {
        let mut img = cached_image();
        // Obtaining a &mut RgbaImage via DerefMut should clear caches
        let _inner: &mut RgbaImage = &mut img;
        assert_caches_cleared(&img);
    }

    #[test]
    fn cache_not_cleared_by_immutable_access() {
        let img = cached_image();
        // Read-only operations should not clear caches
        let _w = img.width();
        let _h = img.height();
        let _pixel = img.get_pixel_color(point(0, 0)).unwrap();
        let _rgba = img.as_rgba8();
        assert!(img.has_cached_source());
        assert!(img.has_cached_template());
    }

    #[test]
    fn immutable_variants_return_fresh_image_without_caches() {
        let img = cached_image();

        // Non-mutating methods return a new Image; original caches should be untouched
        let inv = img.inverted();
        assert!(!inv.has_cached_source());
        assert!(!inv.has_cached_template());
        assert!(img.has_cached_source());
        assert!(img.has_cached_template());
    }
}
