//! Linux X11 presenter — converts the framebuffer to X11's expected
//! pixel format and pushes it to the window with `PutImage`.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};

use crate::error::{Error, Result};
use crate::framebuffer::Framebuffer;

pub struct PlatformPresenter {
    conn: RustConnection,
    window: u32,
    gc: u32,
    depth: u8,
    /// True if the server expects BGR byte order (rare but possible).
    bgr: bool,
}

impl PlatformPresenter {
    pub fn new<W: HasWindowHandle + HasDisplayHandle>(handle: &W) -> Result<Self> {
        // Extract X11 window id from raw handle
        let window = match handle
            .window_handle()
            .map_err(|_| Error::InvalidHandle)?
            .as_raw()
        {
            RawWindowHandle::Xlib(h) => h.window as u32,
            RawWindowHandle::Xcb(h) => h.window.get(),
            _ => return Err(Error::InvalidHandle),
        };

        let (conn, screen_num) =
            RustConnection::connect(None).map_err(|e| Error::Platform(e.to_string()))?;

        let screen = &conn.setup().roots[screen_num];
        let depth = screen.root_depth;

        // Check byte order: X11 setup reports MSBFirst or LSBFirst
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
        })
    }

    /// Blit the entire framebuffer to the window.
    ///
    /// The framebuffer size does not need to match the window — the image
    /// will be rendered at (0,0) at its native pixel dimensions.
    pub fn present(&self, fb: &Framebuffer) -> Result<()> {
        let w = fb.width() as u16;
        let h = fb.height() as u16;

        // Convert ARGB → the format PutImage expects.
        // ZPixmap with 32 bpp: on most modern X11 servers the expected
        // pixel format is 0x00RRGGBB (native u32, little-endian).
        // We need to write bytes as B G R X.
        let mut data: Vec<u8> = Vec::with_capacity(fb.len() * 4);

        for &argb in fb.pixels() {
            let a = ((argb >> 24) & 0xFF) as u8;
            let r = ((argb >> 16) & 0xFF) as u8;
            let g = ((argb >> 8) & 0xFF) as u8;
            let b = (argb & 0xFF) as u8;

            if self.bgr {
                // LSBFirst: bytes in memory order B G R A
                data.push(b);
                data.push(g);
                data.push(r);
                data.push(a);
            } else {
                // MSBFirst: bytes in memory order A R G B
                data.push(a);
                data.push(r);
                data.push(g);
                data.push(b);
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
                &data,
            )
            .map_err(|e| Error::Platform(e.to_string()))?;

        self.conn
            .flush()
            .map_err(|e| Error::Platform(e.to_string()))?;
        Ok(())
    }
}

impl Drop for PlatformPresenter {
    fn drop(&mut self) {
        let _ = self.conn.free_gc(self.gc);
        let _ = self.conn.flush();
    }
}
