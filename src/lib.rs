mod platform;

pub mod color;
pub mod error;
pub mod framebuffer;
pub mod presenter;

pub use color::Color;
pub use error::{Error, Result};
pub use framebuffer::Framebuffer;
pub use presenter::Presenter;
