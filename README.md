# Course overview
This course is targeted at developers experienced in other procedural or object-oriented programming languages.

* Day 1: Rust foundations and the concept of ownership
* Day 2: Type system and error handling
* Day 3: Systems programming & concurrency
* Transfer day: other languages to Rust

Each day is a mix of theory and exercises. day 1 and 2 feature exercises in a std environment (building cli applications on desktop). day 3 and transfer day feature no_std and building embedded applications on an ESP32C3 microcontroller.

# This repository
Contains the course slides/script as an mdbook and solutions to the exercises in the `solutions` directory. Will be updated before and during the course.

# Installation Instructions Day 1 and 2
Please ensure the following software is installed on the device you bring to the course.

If there are any questions or difficulties during the installation please don't hesitate to contact the instructor (rolandbrand11@gmail.com).

## Rust
Install Rust using rustup (Rust's official installer)
- Visit [rust-lang.org](https://rust-lang.org/tools/install) and follow the installation instructions for your operating system.
- Verify installation with: `rustc --version` and `cargo --version`

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

### Rust Installation Issues
- On Unix-like systems, you might need to install build essentials: `sudo apt install build-essential` (Ubuntu/Debian)
- On Windows, you might need to install Visual Studio C++ Build Tools

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

# Installation Instructions Day 3 and 4 - ESP32-C3 Embedded Development

From day 3, we will be using ESP32-C3 boards. Please install the following tooling in advance:

## Required ESP32-C3 Tooling

### 1. Rust Source Code
This downloads the rust source code. Needed to build the std or core library, no pre-compiled provided:

```bash
rustup component add rust-src
```

### 2. ESP32-C3 Target Architecture
The toolchain for the ESP32-C3 (RISC-V architecture):

```bash
rustup target add riscv32imc-unknown-none-elf
```

### 3. cargo-espflash for Flashing
cargo-espflash is the recommended tool for flashing ESP32-C3 boards across all platforms.

**Installation:**
```bash
# Install cargo-espflash
cargo install cargo-espflash
```

### 4. probe-rs for Debugging (Optional - Linux/macOS)
probe-rs provides debugging capabilities and works best on Linux and macOS.

**Installation (Optional):**
```bash
# Install probe-rs (optional, primarily for debugging)
cargo install probe-rs --features cli
```

### 5. esp-generate for Project Scaffolding
Tool for creating no_std projects targeting ESP32 chips:

```bash
cargo install esp-generate
```

## Verification Steps

### Test ESP32-C3 Setup
1. **Connect your ESP32-C3 board** via USB cable
2. **Generate a test project**:
   ```bash
   esp-generate --chip esp32c3 test-esp32c3
   cd test-esp32c3
   ```
3. **Build the project**:
   ```bash
   cargo build --release
   ```
4. **Flash to the board**:
   ```bash
   cargo run --release
   ```

### Zed Editor ESP32 Debugging Setup
If using Zed editor:
1. **Install probe-rs extension** in Zed: [https://zed.dev/extensions/probe-rs](https://zed.dev/extensions/probe-rs)
2. probe-rs integrates seamlessly with Zed for debugging ESP32-C3 projects

## Platform-Specific Instructions

### Windows
- Use PowerShell or Command Prompt
- Consider adding Windows Defender exclusions for Cargo directories
- Ensure you have the latest USB drivers

### macOS/Linux
- Installation should work out of the box
- Use Terminal for all commands
- May need to add user to dialout group on Linux: `sudo usermod -a -G dialout $USER`

## Troubleshooting ESP32-C3 Setup

### Common Issues and Solutions

**Flashing Issues:**
- If cargo-espflash fails to detect the board, ensure the ESP32-C3 is connected via USB and the correct port is being used

**Port Detection Issues:**
- On Windows: Check Device Manager for COM port assignments
- On Linux: Ensure user is in dialout group (see below)
- On macOS: Look for `/dev/cu.usbserial-*` or `/dev/cu.usbmodem*` devices

**ESP32-C3 Chip Revision:**
- Most ESP32-C3 boards work with cargo-espflash regardless of revision
- Check revision during flashing: Look for "Chip is ESP32-C3 (revision 3)" message

**Permission Issues (Linux):**
- Add user to dialout group: `sudo usermod -a -G dialout $USER`
- Log out and back in for changes to take effect

### Alternative Debugging Tools
For advanced debugging beyond cargo-espflash:
- **probe-rs**: Best on Linux/macOS for hardware debugging
- **ESP-IDF monitor**: Traditional ESP toolchain option
- **Serial monitor**: Use any serial terminal for basic output monitoring

## Resources
- [ESP-RS Documentation](https://docs.esp-rs.org/)
- [probe-rs Documentation](https://probe.rs/)
- [ESP32-C3 Hardware Reference](https://www.espressif.com/en/products/socs/esp32-c3)

**→ Regularly pull updates to the repo. There will also be additional setup instructions for days 3 and 4.**

