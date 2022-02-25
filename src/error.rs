use thiserror::Error;

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
}

pub type Result<T> = ::core::result::Result<T, Error>;
