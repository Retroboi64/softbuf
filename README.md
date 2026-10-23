# softbuf

A fast, lightweight, and cross-platform software framebuffer, 2D drawing, and frame-pacing library for Rust. Designed to pair seamlessly with [`windowed`](https://github.com/Retroboi64/windowed) (v0.4.x+) and standard raw window handles.

---

## Highlights

- **Zero External Dependencies on Windows & macOS**: Direct Win32 GDI and Cocoa `CALayer`/CoreGraphics blitters.
- **High-Precision Frame Pacing**:
  - `FramePacer` using hybrid OS sleep and spin-wait timing to achieve sub-millisecond precision.
  - Target framerates: 30 FPS, 60 FPS, 120 FPS, 144 FPS, uncapped, and dynamic oscillating **ranging framerates**.
  - Delta-time calculation (`delta_secs`) for framerate-independent physics and animations.
  - On-screen performance HUD (`draw_fps_overlay`) and real-time frame time graph (`draw_fps_graph`).
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
softbuf = "0.3"
windowed = { version = "0.4", features = ["raw-window-handle"] }
```

### Basic Window + Frame Pacing

```rust
use softbuf::{Color, FramePacer, Framebuffer, Presenter};
use windowed::{ControlFlow, Event, Key, Window, WindowConfig};

fn main() -> windowed::Result<()> {
    let mut window = Window::new(
        WindowConfig::new("softbuf Example")
            .size(800, 600)
            .resizable(true),
    )?;

    let mut presenter = Presenter::from_window(&window).expect("presenter");
    let mut fb = Framebuffer::new(800, 600);
    let mut pacer = FramePacer::new(60.0); // 60 FPS target

    window.request_redraw();

    window.run(|event, window| match event {
        Event::CloseRequested
        | Event::KeyDown { key: Key::Escape, .. } => ControlFlow::Exit,

        Event::RedrawRequested => {
            let stats = pacer.tick(); // paces frame and yields timing metrics

            fb.clear(Color::from_rgb_hex(0x1E1E2E));
            fb.draw_text_scaled(40, 40, "Hello from softbuf!", 2, Color::CYAN);
            fb.draw_fps_overlay(40, 80, &stats);

            presenter.present(&fb).ok();
            window.request_redraw();
            ControlFlow::Poll
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

## Frame Pacing & Ranging Framerates

```rust
use softbuf::{FramePacer, Framerate};

// 1. Target 60 FPS (standard)
let mut pacer = FramePacer::new(60.0);

// 2. High refresh rate (144 FPS)
pacer.set_target(Framerate::Fps(144.0));

// 3. Uncapped benchmarking
pacer.set_target(Framerate::Uncapped);

// 4. Dynamic ranging framerate (oscillates between 30 and 120 FPS over 4 seconds)
pacer.set_target(Framerate::Ranging {
    min_fps: 30.0,
    max_fps: 120.0,
    cycle_secs: 4.0,
});

// In render loop:
let stats = pacer.tick();
let dt = stats.delta_secs; // Use dt for physics simulation: position += velocity * dt
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
- `ScaleMode::IntegerScale`: Perfect integer multiples (1×, 2×, 3×) for pixel art games.
- `ScaleMode::Stretch`: Fills entire window.
- `ScaleMode::Center`: 1:1 pixel rendering at window center.

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

# Frame pacing, ranging framerates, and real-time jitter graph
cargo run --example framerate_pacing
```

---

## License

MIT License &copy; 2025-2026 Patrick Reese (Retroboi64).
