# Chapter 20: imgforge CLI -- Project Setup & Image Processing

## Day 4 Overview

Day 4 is all application -- the theory was covered in Day 3. You will build **imgforge**, an image processing tool that starts as a CLI and evolves into an async web server with FFI acceleration, multithreading, and testing. Each stage adds a layer without rewriting previous work.

### Roadmap

| Chapter | Stage | What you build | Key concepts applied |
|---------|-------|----------------|---------------------|
| **Ch20** | CLI foundation | `imgforge cli resize --width 800 --height 600 in.jpg out.jpg` | Project structure, error handling (Ch13, Ch19), modules (Ch12) |
| **Ch21** | FFI acceleration | Optional `turbojpeg` backend via feature flag | `unsafe`, FFI, `bindgen`, conditional compilation (Ch16) |
| **Ch22** | Async web server | `POST /transform` endpoint with Axum | Async/await, Tokio, routing, JSON (Ch18) |
| **Ch23** | Thread pool | CPU-bound work on a thread pool, job status API | Threads vs async, `Arc<Mutex<_>>`, channels (Ch18) |
| **Ch24** | Testing & polish | Unit tests, integration tests, coverage, batch CLI mode | Testing strategies, mocking, coverage (Ch14) |
| **Ch25** | Optional stretch | Desktop GUI with `egui`, `#[cfg(feature = "gui")]` | Conditional compilation at scale (Ch16) |

This chapter covers Stage 1: setting up the project, establishing idiomatic Rust project structure, implementing error handling, CLI argument parsing, and the first image transform operations.

## Project Scaffolding

Create the project:

```bash
cargo new imgforge
cd imgforge
```

Replace the generated `Cargo.toml` with the following:

```toml
[package]
name = "imgforge"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4", features = ["derive"] }
image = "0.25"
derive_more = { version = "1", features = ["from", "display"] }
```

Why these three crates:

| Crate | Purpose |
|-------|---------|
| `clap` (with `derive`) | Declarative CLI argument parsing -- define structs, get a full CLI with help text, validation, and subcommands for free |
| `image` | Pure-Rust image decoding/encoding and basic transforms -- no system libraries required |
| `derive_more` (with `from`, `display`) | Derive `From` conversions on error enums so the `?` operator auto-converts external errors into your domain error type |

## Project Structure

### File layout

```text
imgforge/
├── Cargo.toml
├── src/
│   ├── main.rs              # thin: parse args, call lib
│   ├── lib.rs               # mod declarations + re-exports
│   ├── error.rs             # Error enum + Result<T> alias
│   ├── config.rs            # CLI config (clap derive)
│   ├── transform.rs         # Transform trait + Operation enum
│   └── transform_imagers.rs # ImageRs backend implementation
└── tests/
    └── transform_tests.rs
```

### The "thin main.rs" principle

Application logic belongs in `lib.rs`, not `main.rs`. The binary entry point should do exactly two things: parse configuration and call a library function.

- **Testability** -- integration tests (`tests/*.rs`) can only access library code. If your logic lives in `main.rs`, you cannot integration-test it.
- **Reusability** -- the same library powers CLI, server, and tests without duplication.
- **Clarity** -- `main.rs` says "parse config and run." `lib.rs` says what it actually does.

### C#/.NET vs Rust project conventions

The layout will feel unusually flat compared to .NET. Rust projects start flat and nest only when a group reaches 5+ related files.

| C#/.NET habit | Rust idiom | Why |
|---------------|-----------|-----|
| One class per file, always | Group by semantic cohesion | A struct, its trait impl, and its error variants often belong in one file |
| Deep namespace hierarchy from day one (`Company.Product.Feature.SubFeature`) | Start flat, nest when a module group reaches ~5 files | Premature nesting costs longer paths and more `use` statements with no benefit |
| `Foo/` folder containing only `Foo.cs` | `foo.rs` file (modern style, not `foo/mod.rs`) | Fewer files named "mod.rs" cluttering editor tabs |
| Separate `Interfaces/`, `Models/`, `Services/` folders | Types live next to their behavior | Rust modules group by *what the code does*, not *what kind of thing it is* |
| One `FooException.cs` per exception class | One `error.rs` with an enum per architectural layer | `derive_more::From` makes a single enum with `#[from]` variants ergonomic |
| `using` directives resolve automatically | Explicit `mod` declarations + `pub use` re-exports | The module tree is explicit -- Rust's module system is the most common stumbling block for newcomers |

