use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt, ImageFormat},
};

pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn capture_screenshot() -> Screenshot {
    let (connection, screen_number) = x11rb::connect(None).expect("X11 connect failed");
    let screen = &connection.setup().roots[screen_number];
    let width = u32::from(screen.width_in_pixels);
    let height = u32::from(screen.height_in_pixels);

    let image = connection
        .get_image(
            ImageFormat::Z_PIXMAP,
            screen.root,
            0,
            0,
            width as u16,
            height as u16,
            !0u32,
        )
        .expect("GetImage request")
        .reply()
        .expect("GetImage reply");

    // X11 ZPixmap 32bpp: BGRX in memory -> convert to RGBA.
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for (pixel_index, pixel_chunk) in image.data.chunks_exact(4).enumerate() {
        let channel_index = pixel_index * 4;
        rgba[channel_index] = pixel_chunk[2];
        rgba[channel_index + 1] = pixel_chunk[1];
        rgba[channel_index + 2] = pixel_chunk[0];
        rgba[channel_index + 3] = 255;
    }

    Screenshot {
        width,
        height,
        rgba,
    }
}

pub fn screenshot_color_at(
    screenshot: &Screenshot,
    x_position: i32,
    y_position: i32,
) -> Option<[u8; 3]> {
    if x_position < 0
        || y_position < 0
        || x_position as u32 >= screenshot.width
        || y_position as u32 >= screenshot.height
    {
        return None;
    }

    let pixel_index = ((y_position as u32 * screenshot.width + x_position as u32) * 4) as usize;
    Some([
        screenshot.rgba[pixel_index],
        screenshot.rgba[pixel_index + 1],
        screenshot.rgba[pixel_index + 2],
    ])
}
