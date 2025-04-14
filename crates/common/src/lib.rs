use std::sync::OnceLock;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};

pub mod selection;
pub mod sentry;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub const FONT_BYTES: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");
pub const FONT_SIZE: f32 = 16.0;

fn font() -> &'static FontRef<'static> {
    static FONT: OnceLock<FontRef<'static>> = OnceLock::new();
    FONT.get_or_init(|| FontRef::try_from_slice(FONT_BYTES).expect("valid font"))
}

/// Returns the line height (ascent + |descent|) at `FONT_SIZE`.
pub fn line_height() -> i32 {
    let font = font();
    let scaled = font.as_scaled(PxScale::from(FONT_SIZE));
    (scaled.ascent() - scaled.descent()).ceil() as i32
}

pub fn draw_text(
    frame: &mut [u8],
    window_width: u32,
    window_height: u32,
    x: i32,
    y: i32,
    text: &str,
    color: [u8; 4],
) {
    let font = font();
    let scale = PxScale::from(FONT_SIZE);
    let scaled = font.as_scaled(scale);
    let ascent = scaled.ascent();

    let mut x_cursor = x as f32;
    let mut prev = None;

    for ch in text.chars() {
        let gid = scaled.glyph_id(ch);
        if let Some(p) = prev {
            x_cursor += scaled.kern(p, gid);
        }

        let glyph =
            gid.with_scale_and_position(scale, ab_glyph::point(x_cursor, y as f32 + ascent));

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|dx, dy, coverage| {
                let px = bounds.min.x as i32 + dx as i32;
                let py = bounds.min.y as i32 + dy as i32;
                if px >= 0 && py >= 0 && (px as u32) < window_width && (py as u32) < window_height {
                    let idx = ((py as u32 * window_width + px as u32) * 4) as usize;
                    if let Some(pixel) = frame.get_mut(idx..idx + 4) {
                        let a = coverage;
                        pixel[0] = (color[0] as f32 * a + pixel[0] as f32 * (1.0 - a)) as u8;
                        pixel[1] = (color[1] as f32 * a + pixel[1] as f32 * (1.0 - a)) as u8;
                        pixel[2] = (color[2] as f32 * a + pixel[2] as f32 * (1.0 - a)) as u8;
                        pixel[3] = 255;
                    }
                }
            });
        }

        x_cursor += scaled.h_advance(gid);
        prev = Some(gid);
    }
}