## Error Handling -- error.rs

Create `src/error.rs`:

```rust,ignore
use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    // -- Domain errors (constructed manually, no #[from])
    UnsupportedFormat { format: String },
    DimensionTooLarge { width: u32, height: u32 },
    InvalidOperation { message: String },

    // -- External errors (auto-converted via #[from])
    #[from]
    Io(std::io::Error),
    #[from]
    Image(image::ImageError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
```

### Domain errors vs external errors

**Domain errors** (`UnsupportedFormat`, `DimensionTooLarge`, `InvalidOperation`) represent business-rule violations. They have no `#[from]` -- you construct them explicitly at the validation site. `InvalidOperation` is a general-purpose variant for cases like invalid multipart data or unknown operations (you will use it in Ch22).

**External errors** (`Io`, `Image`) wrap errors from other crates. The `#[from]` attribute generates `From<std::io::Error> for Error`, so the `?` operator auto-converts them. Writing `std::fs::read(path)?` inside a function returning `Result<T>` automatically wraps the `io::Error` into `Error::Io`.

### Why derive_more::From

Three common approaches compared:

| Approach | Pros | Cons |
|----------|------|------|
| `derive_more::From` | Lightweight, selective `#[from]`, you control Display | You write the `Display` impl yourself |
| `thiserror` | Generates `Display` from `#[error("...")]` annotations | Heavier proc macro just for Display generation |
| `anyhow` | Minimal boilerplate, good for scripts | Erases the error type -- callers cannot match on variants |

For imgforge, `derive_more::From` is the right balance: automatic `From` conversions where needed, concrete error types for pattern matching, and Display-as-Debug is adequate for CLI output. See Ch19 for more on error handling philosophy.

## CLI Configuration -- config.rs

Create `src/config.rs`:

```rust,ignore
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "imgforge", about = "Image processing CLI & server")]
pub struct Config {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a single image operation from the command line
    Cli {
        #[command(subcommand)]
        operation: CliOperation,
    },
    /// Start the HTTP server
    Serve {
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliOperation {
    /// Resize an image to the given dimensions
    Resize {
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        input: PathBuf,
        output: PathBuf,
    },
    /// Convert an image to grayscale
    Grayscale {
        input: PathBuf,
        output: PathBuf,
    },
    /// Apply Gaussian blur
    Blur {
        #[arg(long, default_value = "3.0")]
        sigma: f32,
        input: PathBuf,
        output: PathBuf,
    },
}
```

### The clap derive API

`clap` is the standard CLI argument parsing crate. The derive API lets you define your CLI as structs and enums -- clap generates the parser, help text, and validation at compile time. Key attributes:

- `#[derive(Parser)]` -- marks the root config struct.
- `#[derive(Subcommand)]` -- turns an enum into subcommands (like `git commit`, `git push`).
- `#[arg(long)]` -- creates a `--name` flag. Without it, the field is a positional argument.
- `#[arg(default_value = "...")]` -- provides a default when the flag is omitted.
- `#[command(subcommand)]` -- nests another level of subcommands.

The `Serve` variant is a placeholder for Ch22.

### Example invocations

```bash
# Resize an image
imgforge cli resize --width 800 --height 600 input.jpg output.jpg

# Convert to grayscale
imgforge cli grayscale photo.png gray_photo.png

# Apply blur with custom sigma
imgforge cli blur --sigma 5.0 sharp.jpg blurred.jpg

# Apply blur with default sigma (3.0)
imgforge cli blur sharp.jpg blurred.jpg

# Show help
imgforge --help
imgforge cli --help
imgforge cli resize --help
```

## Transform Trait + ImageRs Backend

