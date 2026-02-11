#![cfg(feature = "turbojpeg")]

use crate::error::{Error, Result};
use crate::transform::{Operation, Transform, validate_dimensions};

/// TurboJPEG-accelerated backend for JPEG images.
///
/// Falls back to the ImageRs backend for non-JPEG formats.
pub struct TurboJpegBackend;

impl Transform for TurboJpegBackend {
    fn apply(&self, input: &[u8], operation: &Operation) -> Result<Vec<u8>> {
        // Only handle JPEG; for other formats, delegate to ImageRs
        let is_jpeg = input.starts_with(&[0xFF, 0xD8]);
        if !is_jpeg {
            return crate::transform_imagers::ImageRsBackend.apply(input, operation);
        }

        match operation {
            Operation::Resize { width, height } => {
                validate_dimensions(*width, *height)?;
                resize_jpeg(input, *width, *height)
            }
            // TurboJPEG doesn't support grayscale/blur natively; fall back
            _ => crate::transform_imagers::ImageRsBackend.apply(input, operation),
        }
    }

    fn name(&self) -> &str {
        "turbojpeg"
    }
}

fn resize_jpeg(input: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    use turbojpeg::{Compressor, Decompressor, Image, PixelFormat};

    let mut decompressor = Decompressor::new().map_err(|e| Error::InvalidOperation {
        message: format!("TurboJPEG decompressor init: {e}"),
    })?;

    let header = decompressor
        .read_header(input)
        .map_err(|e| Error::InvalidOperation {
            message: format!("TurboJPEG read header: {e}"),
        })?;

    let mut pixels = vec![0u8; header.width * header.height * 3];
    let image = Image {
        pixels: pixels.as_mut_slice(),
        width: header.width,
        pitch: header.width * 3,
        height: header.height,
        format: PixelFormat::RGB,
    };

    decompressor
        .decompress(input, image)
        .map_err(|e| Error::InvalidOperation {
            message: format!("TurboJPEG decompress: {e}"),
        })?;

    // Use image crate for the actual resize, then re-encode with turbojpeg
    let img = image::RgbImage::from_raw(header.width as u32, header.height as u32, pixels)
        .ok_or_else(|| Error::InvalidOperation {
            message: "Failed to create image from decompressed data".to_string(),
        })?;

    let resized =
        image::imageops::resize(&img, width, height, image::imageops::FilterType::Lanczos3);

    let out_image = Image {
        pixels: resized.as_raw().as_slice(),
        width: width as usize,
        pitch: width as usize * 3,
        height: height as usize,
        format: PixelFormat::RGB,
    };

    let mut compressor = Compressor::new().map_err(|e| Error::InvalidOperation {
        message: format!("TurboJPEG compressor init: {e}"),
    })?;
    compressor.set_quality(85);

    let compressed = compressor
        .compress_to_vec(out_image)
        .map_err(|e| Error::InvalidOperation {
            message: format!("TurboJPEG compress: {e}"),
        })?;

    Ok(compressed)
}
