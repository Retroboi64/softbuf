# softbuf

A fast, lightweight, and cross-platform software framebuffer and 2D drawing library for Rust. Designed to pair seamlessly with [`windowed`](https://github.com/Retroboi64/windowed) and standard raw window handles.

---

## Highlights

- **Zero External Dependencies on Windows & macOS**: Direct Win32 GDI and Cocoa `CALayer`/CoreGraphics blitters.
- **Rich 2D Drawing Primitives**:
  - Aliased lines (Bresenham) & anti-aliased lines (Xiaolin Wu)
  - Thick lines with rounded end-caps
  - Rectangles & rounded rectangles (unfilled & filled)
  - Circles, thick circles, and ellipses (unfilled & filled)
  - Triangles and arbitrary polygons (scanline rasterization)
  - Quadratic & cubic Bézier curves
- **Text & Bitmap Font Rendering**: Built-in 8×8 CP437 font with integer scaling and background fill.
- **Clipping Stack**: Hierarchical rectangular scissoring (`push_clip_rect` / `pop_clip_rect`) for UI views and viewports.
- **Image Operations & Blitting**: Fast blits, alpha blending, scaling (`Nearest` & `Bilinear`), color tinting, chroma-key transparency, and rotation.
- **Image Filters**: $O(W \times H)$ separable two-pass box blur, scanline flood fill, grayscale, and color inversion.
- **Presentation & Scaling**:
  - `ScaleMode::Fit`: Scales maintaining aspect ratio with automatic letterboxing / pillarboxing.
  - `ScaleMode::IntegerScale`: 1×, 2×, 3× integer scaling for crisp retro pixel art.
  - `ScaleMode::Stretch`: Fills window client area.
  - `ScaleMode::Center`: 1:1 unscaled centered in window.
- **Damage Tracking (Dirty Rectangles)**: `present_rects` and `present_diff` to minimize compositor bandwidth.

---

## Platform Support Matrix

| Platform | Backend | External Dependencies | Presentation Modes |
|---|---|:---:|:---:|
| **Windows** (Win32) | GDI `StretchDIBits` | **0 (Zero)** | Fit, IntegerScale, Stretch, Center, Dirty Rects |
| **macOS** (Cocoa/AppKit) | CoreGraphics + `CALayer` | **0 (Zero)** | Hardware-composited `CALayer` |
| **Linux** (X11) | `x11rb` `PutImage` | Minimal (`x11rb`) | 32-bit TrueColor ZPixmap |

---

## Quick Start

Add `softbuf` to your `Cargo.toml`:

```toml
[dependencies]
softbuf = "0.2"
windowed = { version = "0.3", features = ["raw-window-handle"] }
```

### Basic Window + Framebuffer

```rust
use softbuf::{Color, Framebuffer, Presenter};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

fn main() -> windowed::Result<()> {
    let mut window = Window::new(
        WindowConfig::new("softbuf Example")
            .size(800, 600)
            .resizable(true),
    )?;

    let mut presenter = Presenter::from_window(&window).expect("presenter");
    let mut fb = Framebuffer::new(800, 600);

    // Draw some graphics
    fb.clear(Color::from_rgb_hex(0x1E1E2E));
    fb.draw_text_scaled(40, 40, "Hello from softbuf!", 2, Color::CYAN);
    fb.fill_rounded_rect(40, 90, 200, 100, 12, Color::from_rgb_hex(0x313244));
    fb.fill_circle(140, 140, 30, Color::RED);

    window.request_redraw();

    window.run(|event, window| match event {
        Event::CloseRequested
        | Event::KeyDown { key: Key::Escape, .. } => ControlFlow::Exit,

        Event::RedrawRequested => {
            presenter.present(&fb).ok();
            ControlFlow::Wait
        }

        Event::Resized { .. } => {
            window.request_redraw();
            ControlFlow::Poll
        }

        _ => ControlFlow::Wait,
    })
}
```

---

## Scaling & Presentation Modes

When a window is resized, `softbuf` automatically scales and centers the framebuffer:

```rust
use softbuf::{Presenter, PresentationOptions, ScaleMode, Filter, Color};

let mut options = PresentationOptions {
    scale_mode: ScaleMode::Fit,      // Aspect-ratio letterbox/pillarbox
    filter: Filter::Nearest,         // Nearest-neighbor sampling
    background_color: Color::BLACK,  // Border bar color
};

presenter.set_options(options);
```

Available modes:
- `ScaleMode::Fit`: Scales up/down maintaining aspect ratio, drawing solid border bars on margins.
- `ScaleMode::IntegerScale`: Perfect integer multiples (1x, 2x, 3x) for pixel art games.
- `ScaleMode::Stretch`: Fills entire window.
- `ScaleMode::Center`: 1:1 pixel rendering at window center.

---

## 2D Drawing Primitives

```rust
// Shapes
fb.draw_line(0, 0, 100, 100, Color::WHITE);
fb.draw_line_thick(10, 10, 90, 90, 4, Color::YELLOW);
fb.draw_line_aa(10.0, 50.0, 90.0, 50.0, Color::CYAN);

fb.draw_rect(20, 20, 100, 60, Color::BLUE);
fb.fill_rect(20, 20, 100, 60, Color::DARK_GRAY);
fb.draw_rounded_rect(140, 20, 100, 60, 8, Color::GREEN);
fb.fill_rounded_rect(140, 20, 100, 60, 8, Color::MAGENTA);

fb.draw_circle(200, 200, 40, Color::WHITE);
fb.fill_circle(200, 200, 40, Color::RED);
fb.draw_ellipse(320, 200, 50, 30, Color::YELLOW);
fb.fill_ellipse(320, 200, 50, 30, Color::PURPLE);

fb.fill_triangle(100, 50, 150, 20, 180, 70, Color::CYAN);
fb.draw_cubic_bezier((10, 100), (40, 20), (80, 180), (120, 100), Color::ORANGE);

// Gradients
fb.fill_rect_gradient_h(0, 0, 800, 40, Color::BLUE, Color::PURPLE);
fb.fill_circle_gradient(400, 300, 50, Color::YELLOW, Color::RED);

// Text & Bitmap Fonts
fb.draw_text(20, 20, "Crisp 8x8 font", Color::WHITE);
fb.draw_text_scaled(20, 40, "Scaled Text", 2, Color::CYAN);
fb.draw_text_with_bg(20, 80, "Text with background", Color::BLACK, Color::WHITE);

// Clipping Stack
fb.push_clip_rect(Rect::new(50, 50, 200, 200));
// Everything drawn here is clipped to [50, 50, 200, 200]
fb.pop_clip_rect();

// Image Filters
fb.blur(); // Fast O(W x H) separable box blur
fb.flood_fill(100, 100, Color::GREEN); // Scanline flood fill
fb.grayscale();
fb.invert();
```

---

## Running the Examples

```sh
# Simple text and window
cargo run --example simple

# Bouncing balls animation
cargo run --example animate

# 2D shapes showcase
cargo run --example shapes

# Interactive text, clipping, gradients, and scaling modes
cargo run --example text_and_ui
```

---

## License

MIT License &copy; 2025-2026 Patrick Reese (Retroboi64).
