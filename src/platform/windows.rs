//! Zero-dependency Win32 software framebuffer presenter.
//!
//! Blits ARGB pixels directly to an HWND using GDI and `StretchDIBits`.

#![allow(non_snake_case, dead_code, clippy::upper_case_acronyms, clippy::too_many_arguments)]

use std::ffi::c_void;

use crate::color::Color;
use crate::error::{Error, Result};
use crate::framebuffer::{Filter, Framebuffer};
use crate::geom::Rect;
use crate::presenter::{PresentationOptions, ScaleMode};

type HWND = isize;
type HDC = isize;
type HBITMAP = isize;
type HBRUSH = isize;
type HGDIOBJ = isize;
type BOOL = i32;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct BITMAPINFOHEADER {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

#[repr(C)]
struct BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER,
    bmiColors: [u32; 3],
}

const BI_BITFIELDS: u32 = 3;
const DIB_RGB_COLORS: u32 = 0;
const SRCCOPY: u32 = 0x00CC0020;
const COLORONCOLOR: i32 = 3;
const HALFTONE: i32 = 4;

#[link(name = "gdi32")]
unsafe extern "system" {
    fn StretchDIBits(
        hdc: HDC,
        xDest: i32,
        yDest: i32,
        DestWidth: i32,
        DestHeight: i32,
        xSrc: i32,
        ySrc: i32,
        SrcWidth: i32,
        SrcHeight: i32,
        lpBits: *const c_void,
        lpbmi: *const BITMAPINFO,
        iUsage: u32,
        rop: u32,
    ) -> i32;
    fn SetStretchBltMode(hdc: HDC, mode: i32) -> i32;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn CreateDIBSection(
        hdc: HDC,
        pbmi: *const BITMAPINFO,
        usage: u32,
        ppvBits: *mut *mut c_void,
        hSection: isize,
        offset: u32,
    ) -> HBITMAP;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn BitBlt(
        hdc: HDC,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        hdcSrc: HDC,
        x1: i32,
        y1: i32,
        rop: u32,
    ) -> BOOL;
    fn CreateSolidBrush(color: u32) -> HBRUSH;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetDC(hwnd: HWND) -> HDC;
    fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32;
    fn GetClientRect(hwnd: HWND, lpRect: *mut RECT) -> BOOL;
    fn FillRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
}

pub struct PlatformPresenter {
    hwnd: HWND,
    last_client_size: (i32, i32),
}

unsafe impl Send for PlatformPresenter {}
unsafe impl Sync for PlatformPresenter {}

impl PlatformPresenter {
    pub fn from_hwnd(hwnd: isize) -> Self {
        Self {
            hwnd,
            last_client_size: (0, 0),
        }
    }

    #[cfg(feature = "raw-window-handle")]
    pub fn new<W: raw_window_handle::HasWindowHandle>(handle: &W) -> Result<Self> {
        let win = handle
            .window_handle()
            .map_err(|_| Error::InvalidHandle)?;
        let hwnd = match win.as_raw() {
            raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get(),
            _ => return Err(Error::InvalidHandle),
        };
        Ok(Self::from_hwnd(hwnd))
    }

    /// Blit the framebuffer with the requested scaling and filtering options.
    pub fn present_with_options(&mut self, fb: &Framebuffer, options: &PresentationOptions) -> Result<()> {
        let fb_w = fb.width() as i32;
        let fb_h = fb.height() as i32;
        if fb_w <= 0 || fb_h <= 0 {
            return Ok(());
        }

        unsafe {
            let hdc = GetDC(self.hwnd);
            if hdc == 0 {
                return Err(Error::Platform("Win32 GetDC failed".into()));
            }

            let mut client_rect: RECT = std::mem::zeroed();
            GetClientRect(self.hwnd, &mut client_rect);
            let win_w = client_rect.right - client_rect.left;
            let win_h = client_rect.bottom - client_rect.top;

            // Compute destination blit bounds according to ScaleMode
            let (dst_x, dst_y, dst_w, dst_h) = calculate_dst_rect(fb_w, fb_h, win_w, win_h, options.scale_mode);

            // Clear borders if letterboxed / pillarboxed
            if dst_w < win_w || dst_h < win_h {
                clear_borders(hdc, win_w, win_h, dst_x, dst_y, dst_w, dst_h, options.background_color);
            }

            // Set stretch filtering mode
            let blt_mode = if options.filter == Filter::Bilinear {
                HALFTONE
            } else {
                COLORONCOLOR
            };
            SetStretchBltMode(hdc, blt_mode);

            let bmi = make_bitmap_info(fb_w, fb_h);

            let res = StretchDIBits(
                hdc,
                dst_x,
                dst_y,
                dst_w,
                dst_h,
                0,
                0,
                fb_w,
                fb_h,
                fb.pixels().as_ptr() as *const c_void,
                &bmi,
                DIB_RGB_COLORS,
                SRCCOPY,
            );

            ReleaseDC(self.hwnd, hdc);

            if res == 0 {
                return Err(Error::Platform("Win32 StretchDIBits failed".into()));
            }
        }

        Ok(())
    }

