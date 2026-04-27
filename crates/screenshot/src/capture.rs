use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};

/// A raw screen capture in BGRA pixel order, top-down.
#[derive(Clone, Debug)]
pub struct Capture {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl Capture {
    /// Consume the capture and return its bytes converted to RGBA.
    pub fn into_rgba(mut self) -> Vec<u8> {
        bgra_to_rgba_in_place(&mut self.bgra);
        self.bgra
    }
}

/// Convert a BGRA buffer to RGBA in place: swap the R and B channels and set
/// alpha to 255 for every pixel.
pub fn bgra_to_rgba_in_place(buffer: &mut [u8]) {
    buffer.par_chunks_exact_mut(4).for_each(|pixel| {
        pixel.swap(0, 2);
        pixel[3] = 255;
    });
}
