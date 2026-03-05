use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::error::Result;
use crate::framebuffer::Framebuffer;
use crate::platform::PlatformPresenter;

pub struct Presenter {
    inner: PlatformPresenter,
}

impl Presenter {
    pub fn new<W: HasWindowHandle + HasDisplayHandle>(window: &W) -> Result<Self> {
        Ok(Self {
            inner: PlatformPresenter::new(window)?,
        })
    }

    pub fn present(&self, fb: &Framebuffer) -> Result<()> {
        self.inner.present(fb)
    }
}
