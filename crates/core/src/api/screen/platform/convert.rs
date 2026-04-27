use color_eyre::{Result, eyre::eyre};
use image::RgbaImage;
use screenshot::{Capture, bgra_to_rgba_in_place};

use crate::{api::image::Image, types::su32::Su32};

impl Image {
    /// Build an [`Image`] from a raw BGRA buffer, converting to RGBA.
    pub fn from_bgra(data: &[u8], width: u32, height: u32) -> Result<Self> {
        const BYTES_PER_PIXEL: usize = 4;

        let needed = usize::from(Su32::from(width))
            .checked_mul(Su32::from(height).into())
            .and_then(|pixel_count| pixel_count.checked_mul(BYTES_PER_PIXEL))
            .ok_or_else(|| eyre!("image dimensions overflow: {width}x{height}"))?;

        if data.len() < needed {
            return Err(eyre!(
                "image data too small: expected {needed} bytes, got {}",
                data.len()
            ));
        }

        let mut image = RgbaImage::new(width, height);
        let image_data: &mut [u8] = image.as_mut();
        image_data.copy_from_slice(&data[..needed]);
        bgra_to_rgba_in_place(image_data);

        Ok(Self::from_rgba8(image))
    }

    /// Build an [`Image`] from a [`Capture`], converting BGRA→RGBA in place.
    pub fn from_capture(capture: Capture) -> Result<Self> {
        let Capture {
            width,
            height,
            mut bgra,
        } = capture;
        const BYTES_PER_PIXEL: usize = 4;
        let needed = usize::from(Su32::from(width))
            .checked_mul(Su32::from(height).into())
            .and_then(|pixel_count| pixel_count.checked_mul(BYTES_PER_PIXEL))
            .ok_or_else(|| eyre!("image dimensions overflow: {width}x{height}"))?;
        if bgra.len() < needed {
            return Err(eyre!(
                "capture data too small: expected {needed} bytes, got {}",
                bgra.len()
            ));
        }
        bgra.truncate(needed);
        bgra_to_rgba_in_place(&mut bgra);
        let image = RgbaImage::from_raw(width, height, bgra)
            .ok_or_else(|| eyre!("failed to build RgbaImage from capture buffer"))?;
        Ok(Self::from_rgba8(image))
    }
}
