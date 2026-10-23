//! Color representations, color spaces, and Porter-Duff alpha blending.

/// A 32-bit RGBA color representation (8 bits per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Creates an opaque color with red, green, and blue components (`alpha = 255`).
    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Creates a color with red, green, blue, and alpha components.
    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a fully transparent color `(0, 0, 0, 0)`.
    #[inline]
    pub const fn transparent() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    /// Creates an opaque color from a 24-bit RGB hex literal `0xRRGGBB`.
    #[inline]
    pub const fn from_rgb_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    /// Creates a color from a 32-bit RGBA hex literal `0xRRGGBBAA`.
    #[inline]
    pub const fn from_rgba_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    /// Returns a copy of this color with the alpha channel replaced.
    #[inline]
    pub const fn with_alpha(mut self, a: u8) -> Self {
        self.a = a;
        self
    }
}

// ── Standard Color Palette ───────────────────────────────────────────────────
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

// ── Conversions ──────────────────────────────────────────────────────────────
impl Color {
    /// Encodes as packed 32-bit integer `0xAARRGGBB` (native format for Windows DIB and softbuf).
    #[inline]
    pub const fn to_argb(self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.r as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.b as u32)
    }

    /// Encodes as packed 32-bit integer `0xAABBGGRR`.
    #[inline]
    pub const fn to_abgr(self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.b as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.r as u32)
    }

    /// Encodes as packed 32-bit integer `0xRRGGBBAA`.
    #[inline]
    pub const fn to_rgba(self) -> u32 {
        ((self.r as u32) << 24)
            | ((self.g as u32) << 16)
            | ((self.b as u32) << 8)
            | (self.a as u32)
    }

    /// Encodes as packed 32-bit integer `0xBBGGRRAA`.
    #[inline]
    pub const fn to_bgra(self) -> u32 {
        ((self.b as u32) << 24)
            | ((self.g as u32) << 16)
            | ((self.r as u32) << 8)
            | (self.a as u32)
    }

    /// Decodes from packed 32-bit `0xAARRGGBB`.
    #[inline]
    pub const fn from_argb(v: u32) -> Self {
        Self {
            a: ((v >> 24) & 0xFF) as u8,
            r: ((v >> 16) & 0xFF) as u8,
            g: ((v >> 8) & 0xFF) as u8,
            b: (v & 0xFF) as u8,
        }
    }

    /// Decodes from packed 32-bit `0xRRGGBBAA`.
    #[inline]
    pub const fn from_rgba(v: u32) -> Self {
        Self {
            r: ((v >> 24) & 0xFF) as u8,
            g: ((v >> 16) & 0xFF) as u8,
            b: ((v >> 8) & 0xFF) as u8,
            a: (v & 0xFF) as u8,
        }
    }

    /// Returns the bytes in RGBA memory order `[r, g, b, a]`.
    #[inline]
    pub const fn to_bytes(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

// ── Blending and Manipulation ────────────────────────────────────────────────
impl Color {
    /// Linearly interpolates between `self` and `other` by parameter `t` in `[0.0, 1.0]`.
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: lerp_u8(self.r, other.r, t),
            g: lerp_u8(self.g, other.g, t),
            b: lerp_u8(self.b, other.b, t),
            a: lerp_u8(self.a, other.a, t),
        }
    }

    /// Performs standard Porter-Duff Source Over alpha blending over destination color `dst`.
    pub fn blend_over(self, dst: Color) -> Color {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return dst;
        }
        if dst.a == 0 {
            return self;
        }

        let sa = self.a as u32;
        let da = dst.a as u32;
        let inv_sa = 255 - sa;

        // Resulting alpha channel: sa + da * (255 - sa) / 255
        let out_a = sa + (da * inv_sa + 127) / 255;
        if out_a == 0 {
            return Color::TRANSPARENT;
        }

        Color {
            r: blend_channel(self.r, sa, dst.r, da, inv_sa, out_a),
            g: blend_channel(self.g, sa, dst.g, da, inv_sa, out_a),
            b: blend_channel(self.b, sa, dst.b, da, inv_sa, out_a),
            a: out_a.min(255) as u8,
        }
    }

    /// Multiplies the RGB components by `factor` in `[0.0, 1.0]`.
    pub fn darken(self, factor: f32) -> Color {
        let f = factor.clamp(0.0, 1.0);
        Color {
            r: (self.r as f32 * f).round() as u8,
            g: (self.g as f32 * f).round() as u8,
            b: (self.b as f32 * f).round() as u8,
            a: self.a,
        }
    }

    /// Brightens the color toward white by `factor` in `[0.0, 1.0]`.
    pub fn lighten(self, factor: f32) -> Color {
        self.lerp(Color::WHITE, factor)
    }

    /// Computes the ITU-R BT.709 relative luminance of the color in `[0.0, 1.0]`.
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
fn blend_channel(sc: u8, sa: u32, dc: u8, da: u32, inv_sa: u32, out_a: u32) -> u8 {
    let num = (sc as u32 * sa) + ((dc as u32 * da * inv_sa + 127) / 255);
    (num / out_a).min(255) as u8
}
