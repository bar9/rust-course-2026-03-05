# Chapter 21: FFI Acceleration -- TurboJPEG Feature Flag

This chapter adds an optional FFI backend to imgforge using TurboJPEG for faster JPEG processing. It applies the unsafe and FFI theory from Chapter 16 in a practical, production-style context.

## 1. Motivation

The imgforge project already works with the pure-Rust `image` crate. So why bother with FFI?

- **Performance**: TurboJPEG (libjpeg-turbo) uses hand-tuned SIMD assembly for JPEG decode/encode. On typical workloads it is 2-5x faster than the pure-Rust JPEG codec.
- **Real-world pattern**: Many production Rust projects follow this model -- a safe default backend with an optional accelerated path via a system library (e.g., `ring` vs `aws-lc-rs` for crypto).
- **Optional by design**: If TurboJPEG is not installed, the feature flag stays off and everything still compiles and runs.

The theory was covered in Chapter 16 (Unsafe Rust & FFI). Here we focus on *applying* it: wrapping an existing FFI crate, managing unsafe boundaries, and using conditional compilation to keep the dependency optional. See also Chapter 13 for feature flag fundamentals.

## 2. Feature Flag Setup

### Cargo.toml Additions

Add the `turbojpeg` crate as an optional dependency and declare a feature that enables it:

```toml
[dependencies]
turbojpeg = { version = "1", optional = true }

[features]
default = []
turbojpeg = ["dep:turbojpeg"]
```

### The `dep:` Prefix

The `dep:turbojpeg` syntax (stabilized in Rust 1.60) explicitly ties a feature to an optional dependency. Without `dep:`, Cargo implicitly creates a feature with the same name as the dependency. Using `dep:` makes the relationship explicit and avoids accidental feature/dependency name collisions:

| Syntax | Meaning |
|--------|---------|
| `turbojpeg = ["dep:turbojpeg"]` | Feature `turbojpeg` enables the optional dependency `turbojpeg` |
| `fast-jpeg = ["dep:turbojpeg"]` | Feature `fast-jpeg` enables the optional dependency `turbojpeg` |

### Building With the Feature

```bash
# Default build -- pure-Rust backend only
cargo build

# With TurboJPEG acceleration
cargo build --features turbojpeg

# Run tests with the feature enabled
cargo test --features turbojpeg

# Check that the project compiles both ways
cargo check && cargo check --features turbojpeg
```

## 3. TurboJPEG Backend -- transform_turbojpeg.rs

The entire module is conditionally compiled. When the `turbojpeg` feature is off, this file does not exist as far as the compiler is concerned.

