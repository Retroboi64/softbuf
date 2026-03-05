use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Platform(String),
    SizeMismatch { fb: (u32, u32), window: (u32, u32) },
    InvalidHandle,
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
        }
    }
}

impl std::error::Error for Error {}
