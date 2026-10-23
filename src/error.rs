//! Error types for softbuf.

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Platform presentation failed with description.
    Platform(String),
    /// Framebuffer size does not match required dimensions.
    SizeMismatch { fb: (u32, u32), window: (u32, u32) },
    /// The window or display handle provided was invalid or unsupported.
    InvalidHandle,
    /// An I/O error occurred.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Platform(msg) => write!(f, "Platform error: {msg}"),
            Error::SizeMismatch { fb, window } => write!(
                f,
                "Size mismatch: framebuffer is {}×{} but window is {}×{}",
                fb.0, fb.1, window.0, window.1
            ),
            Error::InvalidHandle => write!(f, "Invalid raw window handle"),
            Error::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
