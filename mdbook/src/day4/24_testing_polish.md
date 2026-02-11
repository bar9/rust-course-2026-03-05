# Chapter 24: Testing & Polish -- Coverage & Batch Mode

This chapter adds comprehensive testing and a batch processing CLI mode to imgforge. We apply the testing theory from [Chapter 14](../day3/14_testing.md) to a real async web application, introducing Axum-specific testing patterns along the way.

## 1. Testing Strategy

In Chapter 14, we covered unit tests, integration tests, doc tests, and property-based testing in isolation. Now we apply those concepts to a real project. The imgforge crate has three distinct layers to test:

| Layer | What to test | Test kind |
|-------|-------------|-----------|
| **Transform operations** | Grayscale, resize, rotate produce valid output; invalid dimensions rejected | Unit tests |
| **HTTP endpoints** | Routes return correct status codes; multipart upload works; error responses are JSON | Integration tests |
| **Batch processing** | Directory traversal, skipping non-images, error counting | Unit + integration tests |

This maps naturally to the test pyramid: many fast unit tests at the base, fewer integration tests for HTTP behavior, and occasional manual end-to-end verification with a browser or `curl`.

## 2. Unit Tests -- Transform Operations

Create `tests/transform_tests.rs` at the project root. These tests exercise the `Transform` trait through its `ImageRsBackend` implementation. No HTTP server, no filesystem -- everything happens in memory.

```rust,ignore
use imgforge::transform_imagers::ImageRsBackend;
use imgforge::{Operation, Transform};

/// Generate a minimal 2x2 red PNG image entirely in memory.
/// No test fixtures on disk -- portable and fast.
fn test_png() -> Vec<u8> {
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
    let result = backend.apply(&test_png(), &Operation::Grayscale).unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_resize() {
    let backend = ImageRsBackend;
    let op = Operation::Resize {
        width: 4,
        height: 4,
    };
    let result = backend.apply(&test_png(), &op).unwrap();
    let img = image::load_from_memory(&result).unwrap();
    assert_eq!(img.width(), 4);
}

#[test]
fn test_dimension_too_large() {
    let backend = ImageRsBackend;
    let op = Operation::Resize {
        width: 20000,
        height: 20000,
    };
    assert!(backend.apply(&test_png(), &op).is_err());
}
```

### Key patterns

- **In-memory test fixtures.** The `test_png()` helper generates a valid PNG programmatically. No external files means no broken paths on CI, no git-LFS for test data, and instant execution.
- **One assertion per test.** Each test verifies a single behavior. When `test_dimension_too_large` fails, you know exactly what broke.
- **Testing through the public trait.** These tests use `Transform::apply`, not internal helper functions. This keeps tests stable when you refactor internals.

## 3. Integration Tests -- HTTP Endpoints

Testing an Axum application does not require starting a real TCP server. Axum routers implement Tower's `Service` trait, so you can send synthetic requests with `oneshot` -- a single request/response cycle without binding a port.

Create `tests/api_tests.rs`:

```rust,ignore
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::util::ServiceExt;

use imgforge::server::AppState;
use imgforge::transform;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Build a test-only AppState with default configuration.
fn test_state() -> AppState {
    AppState {
        backend: Arc::from(transform::default_backend()),
        jobs: Arc::new(Mutex::new(HashMap::new())),
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = imgforge::server::router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_index_returns_html() {
    let app = imgforge::server::router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("imgforge"));
    assert!(html.contains("<form"));
}

#[tokio::test]
async fn test_job_not_found() {
    let app = imgforge::server::router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/jobs/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
```

### How `oneshot` works

The `oneshot` method consumes the service and sends exactly one request. Under the hood, Axum's `Router` is a Tower service that processes the request through the same middleware stack and handler chain as production -- but entirely in-process, with no network I/O.

```text
Production:  Client --(TCP)--> Axum Router --> Handler
Testing:     Test   --(oneshot)--> Axum Router --> Handler
```

Both paths exercise the same router logic. The only difference is the transport layer.

### Comparison with C#/.NET

| ASP.NET Core | Axum / Tower | Notes |
|-------------|-------------|-------|
| `WebApplicationFactory<T>` | `router(state)` | Build the app in test configuration |
| `HttpClient` from factory | `oneshot(request)` | Send a request without real HTTP |
| `TestServer` | Not needed | No server process at all |
| `IServiceCollection` mocking | Pass mock `AppState` | Inject a test backend via the trait |

The Axum approach is lighter: there is no test server process, no port allocation, and no race conditions from parallel test suites binding the same port.

## 4. Test Fixtures

### Programmatic image generation (preferred)

The `test_png()` helper from Section 2 is the recommended approach. It is self-contained, requires no external files, and produces a valid image every time.

For more complex test scenarios, you can generate images of specific sizes or pixel patterns:

```rust,ignore
fn test_image(width: u32, height: u32) -> Vec<u8> {
    let img = image::RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([(x % 256) as u8, (y % 256) as u8, 128])
    });
    let mut buf = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )
    .unwrap();
    buf
}
```

