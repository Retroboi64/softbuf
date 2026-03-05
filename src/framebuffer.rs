use crate::color::Color;

pub struct Framebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u32>, // ARGB
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0u32; (width * height) as usize],
        }
    }

    pub fn filled(width: u32, height: u32, color: Color) -> Self {
        let argb = color.to_argb();
        Self {
            width,
            height,
            pixels: vec![argb; (width * height) as usize],
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    #[inline]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    #[inline]
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.pixels.as_ptr() as *const u8, self.pixels.len() * 4)
        }
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }

    #[inline]
    pub fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: Color) {
        let mut _p: u32 = unsafe { *self.pixels.get_unchecked_mut((y * self.width + x) as usize) };
        _p = color.to_argb();
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if self.in_bounds(x, y) {
            self.pixels[(y as u32 * self.width + x as u32) as usize] = color.to_argb();
        }
    }

    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Color {
        if self.in_bounds(x, y) {
            Color::from_argb(self.pixels[(y as u32 * self.width + x as u32) as usize])
        } else {
            Color::TRANSPARENT
        }
    }

    #[inline]
    pub fn blend_pixel(&mut self, x: i32, y: i32, src: Color) {
        if self.in_bounds(x, y) {
            let idx = (y as u32 * self.width + x as u32) as usize;
            let dst = Color::from_argb(self.pixels[idx]);
            self.pixels[idx] = src.blend_over(dst).to_argb();
        }
    }

    pub fn clear(&mut self, color: Color) {
        let argb = color.to_argb();
        self.pixels.iter_mut().for_each(|p| *p = argb);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color) {
        if w <= 0 || h <= 0 {
            return;
        }

        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = (x + w).min(self.width as i32).max(0) as u32;
        let y1 = (y + h).min(self.height as i32).max(0) as u32;

        let argb = color.to_argb();
        for row in y0..y1 {
            let start = (row * self.width + x0) as usize;
            let end = (row * self.width + x1) as usize;
            self.pixels[start..end].fill(argb);
        }
    }

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

    pub fn draw_hline(&mut self, x0: i32, x1: i32, y: i32, color: Color) {
        if y < 0 || y as u32 >= self.height {
            return;
        }
        let xa = x0.max(0) as u32;
        let xb = (x1 + 1).min(self.width as i32).max(0) as u32;
        if xa >= xb {
            return;
        }
        let argb = color.to_argb();
        let row_start = (y as u32 * self.width + xa) as usize;
        self.pixels[row_start..row_start + (xb - xa) as usize].fill(argb);
    }

    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {
        if x < 0 || x as u32 >= self.width {
            return;
        }
        let ya = y0.max(0) as u32;
        let yb = (y1 + 1).min(self.height as i32).max(0) as u32;
        let argb = color.to_argb();
        for y in ya..yb {
            self.pixels[(y * self.width + x as u32) as usize] = argb;
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1i32 } else { -1 };
        let sy = if y0 < y1 { 1i32 } else { -1 };

        let mut x = x0;
        let mut y = y0;
        let mut err = dx - dy;

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

    pub fn draw_line_aa(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
        let steep = (y1 - y0).abs() > (x1 - x0).abs();

        let (x0, y0, x1, y1) = if steep {
            (y0, x0, y1, x1)
        } else {
            (x0, y0, x1, y1)
        };
        let (x0, y0, x1, y1) = if x0 > x1 {
            (x1, y1, x0, y0)
        } else {
            (x0, y0, x1, y1)
        };

        let dx = x1 - x0;
        let dy = y1 - y0;
        let grad = if dx.abs() < 1e-6 { 1.0 } else { dy / dx };

        let mut y_intersect = y0 + grad;

        let x_end = x0.round();
        let y_end = y0 + grad * (x_end - x0);
        let rf = x_end - x0;
        let x_pxl1 = x_end as i32;
        let y_pxl1 = y_end as i32;
        self.plot_aa(x_pxl1, y_pxl1, steep, color, 1.0 - rf * frac(y_end));
        self.plot_aa(x_pxl1, y_pxl1 + 1, steep, color, rf * frac(y_end));

        let x_end = x1.round();
        let y_end = y1 + grad * (x_end - x1);
        let rf = x_end - x1;
        let x_pxl2 = x_end as i32;
        let y_pxl2 = y_end as i32;
        self.plot_aa(x_pxl2, y_pxl2, steep, color, 1.0 - rf * frac(y_end));
        self.plot_aa(x_pxl2, y_pxl2 + 1, steep, color, rf * frac(y_end));

        for x in (x_pxl1 + 1)..x_pxl2 {
            self.plot_aa(x, y_intersect as i32, steep, color, 1.0 - frac(y_intersect));
            self.plot_aa(x, y_intersect as i32 + 1, steep, color, frac(y_intersect));
            y_intersect += grad;
        }
    }

    fn plot_aa(&mut self, x: i32, y: i32, steep: bool, color: Color, brightness: f32) {
        let a = (color.a as f32 * brightness) as u8;
        let c = color.with_alpha(a);
        if steep {
            self.blend_pixel(y, x, c);
        } else {
            self.blend_pixel(x, y, c);
        }
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        if r <= 0 {
            self.set_pixel(cx, cy, color);
            return;
        }

        let mut x = r;
        let mut y = 0i32;
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

    pub fn fill_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color) {
        if r <= 0 {
            self.set_pixel(cx, cy, color);
            return;
        }

        let mut x = r;
        let mut y = 0i32;
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

    pub fn draw_circle_thick(&mut self, cx: i32, cy: i32, r: i32, thickness: i32, color: Color) {
        let half = thickness / 2;
        for dr in -half..=(thickness - half) {
            self.draw_circle(cx, cy, r + dr, color);
        }
    }

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

        let fill_flat_bottom = |fb: &mut Self, x0, y0, x1, y1, x2, y2| {
            let inv1 = (x1 - x0) as f32 / (y1 - y0) as f32;
            let inv2 = (x2 - x0) as f32 / (y2 - y0) as f32;
            let mut cur1 = x0 as f32;
            let mut cur2 = x0 as f32;
            for y in y0..=y1 {
                fb.draw_hline(cur1 as i32, cur2 as i32, y, color);
                cur1 += inv1;
                cur2 += inv2;
            }
        };

        let fill_flat_top = |fb: &mut Self, x0, y0, x1, y1, x2, y2| {
            let inv1 = (x2 - x0) as f32 / (y2 - y0) as f32;
            let inv2 = (x2 - x1) as f32 / (y2 - y1) as f32;
            let mut cur1 = x2 as f32;
            let mut cur2 = x2 as f32;
            for y in (y0..=y2).rev() {
                fb.draw_hline(cur1 as i32, cur2 as i32, y, color);
                cur1 -= inv1;
                cur2 -= inv2;
            }
        };

        if y1 == y2 {
            fill_flat_bottom(self, x0, y0, x1, y1, x2, y2);
        } else if y0 == y1 {
            fill_flat_top(self, x0, y0, x1, y1, x2, y2);
        } else {
            let mx = (x0 as f32 + (y1 - y0) as f32 / (y2 - y0) as f32 * (x2 - x0) as f32) as i32;
            let my = y1;
            fill_flat_bottom(self, x0, y0, x1, y1, mx, my);
            fill_flat_top(self, x1, y1, mx, my, x2, y2);
        }
    }

    pub fn blit(&mut self, src: &Framebuffer, dst_x: i32, dst_y: i32) {
        self.blit_rect(src, 0, 0, src.width as i32, src.height as i32, dst_x, dst_y);
    }

    pub fn blit_rect(
        &mut self,
        src: &Framebuffer,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        dst_x: i32,
        dst_y: i32,
    ) {
        let sx0 = src_x.max(0);
        let sy0 = src_y.max(0);
        let sx1 = (src_x + src_w).min(src.width as i32);
        let sy1 = (src_y + src_h).min(src.height as i32);
        if sx0 >= sx1 || sy0 >= sy1 {
            return;
        }

        let ox = dst_x - src_x;
        let oy = dst_y - src_y;

        let dw = self.width as i32;
        let dh = self.height as i32;

        for sy in sy0..sy1 {
            let dy = sy + oy;
            if dy < 0 || dy >= dh {
                continue;
            }

            for sx in sx0..sx1 {
                let dx = sx + ox;
                if dx < 0 || dx >= dw {
                    continue;
                }

                let src_px = src.pixels[(sy as u32 * src.width + sx as u32) as usize];
                self.pixels[(dy as u32 * self.width as u32 + dx as u32) as usize] = src_px;
            }
        }
    }

    pub fn blit_blend(&mut self, src: &Framebuffer, dst_x: i32, dst_y: i32) {
        let dw = self.width as i32;
        let dh = self.height as i32;

        for sy in 0..src.height as i32 {
            let dy = dst_y + sy;
            if dy < 0 || dy >= dh {
                continue;
            }
            for sx in 0..src.width as i32 {
                let dx = dst_x + sx;
                if dx < 0 || dx >= dw {
                    continue;
                }

                let src_c =
                    Color::from_argb(src.pixels[(sy as u32 * src.width + sx as u32) as usize]);
                let di = (dy as u32 * self.width + dx as u32) as usize;
                let dst_c = Color::from_argb(self.pixels[di]);
                self.pixels[di] = src_c.blend_over(dst_c).to_argb();
            }
        }
    }

    pub fn flood_fill(&mut self, x: i32, y: i32, fill_color: Color) {
        if !self.in_bounds(x, y) {
            return;
        }
        let target = self.get_pixel(x, y);
        if target == fill_color {
            return;
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back((x, y));

        while let Some((px, py)) = queue.pop_front() {
            if !self.in_bounds(px, py) {
                continue;
            }
            if self.get_pixel(px, py) != target {
                continue;
            }
            self.set_pixel(px, py, fill_color);
            queue.push_back((px + 1, py));
            queue.push_back((px - 1, py));
            queue.push_back((px, py + 1));
            queue.push_back((px, py - 1));
        }
    }

    pub fn blur(&mut self) {
        let w = self.width as i32;
        let h = self.height as i32;
        let src = self.pixels.clone();

        for y in 0..h {
            for x in 0..w {
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut a = 0u32;
                let mut count = 0u32;

                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx < 0 || ny < 0 || nx >= w || ny >= h {
                            continue;
                        }
                        let p = src[(ny * w + nx) as usize];
                        r += (p >> 16) & 0xFF;
                        g += (p >> 8) & 0xFF;
                        b += p & 0xFF;
                        a += (p >> 24) & 0xFF;
                        count += 1;
                    }
                }

                self.pixels[(y * w + x) as usize] =
                    ((a / count) << 24) | ((r / count) << 16) | ((g / count) << 8) | (b / count);
            }
        }
    }

    pub fn invert(&mut self) {
        for p in &mut self.pixels {
            let a = *p & 0xFF000000;
            *p = a | (!*p & 0x00FFFFFF);
        }
    }

    pub fn grayscale(&mut self) {
        for p in &mut self.pixels {
            let r = ((*p >> 16) & 0xFF) as f32;
            let g = ((*p >> 8) & 0xFF) as f32;
            let b = (*p & 0xFF) as f32;
            let luma = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u32;
            *p = (*p & 0xFF000000) | (luma << 16) | (luma << 8) | luma;
        }
    }
}

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