```rust,ignore
#![cfg(feature = "turbojpeg")]

use crate::error::{Error, Result};
use crate::transform::{Operation, Transform, validate_dimensions};
use turbojpeg::{Compressor, Decompressor, Image, PixelFormat};

pub struct TurboJpegBackend;

impl Transform for TurboJpegBackend {
    fn apply(&self, input: &[u8], operation: &Operation) -> Result<Vec<u8>> {
        // JPEG files start with the magic bytes 0xFF 0xD8.
        // If the input is not JPEG, fall back to the pure-Rust backend.
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
    // Step 1: Decompress JPEG to raw pixels
    let mut decompressor = Decompressor::new()
        .map_err(|e| Error::InvalidOperation {
            message: format!("TurboJPEG decompressor init: {e}"),
        })?;

    let header = decompressor.read_header(input)
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

    decompressor.decompress(input, image)
        .map_err(|e| Error::InvalidOperation {
            message: format!("TurboJPEG decompress: {e}"),
        })?;

    // Step 2: Resize using the image crate (turbojpeg does not resize)
    let img = image::RgbImage::from_raw(
        header.width as u32,
        header.height as u32,
        pixels,
    )
    .ok_or_else(|| Error::InvalidOperation {
        message: "Failed to create image from decompressed data".into(),
    })?;

    let resized = image::imageops::resize(
        &img,
        width,
        height,
        image::imageops::FilterType::Lanczos3,
    );

    // Step 3: Compress back to JPEG using TurboJPEG (the fast path)
    let out_image = Image {
        pixels: resized.as_raw().as_slice(),
        width: width as usize,
        pitch: width as usize * 3,
        height: height as usize,
        format: PixelFormat::RGB,
    };

    let mut compressor = Compressor::new()
        .map_err(|e| Error::InvalidOperation {
            message: format!("TurboJPEG compressor init: {e}"),
        })?;
    compressor.set_quality(85);

    let compressed = compressor.compress_to_vec(out_image)
        .map_err(|e| Error::InvalidOperation {
            message: format!("TurboJPEG compress: {e}"),
        })?;

    Ok(compressed)
}
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Magic byte check at the top | TurboJPEG only handles JPEG. PNG, WebP, etc. fall through to ImageRs. |
| Decompress with TurboJPEG, resize with `image` crate | TurboJPEG provides fast decode/encode but no resize. Mixing backends is normal. |
| Compress with TurboJPEG | The encode step is where the biggest speedup occurs -- SIMD-accelerated Huffman coding. |
| All errors wrapped in `Error::InvalidOperation` | The `turbojpeg` crate returns its own error type. We convert it to our domain error at the boundary. This is the safe abstraction pattern from Chapter 16. |

### Where Is the `unsafe`?

You may notice that the code above contains no `unsafe` blocks. That is because the `turbojpeg` crate itself is a safe wrapper around the C library -- it uses `unsafe` internally but exposes a safe API. This is exactly the layered abstraction pattern taught in Chapter 16:

```text
libjpeg-turbo (C library, inherently unsafe)
    -> turbojpeg crate (unsafe FFI internally, safe Rust API externally)
        -> our TurboJpegBackend (pure safe Rust, implements our Transform trait)
```

If you were writing FFI bindings from scratch, you would use `bindgen` and write the `unsafe extern "C"` blocks yourself. Using an existing `-sys` or wrapper crate is the pragmatic choice for production code.

## 4. Backend Selection -- Dynamic Dispatch

In `transform.rs`, add a function that returns the appropriate backend at runtime:

```rust,ignore
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
```

### Why `Box<dyn Transform>`?

This is **dynamic dispatch** -- the concrete backend type is erased behind a trait object. The caller does not know (or care) whether it is talking to `ImageRsBackend` or `TurboJpegBackend`:

```rust,ignore
// In main.rs or wherever the transform is invoked
let backend = imgforge::transform::default_backend();
println!("Using backend: {}", backend.name());
let output = backend.apply(&input_bytes, &operation)?;
```

This pattern is common in Rust for plugin and backend architectures. For C++/C# developers: `Box<dyn Transform>` is similar to `std::unique_ptr<ITransform>` in C++ or `ITransform` in C#. The vtable dispatch cost is negligible compared to the image processing work.

## 5. Conditional Module Declaration

In `lib.rs`, the module is declared only when the feature is active:

```rust,ignore
mod error;
pub mod config;
pub mod transform;
pub mod transform_imagers;

#[cfg(feature = "turbojpeg")]
pub mod transform_turbojpeg;
```

When `turbojpeg` is not in the feature set, the compiler does not parse the file, produces no linker errors, and the binary size is unaffected. This is a compile-time guarantee -- the module does not exist in the compiled binary.

The `#![cfg(feature = "turbojpeg")]` at the top of the file itself is a second layer of protection -- it prevents compilation even if someone adds `mod transform_turbojpeg;` without the corresponding feature check in `lib.rs`.

## 6. Practical Concerns

### System Library Installation

TurboJPEG requires the `libjpeg-turbo` development headers on the system:

