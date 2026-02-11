use crate::error::Result;
use crate::transform::{Operation, Transform, validate_dimensions};
use image::ImageFormat;
use std::io::Cursor;

/// Pure-Rust image processing backend using the `image` crate.
pub struct ImageRsBackend;

impl Transform for ImageRsBackend {
    fn apply(&self, input: &[u8], operation: &Operation) -> Result<Vec<u8>> {
        let format = image::guess_format(input)?;
        let img = image::load_from_memory(input)?;

        let processed = match operation {
            Operation::Resize { width, height } => {
                validate_dimensions(*width, *height)?;
                img.resize_exact(*width, *height, image::imageops::FilterType::Lanczos3)
            }
            Operation::Grayscale => img.grayscale(),
            Operation::Blur { sigma } => img.blur(*sigma),
        };

        let mut output = Vec::new();
        let output_format = match format {
            ImageFormat::Jpeg => ImageFormat::Jpeg,
            ImageFormat::Png => ImageFormat::Png,
            _ => ImageFormat::Png, // default fallback
        };
        processed.write_to(&mut Cursor::new(&mut output), output_format)?;

        Ok(output)
    }

    fn name(&self) -> &str {
        "image-rs"
    }
}