### Embedded test data with `include_bytes!`

When you need a real-world image (e.g., a JPEG with EXIF metadata), embed it at compile time:

```rust,ignore
const TEST_JPEG: &[u8] = include_bytes!("fixtures/sample.jpg");

#[test]
fn test_jpeg_processing() {
    let backend = ImageRsBackend;
    let result = backend.apply(TEST_JPEG, &Operation::Grayscale).unwrap();
    assert!(!result.is_empty());
}
```

The file is baked into the test binary -- no filesystem access at runtime. The trade-off is binary size and the need to commit test data to the repository.

### Helper function pattern for AppState

Centralize test state construction so every integration test uses the same defaults:

```rust,ignore
fn test_state() -> AppState {
    AppState {
        backend: Arc::from(transform::default_backend()),
        jobs: Arc::new(Mutex::new(HashMap::new())),
    }
}
```

If your `AppState` grows (e.g., adding configuration or rate limiting), update this single function rather than fixing every test.

## 5. Coverage with cargo-llvm-cov

### Setup and basic usage

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

# Run tests and print a coverage summary
cargo llvm-cov --manifest-path solutions/day4/imgforge/Cargo.toml

# Generate an HTML report (opens in browser)
cargo llvm-cov --html

# Generate LCOV format for CI integration
cargo llvm-cov --lcov --output-path lcov.info
```

### Reading the report

```text
Filename                  Regions  Missed   Cover   Lines  Missed   Cover
-------------------------------------------------------------------------
src/transform.rs               8       1  87.50%      32       2  93.75%
src/transform_imagers.rs      12       2  83.33%      48       4  91.67%
src/server.rs                 18       6  66.67%      72      18  75.00%
src/batch.rs                  10       4  60.00%      40      12  70.00%
src/error.rs                   4       2  50.00%      16       8  50.00%
-------------------------------------------------------------------------
TOTAL                         52      15  71.15%     208      44  78.85%
```

The HTML report highlights each source file with green (covered) and red (uncovered) lines. Focus on the red lines -- they reveal untested code paths, often error branches and edge cases.

### Realistic expectations

As discussed in [Chapter 14](../day3/14_testing.md), coverage measures which lines your tests execute, not whether your logic is correct. Practical targets:

| Code category | Target | Rationale |
|--------------|--------|-----------|
| Transform logic | 90%+ | Core business logic, high value |
| HTTP handlers | 70-80% | Error paths are harder to trigger |
| CLI/batch mode | 70-80% | Filesystem edge cases vary by OS |
| Error Display impls | Not worth chasing | Boilerplate, low risk |

A sudden drop in coverage after a commit usually means new code was added without tests. Track the trend, not the absolute number.

## 6. Batch CLI Mode

Batch mode lets users process an entire directory of images from the command line, without starting the web server.

### The BatchResult struct

```rust,ignore
// src/batch.rs
use std::path::Path;
use crate::error::{Error, Result};
use crate::transform::{Operation, Transform};

#[derive(Debug)]
pub struct BatchResult {
    pub processed: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl std::fmt::Display for BatchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Batch complete: {} processed, {} skipped, {} errors",
            self.processed, self.skipped, self.errors.len()
        )?;
        for err in &self.errors {
            write!(f, "\n  - {err}")?;
        }
        Ok(())
    }
}
```

### The process_directory function

```rust,ignore
pub fn process_directory(
    backend: &dyn Transform,
    input_dir: &Path,
    output_dir: &Path,
    operation: &Operation,
) -> Result<BatchResult> {
    if !input_dir.is_dir() {
        return Err(Error::InvalidOperation {
            message: format!("Not a directory: {}", input_dir.display()),
        });
    }

    std::fs::create_dir_all(output_dir)?;

    let mut result = BatchResult {
        processed: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !matches!(ext.as_str(), "jpg" | "jpeg" | "png") {
            result.skipped += 1;
            continue;
        }

        let file_name = path.file_name().unwrap();
        let output_path = output_dir.join(file_name);

        match process_single_file(backend, &path, &output_path, operation) {
            Ok(()) => result.processed += 1,
            Err(e) => {
                result.errors.push(format!("{}: {e}", path.display()));
            }
        }
    }

    Ok(result)
}

fn process_single_file(
    backend: &dyn Transform,
    input: &Path,
    output: &Path,
    operation: &Operation,
) -> Result<()> {
    let input_data = std::fs::read(input)?;
    let output_data = backend.apply(&input_data, operation)?;
    std::fs::write(output, output_data)?;
    Ok(())
}
```

### Adding the Batch subcommand

Extend `config.rs` with a `Batch` variant in the CLI subcommand enum:

```rust,ignore
// In config.rs, add the Batch variant to the Command enum:
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Process a single image
    Cli {
        #[command(subcommand)]
        operation: CliOperation,
    },
    /// Process all images in a directory
    Batch {
        #[arg(long)]
        input_dir: PathBuf,
        #[arg(long)]
        output_dir: PathBuf,
        #[command(subcommand)]
        operation: CliOperation,
    },
    /// Start the HTTP server
    Serve {
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}
```

### Usage

```bash
# Convert all images in ./photos to grayscale
imgforge batch --input-dir ./photos --output-dir ./output grayscale

