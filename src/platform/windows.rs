//! Win32 presenter — blits the framebuffer to an HWND using `StretchDIBits`.

use winapi::shared::windef::HWND;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::error::{Error, Result};
use crate::framebuffer::Framebuffer;

pub struct PlatformPresenter {
    hwnd: HWND,
}

unsafe impl Send for PlatformPresenter {}

impl PlatformPresenter {
    pub fn new<W: HasWindowHandle>(handle: &W) -> Result<Self> {
        let hwnd = match handle.window_handle().map_err(|_| Error::InvalidHandle)?.as_raw() {
            RawWindowHandle::Win32(h) => h.hwnd.get() as HWND,
            _ => return Err(Error::InvalidHandle),
        };
        Ok(Self { hwnd })
    }

    /// Blit the framebuffer to the window using `SetDIBitsToDevice`.
    ///
    /// The DIB uses `BI_BITFIELDS` with RGB masks so we can feed our ARGB
    /// pixels directly without a per-pixel format conversion.
    pub fn present(&self, fb: &Framebuffer) -> Result<()> {
        let w = fb.width()  as i32;
        let h = fb.height() as i32;

        unsafe {
            let hdc = GetDC(self.hwnd);
            if hdc.is_null() {
                return Err(Error::Platform("GetDC failed".into()));
            }

            // Build a BITMAPV4HEADER so we can specify channel masks.
            // Our pixels are 0xAARRGGBB — which maps directly to the
            // BGRA DIB format with the masks below.
            let mut bmi: BITMAPV4HEADER = std::mem::zeroed();
            bmi.bV4Size          = std::mem::size_of::<BITMAPV4HEADER>() as u32;
            bmi.bV4Width         = w;
            bmi.bV4Height        = -h; // negative = top-down
            bmi.bV4Planes        = 1;
            bmi.bV4BitCount      = 32;
            bmi.bV4V4Compression = BI_BITFIELDS;
            bmi.bV4RedMask       = 0x00FF0000;
            bmi.bV4GreenMask     = 0x0000FF00;
            bmi.bV4BlueMask      = 0x000000FF;
            bmi.bV4AlphaMask     = 0xFF000000;

            let result = SetDIBitsToDevice(
                hdc,
                0,              // x dest
                0,              // y dest
                w as u32,
                h as u32,
                0,              // x src
                0,              // y src
                0,              // start scan
                h as u32,       // scan lines
                fb.pixels().as_ptr() as *const _,
                &bmi as *const BITMAPV4HEADER as *const BITMAPINFO,
                DIB_RGB_COLORS,
            );

            ReleaseDC(self.hwnd, hdc);

            if result == 0 {
                return Err(Error::Platform("SetDIBitsToDevice failed".into()));
            }
        }

        Ok(())
    }
}