Create `src/transform.rs`. This module defines the operations as an enum, a trait for image processing backends, and the first concrete backend using the `image` crate.

### The Operation enum

```rust,ignore
use crate::error::{Error, Result};

/// Supported image operations.
pub enum Operation {
    Resize { width: u32, height: u32 },
    Grayscale,
    Blur { sigma: f32 },
}
```

### The Transform trait

```rust,ignore
/// A backend that can apply image operations.
///
/// The trait is object-safe (`Send + Sync`) so backends can be shared
/// across threads and used as trait objects in later chapters.
pub trait Transform: Send + Sync {
    fn apply(&self, input: &[u8], operation: &Operation) -> Result<Vec<u8>>;
    fn name(&self) -> &str;
}
```

The `input` is raw file bytes (from `std::fs::read`), the output is encoded bytes ready to write. This byte-in/byte-out interface keeps the trait testable without filesystem access.

Why a trait? In Ch21 you will add an FFI-based `TurboJpegBackend`. The trait lets you swap backends at runtime and inject mocks in tests (Ch24) -- the same trait-based dependency injection pattern from Ch14 and Ch19.

### Dimension validation

```rust,ignore
/// Validate that image dimensions are within reasonable bounds.
///
/// `pub(crate)` makes this visible to other modules in the crate
/// but not to external consumers. This is Rust's equivalent of
/// C#'s `internal` access modifier.
pub(crate) fn validate_dimensions(width: u32, height: u32) -> Result<()> {
    const MAX_DIMENSION: u32 = 16384;
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(Error::DimensionTooLarge { width, height });
    }
    Ok(())
}
```

### The ImageRsBackend -- transform_imagers.rs

The backend implementation lives in a separate file `src/transform_imagers.rs`. This separation makes it easy to add alternative backends (Ch21) without modifying the trait definition.

```rust,ignore
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
                img.resize_exact(
                    *width,
                    *height,
                    image::imageops::FilterType::Lanczos3,
                )
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
```

Two details worth noting:

- **Format preservation**: `guess_format` detects the input format, and the output uses the same format. A JPEG input produces a JPEG output, not an unexpected PNG conversion.
- **`img.blur(*sigma)`**: The `image` crate's `blur()` takes `f32` -- no cast needed since `sigma` is already `f32`.

The `?` operator works seamlessly: `guess_format` returns `Result<_, ImageError>` and `load_from_memory` returns `Result<_, ImageError>` -- both auto-converted to our `Error` type via the `#[from]` attribute.

## Wiring in lib.rs

Create `src/lib.rs`:

```rust,ignore
mod error;
pub mod config;
pub mod transform;
pub mod transform_imagers;

pub use error::{Error, Result};
pub use config::Config;
pub use transform::{Transform, Operation};

use config::{Command, CliOperation};

/// Main entry point: dispatch CLI or server mode.
pub fn run(config: Config) -> Result<()> {
    match config.command {
        Command::Cli { operation } => run_cli(operation),
        Command::Serve { port } => {
            eprintln!("Server mode on port {port} -- not yet implemented (see Ch22)");
            Ok(())
        }
    }
}

fn run_cli(operation: CliOperation) -> Result<()> {
    let backend = transform_imagers::ImageRsBackend;
    println!("Using backend: {}", backend.name());

    let (input_path, output_path, op) = match operation {
        CliOperation::Resize { width, height, input, output } => {
            (input, output, Operation::Resize { width, height })
        }
        CliOperation::Grayscale { input, output } => {
            (input, output, Operation::Grayscale)
        }
        CliOperation::Blur { sigma, input, output } => {
            (input, output, Operation::Blur { sigma })
        }
    };

    let input_bytes = std::fs::read(&input_path)?;
    let output_bytes = backend.apply(&input_bytes, &op)?;
    std::fs::write(&output_path, &output_bytes)?;

    println!(
        "Processed {} -> {}",
        input_path.display(),
        output_path.display(),
    );

    Ok(())
}
```

### Re-exports for a flat public API