| Platform | Command |
|----------|---------|
| Ubuntu/Debian | `sudo apt install libturbojpeg0-dev` |
| Fedora/RHEL | `sudo dnf install turbojpeg-devel` |
| macOS (Homebrew) | `brew install jpeg-turbo` |
| Windows (vcpkg) | `vcpkg install libjpeg-turbo` |

If the library is not installed and you try to build with `--features turbojpeg`, the build will fail at the link step with a clear error about missing libraries. The fix is either to install the library or to build without the feature.

### The Project Works Without It

This is worth emphasizing: **the project compiles and passes all tests without TurboJPEG installed**. The feature flag is off by default. Students who cannot install system libraries (corporate machines, unusual OS configurations) are not blocked.

### CI Configuration

A production CI pipeline should test both configurations. Use a GitHub Actions matrix with `features: ["", "--features turbojpeg"]` and conditionally install `libturbojpeg0-dev` only for the turbojpeg job. This ensures neither the default path nor the accelerated path regresses.

## 7. Exercise

### Task A: Implement the TurboJPEG Resize

The `resize_jpeg` function in `transform_turbojpeg.rs` is left as a `todo!()` in the starter code. Your job:

1. Decompress the input JPEG bytes using `turbojpeg::decompress`
2. Resize using the `image` crate (TurboJPEG does not provide resize)
3. Compress the result back to JPEG using `turbojpeg::compress`
4. Wrap all errors into the `Error` type

The reference implementation is shown in Section 3 above. Try it yourself first.

### Task B: Feature-Gated Test Module

Add a test module to `transform_turbojpeg.rs` that only compiles when the feature is active:

```rust,ignore
#[cfg(test)]
#[cfg(feature = "turbojpeg")]
mod tests {
    use super::*;

    #[test]
    fn turbojpeg_backend_reports_name() {
        let backend = TurboJpegBackend;
        assert_eq!(backend.name(), "turbojpeg");
    }

    #[test]
    fn non_jpeg_falls_back_to_imagers() {
        let png_magic = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
        let backend = TurboJpegBackend;
        // Should not panic -- falls through to ImageRs backend
        let result = backend.apply(
            &png_magic,
            &Operation::Resize { width: 100, height: 100 },
        );
        // Will fail because png_magic is not a valid PNG,
        // but the point is it did NOT try to JPEG-decode it
        assert!(result.is_err());
    }
}
```

Note the double `#[cfg]`: `#[cfg(test)]` for test-only compilation and `#[cfg(feature = "turbojpeg")]` to exclude the tests when the feature is off. Both conditions must be true for the module to compile.

### Task C: Works Without the Feature

Verify that the project compiles and all non-turbojpeg tests pass without the feature:

```bash
cargo test
```

No turbojpeg-related code should compile. No linker errors. This confirms the conditional compilation is correct.

### Bonus: Backend Reporting via CLI

Add a `--backend` flag (or subcommand) that prints which backend is active:

```rust,ignore
// In your CLI handling code
if args.show_backend {
    let backend = imgforge::transform::default_backend();
    println!("Active backend: {}", backend.name());
}
```

```bash
$ imgforge --backend
Active backend: imagers

$ cargo run --features turbojpeg -- --backend
Active backend: turbojpeg
```

## Summary

This chapter applied the FFI and unsafe theory from Chapter 16 to a real project feature:

| Concept | How it was applied |
|---------|--------------------|
| Feature flags (`Cargo.toml`) | `turbojpeg` optional dependency with `dep:` syntax |
| Conditional compilation (`#[cfg]`) | Module excluded from compilation when feature is off |
| Safe abstraction over unsafe | `turbojpeg` crate wraps C library; our code is pure safe Rust |
| Trait-based dispatch (`Box<dyn T>`) | `default_backend()` returns the right implementation |
| Error conversion | FFI errors mapped to domain `Error` enum |
| Graceful degradation | Project works without the system library installed |

The key takeaway: FFI does not have to be scary or all-or-nothing. With feature flags and trait-based backends, you can offer an accelerated path while keeping the default path dependency-free and safe.
