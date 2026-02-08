use std::ffi::c_void;

use color_eyre::Result;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, RGBQUAD, ReleaseDC, SRCCOPY,
    SelectObject,
};

use crate::{core::rect::Rect, types::su32::Su32};

/// Captures a screen region using BitBlt and returns raw BGRA pixel data.
#[allow(unsafe_code)]
pub fn capture_rect(rect: Rect) -> Result<Vec<u8>> {
    let width: i32 = rect.size.width.into();
    let height: i32 = rect.size.height.into();
    let x: i32 = rect.top_left.x.into();
    let y: i32 = rect.top_left.y.into();

    let hdc_screen = unsafe { GetDC(None) };
    let hdc_mem = unsafe { CreateCompatibleDC(Some(hdc_screen)) };

    let hbm = unsafe { CreateCompatibleBitmap(hdc_screen, width, height) };
    unsafe { SelectObject(hdc_mem, hbm.into()) };

    unsafe {
        BitBlt(
            hdc_mem,
            0,
            0,
            width,
            height,
            Some(hdc_screen),
            x,
            y,
            SRCCOPY,
        )?;
    }

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: Su32::from(size_of::<BITMAPINFOHEADER>()).into(),
            biWidth: width,
            biHeight: -height, // Top-down bitmap
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    };

    let buffer_size: usize = Su32::from(width).into();
    let buffer_size = buffer_size * usize::from(Su32::from(height)) * 4;
    let mut data = vec![0u8; buffer_size];

    #[allow(clippy::as_conversions)] // pointer cast
    let data_ptr = data.as_mut_ptr() as *mut c_void;

    unsafe {
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            u32::from(Su32::from(height)),
            #[allow(clippy::as_conversions)] // pointer cast
            Some(data_ptr as *mut _),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };

    unsafe { ReleaseDC(None, hdc_screen) };
    unsafe {
        _ = DeleteDC(hdc_mem);
    };
    unsafe {
        _ = DeleteObject(hbm.into());
    };

    Ok(data)
}
