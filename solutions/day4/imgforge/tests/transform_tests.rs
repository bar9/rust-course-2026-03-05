use imgforge::transform_imagers::ImageRsBackend;
use imgforge::{Operation, Transform};

fn test_png() -> Vec<u8> {
    // Create a minimal 2x2 red PNG in memory
    let mut img = image::RgbImage::new(2, 2);
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255, 0, 0]);
    }
    let mut buf = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )
    .unwrap();
    buf
}

#[test]
fn test_grayscale() {
    let backend = ImageRsBackend;
    let input = test_png();
    let result = backend.apply(&input, &Operation::Grayscale).unwrap();
    assert!(!result.is_empty());
    // Output should be valid PNG
    let img = image::load_from_memory(&result).unwrap();
    assert_eq!(img.width(), 2);
    assert_eq!(img.height(), 2);
}

#[test]
fn test_resize() {
    let backend = ImageRsBackend;
    let input = test_png();
    let op = Operation::Resize {
        width: 4,
        height: 4,
    };
    let result = backend.apply(&input, &op).unwrap();
    let img = image::load_from_memory(&result).unwrap();
    assert_eq!(img.width(), 4);
    assert_eq!(img.height(), 4);
}

#[test]
fn test_blur() {
    let backend = ImageRsBackend;
    let input = test_png();
    let result = backend.apply(&input, &Operation::Blur { sigma: 1.0 }).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_dimension_too_large() {
    let backend = ImageRsBackend;
    let input = test_png();
    let op = Operation::Resize {
        width: 20000,
        height: 20000,
    };
    let result = backend.apply(&input, &op);
    assert!(result.is_err());
}

#[test]
fn test_backend_name() {
    let backend = ImageRsBackend;
    assert_eq!(backend.name(), "image-rs");
}
