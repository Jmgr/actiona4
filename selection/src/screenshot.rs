pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[cfg(not(windows))]
pub fn capture_screenshot() -> Screenshot {
    use x11rb::{
        connection::Connection,
        protocol::xproto::{ConnectionExt, ImageFormat},
    };

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

#[cfg(windows)]
pub fn capture_screenshot() -> Screenshot {
    use std::ffi::c_void;
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap,
        CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, RGBQUAD,
        ReleaseDC, SRCCOPY, SelectObject,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };

    unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32;
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32;

        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbm = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);
        SelectObject(hdc_mem, hbm.into());

        BitBlt(
            hdc_mem,
            0,
            0,
            width as i32,
            height as i32,
            Some(hdc_screen),
            x,
            y,
            SRCCOPY,
        )
        .expect("BitBlt failed");

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // negative = top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default()],
        };

        let mut bgra = vec![0u8; (width * height * 4) as usize];
        #[allow(clippy::as_conversions)]
        let data_ptr = bgra.as_mut_ptr() as *mut c_void;
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            height,
            Some(data_ptr as *mut _),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        );

        ReleaseDC(None, hdc_screen);
        _ = DeleteDC(hdc_mem);
        _ = DeleteObject(hbm.into());

        // GDI BitBlt returns BGRA; convert to RGBA.
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        for (i, chunk) in bgra.chunks_exact(4).enumerate() {
            let idx = i * 4;
            rgba[idx] = chunk[2];
            rgba[idx + 1] = chunk[1];
            rgba[idx + 2] = chunk[0];
            rgba[idx + 3] = 255;
        }

        Screenshot {
            width,
            height,
            rgba,
        }
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
