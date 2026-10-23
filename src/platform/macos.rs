//! Zero-dependency macOS Cocoa/AppKit software framebuffer presenter.
//!
//! Blits ARGB pixels directly to an NSView's backing CALayer using CoreGraphics and the Objective-C runtime.

#![allow(non_snake_case, dead_code)]

use std::ffi::{c_char, c_void};

use crate::error::{Error, Result};
use crate::framebuffer::Framebuffer;
use crate::geom::Rect;
use crate::presenter::PresentationOptions;

type Id = *mut c_void;
type Sel = *mut c_void;

#[link(name = "objc")]
unsafe extern "C" {
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
}

#[inline(always)]
unsafe fn msg_send_id(recv: Id, sel: Sel) -> Id {
    let f: unsafe extern "C" fn(Id, Sel) -> Id =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { f(recv, sel) }
}

#[inline(always)]
unsafe fn msg_send_bool(recv: Id, sel: Sel, arg: i8) {
    let f: unsafe extern "C" fn(Id, Sel, i8) =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { f(recv, sel, arg) };
}

#[inline(always)]
unsafe fn msg_send_ptr(recv: Id, sel: Sel, arg: *mut c_void) {
    let f: unsafe extern "C" fn(Id, Sel, *mut c_void) =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { f(recv, sel, arg) };
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGColorSpaceCreateDeviceRGB() -> *mut c_void;
    fn CGColorSpaceRelease(space: *mut c_void);
    fn CGDataProviderCreateWithData(
        info: *mut c_void,
        data: *const c_void,
        size: usize,
        release_data: *mut c_void,
    ) -> *mut c_void;
    fn CGDataProviderRelease(provider: *mut c_void);
    fn CGImageCreate(
        width: usize,
        height: usize,
        bits_per_component: usize,
        bits_per_pixel: usize,
        bytes_per_row: usize,
        space: *mut c_void,
        bitmap_info: u32,
        provider: *mut c_void,
        decode: *const f64,
        should_interpolate: bool,
        intent: u32,
    ) -> *mut c_void;
    fn CGImageRelease(image: *mut c_void);
}

// macOS CGImage bitmap info constants:
// kCGBitmapByteOrder32Little (0x2000) | kCGImageAlphaPremultipliedFirst (2) = ARGB little-endian (BGRA in memory)
const BITMAP_INFO_ARGB_LITTLE: u32 = 0x2000 | 2;

pub struct PlatformPresenter {
    ns_view: Id,
}

unsafe impl Send for PlatformPresenter {}
unsafe impl Sync for PlatformPresenter {}

impl PlatformPresenter {
    pub fn from_ns_view(ns_view: *mut c_void) -> Self {
        unsafe {
            // Enable layer backing on the view: [view setWantsLayer:YES]
            let sel_set_wants_layer = sel_registerName(b"setWantsLayer:\0".as_ptr() as *const c_char);
            msg_send_bool(ns_view, sel_set_wants_layer, 1i8);
        }
        Self { ns_view }
    }

    #[cfg(feature = "raw-window-handle")]
    pub fn new<W: raw_window_handle::HasWindowHandle>(handle: &W) -> Result<Self> {
        let win = handle
            .window_handle()
            .map_err(|_| Error::InvalidHandle)?;
        let ns_view = match win.as_raw() {
            raw_window_handle::RawWindowHandle::AppKit(h) => h.ns_view.as_ptr(),
            _ => return Err(Error::InvalidHandle),
        };
        Ok(Self::from_ns_view(ns_view))
    }

    pub fn present_with_options(
        &mut self,
        fb: &Framebuffer,
        _options: &PresentationOptions,
    ) -> Result<()> {
        let w = fb.width() as usize;
        let h = fb.height() as usize;
        if w == 0 || h == 0 {
            return Ok(());
        }

        unsafe {
            let sel_layer = sel_registerName(b"layer\0".as_ptr() as *const c_char);
            let layer: Id = msg_send_id(self.ns_view, sel_layer);
            if layer.is_null() {
                return Err(Error::Platform("NSView does not have an active layer".into()));
            }

            let cs = CGColorSpaceCreateDeviceRGB();
            let provider = CGDataProviderCreateWithData(
                std::ptr::null_mut(),
                fb.pixels().as_ptr() as *const c_void,
                fb.len() * 4,
                std::ptr::null_mut(),
            );

            let cg_img = CGImageCreate(
                w,
                h,
                8,
                32,
                w * 4,
                cs,
                BITMAP_INFO_ARGB_LITTLE,
                provider,
                std::ptr::null(),
                false,
                0,
            );

            if !cg_img.is_null() {
                let sel_set_contents = sel_registerName(b"setContents:\0".as_ptr() as *const c_char);
                msg_send_ptr(layer, sel_set_contents, cg_img);
                CGImageRelease(cg_img);
            }

            CGDataProviderRelease(provider);
            CGColorSpaceRelease(cs);
        }

        Ok(())
    }

    pub fn present_rects(
        &mut self,
        fb: &Framebuffer,
        _rects: &[Rect],
        options: &PresentationOptions,
    ) -> Result<()> {
        // macOS CALayer compositing updates full layer contents efficiently
        self.present_with_options(fb, options)
    }
}
