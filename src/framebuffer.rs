//! The core software framebuffer containing pixel buffers, 2D drawing primitives,
//! text rendering, clipping, and image filters.

#![allow(clippy::too_many_arguments)]

use crate::color::Color;
use crate::font::{get_glyph_8x8, FONT_HEIGHT, FONT_WIDTH};
use crate::geom::Rect;

/// Filtering mode used when scaling images during blit operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    /// Nearest-neighbor sampling (crisp pixel art).
    #[default]
    Nearest,
    /// Bilinear interpolation (smooth gradient transitions).
    Bilinear,
}

/// A 2D software pixel canvas storing packed 32-bit ARGB pixels.
#[derive(Clone)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u32>, // 0xAARRGGBB
    clip_stack: Vec<Rect>,
}

impl Framebuffer {
    /// Creates a new black-initialized framebuffer with dimensions `width` × `height`.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::BLACK.to_argb(); (width * height) as usize],
            clip_stack: Vec::new(),
        }
    }

    /// Creates a new framebuffer filled with the specified background `color`.
    pub fn filled(width: u32, height: u32, color: Color) -> Self {
        let argb = color.to_argb();
        Self {
            width,
            height,
            pixels: vec![argb; (width * height) as usize],
            clip_stack: Vec::new(),
        }
    }

    /// Width of the framebuffer in pixels.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the framebuffer in pixels.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Dimensions of the framebuffer as `(width, height)`.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Bounding rectangle covering `(0, 0, width, height)`.
    #[inline]
    pub fn bounds(&self) -> Rect {
        Rect::from_size(self.width, self.height)
    }

    /// Total number of pixels in the framebuffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    /// Returns `true` if the framebuffer has zero width or height.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    /// Borrow the underlying pixel buffer as an immutable slice of packed `0xAARRGGBB` `u32`s.
    #[inline]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Borrow the underlying pixel buffer as a mutable slice of packed `0xAARRGGBB` `u32`s.
    #[inline]
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    /// Borrow the raw byte representation of the pixels.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.pixels.as_ptr() as *const u8, self.pixels.len() * 4)
        }
    }

    /// Resizes the framebuffer, preserving existing pixel content where it fits.
    pub fn resize(&mut self, new_width: u32, new_height: u32, fill: Color) {
        if self.width == new_width && self.height == new_height {
            return;
        }

        let mut new_pixels = vec![fill.to_argb(); (new_width * new_height) as usize];
        let copy_w = self.width.min(new_width) as usize;
        let copy_h = self.height.min(new_height) as usize;

        for y in 0..copy_h {
            let src_idx = y * self.width as usize;
            let dst_idx = y * new_width as usize;
            new_pixels[dst_idx..dst_idx + copy_w]
                .copy_from_slice(&self.pixels[src_idx..src_idx + copy_w]);
        }

        self.width = new_width;
        self.height = new_height;
        self.pixels = new_pixels;
        self.clip_stack.clear();
    }

    // ── Clipping ─────────────────────────────────────────────────────────────

    /// Pushes a new clipping rectangle onto the clipping stack.
    ///
    /// The active clip rectangle becomes the intersection of all rects on the stack.
    pub fn push_clip_rect(&mut self, rect: Rect) {
        let current = self.clip_rect();
        let clamped = rect.clamp_to(current).unwrap_or(Rect::new(0, 0, 0, 0));
        self.clip_stack.push(clamped);
    }

    /// Pops the topmost clipping rectangle from the stack.
    pub fn pop_clip_rect(&mut self) -> Option<Rect> {
        self.clip_stack.pop()
    }

    /// Clears all clipping rectangles, restoring clipping to the full framebuffer boundaries.
    pub fn reset_clip(&mut self) {
        self.clip_stack.clear();
    }

    /// Returns the currently active clipping rectangle.
    #[inline]
    pub fn clip_rect(&self) -> Rect {
        self.clip_stack
            .last()
            .copied()
            .unwrap_or_else(|| self.bounds())
    }

    /// Returns `true` if `(x, y)` is within both canvas bounds and the active clip rectangle.
    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        self.clip_rect().contains(x, y)
    }

    // ── Pixel Manipulation ───────────────────────────────────────────────────

    /// Sets the pixel at `(x, y)` without any bounds or clip checking.
    ///
    /// # Safety
    /// Calling this with `x >= width` or `y >= height` invokes undefined behavior.
    #[inline]
    pub fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: Color) {
        unsafe {
            *self.pixels.get_unchecked_mut((y * self.width + x) as usize) = color.to_argb();
        }
    }

    /// Sets the pixel at `(x, y)` to `color` if it falls within the active clip bounds.
    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if self.in_bounds(x, y) {
            self.pixels[(y as u32 * self.width + x as u32) as usize] = color.to_argb();
        }
    }

    /// Retrieves the color of the pixel at `(x, y)`, or `Color::TRANSPARENT` if out of bounds.
    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Color {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            Color::from_argb(self.pixels[(y as u32 * self.width + x as u32) as usize])
        } else {
            Color::TRANSPARENT
        }
    }

    /// Blends `src` over the pixel at `(x, y)` using Porter-Duff alpha blending.
    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, src: Color) {
        if self.in_bounds(x, y) {
            let idx = (y as u32 * self.width + x as u32) as usize;
            let dst = Color::from_argb(self.pixels[idx]);
            self.pixels[idx] = src.blend_over(dst).to_argb();
        }
    }

    /// Clears the entire framebuffer to `color` (ignoring clip rect).
    pub fn clear(&mut self, color: Color) {
        let argb = color.to_argb();
        self.pixels.fill(argb);
    }

    /// Clears only the active clipping area (or a specified rectangle) to `color`.
    pub fn clear_rect(&mut self, rect: Rect, color: Color) {
        if let Some(c) = rect.clamp_to(self.clip_rect()) {
            let argb = color.to_argb();
            for y in c.y..c.bottom() {
                let start = (y as u32 * self.width + c.x as u32) as usize;
                let end = start + c.width as usize;
                self.pixels[start..end].fill(argb);
            }
        }
    }

    // ── Lines & Rectangles ───────────────────────────────────────────────────

    /// Draws a horizontal line from `x0` to `x1` at `y` with clipping.
    pub fn draw_hline(&mut self, x0: i32, x1: i32, y: i32, color: Color) {
        let clip = self.clip_rect();
        if y < clip.y || y >= clip.bottom() {
            return;
        }

        let xa = x0.min(x1).max(clip.x);
        let xb = x0.max(x1).min(clip.right() - 1);
        if xa > xb {
            return;
        }

        let argb = color.to_argb();
        let row_start = (y as u32 * self.width + xa as u32) as usize;
        let count = (xb - xa + 1) as usize;
        self.pixels[row_start..row_start + count].fill(argb);
    }

    /// Draws a vertical line from `y0` to `y1` at `x` with clipping.
    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {
        let clip = self.clip_rect();
        if x < clip.x || x >= clip.right() {
            return;
        }

        let ya = y0.min(y1).max(clip.y);
        let yb = y0.max(y1).min(clip.bottom() - 1);
        if ya > yb {
            return;
        }

        let argb = color.to_argb();
        for y in ya..=yb {
            self.pixels[(y as u32 * self.width + x as u32) as usize] = argb;
        }
    }

    /// Draws an aliased line from `(x0, y0)` to `(x1, y1)` using Bresenham's algorithm.
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            self.set_pixel(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draws a line with the specified pixel thickness.
    pub fn draw_line_thick(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        thickness: i32,
        color: Color,
    ) {
        if thickness <= 1 {
            self.draw_line(x0, y0, x1, y1, color);
            return;
        }

        let dx = (x1 - x0) as f32;
        let dy = (y1 - y0) as f32;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-4 {
            self.fill_circle(x0, y0, thickness / 2, color);
            return;
        }

        // Compute normal vector
        let nx = -dy / len;
        let ny = dx / len;
        let half = thickness as f32 / 2.0;

        let p0 = (x0 as f32 + nx * half, y0 as f32 + ny * half);
        let p1 = (x1 as f32 + nx * half, y1 as f32 + ny * half);
        let p2 = (x1 as f32 - nx * half, y1 as f32 - ny * half);
        let p3 = (x0 as f32 - nx * half, y0 as f32 - ny * half);

        self.fill_polygon(
            &[
                (p0.0.round() as i32, p0.1.round() as i32),
                (p1.0.round() as i32, p1.1.round() as i32),
                (p2.0.round() as i32, p2.1.round() as i32),
                (p3.0.round() as i32, p3.1.round() as i32),
            ],
            color,
        );

        // Add rounded caps
        self.fill_circle(x0, y0, (half - 0.5).max(1.0) as i32, color);
        self.fill_circle(x1, y1, (half - 0.5).max(1.0) as i32, color);
    }

    /// Draws an anti-aliased line using Xiaolin Wu's algorithm.
    pub fn draw_line_aa(&mut self, mut x0: f32, mut y0: f32, mut x1: f32, mut y1: f32, color: Color) {
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let gradient = if dx.abs() < 1e-6 { 1.0 } else { dy / dx };

        // Handle first endpoint
        let xend = x0.round();
        let yend = y0 + gradient * (xend - x0);
        let xgap = 1.0 - frac(x0 + 0.5);
        let xpxl1 = xend as i32;
        let ypxl1 = yend.floor() as i32;

        self.plot_aa(xpxl1, ypxl1, steep, color, (1.0 - frac(yend)) * xgap);
        self.plot_aa(xpxl1, ypxl1 + 1, steep, color, frac(yend) * xgap);

        let mut intery = yend + gradient;

        // Handle second endpoint
        let xend2 = x1.round();
        let yend2 = y1 + gradient * (xend2 - x1);
        let xgap2 = frac(x1 + 0.5);
        let xpxl2 = xend2 as i32;
        let ypxl2 = yend2.floor() as i32;

        self.plot_aa(xpxl2, ypxl2, steep, color, (1.0 - frac(yend2)) * xgap2);
        self.plot_aa(xpxl2, ypxl2 + 1, steep, color, frac(yend2) * xgap2);

        // Main loop
        for x in (xpxl1 + 1)..xpxl2 {
            let y = intery.floor() as i32;
            self.plot_aa(x, y, steep, color, 1.0 - frac(intery));
            self.plot_aa(x, y + 1, steep, color, frac(intery));
            intery += gradient;
        }
    }

    #[inline]
    fn plot_aa(&mut self, x: i32, y: i32, steep: bool, color: Color, brightness: f32) {
        let b = brightness.clamp(0.0, 1.0);
        let alpha = ((color.a as f32) * b).round() as u8;
        if alpha == 0 {
            return;
        }
        let c = color.with_alpha(alpha);
        if steep {
            self.blend_pixel(y, x, c);
        } else {
            self.blend_pixel(x, y, c);
        }
    }

    /// Draws an unfilled rectangle.
    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        let x1 = x + w - 1;
        let y1 = y + h - 1;
        self.draw_hline(x, x1, y, color);
        self.draw_hline(x, x1, y1, color);
        self.draw_vline(x, y, y1, color);
        self.draw_vline(x1, y, y1, color);
    }

    /// Fills a solid rectangle with clipping.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        let rect = Rect::new(x, y, w as u32, h as u32);
        self.clear_rect(rect, color);
    }

    /// Draws an unfilled rounded rectangle with border radius `r`.
    pub fn draw_rounded_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        let radius = r.min(w / 2).min(h / 2).max(0);
        if radius == 0 {
            self.draw_rect(x, y, w, h, color);
            return;
        }

        // Horizontal edges
        self.draw_hline(x + radius, x + w - 1 - radius, y, color);
        self.draw_hline(x + radius, x + w - 1 - radius, y + h - 1, color);
        // Vertical edges
        self.draw_vline(x, y + radius, y + h - 1 - radius, color);
        self.draw_vline(x + w - 1, y + radius, y + h - 1 - radius, color);

        // Four corner arcs
        self.draw_circle_quadrant(x + radius, y + radius, radius, 1, color);
        self.draw_circle_quadrant(x + w - 1 - radius, y + radius, radius, 2, color);
        self.draw_circle_quadrant(x + w - 1 - radius, y + h - 1 - radius, radius, 3, color);
        self.draw_circle_quadrant(x + radius, y + h - 1 - radius, radius, 4, color);
    }

    /// Fills a solid rounded rectangle with border radius `r`.
    pub fn fill_rounded_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        let radius = r.min(w / 2).min(h / 2).max(0);
        if radius == 0 {
            self.fill_rect(x, y, w, h, color);
            return;
        }

        // Center rectangular block
        self.fill_rect(x + radius, y, w - 2 * radius, h, color);
        // Left and right side blocks
        self.fill_rect(x, y + radius, radius, h - 2 * radius, color);
        self.fill_rect(x + w - radius, y + radius, radius, h - 2 * radius, color);

        // Four corner disc quadrants
        self.fill_circle_quadrant(x + radius, y + radius, radius, 1, color);
        self.fill_circle_quadrant(x + w - 1 - radius, y + radius, radius, 2, color);
        self.fill_circle_quadrant(x + w - 1 - radius, y + h - 1 - radius, radius, 3, color);
        self.fill_circle_quadrant(x + radius, y + h - 1 - radius, radius, 4, color);
    }

    // ── Circles & Ellipses ───────────────────────────────────────────────────

    /// Draws an unfilled circle outline.
    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        if r <= 0 {
            self.set_pixel(cx, cy, color);
            return;
        }

        let mut x = r;
        let mut y = 0;
        let mut p = 1 - r;

        while x >= y {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx + x, cy - y, color);
            self.set_pixel(cx - x, cy - y, color);
            self.set_pixel(cx + y, cy + x, color);
            self.set_pixel(cx - y, cy + x, color);
            self.set_pixel(cx + y, cy - x, color);
            self.set_pixel(cx - y, cy - x, color);

            y += 1;
            if p <= 0 {
                p += 2 * y + 1;
            } else {
                x -= 1;
                p += 2 * (y - x) + 1;
            }
        }
    }

    /// Fills a solid circle.
    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        if r <= 0 {
            self.set_pixel(cx, cy, color);
            return;
        }

        let mut x = r;
        let mut y = 0;
        let mut p = 1 - r;

        while x >= y {
            self.draw_hline(cx - x, cx + x, cy + y, color);
            self.draw_hline(cx - x, cx + x, cy - y, color);
            self.draw_hline(cx - y, cx + y, cy + x, color);
            self.draw_hline(cx - y, cx + y, cy - x, color);

            y += 1;
            if p <= 0 {
                p += 2 * y + 1;
            } else {
                x -= 1;
                p += 2 * (y - x) + 1;
            }
        }
    }

    /// Draws a circle with specified border thickness.
    pub fn draw_circle_thick(&mut self, cx: i32, cy: i32, r: i32, thickness: i32, color: Color) {
        let half = thickness / 2;
        for dr in -half..=(thickness - half) {
            let cur_r = r + dr;
            if cur_r >= 0 {
                self.draw_circle(cx, cy, cur_r, color);
            }
        }
    }

    /// Draws an unfilled ellipse outline.
    pub fn draw_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: Color) {
        if rx <= 0 || ry <= 0 {
            return;
        }

        let rx64 = rx as i64;
        let ry64 = ry as i64;
        let mut x = 0i32;
        let mut y = ry;
        let mut d1 = (ry64 * ry64) - (rx64 * rx64 * ry64) + ((rx64 * rx64) / 4);
        let mut dx = 2 * (ry64 * ry64) * x as i64;
        let mut dy = 2 * (rx64 * rx64) * y as i64;

        // Region 1
        while dx < dy {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx + x, cy - y, color);
            self.set_pixel(cx - x, cy - y, color);

            x += 1;
            dx += 2 * (ry64 * ry64);
            if d1 < 0 {
                d1 += dx + (ry64 * ry64);
            } else {
                y -= 1;
                dy -= 2 * (rx64 * rx64);
                d1 += dx - dy + (ry64 * ry64);
            }
        }

        // Region 2
        let mut d2 = (ry64 * ry64) * (2 * x as i64 + 1) * (2 * x as i64 + 1) / 4
            + (rx64 * rx64) * (y as i64 - 1) * (y as i64 - 1)
            - (rx64 * rx64 * ry64 * ry64);

        while y >= 0 {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx + x, cy - y, color);
            self.set_pixel(cx - x, cy - y, color);

            y -= 1;
            dy -= 2 * (rx64 * rx64);
            if d2 > 0 {
                d2 += (rx64 * rx64) - dy;
            } else {
                x += 1;
                dx += 2 * (ry64 * ry64);
                d2 += dx - dy + (rx64 * rx64);
            }
        }
    }

    /// Fills a solid ellipse.
    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, color: Color) {
        if rx <= 0 || ry <= 0 {
            return;
        }

        let rx64 = rx as i64;
        let ry64 = ry as i64;
        let mut x = 0i32;
        let mut y = ry;
        let mut d1 = (ry64 * ry64) - (rx64 * rx64 * ry64) + ((rx64 * rx64) / 4);
        let mut dx = 2 * (ry64 * ry64) * x as i64;
        let mut dy = 2 * (rx64 * rx64) * y as i64;

        while dx < dy {
            self.draw_hline(cx - x, cx + x, cy + y, color);
            self.draw_hline(cx - x, cx + x, cy - y, color);

            x += 1;
            dx += 2 * (ry64 * ry64);
            if d1 < 0 {
                d1 += dx + (ry64 * ry64);
            } else {
                y -= 1;
                dy -= 2 * (rx64 * rx64);
                d1 += dx - dy + (ry64 * ry64);
            }
        }

        let mut d2 = (ry64 * ry64) * (2 * x as i64 + 1) * (2 * x as i64 + 1) / 4
            + (rx64 * rx64) * (y as i64 - 1) * (y as i64 - 1)
            - (rx64 * rx64 * ry64 * ry64);

        while y >= 0 {
            self.draw_hline(cx - x, cx + x, cy + y, color);
            self.draw_hline(cx - x, cx + x, cy - y, color);

            y -= 1;
            dy -= 2 * (rx64 * rx64);
            if d2 > 0 {
                d2 += (rx64 * rx64) - dy;
            } else {
                x += 1;
                dx += 2 * (ry64 * ry64);
                d2 += dx - dy + (rx64 * rx64);
            }
        }
    }

    fn draw_circle_quadrant(&mut self, cx: i32, cy: i32, r: i32, quad: u8, color: Color) {
        let mut x = r;
        let mut y = 0;
        let mut p = 1 - r;

        while x >= y {
            match quad {
                1 => {
                    // Top-left
                    self.set_pixel(cx - x, cy - y, color);
                    self.set_pixel(cx - y, cy - x, color);
                }
                2 => {
                    // Top-right
                    self.set_pixel(cx + x, cy - y, color);
                    self.set_pixel(cx + y, cy - x, color);
                }
                3 => {
                    // Bottom-right
                    self.set_pixel(cx + x, cy + y, color);
                    self.set_pixel(cx + y, cy + x, color);
                }
                4 => {
                    // Bottom-left
                    self.set_pixel(cx - x, cy + y, color);
                    self.set_pixel(cx - y, cy + x, color);
                }
                _ => {}
            }

            y += 1;
            if p <= 0 {
                p += 2 * y + 1;
            } else {
                x -= 1;
                p += 2 * (y - x) + 1;
            }
        }
    }

    fn fill_circle_quadrant(&mut self, cx: i32, cy: i32, r: i32, quad: u8, color: Color) {
        let mut x = r;
        let mut y = 0;
        let mut p = 1 - r;

        while x >= y {
            match quad {
                1 => {
                    // Top-left
                    self.draw_hline(cx - x, cx, cy - y, color);
                    self.draw_hline(cx - y, cx, cy - x, color);
                }
                2 => {
                    // Top-right
                    self.draw_hline(cx, cx + x, cy - y, color);
                    self.draw_hline(cx, cx + y, cy - x, color);
                }
                3 => {
                    // Bottom-right
                    self.draw_hline(cx, cx + x, cy + y, color);
                    self.draw_hline(cx, cx + y, cy + x, color);
                }
                4 => {
                    // Bottom-left
                    self.draw_hline(cx - x, cx, cy + y, color);
                    self.draw_hline(cx - y, cx, cy + x, color);
                }
                _ => {}
            }

            y += 1;
            if p <= 0 {
                p += 2 * y + 1;
            } else {
                x -= 1;
                p += 2 * (y - x) + 1;
            }
        }
    }

    // ── Triangles & Polygons ─────────────────────────────────────────────────

    /// Draws an unfilled triangle.
    pub fn draw_triangle(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Color,
    ) {
        self.draw_line(x0, y0, x1, y1, color);
        self.draw_line(x1, y1, x2, y2, color);
        self.draw_line(x2, y2, x0, y0, color);
    }

    /// Fills a solid triangle using robust scanline decomposition.
    pub fn fill_triangle(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Color,
    ) {
        let (x0, y0, x1, y1, x2, y2) = sort_tri_verts(x0, y0, x1, y1, x2, y2);
        if y0 == y2 {
            return;
        }

        let total_height = (y2 - y0) as f32;

        for y in y0..=y2 {
            let second_half = y > y1 || y1 == y0;
            let segment_height = if second_half {
                (y2 - y1) as f32
            } else {
                (y1 - y0) as f32
            };

            let alpha = (y - y0) as f32 / total_height;
            let beta = if segment_height == 0.0 {
                0.0
            } else if second_half {
                (y - y1) as f32 / segment_height
            } else {
                (y - y0) as f32 / segment_height
            };

            let mut ax = x0 as f32 + (x2 - x0) as f32 * alpha;
            let mut bx = if second_half {
                x1 as f32 + (x2 - x1) as f32 * beta
            } else {
                x0 as f32 + (x1 - x0) as f32 * beta
            };

            if ax > bx {
                std::mem::swap(&mut ax, &mut bx);
            }
            self.draw_hline(ax.round() as i32, bx.round() as i32, y, color);
        }
    }

    /// Draws an unfilled polygon outline.
    pub fn draw_polygon(&mut self, points: &[(i32, i32)], color: Color) {
        if points.len() < 2 {
            return;
        }
        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            self.draw_line(p1.0, p1.1, p2.0, p2.1, color);
        }
    }

    /// Fills an arbitrary solid polygon using scanline rasterization.
    pub fn fill_polygon(&mut self, points: &[(i32, i32)], color: Color) {
        if points.len() < 3 {
            return;
        }

        let min_y = points.iter().map(|p| p.1).min().unwrap();
        let max_y = points.iter().map(|p| p.1).max().unwrap();

        let mut node_x = Vec::with_capacity(points.len());

        for y in min_y..=max_y {
            node_x.clear();
            let mut j = points.len() - 1;
            for i in 0..points.len() {
                let pi = points[i];
                let pj = points[j];

                if (pi.1 < y && pj.1 >= y) || (pj.1 < y && pi.1 >= y) {
                    let x = (pi.0 as f32
                        + (y - pi.1) as f32 / (pj.1 - pi.1) as f32 * (pj.0 - pi.0) as f32)
                        .round() as i32;
                    node_x.push(x);
                }
                j = i;
            }

            node_x.sort_unstable();

            for chunk in node_x.chunks(2) {
                if chunk.len() == 2 {
                    self.draw_hline(chunk[0], chunk[1], y, color);
                }
            }
        }
    }

    // ── Bezier Curves ────────────────────────────────────────────────────────

    /// Draws a quadratic Bezier curve `(p0 -> p1 -> p2)`.
    pub fn draw_quadratic_bezier(
        &mut self,
        p0: (i32, i32),
        p1: (i32, i32),
        p2: (i32, i32),
        color: Color,
    ) {
        let approx_len = ((p1.0 - p0.0).abs() + (p1.1 - p0.1).abs()
            + (p2.0 - p1.0).abs() + (p2.1 - p1.1).abs()) as usize;
        let steps = approx_len.clamp(10, 100);

        let mut prev = p0;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let inv = 1.0 - t;
            let x = (inv * inv * p0.0 as f32 + 2.0 * inv * t * p1.0 as f32 + t * t * p2.0 as f32).round() as i32;
            let y = (inv * inv * p0.1 as f32 + 2.0 * inv * t * p1.1 as f32 + t * t * p2.1 as f32).round() as i32;
            self.draw_line(prev.0, prev.1, x, y, color);
            prev = (x, y);
        }
    }

    /// Draws a cubic Bezier curve `(p0 -> p1 -> p2 -> p3)`.
    pub fn draw_cubic_bezier(
        &mut self,
        p0: (i32, i32),
        p1: (i32, i32),
        p2: (i32, i32),
        p3: (i32, i32),
        color: Color,
    ) {
        let approx_len = ((p1.0 - p0.0).abs() + (p1.1 - p0.1).abs()
            + (p2.0 - p1.0).abs() + (p2.1 - p1.1).abs()
            + (p3.0 - p2.0).abs() + (p3.1 - p2.1).abs()) as usize;
        let steps = approx_len.clamp(16, 120);

        let mut prev = p0;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let inv = 1.0 - t;
            let x = (inv * inv * inv * p0.0 as f32
                + 3.0 * inv * inv * t * p1.0 as f32
                + 3.0 * inv * t * t * p2.0 as f32
                + t * t * t * p3.0 as f32).round() as i32;
            let y = (inv * inv * inv * p0.1 as f32
                + 3.0 * inv * inv * t * p1.1 as f32
                + 3.0 * inv * t * t * p2.1 as f32
                + t * t * t * p3.1 as f32).round() as i32;
            self.draw_line(prev.0, prev.1, x, y, color);
            prev = (x, y);
        }
    }

    // ── Text & Font Rendering ────────────────────────────────────────────────

    /// Draws an 8×8 character glyph at `(x, y)` using the embedded bitmap font.
    pub fn draw_char(&mut self, x: i32, y: i32, ch: char, color: Color) {
        self.draw_char_scaled(x, y, ch, 1, color);
    }

    /// Draws a character glyph scaled by integer factor `scale`.
    pub fn draw_char_scaled(&mut self, x: i32, y: i32, ch: char, scale: u32, color: Color) {
        let glyph = get_glyph_8x8(ch);
        let s = scale.max(1) as i32;

        for row in 0..FONT_HEIGHT as i32 {
            let byte = glyph[row as usize];
            for col in 0..FONT_WIDTH as i32 {
                if (byte & (0x80 >> col)) != 0 {
                    if s == 1 {
                        self.set_pixel(x + col, y + row, color);
                    } else {
                        self.fill_rect(x + col * s, y + row * s, s, s, color);
                    }
                }
            }
        }
    }

    /// Draws a string of text at `(x, y)` with the embedded bitmap font.
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color) {
        self.draw_text_scaled(x, y, text, 1, color);
    }

    /// Draws a string of text scaled by integer factor `scale`.
    pub fn draw_text_scaled(&mut self, x: i32, y: i32, text: &str, scale: u32, color: Color) {
        let s = scale.max(1) as i32;
        let char_w = FONT_WIDTH as i32 * s;
        let char_h = FONT_HEIGHT as i32 * s;

        let mut cur_x = x;
        let mut cur_y = y;

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = x;
                cur_y += char_h + s;
                continue;
            }
            if ch == '\r' {
                continue;
            }
            self.draw_char_scaled(cur_x, cur_y, ch, scale, color);
            cur_x += char_w;
        }
    }

    /// Draws text with a solid background fill behind the characters.
    pub fn draw_text_with_bg(&mut self, x: i32, y: i32, text: &str, fg: Color, bg: Color) {
        let lines: Vec<&str> = text.split('\n').collect();
        let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
        let w = (max_line_len as u32 * FONT_WIDTH) as i32;
        let h = (lines.len() as u32 * (FONT_HEIGHT + 1)) as i32;

        self.fill_rect(x - 2, y - 2, w + 4, h + 4, bg);
        self.draw_text(x, y, text, fg);
    }

    /// Draws a formatted performance HUD overlay displaying instantaneous FPS,
    /// average FPS, and frame time in milliseconds.
    pub fn draw_fps_overlay(&mut self, x: i32, y: i32, stats: &crate::pacing::FrameStats) {
        let label = format!(
            "FPS: {:5.1} | AVG: {:5.1} | {:5.2} ms",
            stats.fps, stats.avg_fps, stats.frame_time_ms
        );
        self.draw_text_with_bg(x, y, &label, Color::WHITE, Color::rgba(0, 0, 0, 180));
    }

    /// Draws a real-time frame time graph / histogram (in ms) to visualize pacing and micro-stutter.
    pub fn draw_fps_graph(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        history: &std::collections::VecDeque<f32>,
        line_color: Color,
        bg_color: Color,
    ) {
        if width < 10 || height < 10 || history.is_empty() {
            return;
        }

        let w = width as i32;
        let h = height as i32;

        // Background box and border
        self.fill_rect(x, y, w, h, bg_color);
        self.draw_rect(x, y, w, h, Color::DARK_GRAY);

        // Guide lines for 16.6ms (60 FPS) and 33.3ms (30 FPS)
        let max_ms = 50.0f32;
        let y_60fps = y + h - (16.666 / max_ms * h as f32).round() as i32;
        let y_30fps = y + h - (33.333 / max_ms * h as f32).round() as i32;

        if y_60fps > y && y_60fps < y + h {
            self.draw_hline(x + 1, x + w - 2, y_60fps, Color::rgba(0, 255, 0, 70));
        }
        if y_30fps > y && y_30fps < y + h {
            self.draw_hline(x + 1, x + w - 2, y_30fps, Color::rgba(255, 255, 0, 70));
        }

        // Plot bars for recent history
        let count = history.len().min(width as usize - 2);
        let start_idx = history.len() - count;

        for (i, &ms) in history.iter().skip(start_idx).enumerate() {
            let bar_x = x + 1 + i as i32;
            let bar_h = ((ms / max_ms) * (h - 2) as f32).clamp(1.0, (h - 2) as f32).round() as i32;
            let bar_y = y + h - 1 - bar_h;

            let col = if ms > 35.0 {
                Color::RED
            } else if ms > 18.0 {
                Color::YELLOW
            } else {
                line_color
            };

            self.draw_vline(bar_x, bar_y, y + h - 2, col);
        }
    }

    // ── Gradients ────────────────────────────────────────────────────────────

    /// Fills a rectangle with a horizontal linear gradient from `left` to `right`.
    pub fn fill_rect_gradient_h(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        left: Color,
        right: Color,
    ) {
        if w <= 0 || h <= 0 {
            return;
        }
        for col in 0..w {
            let t = col as f32 / (w - 1).max(1) as f32;
            let c = left.lerp(right, t);
            self.draw_vline(x + col, y, y + h - 1, c);
        }
    }

    /// Fills a rectangle with a vertical linear gradient from `top` to `bottom`.
    pub fn fill_rect_gradient_v(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        top: Color,
        bottom: Color,
    ) {
        if w <= 0 || h <= 0 {
            return;
        }
        for row in 0..h {
            let t = row as f32 / (h - 1).max(1) as f32;
            let c = top.lerp(bottom, t);
            self.draw_hline(x, x + w - 1, y + row, c);
        }
    }

    /// Fills a circle with a radial gradient from `inner` color at center to `outer` at radius.
    pub fn fill_circle_gradient(&mut self, cx: i32, cy: i32, r: i32, inner: Color, outer: Color) {
        if r <= 0 {
            self.set_pixel(cx, cy, inner);
            return;
        }

        let rf = r as f32;
        let clip = self.clip_rect();
        let y_start = (cy - r).max(clip.y);
        let y_end = (cy + r).min(clip.bottom() - 1);

        for y in y_start..=y_end {
            let dy = (y - cy) as f32;
            let half_w = (rf * rf - dy * dy).max(0.0).sqrt() as i32;
            let x_start = (cx - half_w).max(clip.x);
            let x_end = (cx + half_w).min(clip.right() - 1);

            for x in x_start..=x_end {
                let dx = (x - cx) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                let t = (dist / rf).clamp(0.0, 1.0);
                self.set_pixel(x, y, inner.lerp(outer, t));
            }
        }
    }

    // ── Blitting & Transformations ───────────────────────────────────────────

    /// Fast copies the entire source framebuffer onto this one at `(dst_x, dst_y)`.
    pub fn blit(&mut self, src: &Framebuffer, dst_x: i32, dst_y: i32) {
        self.blit_rect(
            src,
            Rect::new(0, 0, src.width, src.height),
            dst_x,
            dst_y,
        );
    }

    /// Copies a sub-rectangle of `src` onto this framebuffer at `(dst_x, dst_y)`.
    pub fn blit_rect(&mut self, src: &Framebuffer, src_rect: Rect, dst_x: i32, dst_y: i32) {
        let clip = self.clip_rect();
        let valid_src = match src_rect.clamp_to(src.bounds()) {
            Some(r) => r,
            None => return,
        };

        let ox = dst_x - valid_src.x;
        let oy = dst_y - valid_src.y;

        let dst_rect = Rect::new(dst_x, dst_y, valid_src.width, valid_src.height);
        let clamped_dst = match dst_rect.clamp_to(clip) {
            Some(r) => r,
            None => return,
        };

        let sx0 = (clamped_dst.x - ox) as u32;
        let sy0 = (clamped_dst.y - oy) as u32;
        let w = clamped_dst.width as usize;

        for (i, dy) in (clamped_dst.y..clamped_dst.bottom()).enumerate() {
            let sy = sy0 + i as u32;
            let src_idx = (sy * src.width + sx0) as usize;
            let dst_idx = (dy as u32 * self.width + clamped_dst.x as u32) as usize;
            self.pixels[dst_idx..dst_idx + w].copy_from_slice(&src.pixels[src_idx..src_idx + w]);
        }
    }

    /// Blends the entire `src` framebuffer over this one using alpha blending.
    pub fn blit_blend(&mut self, src: &Framebuffer, dst_x: i32, dst_y: i32) {
        self.blit_blend_rect(src, Rect::new(0, 0, src.width, src.height), dst_x, dst_y);
    }

    /// Blends a sub-rectangle of `src` over this framebuffer with alpha blending.
    pub fn blit_blend_rect(&mut self, src: &Framebuffer, src_rect: Rect, dst_x: i32, dst_y: i32) {
        let clip = self.clip_rect();
        let valid_src = match src_rect.clamp_to(src.bounds()) {
            Some(r) => r,
            None => return,
        };

        let ox = dst_x - valid_src.x;
        let oy = dst_y - valid_src.y;

        let dst_rect = Rect::new(dst_x, dst_y, valid_src.width, valid_src.height);
        let clamped_dst = match dst_rect.clamp_to(clip) {
            Some(r) => r,
            None => return,
        };

        let sx0 = (clamped_dst.x - ox) as u32;
        let sy0 = (clamped_dst.y - oy) as u32;

        for (i, dy) in (clamped_dst.y..clamped_dst.bottom()).enumerate() {
            let sy = sy0 + i as u32;
            for j in 0..clamped_dst.width {
                let sx = sx0 + j;
                let dx = clamped_dst.x as u32 + j;
                let src_px = src.pixels[(sy * src.width + sx) as usize];
                let di = (dy as u32 * self.width + dx) as usize;
                let src_c = Color::from_argb(src_px);
                let dst_c = Color::from_argb(self.pixels[di]);
                self.pixels[di] = src_c.blend_over(dst_c).to_argb();
            }
        }
    }

    /// Scaled blit with either `Filter::Nearest` or `Filter::Bilinear` interpolation.
    pub fn blit_scaled(
        &mut self,
        src: &Framebuffer,
        src_rect: Rect,
        dst_rect: Rect,
        filter: Filter,
    ) {
        if src_rect.is_empty() || dst_rect.is_empty() {
            return;
        }

        let clip = self.clip_rect();
        let visible_dst = match dst_rect.clamp_to(clip) {
            Some(r) => r,
            None => return,
        };

        let x_scale = src_rect.width as f32 / dst_rect.width as f32;
        let y_scale = src_rect.height as f32 / dst_rect.height as f32;

        match filter {
            Filter::Nearest => {
                for dy in visible_dst.y..visible_dst.bottom() {
                    let sy = src_rect.y + ((dy - dst_rect.y) as f32 * y_scale) as i32;
                    let clamped_sy = sy.clamp(src_rect.y, src_rect.bottom() - 1) as u32;

                    for dx in visible_dst.x..visible_dst.right() {
                        let sx = src_rect.x + ((dx - dst_rect.x) as f32 * x_scale) as i32;
                        let clamped_sx = sx.clamp(src_rect.x, src_rect.right() - 1) as u32;
                        let px = src.pixels[(clamped_sy * src.width + clamped_sx) as usize];
                        self.pixels[(dy as u32 * self.width + dx as u32) as usize] = px;
                    }
                }
            }
            Filter::Bilinear => {
                for dy in visible_dst.y..visible_dst.bottom() {
                    let src_yf = src_rect.y as f32 + (dy - dst_rect.y) as f32 * y_scale;
                    let y0 = (src_yf.floor() as i32).clamp(src_rect.y, src_rect.bottom() - 1);
                    let y1 = (y0 + 1).min(src_rect.bottom() - 1);
                    let fy = frac(src_yf);

                    for dx in visible_dst.x..visible_dst.right() {
                        let src_xf = src_rect.x as f32 + (dx - dst_rect.x) as f32 * x_scale;
                        let x0 = (src_xf.floor() as i32).clamp(src_rect.x, src_rect.right() - 1);
                        let x1 = (x0 + 1).min(src_rect.right() - 1);
                        let fx = frac(src_xf);

                        let c00 = Color::from_argb(src.pixels[(y0 as u32 * src.width + x0 as u32) as usize]);
                        let c10 = Color::from_argb(src.pixels[(y0 as u32 * src.width + x1 as u32) as usize]);
                        let c01 = Color::from_argb(src.pixels[(y1 as u32 * src.width + x0 as u32) as usize]);
                        let c11 = Color::from_argb(src.pixels[(y1 as u32 * src.width + x1 as u32) as usize]);

                        let top = c00.lerp(c10, fx);
                        let bottom = c01.lerp(c11, fx);
                        let sample = top.lerp(bottom, fy);

                        self.pixels[(dy as u32 * self.width + dx as u32) as usize] = sample.to_argb();
                    }
                }
            }
        }
    }

    /// Blits `src` with a color tint multiplier.
    pub fn blit_tint(&mut self, src: &Framebuffer, dst_x: i32, dst_y: i32, tint: Color) {
        let clip = self.clip_rect();
        let dst_rect = Rect::new(dst_x, dst_y, src.width, src.height);
        let clamped = match dst_rect.clamp_to(clip) {
            Some(r) => r,
            None => return,
        };

        let tr = tint.r as u32;
        let tg = tint.g as u32;
        let tb = tint.b as u32;
        let ta = tint.a as u32;

        for dy in clamped.y..clamped.bottom() {
            let sy = (dy - dst_y) as u32;
            for dx in clamped.x..clamped.right() {
                let sx = (dx - dst_x) as u32;
                let p = src.pixels[(sy * src.width + sx) as usize];
                let a = (((p >> 24) & 0xFF) * ta) / 255;
                let r = (((p >> 16) & 0xFF) * tr) / 255;
                let g = (((p >> 8) & 0xFF) * tg) / 255;
                let b = ((p & 0xFF) * tb) / 255;
                self.pixels[(dy as u32 * self.width + dx as u32) as usize] =
                    (a << 24) | (r << 16) | (g << 8) | b;
            }
        }
    }

    /// Blits `src`, skipping pixels matching the specified chroma `key` color.
    pub fn blit_keyed(&mut self, src: &Framebuffer, dst_x: i32, dst_y: i32, key: Color) {
        let key_argb = key.to_argb() & 0x00FFFFFF;
        let clip = self.clip_rect();
        let dst_rect = Rect::new(dst_x, dst_y, src.width, src.height);
        let clamped = match dst_rect.clamp_to(clip) {
            Some(r) => r,
            None => return,
        };

        for dy in clamped.y..clamped.bottom() {
            let sy = (dy - dst_y) as u32;
            for dx in clamped.x..clamped.right() {
                let sx = (dx - dst_x) as u32;
                let p = src.pixels[(sy * src.width + sx) as usize];
                if (p & 0x00FFFFFF) != key_argb {
                    self.pixels[(dy as u32 * self.width + dx as u32) as usize] = p;
                }
            }
        }
    }

    /// Blits a rotated sprite centered at `(center_x, center_y)`.
    pub fn blit_rotated(&mut self, src: &Framebuffer, center_x: i32, center_y: i32, angle_rad: f32) {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        let hw = src.width as f32 / 2.0;
        let hh = src.height as f32 / 2.0;

        let max_r = (hw * hw + hh * hh).sqrt().ceil() as i32;
        let clip = self.clip_rect();

        let y_start = (center_y - max_r).max(clip.y);
        let y_end = (center_y + max_r).min(clip.bottom() - 1);

        for dy in y_start..=y_end {
            let y_rel = (dy - center_y) as f32;
            for dx in (center_x - max_r).max(clip.x)..=(center_x + max_r).min(clip.right() - 1) {
                let x_rel = (dx - center_x) as f32;
                let sx = x_rel * cos_a + y_rel * sin_a + hw;
                let sy = -x_rel * sin_a + y_rel * cos_a + hh;

                if sx >= 0.0 && sx < src.width as f32 && sy >= 0.0 && sy < src.height as f32 {
                    let p = src.pixels[(sy as u32 * src.width + sx as u32) as usize];
                    let c = Color::from_argb(p);
                    self.blend_pixel(dx, dy, c);
                }
            }
        }
    }

    // ── Image Processing & Filters ───────────────────────────────────────────

    /// Fills a contiguous enclosed region using an efficient scanline span-fill algorithm.
    pub fn flood_fill(&mut self, x: i32, y: i32, fill_color: Color) {
        if !self.in_bounds(x, y) {
            return;
        }

        let target = self.get_pixel(x, y);
        if target == fill_color {
            return;
        }

        let target_argb = target.to_argb();
        let fill_argb = fill_color.to_argb();
        let clip = self.clip_rect();

        let mut stack = Vec::with_capacity(256);
        stack.push((x, x, y, 1i32));
        stack.push((x, x, y - 1, -1i32));

        while let Some((x1, x2, y, dy)) = stack.pop() {
            if y < clip.y || y >= clip.bottom() {
                continue;
            }

            let mut cur_x = x1;
            if cur_x >= clip.x && cur_x < clip.right() {
                // Scan left
                while cur_x > clip.x
                    && self.pixels[(y as u32 * self.width + (cur_x - 1) as u32) as usize] == target_argb
                {
                    cur_x -= 1;
                    self.pixels[(y as u32 * self.width + cur_x as u32) as usize] = fill_argb;
                }
            }

            if cur_x < x1 {
                stack.push((cur_x, x1 - 1, y - dy, -dy));
            }

            let mut start_x = cur_x;
            while start_x <= x2 {
                while start_x < clip.right()
                    && self.pixels[(y as u32 * self.width + start_x as u32) as usize] == target_argb
                {
                    self.pixels[(y as u32 * self.width + start_x as u32) as usize] = fill_argb;
                    start_x += 1;
                }

                if start_x > cur_x {
                    stack.push((cur_x, start_x - 1, y + dy, dy));
                    if start_x - 1 > x2 {
                        stack.push((x2 + 1, start_x - 1, y - dy, -dy));
                    }
                }

                start_x += 1;
                while start_x <= x2
                    && start_x < clip.right()
                    && self.pixels[(y as u32 * self.width + start_x as u32) as usize] != target_argb
                {
                    start_x += 1;
                }
                cur_x = start_x;
            }
        }
    }

    /// Blurs the entire framebuffer using a fast separable box filter (radius = 1).
    pub fn blur(&mut self) {
        self.blur_radius(1);
    }

    /// Blurs the framebuffer using an O(W × H) separable two-pass horizontal/vertical box filter.
    pub fn blur_radius(&mut self, radius: u32) {
        if radius == 0 || self.width == 0 || self.height == 0 {
            return;
        }

        let r = radius as i32;
        let w = self.width as i32;
        let h = self.height as i32;
        let mut temp = vec![0u32; (self.width * self.height) as usize];

        // Horizontal Pass
        for y in 0..h {
            for x in 0..w {
                let mut acc_a = 0u32;
                let mut acc_r = 0u32;
                let mut acc_g = 0u32;
                let mut acc_b = 0u32;
                let mut count = 0u32;

                for k in -r..=r {
                    let nx = (x + k).clamp(0, w - 1);
                    let p = self.pixels[(y * w + nx) as usize];
                    acc_a += (p >> 24) & 0xFF;
                    acc_r += (p >> 16) & 0xFF;
                    acc_g += (p >> 8) & 0xFF;
                    acc_b += p & 0xFF;
                    count += 1;
                }

                temp[(y * w + x) as usize] = ((acc_a / count) << 24)
                    | ((acc_r / count) << 16)
                    | ((acc_g / count) << 8)
                    | (acc_b / count);
            }
        }

        // Vertical Pass
        for x in 0..w {
            for y in 0..h {
                let mut acc_a = 0u32;
                let mut acc_r = 0u32;
                let mut acc_g = 0u32;
                let mut acc_b = 0u32;
                let mut count = 0u32;

                for k in -r..=r {
                    let ny = (y + k).clamp(0, h - 1);
                    let p = temp[(ny * w + x) as usize];
                    acc_a += (p >> 24) & 0xFF;
                    acc_r += (p >> 16) & 0xFF;
                    acc_g += (p >> 8) & 0xFF;
                    acc_b += p & 0xFF;
                    count += 1;
                }

                self.pixels[(y * w + x) as usize] = ((acc_a / count) << 24)
                    | ((acc_r / count) << 16)
                    | ((acc_g / count) << 8)
                    | (acc_b / count);
            }
        }
    }

    /// Inverts all RGB color channels while preserving alpha.
    pub fn invert(&mut self) {
        for p in &mut self.pixels {
            let a = *p & 0xFF000000;
            *p = a | (!*p & 0x00FFFFFF);
        }
    }

    /// Converts all pixels to grayscale while preserving alpha.
    pub fn grayscale(&mut self) {
        for p in &mut self.pixels {
            let r = ((*p >> 16) & 0xFF) as f32;
            let g = ((*p >> 8) & 0xFF) as f32;
            let b = (*p & 0xFF) as f32;
            let luma = (0.2126 * r + 0.7152 * g + 0.0722 * b).round() as u32;
            *p = (*p & 0xFF000000) | (luma << 16) | (luma << 8) | luma;
        }
    }

    /// Computes dirty / damage rectangles representing areas differing from `other`.
    pub fn diff_rects(&self, other: &Framebuffer) -> Vec<Rect> {
        if self.width != other.width || self.height != other.height {
            return vec![self.bounds()];
        }

        let mut diffs = Vec::new();
        let tile_size = 32u32;

        for ty in (0..self.height).step_by(tile_size as usize) {
            let th = tile_size.min(self.height - ty);
            for tx in (0..self.width).step_by(tile_size as usize) {
                let tw = tile_size.min(self.width - tx);
                let mut dirty = false;

                'check: for dy in 0..th {
                    let y = ty + dy;
                    let start = (y * self.width + tx) as usize;
                    let end = start + tw as usize;
                    if self.pixels[start..end] != other.pixels[start..end] {
                        dirty = true;
                        break 'check;
                    }
                }

                if dirty {
                    diffs.push(Rect::new(tx as i32, ty as i32, tw, th));
                }
            }
        }

        diffs
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn sort_tri_verts(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) -> (i32, i32, i32, i32, i32, i32) {
    let mut verts = [(x0, y0), (x1, y1), (x2, y2)];
    verts.sort_by_key(|v| v.1);
    (
        verts[0].0, verts[0].1, verts[1].0, verts[1].1, verts[2].0, verts[2].1,
    )
}

#[inline]
fn frac(x: f32) -> f32 {
    x - x.floor()
}
