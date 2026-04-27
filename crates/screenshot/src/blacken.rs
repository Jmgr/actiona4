use types::{rect::Rect, su32::Su32, su32::su32};

/// Fill all pixels of `image` that lie outside every display rectangle with
/// black.
///
/// `image` is a packed RGBA (or BGRA) buffer of `desktop_rect.size.width *
/// desktop_rect.size.height * 4` bytes whose origin maps to
/// `desktop_rect.top_left` in screen coordinates. Pixels outside every
/// display are zeroed in place (no extra allocation).
pub fn blacken_non_display_areas(image: &mut [u8], desktop_rect: Rect, display_rects: &[Rect]) {
    let width = desktop_rect.size.width;

    let mut bands: Vec<(Su32, Su32, Su32, Su32)> = display_rects
        .iter()
        .filter_map(|&display_rect| {
            let overlap = display_rect.intersection(desktop_rect)?;
            let offset = overlap.top_left - desktop_rect.top_left;
            let img_x0: Su32 = offset.x.into();
            let img_y0: Su32 = offset.y.into();
            let img_x1 = img_x0 + overlap.size.width;
            let img_y1 = img_y0 + overlap.size.height;
            Some((img_y0, img_y1, img_x0, img_x1))
        })
        .collect();
    bands.sort_unstable_by_key(|&(_, _, x0, _)| x0);

    for (y_idx, row) in image.chunks_exact_mut(usize::from(width) * 4).enumerate() {
        let y = su32(y_idx);

        let mut cursor = Su32::ZERO;
        for &(y0, y1, x0, x1) in &bands {
            if y < y0 || y >= y1 {
                continue;
            }
            if cursor < x0 {
                row[usize::from(cursor) * 4..usize::from(x0) * 4].fill(0);
            }
            cursor = cursor.max(x1);
        }
        if cursor < width {
            row[usize::from(cursor) * 4..].fill(0);
        }
    }
}
