# Course overview
This course is targeted at developers experienced in other procedural or object-oriented programming languages.

* Day 1: Rust foundations and the concept of ownership
* Day 2: Type system and error handling
* Day 3: Building programs & advanced topics
* Day 4: imgforge — hands-on image processing service

Each day is a mix of theory and exercises. Days 1 to 3 feature exercises in a std environment (building CLI applications on desktop). Day 4 builds an incremental image processing service (CLI, web server, FFI acceleration) applying the theory from Day 3.

# This repository
Contains the course slides/script as an mdbook and solutions to the exercises in the `solutions` directory. Will be updated before and during the course.

# Installation Instructions Days 1 to 3
Please ensure the following software is installed on the device you bring to the course.

If there are any questions or difficulties during the installation please don't hesitate to contact the instructor (rolandbrand11@gmail.com).

## Rust
Install Rust using rustup (Rust's official installer)
- Visit [rust-lang.org](https://rust-lang.org/tools/install) and follow the installation instructions for your operating system.
- Verify installation with: `rustc --version` and `cargo --version`

Recommended: Install rust-analyzer for a better development experience:
- **rust-analyzer**: A language server that provides IDE features like autocompletion, go-to-definition, and inline errors. Install with: `rustup component add rust-analyzer`

The following tools are included by default with the stable toolchain:
- **clippy**: Run with `cargo clippy` to catch common mistakes and improve your code.
- **rustfmt**: Run with `cargo fmt` to automatically format your code.

## Git
Git for version control: [git-scm.com](https://git-scm.com/)
- Make sure you can access it through the command line: `git --version`

## Zed Editor
Download from [zed.dev](https://zed.dev/)

During the course the trainer will use Zed - participants are recommended to use the same editor, but are free to choose any other editor or IDE. The trainer will not be able to provide setup or configuration support for other editors or IDEs during the course.

## Create a Test Project

Create a new Rust project and build it:

```bash
cargo new hello-rust
cd hello-rust
cargo build
```

## Run the Project

Execute the project to verify your Rust installation:

```bash
cargo run
```

You should see "Hello, world!" printed to your terminal.

## Troubleshooting
If you encounter any issues:

### Rust Installation Notes
- On Linux, you will need a C linker (and optionally a C compiler for crates with native code). Install build essentials if not already present: `sudo apt install build-essential` (Ubuntu/Debian) or equivalent for your distribution.
- On Windows, the Visual Studio C++ Build Tools are required by default, as Rust's default toolchain uses the MSVC linker. The rustup installer will guide you through this.

### Cargo Issues
- Try clearing the cargo cache: `cargo clean`
- Update rust: `rustup update`

## Cleanup
To remove the test project:

```bash
cd
rm -rf hello-rust
```

If you can complete all these steps successfully, your environment is ready for the first two days of the Rust course!

# Additional Setup for Day 4 — imgforge

Day 4 builds an image processing service. The standard Rust toolchain from Days 1-3 is sufficient. No additional setup is strictly required.

## Optional: TurboJPEG System Library

Chapter 21 demonstrates FFI acceleration using TurboJPEG. This is **optional** — the project works without it using the pure-Rust `image` crate.

If you want to try the FFI chapter with the real TurboJPEG library:

```bash
# Ubuntu/Debian
sudo apt install libturbojpeg0-dev

# macOS
brew install jpeg-turbo

# Windows — the turbojpeg crate bundles pre-built binaries, no action needed
```

To verify (optional):
```bash
cargo build --features turbojpeg --manifest-path solutions/day4/imgforge/Cargo.toml
```

If this fails, don't worry — the feature flag stays off and everything else works fine.

**→ Regularly pull updates to the repo**