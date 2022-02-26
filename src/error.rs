use thiserror::Error;

/// An error that can occur in the library.
#[derive(Debug, Error)]
pub enum Error {
    #[error("no suitable GPU adapter")]
    NoSuitableAdapter,
    #[error("cannot create logical GPU device")]
    FailedDeviceCreation(#[from] wgpu::RequestDeviceError),
    #[error("invalid image; unable to open")]
    InvalidImage(#[from] image::ImageError),
    #[error("failed to acquire next swapchain image")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("invalid font")]
    InvalidFont(#[from] wgpu_glyph::ab_glyph::InvalidFont),
    #[error("i/o error")]
    Io(#[from] std::io::Error),
    #[error("error drawing text: {0}")]
    TextDraw(String),
}

pub type Result<T> = ::core::result::Result<T, Error>;