    /// Presents only the specified dirty / damage rectangles.
    pub fn present_rects(
        &mut self,
        fb: &Framebuffer,
        rects: &[Rect],
        options: &PresentationOptions,
    ) -> Result<()> {
        if rects.is_empty() {
            return Ok(());
        }

        // If stretched or scaled, fallback to full present to avoid seam artifacts
        if options.scale_mode != ScaleMode::Center && options.scale_mode != ScaleMode::Stretch {
            return self.present_with_options(fb, options);
        }

        let fb_w = fb.width() as i32;
        let fb_h = fb.height() as i32;

        unsafe {
            let hdc = GetDC(self.hwnd);
            if hdc == 0 {
                return Err(Error::Platform("Win32 GetDC failed".into()));
            }

            let mut client_rect: RECT = std::mem::zeroed();
            GetClientRect(self.hwnd, &mut client_rect);
            let win_w = client_rect.right - client_rect.left;
            let win_h = client_rect.bottom - client_rect.top;

            let (dst_x0, dst_y0, dst_w, dst_h) = calculate_dst_rect(fb_w, fb_h, win_w, win_h, options.scale_mode);
            let scale_x = dst_w as f32 / fb_w as f32;
            let scale_y = dst_h as f32 / fb_h as f32;

            let bmi = make_bitmap_info(fb_w, fb_h);

            for r in rects {
                let dx = dst_x0 + (r.x as f32 * scale_x).round() as i32;
                let dy = dst_y0 + (r.y as f32 * scale_y).round() as i32;
                let dw = (r.width as f32 * scale_x).round() as i32;
                let dh = (r.height as f32 * scale_y).round() as i32;

                // In top-down DIBs, ySrc is measured from top
                StretchDIBits(
                    hdc,
                    dx,
                    dy,
                    dw,
                    dh,
                    r.x,
                    fb_h - (r.y + r.height as i32), // inverted because StretchDIBits reads from bottom if height is positive
                    r.width as i32,
                    r.height as i32,
                    fb.pixels().as_ptr() as *const c_void,
                    &bmi,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }

            ReleaseDC(self.hwnd, hdc);
        }

        Ok(())
    }
}

fn calculate_dst_rect(
    fb_w: i32,
    fb_h: i32,
    win_w: i32,
    win_h: i32,
    mode: ScaleMode,
) -> (i32, i32, i32, i32) {
    if win_w <= 0 || win_h <= 0 {
        return (0, 0, fb_w, fb_h);
    }

    match mode {
        ScaleMode::Stretch => (0, 0, win_w, win_h),
        ScaleMode::Fit => {
            let aspect_fb = fb_w as f32 / fb_h as f32;
            let aspect_win = win_w as f32 / win_h as f32;

            let (dw, dh) = if aspect_win > aspect_fb {
                ((win_h as f32 * aspect_fb).round() as i32, win_h)
            } else {
                (win_w, (win_w as f32 / aspect_fb).round() as i32)
            };
            let dx = (win_w - dw) / 2;
            let dy = (win_h - dh) / 2;
            (dx, dy, dw, dh)
        }
        ScaleMode::IntegerScale => {
            let scale = (win_w / fb_w).min(win_h / fb_h).max(1);
            let dw = fb_w * scale;
            let dh = fb_h * scale;
            let dx = (win_w - dw) / 2;
            let dy = (win_h - dh) / 2;
            (dx, dy, dw, dh)
        }
        ScaleMode::Center => {
            let dx = (win_w - fb_w) / 2;
            let dy = (win_h - fb_h) / 2;
            (dx, dy, fb_w, fb_h)
        }
    }
}

unsafe fn clear_borders(
    hdc: HDC,
    win_w: i32,
    win_h: i32,
    dst_x: i32,
    dst_y: i32,
    dst_w: i32,
    dst_h: i32,
    bg: Color,
) {
    // Win32 GDI COLORREF is 0x00BBGGRR
    let colorref = (bg.r as u32) | ((bg.g as u32) << 8) | ((bg.b as u32) << 16);
    let brush = unsafe { CreateSolidBrush(colorref) };
    if brush == 0 {
        return;
    }

    // Top bar
    if dst_y > 0 {
        let r = RECT {
            left: 0,
            top: 0,
            right: win_w,
            bottom: dst_y,
        };
        unsafe { FillRect(hdc, &r, brush) };
    }
    // Bottom bar
    let dst_bottom = dst_y + dst_h;
    if dst_bottom < win_h {
        let r = RECT {
            left: 0,
            top: dst_bottom,
            right: win_w,
            bottom: win_h,
        };
        unsafe { FillRect(hdc, &r, brush) };
    }
    // Left bar
    if dst_x > 0 {
        let r = RECT {
            left: 0,
            top: dst_y,
            right: dst_x,
            bottom: dst_bottom,
        };
        unsafe { FillRect(hdc, &r, brush) };
    }
    // Right bar
    let dst_right = dst_x + dst_w;
    if dst_right < win_w {
        let r = RECT {
            left: dst_right,
            top: dst_y,
            right: win_w,
            bottom: dst_bottom,
        };
        unsafe { FillRect(hdc, &r, brush) };
    }

    unsafe { DeleteObject(brush) };
}

fn make_bitmap_info(width: i32, height: i32) -> BITMAPINFO {
    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height; // negative indicates top-down row order
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_BITFIELDS;
    bmi.bmiColors[0] = 0x00FF0000; // Red mask
    bmi.bmiColors[1] = 0x0000FF00; // Green mask
    bmi.bmiColors[2] = 0x000000FF; // Blue mask
    bmi
}
