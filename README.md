# Course overview
This course is targeted at developers experienced in other procedural or object-oriented programming languages.

* Day 1: Rust foundations and the concept of ownership
* Day 2: Type system and error handling
* Day 3: Building programs & advanced topics
* Day 4: ESP32-C3 embedded systems — hands-on hardware programming

Each day is a mix of theory and exercises. Days 1 to 3 feature exercises in a std environment (building CLI applications on desktop). Day 4 applies the theory from Day 3 to real hardware: building an embedded temperature monitoring system on an ESP32-C3 microcontroller using `no_std` Rust.

# This repository
Contains the course slides/script as an mdbook and solutions to the exercises in the `solutions` directory. Will be updated before and during the course.

# Installation Instructions Days 1 to 3
Please ensure the following software is installed on the device you bring to the course.

If there are any questions or difficulties during the installation please don't hesitate to contact the instructor (rolandbrand11@gmail.com).

## Rust
Install Rust using rustup (Rust's official installer)
- Visit [rust-lang.org](https://rust-lang.org/tools/install) and follow the installation instructions for your operating system.
- **Windows:** Rust requires the **Visual Studio C++ Build Tools** for linking. The `rustup` installer will offer to set these up — when prompted, select the **"Desktop development with C++"** workload. Make sure the default toolchain is set to **MSVC** (the default), not GNU. You can verify with `rustup show`. See [Troubleshooting](#windows-toolchain) below if you run into linker errors.
- **Linux:** A C linker is required. Install build essentials if not already present: `sudo apt install build-essential` (Ubuntu/Debian) or equivalent.
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

### <a name="windows-toolchain"></a> Windows Toolchain

Rust on Windows supports two ABIs: **MSVC** (default, recommended) and **GNU**. If your toolchain is set to `x86_64-pc-windows-gnu` and you do not have MSYS2/MinGW installed, builds will fail with errors like:

```
error: error calling dlltool 'dlltool.exe': program not found
```

**Fix:** Install the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select "Desktop development with C++") and switch to the MSVC toolchain:

```bash
rustup default stable-x86_64-pc-windows-msvc
```

See the [rustup documentation on Windows](https://rust-lang.github.io/rustup/installation/windows.html) for details.

### Cargo Issues
- Try clearing the cargo cache: `cargo clean`
- Update rust: `rustup update`

## Cleanup
To remove the test project:

```bash
cd
rm -rf hello-rust
```

If you can complete all these steps successfully, your environment is ready for the course!

# Additional Setup for Day 4 — ESP32-C3 Embedded

Day 4 targets ESP32-C3 hardware. The standard Rust toolchain from Days 1-3 is required, plus:

```bash
# Add the RISC-V target for ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Install the flash/monitor tool
cargo install cargo-espflash
```

### Hardware Requirements
- ESP32-C3 development board (e.g. ESP32-C3-DevKitM-1)
- USB-C cable for programming and power

No external sensors or components are needed — the exercises use the ESP32-C3's built-in temperature sensor.

**→ Regularly pull updates to the repo**