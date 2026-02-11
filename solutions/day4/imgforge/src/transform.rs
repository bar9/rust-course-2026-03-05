use crate::error::Result;

/// An image transformation operation.
#[derive(Debug, Clone)]
pub enum Operation {
    Resize { width: u32, height: u32 },
    Grayscale,
    Blur { sigma: f32 },
}

/// Backend-agnostic trait for image transforms.
///
/// Implementations receive raw image bytes and return processed bytes.
pub trait Transform: Send + Sync {
    fn apply(&self, input: &[u8], operation: &Operation) -> Result<Vec<u8>>;

    fn name(&self) -> &str;
}

/// Validate that requested dimensions are within reasonable bounds.
pub(crate) fn validate_dimensions(width: u32, height: u32) -> Result<()> {
    const MAX_DIMENSION: u32 = 16384;
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(crate::error::Error::DimensionTooLarge { width, height });
    }
    Ok(())
}

/// Select the best available transform backend.
///
/// Uses TurboJPEG if the feature is enabled, otherwise falls back to the
/// pure-Rust `image` crate backend.
pub fn default_backend() -> Box<dyn Transform> {
    #[cfg(feature = "turbojpeg")]
    {
        Box::new(crate::transform_turbojpeg::TurboJpegBackend)
    }
    #[cfg(not(feature = "turbojpeg"))]
    {
        Box::new(crate::transform_imagers::ImageRsBackend)
    }
}
