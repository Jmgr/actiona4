use std::{borrow::Cow, io::Cursor, ops::DerefMut, sync::Arc};

use arc_swap::ArcSwapOption;
use color_eyre::Result;
use derive_more::Deref;
use image::{ColorType, DynamicImage, GrayImage, ImageReader, RgbImage, RgbaImage};
use opencv::core::Mat;

pub mod find_image;
pub mod js;

#[derive(Default, Debug)]
struct MatCache(ArcSwapOption<Mat>);

impl MatCache {
    fn reset(&self) {
        self.0.store(None);
    }

    fn get(&self) -> Option<Arc<Mat>> {
        self.0.load_full()
    }

    fn set(&self, mat: Arc<Mat>) {
        self.0.store(Some(mat));
    }
}

#[derive(Debug, Default, Deref)]
pub struct Image {
    inner: DynamicImage,

    #[deref(ignore)]
    rgb_mat: MatCache,

    #[deref(ignore)]
    rgba_mat: MatCache,

    #[deref(ignore)]
    greyscale_mat: MatCache,
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.rgb_mat.reset();
        self.rgba_mat.reset();
        self.greyscale_mat.reset();

        &mut self.inner
    }
}

impl Clone for Image {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            rgb_mat: Default::default(),
            rgba_mat: Default::default(),
            greyscale_mat: Default::default(),
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
            rgb_mat: Default::default(),
            rgba_mat: Default::default(),
            greyscale_mat: Default::default(),
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

/*
 TODO:
 1) search one or search multiple
 2) search for multiple templates (in parallel), label them
 3) track an item (post 1.0)
 4) UI to test parameters and display results on screen (transparent target icon?)
 5) find_image should probably run in a spawn_blocking
*/
