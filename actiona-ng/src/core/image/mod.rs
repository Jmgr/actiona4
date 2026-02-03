use std::{borrow::Cow, fs, io::Cursor, ops::DerefMut, sync::Arc};

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use arc_swap::ArcSwapOption;
use color_eyre::{Result, eyre::eyre};
use derive_more::Deref;
use image::{
    ColorType, DynamicImage, GenericImageView, GrayImage, ImageReader, RgbImage, RgbaImage,
    imageops::{self, FilterType},
    metadata::Orientation,
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

pub mod find_image;
pub mod js;

use crate::core::{
    color::Color,
    image::find_image::{Source, Template},
    point::{Point, point},
    rect::{Rect, rect},
    size::size,
};
use crate::types::{si32::Si32, su32::su32};

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

#[derive(Debug, Default, Deref)]
pub struct Image {
    inner: DynamicImage,

    #[deref(ignore)]
    source: Cache<Source>,

    #[deref(ignore)]
    template: Cache<Template>,
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

impl From<Image> for DynamicImage {
    fn from(value: Image) -> Self {
        value.inner
    }
}

impl Image {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self::from_dynamic_image(DynamicImage::new(width, height, ColorType::Rgba8))
    }

    #[must_use]
    pub fn from_dynamic_image(image: DynamicImage) -> Self {
        Self {
            inner: image,
            source: Default::default(),
            template: Default::default(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;
        Ok(Self::from_dynamic_image(image))
    }

    #[must_use]
    pub fn into_inner(self) -> DynamicImage {
        self.inner
    }

    #[must_use]
    pub const fn to_inner(&self) -> &DynamicImage {
        &self.inner
    }
}

impl Image {
    #[must_use]
    pub fn to_rgb8(&'_ self) -> Cow<'_, RgbImage> {
        if let DynamicImage::ImageRgb8(image) = &self.inner {
            Cow::Borrowed(image)
        } else {
            Cow::Owned(self.inner.to_rgb8())
        }
    }

    #[must_use]
    pub fn to_rgba8(&'_ self) -> Cow<'_, RgbaImage> {
        if let DynamicImage::ImageRgba8(image) = &self.inner {
            Cow::Borrowed(image)
        } else {
            Cow::Owned(self.inner.to_rgba8())
        }
    }

    #[must_use]
    pub fn to_luma8(&'_ self) -> Cow<'_, GrayImage> {
        if let DynamicImage::ImageLuma8(image) = &self.inner {
            Cow::Borrowed(image)
        } else {
            Cow::Owned(self.inner.to_luma8())
        }
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
        self.inner.invert();
    }

    #[must_use]
    pub fn inverted(&self) -> Self {
        let mut cloned = self.clone();
        cloned.invert_mut();
        cloned
    }

    pub fn blur_mut(&mut self, options: BlurOptions) {
        self.inner = self.blur_dynamic(options);
    }

    #[must_use]
    pub fn blurred(&self, options: BlurOptions) -> Self {
        Self::from_dynamic_image(self.blur_dynamic(options))
    }

    fn blur_dynamic(&self, options: BlurOptions) -> DynamicImage {
        if options.fast {
            self.inner.fast_blur(options.sigma)
        } else {
            self.inner.blur(options.sigma)
        }
    }

    pub fn rotate_mut(&mut self, angle: f32, options: RotationOptions) {
        if let Some(center) = options.center {
            let rgba = self.ensure_rgba_mut();
            self.inner = DynamicImage::ImageRgba8(rotate(
                rgba,
                (center.x.into(), center.y.into()),
                angle.to_radians(),
                options.interpolation.into(),
                options.default_color.into(),
            ));
        } else {
            match angle.rem_euclid(360.0) {
                0.0 => {}
                90.0 => self.inner.apply_orientation(Orientation::Rotate90),
                180.0 => self.inner.apply_orientation(Orientation::Rotate180),
                270.0 => self.inner.apply_orientation(Orientation::Rotate270),
                _ => {
                    let rgba = self.ensure_rgba_mut();
                    self.inner = DynamicImage::ImageRgba8(rotate_about_center(
                        rgba,
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
                    90.0 => self.inner.rotate90(),
                    180.0 => self.inner.rotate180(),
                    270.0 => self.inner.rotate270(),
                    _ => DynamicImage::ImageRgba8(rotate_about_center(
                        &self.inner.to_rgba8(),
                        angle.to_radians(),
                        options.interpolation.into(),
                        options.default_color.into(),
                    )),
                };

                Self::from_dynamic_image(result)
            }
        }
    }

    pub fn flip_mut(&mut self, direction: FlipDirection) {
        let orientation = match direction {
            FlipDirection::Horizontal => Orientation::FlipHorizontal,
            FlipDirection::Vertical => Orientation::FlipVertical,
        };
        self.inner.apply_orientation(orientation);
    }

    #[must_use]
    pub fn flipped(&self, direction: FlipDirection) -> Self {
        let image = match direction {
            FlipDirection::Horizontal => self.inner.fliph(),
            FlipDirection::Vertical => self.inner.flipv(),
        };

        Self::from_dynamic_image(image)
    }

    pub fn hue_rotate_mut(&mut self, value: i32) {
        self.inner = self.inner.huerotate(value);
    }

    #[must_use]
    pub fn hue_rotated(&self, value: i32) -> Self {
        Self::from_dynamic_image(self.inner.huerotate(value))
    }

    pub fn grayscale_mut(&mut self) {
        self.inner = self.inner.grayscale();
    }

    #[must_use]
    pub fn grayscaled(&self) -> Self {
        Self::from_dynamic_image(self.inner.grayscale())
    }

    pub fn crop_mut(&mut self, rect: Rect) {
        let (x, y, width, height) = rect.clamped();
        self.inner = self.inner.crop_imm(x, y, width, height);
    }

    #[must_use]
    pub fn cropped(&self, rect: Rect) -> Self {
        let (x, y, width, height) = rect.clamped();
        Self::from_dynamic_image(self.inner.crop_imm(x, y, width, height))
    }

    pub fn resize_mut(&mut self, width: u32, height: u32, options: ResizeOptions) {
        self.inner = self.resize_dynamic(width, height, options);
    }

    #[must_use]
    pub fn resized(&self, width: u32, height: u32, options: ResizeOptions) -> Self {
        Self::from_dynamic_image(self.resize_dynamic(width, height, options))
    }

    fn resize_dynamic(&self, width: u32, height: u32, options: ResizeOptions) -> DynamicImage {
        if options.keep_aspect_ratio {
            self.inner.resize(width, height, options.filter.into())
        } else {
            self.inner
                .resize_exact(width, height, options.filter.into())
        }
    }

    pub fn adjust_brightness_mut(&mut self, value: i32) {
        self.inner = self.inner.brighten(value);
    }

    #[must_use]
    pub fn adjusted_brightness(&self, value: i32) -> Self {
        Self::from_dynamic_image(self.inner.brighten(value))
    }

    pub fn adjust_contrast_mut(&mut self, value: f32) {
        self.inner = self.inner.adjust_contrast(value);
    }

    #[must_use]
    pub fn adjusted_contrast(&self, value: f32) -> Self {
        Self::from_dynamic_image(self.inner.adjust_contrast(value))
    }

    pub fn fill_mut(&mut self, color: Color) {
        let rgba = self.ensure_rgba_mut();
        let fill_color: image::Rgba<u8> = color.into();
        for pixel in rgba.pixels_mut() {
            *pixel = fill_color;
        }
    }

    #[must_use]
    pub fn filled(&self, color: Color) -> Self {
        let mut rgba = self.inner.to_rgba8();
        let fill_color: image::Rgba<u8> = color.into();
        for pixel in rgba.pixels_mut() {
            *pixel = fill_color;
        }

        Self::from_dynamic_image(DynamicImage::ImageRgba8(rgba))
    }

    pub fn get_pixel_color(&mut self, position: Point) -> Result<Color> {
        let (x, y) = self.check_position(position)?;
        let rgba = self.ensure_rgba_mut();
        Ok(Color::from(*rgba.get_pixel(x, y)))
    }

    pub fn set_pixel_color(&mut self, position: Point, color: Color) -> Result<()> {
        let (x, y) = self.check_position(position)?;
        let rgba = self.ensure_rgba_mut();
        rgba.put_pixel(x, y, color.into());
        Ok(())
    }

    pub fn copy_region(&self, rect: Rect) -> Result<Self> {
        let (x, y, width, height) = self.check_rect(&rect)?;
        Ok(Self::from_dynamic_image(DynamicImage::ImageRgba8(
            self.inner.view(x, y, width, height).to_image(),
        )))
    }

    #[must_use]
    pub fn bounds_rect(&self) -> Rect {
        rect(point(0, 0), size(self.width(), self.height()))
    }

    pub fn draw_cross_mut(&mut self, position: Point, color: Color) {
        let rgba: image::Rgba<u8> = color.into();
        draw_cross_mut(&mut self.inner, rgba, position.x.into(), position.y.into());
    }

    #[must_use]
    pub fn with_cross(&self, position: Point, color: Color) -> Self {
        let rgba: image::Rgba<u8> = color.into();
        Self::from_dynamic_image(DynamicImage::ImageRgba8(draw_cross(
            &self.inner,
            rgba,
            position.x.into(),
            position.y.into(),
        )))
    }

    pub fn draw_line_mut(&mut self, start: Point, end: Point, color: Color) {
        let rgba: image::Rgba<u8> = color.into();
        draw_line_segment_mut(
            &mut self.inner,
            (start.x.into(), start.y.into()),
            (end.x.into(), end.y.into()),
            rgba,
        );
    }

    #[must_use]
    pub fn with_line(&self, start: Point, end: Point, color: Color) -> Self {
        let rgba: image::Rgba<u8> = color.into();
        Self::from_dynamic_image(DynamicImage::ImageRgba8(draw_line_segment(
            &self.inner,
            (start.x.into(), start.y.into()),
            (end.x.into(), end.y.into()),
            rgba,
        )))
    }

    pub fn draw_circle_mut(
        &mut self,
        center: Point,
        radius: i32,
        color: Color,
        options: DrawingOptions,
    ) {
        let rgba: image::Rgba<u8> = color.into();
        if options.hollow {
            draw_hollow_circle_mut(
                &mut self.inner,
                (center.x.into(), center.y.into()),
                radius,
                rgba,
            );
        } else {
            draw_filled_circle_mut(
                &mut self.inner,
                (center.x.into(), center.y.into()),
                radius,
                rgba,
            );
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

        Self::from_dynamic_image(DynamicImage::ImageRgba8(image))
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
        if options.hollow {
            draw_hollow_ellipse_mut(
                &mut self.inner,
                (center.x.into(), center.y.into()),
                width_radius,
                height_radius,
                rgba,
            );
        } else {
            draw_filled_ellipse_mut(
                &mut self.inner,
                (center.x.into(), center.y.into()),
                width_radius,
                height_radius,
                rgba,
            );
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

        Self::from_dynamic_image(DynamicImage::ImageRgba8(image))
    }

    pub fn draw_rectangle_mut(
        &mut self,
        rect: Rect,
        color: Color,
        options: DrawingOptions,
    ) -> Result<()> {
        let img_rect: ImgRect = rect.try_into().map_err(|err| eyre!("{err}"))?;
        let rgba: image::Rgba<u8> = color.into();

        if options.hollow {
            draw_hollow_rect_mut(&mut self.inner, img_rect, rgba);
        } else {
            draw_filled_rect_mut(&mut self.inner, img_rect, rgba);
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

        Ok(Self::from_dynamic_image(DynamicImage::ImageRgba8(image)))
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

    fn ensure_rgba_mut(&mut self) -> &mut RgbaImage {
        if self.inner.color() != ColorType::Rgba8 {
            let rgba = self.inner.to_rgba8();
            self.inner = DynamicImage::ImageRgba8(rgba);
        }

        self.inner.as_mut_rgba8().expect("image should be the RGBA")
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

        let source_rgba = source.to_rgba8();
        let cropped =
            imageops::crop_imm(source_rgba.as_ref(), src_x, src_y, width, height).to_image();

        dest.ensure_rgba_mut();
        imageops::overlay(
            &mut dest.inner,
            &DynamicImage::ImageRgba8(cropped),
            i64::from(dest_x),
            i64::from(dest_y),
        );

        Ok(())
    }

    fn load_font(font_path: &str) -> Result<FontArc> {
        let data = fs::read(font_path)
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
            .map(|line| text_size(scale, font, line).0)
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
        let canvas = image.ensure_rgba_mut();

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

/*
 TODO:
 1) search one or search multiple
 2) search for multiple templates (in parallel), label them
 3) track an item (post 1.0)
 4) UI to test parameters and display results on screen (transparent target icon?)
 5) find_image should probably run in a spawn_blocking
*/
