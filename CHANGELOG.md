# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-10-22

### Added
- **`Presenter::from_window`**: Native first-class integration with `windowed::Window`.
- **macOS Cocoa/AppKit Presenter**: Zero-dependency blitter using direct Objective-C runtime calls and CoreGraphics `CALayer`.
- **Windows Zero-Dependency Presenter**: Replaced legacy `winapi` dependency with direct Win32 GDI FFI (`StretchDIBits`).
- **Presentation & Scaling Modes**:
  - `ScaleMode::Fit`: Scales maintaining aspect ratio with automatic letterboxing / pillarboxing.
  - `ScaleMode::IntegerScale`: Integer factor scaling (1×, 2×, 3×...) for crisp retro pixel art.
  - `ScaleMode::Stretch`: Fills window client area.
  - `ScaleMode::Center`: 1:1 unscaled centered in window.
  - `Filter::Nearest` and `Filter::Bilinear` sampling options.
- **Damage Tracking (Dirty Rectangles)**: `present_rects(&[Rect])` and `present_diff(&old_fb, &new_fb)`.
- **Text & Bitmap Font Rendering**: Built-in 8×8 CP437 ASCII font with scaling (`draw_char`, `draw_char_scaled`, `draw_text`, `draw_text_scaled`, `draw_text_with_bg`).
- **Clipping Stack**: Scissoring system with `push_clip_rect`, `pop_clip_rect`, `clip_rect`, and `reset_clip`.
- **Advanced 2D Primitives**:
  - Thick lines with rounded end-caps (`draw_line_thick`).
  - Rounded rectangles (`draw_rounded_rect`, `fill_rounded_rect`).
  - Ellipses (`draw_ellipse`, `fill_ellipse`).
  - Arbitrary polygons (`draw_polygon`, `fill_polygon`) via scanline decomposition.
  - Quadratic and cubic Bézier curves (`draw_quadratic_bezier`, `draw_cubic_bezier`).
  - Gradients: horizontal, vertical, and radial circle gradients.
  - Image scaling (`blit_scaled`), color tinting (`blit_tint`), chroma-key transparency (`blit_keyed`), and rotation (`blit_rotated`).
- **Interactive Example**: Added `examples/text_and_ui.rs` showcasing fonts, clipping viewports, and runtime scale mode switching with Space bar.
- **Unit Test Suite**: Added 10 comprehensive unit tests covering primitives, clipping, math, and bug fixes (`tests/framebuffer_test.rs`).
- **Continuous Integration**: Added GitHub Actions workflow (`.github/workflows/ci.yml`) testing Linux, Windows, and macOS.
- **Comprehensive Documentation**: Added complete root `README.md` and module-level rustdoc.

### Fixed
- **`set_pixel_unchecked`**: Fixed silent no-op bug where pixel assignment modified a local variable copy rather than the buffer.
- **`fill_triangle`**: Guarded flat-edge slope calculations against division by zero and `NaN`.
- **`flood_fill`**: Replaced unbounded recursive/4-way queue with an efficient $O(H)$ scanline span-fill algorithm.
- **`draw_line_aa`**: Fixed Xiaolin Wu anti-aliased line brightness clamping `[0.0, 1.0]` and coordinate boundary safety.
- **Linux X11 Heap Allocation**: Eliminated per-frame multi-megabyte `Vec<u8>` allocation in favor of a reusable presentation buffer.
- **Bundled Examples**: Updated `simple.rs`, `animate.rs`, and `shapes.rs` to compile against `windowed 0.3.1` event loop and key event APIs.

### Changed
- Bumped version to `0.2.0`.
- Synchronized dependency with `windowed = "0.3.1"`.
- Removed unused `numix` dependency and fixed unused manifest keys in `Cargo.toml`.

---

## [0.1.1] - 2025

- Initial release with basic 2D primitives and X11/Win32 blitters.
