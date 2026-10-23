//! The Presenter blits software framebuffers onto OS windows.

use crate::color::Color;
use crate::error::Result;
use crate::framebuffer::{Filter, Framebuffer};
use crate::geom::Rect;
use crate::platform::PlatformPresenter;

/// Specifies how a framebuffer is scaled when window dimensions differ from framebuffer dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    /// Stretches to fill the entire window, distorting the aspect ratio if dimensions do not match.
    Stretch,
    /// Scales maintaining aspect ratio, letterboxing or pillarboxing with solid background bars.
    #[default]
    Fit,
    /// Scales strictly by integer factors (1×, 2×, 3×...) centered in the window for crisp pixel art.
    IntegerScale,
    /// Renders 1:1 unscaled at the center of the window.
    Center,
}

/// Configuration options controlling presentation scaling, filtering, and border colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresentationOptions {
    /// The scaling mode to apply when presenting to the window.
    pub scale_mode: ScaleMode,
    /// The filtering algorithm (`Nearest` or `Bilinear`).
    pub filter: Filter,
    /// The background color used to clear letterbox/pillarbox borders.
    pub background_color: Color,
}

impl Default for PresentationOptions {
    fn default() -> Self {
        Self {
            scale_mode: ScaleMode::Fit,
            filter: Filter::Nearest,
            background_color: Color::BLACK,
        }
    }
}

/// Blits framebuffers onto OS windows with scaling, filtering, and dirty-rect acceleration.
pub struct Presenter {
    inner: PlatformPresenter,
    options: PresentationOptions,
}

impl Presenter {
    /// Creates a presenter from any window handle implementing `raw-window-handle` 0.6.
    #[cfg(feature = "raw-window-handle")]
    pub fn new<W: raw_window_handle::HasWindowHandle>(window: &W) -> Result<Self> {
        Ok(Self {
            inner: PlatformPresenter::new(window)?,
            options: PresentationOptions::default(),
        })
    }

    /// Creates a presenter directly from a `windowed::Window`.
    #[cfg(feature = "windowed")]
    pub fn from_window(window: &windowed::Window) -> Result<Self> {
        #[cfg(feature = "raw-window-handle")]
        {
            Self::new(window)
        }
        #[cfg(not(feature = "raw-window-handle"))]
        {
            // Fallback platform-specific instantiation
            #[cfg(target_os = "windows")]
            {
                use windowed::rwd::HasWindowHandle;
                let handle = window.window_handle().map_err(|_| crate::error::Error::InvalidHandle)?;
                match handle.as_raw() {
                    windowed::rwd::RawWindowHandle::Win32(h) => {
                        Ok(Self {
                            inner: PlatformPresenter::from_hwnd(h.hwnd.get()),
                            options: PresentationOptions::default(),
                        })
                    }
                    _ => Err(crate::error::Error::InvalidHandle),
                }
            }
            #[cfg(target_os = "macos")]
            {
                use windowed::rwd::HasWindowHandle;
                let handle = window.window_handle().map_err(|_| crate::error::Error::InvalidHandle)?;
                match handle.as_raw() {
                    windowed::rwd::RawWindowHandle::AppKit(h) => {
                        Ok(Self {
                            inner: PlatformPresenter::from_ns_view(h.ns_view.as_ptr()),
                            options: PresentationOptions::default(),
                        })
                    }
                    _ => Err(crate::error::Error::InvalidHandle),
                }
            }
            #[cfg(target_os = "linux")]
            {
                use windowed::rwd::HasWindowHandle;
                let handle = window.window_handle().map_err(|_| crate::error::Error::InvalidHandle)?;
                match handle.as_raw() {
                    windowed::rwd::RawWindowHandle::Xlib(h) => {
                        Ok(Self {
                            inner: PlatformPresenter::from_x11(h.window as u32)?,
                            options: PresentationOptions::default(),
                        })
                    }
                    _ => Err(crate::error::Error::InvalidHandle),
                }
            }
        }
    }

    /// Blits the framebuffer using the current presentation options.
    pub fn present(&mut self, fb: &Framebuffer) -> Result<()> {
        self.inner.present_with_options(fb, &self.options)
    }

    /// Blits the framebuffer using custom one-off presentation options.
    pub fn present_with_options(
        &mut self,
        fb: &Framebuffer,
        options: &PresentationOptions,
    ) -> Result<()> {
        self.inner.present_with_options(fb, options)
    }

    /// Blits only the specified dirty / damage rectangles.
    pub fn present_rects(&mut self, fb: &Framebuffer, rects: &[Rect]) -> Result<()> {
        self.inner.present_rects(fb, rects, &self.options)
    }

    /// Blits only the regions that differ between `current` and `previous`.
    pub fn present_diff(&mut self, current: &Framebuffer, previous: &Framebuffer) -> Result<()> {
        let diffs = current.diff_rects(previous);
        if !diffs.is_empty() {
            self.present_rects(current, &diffs)?;
        }
        Ok(())
    }

    /// Sets the active presentation options.
    pub fn set_options(&mut self, options: PresentationOptions) {
        self.options = options;
    }

    /// Gets a reference to the active presentation options.
    pub fn options(&self) -> &PresentationOptions {
        &self.options
    }

    /// Gets a mutable reference to the active presentation options.
    pub fn options_mut(&mut self) -> &mut PresentationOptions {
        &mut self.options
    }
}
