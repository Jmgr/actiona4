use color_eyre::{Result, eyre::eyre};
use image::RgbaImage;
use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};

use crate::{core::image::Image, types::su32::Su32};

impl Image {
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

        // Convert from BGRA to RGBA
        image_data.par_chunks_exact_mut(4).for_each(|c| {
            c.swap(0, 2); // Swap R and B
            c[3] = 255; // Set alpha to 255 (fully opaque)
        });

        Ok(Self::from_rgba8(image))
    }
}
