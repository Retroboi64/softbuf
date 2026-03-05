#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub const fn transparent() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    #[inline]
    pub const fn from_rgb_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    #[inline]
    pub const fn from_rgba_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    #[inline]
    pub const fn with_alpha(mut self, a: u8) -> Self {
        self.a = a;
        self
    }
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 255, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);
    pub const YELLOW: Color = Color::rgb(255, 255, 0);
    pub const CYAN: Color = Color::rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::rgb(255, 0, 255);
    pub const ORANGE: Color = Color::rgb(255, 165, 0);
    pub const PURPLE: Color = Color::rgb(128, 0, 128);
    pub const GRAY: Color = Color::rgb(128, 128, 128);
    pub const DARK_GRAY: Color = Color::rgb(64, 64, 64);
    pub const LIGHT_GRAY: Color = Color::rgb(192, 192, 192);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
}

impl Color {
    #[inline]
    pub const fn to_argb(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    #[inline]
    pub const fn to_bgra(self) -> u32 {
        ((self.a as u32) << 24) | ((self.b as u32) << 16) | ((self.g as u32) << 8) | (self.r as u32)
    }

    #[inline]
    pub const fn from_argb(v: u32) -> Self {
        Self {
            a: ((v >> 24) & 0xFF) as u8,
            r: ((v >> 16) & 0xFF) as u8,
            g: ((v >> 8) & 0xFF) as u8,
            b: (v & 0xFF) as u8,
        }
    }
}

impl Color {
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: lerp_u8(self.r, other.r, t),
            g: lerp_u8(self.g, other.g, t),
            b: lerp_u8(self.b, other.b, t),
            a: lerp_u8(self.a, other.a, t),
        }
    }

    pub fn blend_over(self, dst: Color) -> Color {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return dst;
        }

        let sa = self.a as u32;
        let da = dst.a as u32;
        let out_a = sa + da * (255 - sa) / 255;

        if out_a == 0 {
            return Color::TRANSPARENT;
        }

        Color {
            r: blend_channel(self.r, sa, dst.r, da, out_a),
            g: blend_channel(self.g, sa, dst.g, da, out_a),
            b: blend_channel(self.b, sa, dst.b, da, out_a),
            a: out_a as u8,
        }
    }

    pub fn darken(self, factor: f32) -> Color {
        let f = factor.clamp(0.0, 1.0);
        Color {
            r: (self.r as f32 * f) as u8,
            g: (self.g as f32 * f) as u8,
            b: (self.b as f32 * f) as u8,
            a: self.a,
        }
    }

    pub fn lighten(self, factor: f32) -> Color {
        self.lerp(Color::WHITE, factor)
    }

    pub fn luminance(self) -> f32 {
        0.2126 * (self.r as f32 / 255.0)
            + 0.7152 * (self.g as f32 / 255.0)
            + 0.0722 * (self.b as f32 / 255.0)
    }
}

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

#[inline]
fn blend_channel(sc: u8, sa: u32, dc: u8, da: u32, out_a: u32) -> u8 {
    ((sc as u32 * sa + dc as u32 * da * (255 - sa) / 255) / out_a) as u8
}
