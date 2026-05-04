use color_eyre::{Result, eyre::eyre};
use image::RgbaImage;
use satint::{SaturatingInto, Su32, su32};
use screenshot::{Capture, bgra_to_rgba_in_place};
use types::size::Size;

use crate::api::image::Image;

impl Image {
    /// Build an [`Image`] from a raw BGRA buffer, converting to RGBA.
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

        let mut image = RgbaImage::new(size.width.into(), size.height.into());
        let image_data: &mut [u8] = image.as_mut();
        image_data.copy_from_slice(&data[..needed]);
        bgra_to_rgba_in_place(image_data);

        Ok(Self::from_rgba8(image))
    }

    /// Build an [`Image`] from a [`Capture`], converting BGRA→RGBA in place.
    pub fn from_capture(capture: Capture) -> Result<Self> {
        let Capture { size, mut bgra } = capture;
        const BYTES_PER_PIXEL: usize = 4;
        let width: usize = size.width.saturating_into();
        let height: usize = size.height.saturating_into();
        let needed = width
            .checked_mul(height)
            .and_then(|pixel_count| pixel_count.checked_mul(BYTES_PER_PIXEL))
            .ok_or_else(|| eyre!("image dimensions overflow: {size}"))?;
        if bgra.len() < needed {
            return Err(eyre!(
                "capture data too small: expected {needed} bytes, got {}",
                bgra.len()
            ));
        }
        bgra.truncate(needed);
        bgra_to_rgba_in_place(&mut bgra);
        let image = RgbaImage::from_raw(size.width.into(), size.height.into(), bgra)
            .ok_or_else(|| eyre!("failed to build RgbaImage from capture buffer"))?;
        Ok(Self::from_rgba8(image))
    }
}