# Resize a directory of images
imgforge batch --input-dir ./uploads --output-dir ./thumbnails resize --width 200 --height 200
```

Sample output:

```text
Batch complete: 47 processed, 3 skipped, 1 errors
  - photos/corrupt.jpg: Image error: ...
```

## 7. Doc Comments and cargo doc

Rust generates HTML documentation from `///` comments on public items. Adding doc comments to imgforge's public API serves two purposes: it documents the library for consumers, and `cargo test` verifies that code examples in doc comments compile and run.

### Adding doc comments

```rust,ignore
/// A backend-agnostic image transformation operation.
///
/// # Examples
///
/// ```
/// use imgforge::Operation;
///
/// let op = Operation::Resize { width: 800, height: 600 };
/// ```
pub enum Operation {
    /// Resize the image to the given dimensions.
    Resize { width: u32, height: u32 },
    /// Convert the image to grayscale.
    Grayscale,
    /// Apply Gaussian blur with the given sigma.
    Blur { sigma: f32 },
}
```

### Generating documentation

```bash
# Build and open docs in browser
cargo doc --manifest-path solutions/day4/imgforge/Cargo.toml --open

# Include private items (useful during development)
cargo doc --document-private-items --open
```

For crate-level documentation, add a doc comment at the top of `lib.rs`:

```rust,ignore
//! # imgforge
//!
//! An image transformation toolkit with CLI and HTTP server modes.
//! Supports multiple backends via the [`Transform`] trait.
```

Alternatively, pull documentation from README.md:

```rust,ignore
#![doc = include_str!("../README.md")]
```

## 8. Exercise

This exercise has three parts. Each function body is marked with `todo!()` -- the test stubs and type signatures are provided. Run `cargo test` to verify your implementations.

### Part 1: Integration test for the transform endpoint

In `tests/api_tests.rs`, write a test that uploads an image via multipart POST and verifies the response:

```rust,ignore
#[tokio::test]
async fn test_transform_endpoint() {
    let app = imgforge::server::router(test_state());

    // Build a multipart body with a test PNG and a "grayscale" operation
    let boundary = "----test-boundary";
    let body = format!(
        "------test-boundary\r\n\
         Content-Disposition: form-data; name=\"operation\"\r\n\r\n\
         grayscale\r\n\
         ------test-boundary\r\n\
         Content-Disposition: form-data; name=\"image\"; filename=\"test.png\"\r\n\
         Content-Type: image/png\r\n\r\n"
    );
    let mut payload = body.into_bytes();
    payload.extend_from_slice(&test_png());
    payload.extend_from_slice(b"\r\n------test-boundary--\r\n");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/transform")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    todo!("Assert the response status is 200 and the body is non-empty")
}
```

### Part 2: Implement process_directory

The function signature and `BatchResult` struct are provided above. Fill in the `todo!()` in `src/batch.rs`:

```rust,ignore
pub fn process_directory(
    backend: &dyn Transform,
    input_dir: &Path,
    output_dir: &Path,
    operation: &Operation,
) -> Result<BatchResult> {
    todo!("Iterate over input_dir, process image files, skip non-images, collect errors")
}
```

### Part 3: Unit test for batch processing

Write a test that creates a temporary directory, writes test images into it, runs batch processing, and verifies the results:

```rust,ignore
#[test]
fn test_batch_processing() {
    let input_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();

    // Write two test PNGs into input_dir
    std::fs::write(input_dir.path().join("a.png"), &test_png()).unwrap();
    std::fs::write(input_dir.path().join("b.png"), &test_png()).unwrap();
    // Write a non-image file that should be skipped
    std::fs::write(input_dir.path().join("readme.txt"), b"not an image").unwrap();

    let backend = ImageRsBackend;
    let result = process_directory(
        &backend,
        input_dir.path(),
        output_dir.path(),
        &Operation::Grayscale,
    )
    .unwrap();

    todo!("Assert: 2 processed, 1 skipped, 0 errors; output files exist")
}
```

Add `tempfile` as a dev-dependency in `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3"
```

### Bonus

Run `cargo llvm-cov --html` and aim for greater than 80% line coverage across the crate. Identify uncovered branches and add tests to cover them.

## Summary

| Topic | What we did |
|-------|------------|
| Unit tests | Tested transform operations with in-memory PNG fixtures |
| Integration tests | Used Tower `oneshot` to test Axum endpoints without a real server |
| Test fixtures | Programmatic image generation; `include_bytes!` as an alternative |
| Coverage | `cargo llvm-cov` to find untested code paths; realistic targets |
| Batch mode | Directory processing with `BatchResult` reporting |
| Doc comments | `///` comments, `cargo doc`, crate-level documentation |

The testing patterns here -- trait-based backends, in-memory fixtures, `oneshot` for HTTP tests -- are not specific to imgforge. They apply to any Rust project that separates its core logic from its I/O boundaries.