The `pub use` lines let external code write `use imgforge::{Config, Transform, Result}` instead of reaching into submodules like `use imgforge::config::Config`. Note that `Command` and `CliOperation` are used within the crate (via `use config::{...}`) but are not re-exported -- external users only need `Config` to parse CLI args. Internals stay organized; the public surface stays flat.

### The thin main.rs

Replace `src/main.rs` with:

```rust,ignore
use clap::Parser;

fn main() -> imgforge::Result<()> {
    let config = imgforge::Config::parse();
    imgforge::run(config)
}
```

Two lines of logic. Everything testable lives in the library.

## Exercise

The code above gives you a working imgforge CLI. Your task is to extend it.

### Task 1: Complete the Resize case with aspect ratio preservation

The current `Resize` operation uses `resize_exact`, which distorts the image if the aspect ratio does not match. Add a new variant `ResizeFit` that preserves the aspect ratio by fitting within the given bounds. In `ImageRsBackend::apply`, use `image::DynamicImage::resize` (which preserves aspect ratio) instead of `resize_exact`.

Add this variant to the `Operation` enum:

```rust,ignore
pub enum Operation {
    Resize { width: u32, height: u32 },
    ResizeFit { width: u32, height: u32 },  // <-- add this
    Grayscale,
    Blur { sigma: f32 },
}
```

And the corresponding match arm in `apply`:

```rust,ignore
Operation::ResizeFit { width, height } => {
    validate_dimensions(*width, *height)?;
    todo!("Use img.resize() with Lanczos3 filter")
}
```

Wire it up by adding a `ResizeFit` variant to `CliOperation` in `config.rs`.

### Task 2: Add a Rotate90 operation

Add a `Rotate90` variant to `Operation` and implement it in `ImageRsBackend`. The `image` crate provides `DynamicImage::rotate90()`.

```rust,ignore
Operation::Rotate90 => {
    todo!("Call img.rotate90()")
}
```

Add the corresponding `CliOperation` variant and wire it into `run_cli`.

### Task 3: Add an error variant for invalid sigma

Blur with a negative sigma does not make sense. Add a domain error variant:

```rust,ignore
InvalidSigma { sigma: f32 },
```

Add a validation check at the start of the `Blur` match arm:

```rust,ignore
Operation::Blur { sigma } => {
    if *sigma < 0.0 {
        return Err(Error::InvalidSigma { sigma: *sigma });
    }
    // ... existing blur logic
}
```

### Expected test

Add this to `tests/transform_tests.rs` to verify your resize implementation:

```rust,ignore
use imgforge::transform_imagers::ImageRsBackend;
use imgforge::{Operation, Transform};

#[test]
fn resize_produces_correct_dimensions() {
    // Create a 100x100 red test image
    let img = image::RgbImage::from_fn(100, 100, |_, _| image::Rgb([255, 0, 0]));
    let mut input = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut input),
        image::ImageFormat::Png,
    )
    .unwrap();

    let backend = ImageRsBackend;
    let result = backend
        .apply(&input, &Operation::Resize { width: 50, height: 50 })
        .unwrap();

    // Decode the output and check dimensions
    let output = image::load_from_memory(&result).unwrap();
    assert_eq!(output.width(), 50);
    assert_eq!(output.height(), 50);
}

#[test]
fn grayscale_does_not_change_dimensions() {
    let img = image::RgbImage::from_fn(64, 48, |_, _| image::Rgb([0, 128, 255]));
    let mut input = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut input),
        image::ImageFormat::Png,
    )
    .unwrap();

    let backend = ImageRsBackend;
    let result = backend.apply(&input, &Operation::Grayscale).unwrap();

    let output = image::load_from_memory(&result).unwrap();
    assert_eq!(output.width(), 64);
    assert_eq!(output.height(), 48);
}
```

Run the tests:

```bash
cargo test --manifest-path imgforge/Cargo.toml
```

Once all three tasks are complete, you should have five `Operation` variants (`Resize`, `ResizeFit`, `Grayscale`, `Blur`, `Rotate90`), a new error variant (`InvalidSigma`), and passing tests. In the next chapter (Ch21), you will add a second backend using FFI -- and the `Transform` trait will make that a drop-in replacement.
