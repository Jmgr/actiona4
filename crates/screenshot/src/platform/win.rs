use std::ffi::c_void;

use color_eyre::{Result, eyre::eyre};
use satint::{SaturatingInto, Si32};
use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use types::{Rect, size};
use windows::Win32::{
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, RGBQUAD, ReleaseDC, SRCCOPY,
        SelectObject,
    },
    UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    },
};

use crate::Capture;

/// Windows screen capture handle. Every capture spawns a blocking task that
/// performs the GDI calls.
#[derive(Clone, Debug, Default)]
pub struct Screen {
    task_tracker: TaskTracker,
    cancellation_token: CancellationToken,
}

impl Screen {
    #[expect(
        clippy::unused_async,
        reason = "screen implementations expose a uniform async constructor API"
    )]
    pub async fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        Ok(Self {
            task_tracker,
            cancellation_token,
        })
    }

    /// Capture the entire virtual screen (the bounding box of all monitors).
    pub async fn capture_full_screen(&self) -> Result<Capture> {
        self.capture_blocking(capture_virtual_screen).await
    }

    /// Capture an arbitrary rectangle of the virtual screen.
    pub async fn capture_rect(&self, rect: Rect) -> Result<Capture> {
        self.capture_blocking(move || {
            capture_rect_blocking(
                rect.top_left.x,
                rect.top_left.y,
                rect.size.width.saturating_into(),
                rect.size.height.saturating_into(),
            )
        })
        .await
    }

    async fn capture_blocking<F>(&self, capture: F) -> Result<Capture>
    where
        F: FnOnce() -> Result<Capture> + Send + 'static,
    {
        let handle = self.task_tracker.spawn_blocking(capture);

        select! {
            () = self.cancellation_token.cancelled() => Err(eyre!("screen capture cancelled")),
            result = handle => result?,
        }
    }
}

#[allow(unsafe_code)]
fn capture_virtual_screen() -> Result<Capture> {
    // SAFETY: GetSystemMetrics takes only metric constants and returns scalar dimensions.
    let (x, y, width, height) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN).into(),
            GetSystemMetrics(SM_YVIRTUALSCREEN).into(),
            GetSystemMetrics(SM_CXVIRTUALSCREEN).into(),
            GetSystemMetrics(SM_CYVIRTUALSCREEN).into(),
        )
    };
    capture_rect_blocking(x, y, width, height)
}

#[allow(unsafe_code)]
fn capture_rect_blocking(x: Si32, y: Si32, width: Si32, height: Si32) -> Result<Capture> {
    // SAFETY: A null HWND requests the screen device context.
    let hdc_screen = unsafe { GetDC(None) };
    // SAFETY: `hdc_screen` is the device context returned immediately above.
    let hdc_mem = unsafe { CreateCompatibleDC(Some(hdc_screen)) };

    // SAFETY: The source context and requested dimensions are valid capture inputs.
    let hbm = unsafe { CreateCompatibleBitmap(hdc_screen, width.into(), height.into()) };
    // SAFETY: `hbm` is compatible with `hdc_mem` and is selected for the lifetime of the copy.
    unsafe { SelectObject(hdc_mem, hbm.into()) };

    let result = (|| -> Result<Capture> {
        // SAFETY: Both contexts and the selected bitmap are valid; the dimensions were checked by
        // the caller before allocating the destination bitmap.
        unsafe {
            BitBlt(
                hdc_mem,
                0,
                0,
                width.into(),
                height.into(),
                Some(hdc_screen),
                x.into(),
                y.into(),
                SRCCOPY,
            )?;
        }

        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: u32::try_from(size_of::<BITMAPINFOHEADER>())?,
                biWidth: width.into(),
                biHeight: (-height).into(), // top-down bitmap
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

        let width_u: usize = width.saturating_into();
        let height_u: usize = height.saturating_into();
        let buffer_size = width_u
            .checked_mul(height_u)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| color_eyre::eyre::eyre!("capture dimensions overflow"))?;
        let mut data = vec![0_u8; buffer_size];

        #[allow(clippy::as_conversions)]
        let data_ptr = data.as_mut_ptr().cast::<c_void>();

        // SAFETY: `data` is sized for the requested 32-bit bitmap and `bitmap_info` is initialized.
        unsafe {
            GetDIBits(
                hdc_mem,
                hbm,
                0,
                height.saturating_into(),
                Some(data_ptr),
                &raw mut bitmap_info,
                DIB_RGB_COLORS,
            );
        }

        Ok(Capture {
            size: size(width, height),
            bgra: data,
        })
    })();

    // SAFETY: These handles were created above and are released after all GDI operations finish.
    unsafe {
        ReleaseDC(None, hdc_screen);
        _ = DeleteDC(hdc_mem);
        _ = DeleteObject(hbm.into());
    }

    result
}
