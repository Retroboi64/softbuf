//! Linux X11 presenter — blits the framebuffer to an X11 window with x11rb.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use crate::error::{Error, Result};
use crate::framebuffer::Framebuffer;
use crate::geom::Rect;
use crate::presenter::PresentationOptions;

pub struct PlatformPresenter {
    conn: RustConnection,
    window: u32,
    gc: u32,
    depth: u8,
    bgr: bool,
    buffer: Vec<u8>,
}

impl PlatformPresenter {
    #[cfg(feature = "raw-window-handle")]
    pub fn new<W: raw_window_handle::HasWindowHandle>(handle: &W) -> Result<Self> {
        let win = handle
            .window_handle()
            .map_err(|_| Error::InvalidHandle)?;
        let window = match win.as_raw() {
            raw_window_handle::RawWindowHandle::Xlib(h) => h.window as u32,
            raw_window_handle::RawWindowHandle::Xcb(h) => h.window.get(),
            _ => return Err(Error::InvalidHandle),
        };
        Self::from_x11(window)
    }

    pub fn from_x11(window: u32) -> Result<Self> {
        let (conn, screen_num) =
            RustConnection::connect(None).map_err(|e| Error::Platform(e.to_string()))?;

        let screen = &conn.setup().roots[screen_num];
        let depth = screen.root_depth;
        let bgr = conn.setup().image_byte_order == ImageOrder::LSB_FIRST;

        let gc = conn
            .generate_id()
            .map_err(|e| Error::Platform(e.to_string()))?;
        conn.create_gc(gc, window, &CreateGCAux::new())
            .map_err(|e| Error::Platform(e.to_string()))?;

        conn.flush().map_err(|e| Error::Platform(e.to_string()))?;

        Ok(Self {
            conn,
            window,
            gc,
            depth,
            bgr,
            buffer: Vec::new(),
        })
    }

    pub fn present_with_options(
        &mut self,
        fb: &Framebuffer,
        _options: &PresentationOptions,
    ) -> Result<()> {
        let w = fb.width() as u16;
        let h = fb.height() as u16;
        if w == 0 || h == 0 {
            return Ok(());
        }

        let needed_bytes = fb.len() * 4;
        self.buffer.clear();
        self.buffer.reserve(needed_bytes);

        for &argb in fb.pixels() {
            let a = ((argb >> 24) & 0xFF) as u8;
            let r = ((argb >> 16) & 0xFF) as u8;
            let g = ((argb >> 8) & 0xFF) as u8;
            let b = (argb & 0xFF) as u8;

            if self.bgr {
                self.buffer.push(b);
                self.buffer.push(g);
                self.buffer.push(r);
                self.buffer.push(a);
            } else {
                self.buffer.push(a);
                self.buffer.push(r);
                self.buffer.push(g);
                self.buffer.push(b);
            }
        }

        self.conn
            .put_image(
                ImageFormat::Z_PIXMAP,
                self.window,
                self.gc,
                w,
                h,
                0, // dst_x
                0, // dst_y
                0, // left_pad
                self.depth,
                &self.buffer,
            )
            .map_err(|e| Error::Platform(e.to_string()))?;

        self.conn
            .flush()
            .map_err(|e| Error::Platform(e.to_string()))?;
        Ok(())
    }

    pub fn present_rects(
        &mut self,
        fb: &Framebuffer,
        _rects: &[Rect],
        options: &PresentationOptions,
    ) -> Result<()> {
        self.present_with_options(fb, options)
    }
}

impl Drop for PlatformPresenter {
    fn drop(&mut self) {
        let _ = self.conn.free_gc(self.gc);
        let _ = self.conn.flush();
    }
}
