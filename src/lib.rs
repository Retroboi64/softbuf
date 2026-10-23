//! # softbuf
//!
//! A high-performance, cross-platform software framebuffer, 2D drawing, and frame-pacing
//! library designed to pair seamlessly with [`windowed`](https://github.com/Retroboi64/windowed)
//! (v0.4.x+) and standard raw window handles.
//!
//! ## Features
//! - **Zero Dependencies on Windows and macOS**: Pure Win32 GDI and Cocoa/CoreGraphics blitters.
//! - **High-Precision Frame Pacing**: Sub-millisecond pacing for 30/60/120/144 FPS, uncapped, and dynamic ranging framerates.
//! - **Rich 2D Drawing Primitives**: Lines, thick lines, anti-aliased lines (Xiaolin Wu),
//!   rectangles, rounded rectangles, circles, ellipses, triangles, polygons, and Bezier curves.
//! - **Text & Bitmap Font Rendering**: Built-in 8×8 bitmap font with character scaling and background fill.
//! - **Clipping Stack**: Nested hierarchical rectangular clipping for UI and viewports.
//! - **Image Operations**: Blitting, alpha blending, scaling (`Nearest`, `Bilinear`), tints, chroma keys, rotations.
//! - **Image Filters**: Fast O(W × H) separable box blur, scanline flood fill, color inversion, and grayscale.
//! - **Presentation & Scaling**: Aspect-fit letterboxing, integer scaling, stretching, and damage rect tracking.

mod platform;

pub mod color;
pub mod error;
pub mod font;
pub mod framebuffer;
pub mod geom;
pub mod pacing;
pub mod presenter;

pub use color::Color;
pub use error::{Error, Result};
pub use font::{get_glyph_8x8, FONT_HEIGHT, FONT_WIDTH};
pub use framebuffer::{Filter, Framebuffer};
pub use geom::Rect;
pub use pacing::{FramePacer, FrameStats, Framerate};
pub use presenter::{PresentationOptions, Presenter, ScaleMode};
