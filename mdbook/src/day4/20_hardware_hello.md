# Chapter 20: Hello World

## Learning Objectives
- Install the ESP32-C3 toolchain
- Generate a project with `esp-generate`
- Understand the generated project structure
- Build, flash, and monitor a Hello World program

## Prerequisites

Install the tools we need:

```bash
cargo install espflash --locked
cargo install esp-generate --locked
```

- **espflash**: flashes firmware to ESP chips over USB and provides a serial monitor
- **esp-generate**: scaffolds new ESP32 projects with the correct build config, dependencies, and linker setup

On Linux, add yourself to the `dialout` group so you can access the serial port:

```bash
sudo usermod -a -G dialout $USER
# Log out and back in for this to take effect
```

Plug in your ESP32-C3 board via USB. Verify it's detected:

```bash
espflash board-info
```

## Create a Project

Generate a new project:

```bash
esp-generate --chip esp32c3 hello
```

The TUI will ask a series of questions — accept the defaults for everything. This gives us a minimal `no_std` project.

## Generated Files Explained

```
hello/
├── .cargo/
│   └── config.toml     # Build target + espflash as runner
├── rust-toolchain.toml  # Stable Rust + riscv32imc target
├── build.rs             # Linker script setup
├── Cargo.toml           # Dependencies: esp-hal, esp-bootloader-esp-idf
└── src/
    └── bin/
        └── main.rs      # Entry point
```

### `.cargo/config.toml`

```toml
[target.riscv32imc-unknown-none-elf]
runner = "espflash flash --monitor --chip esp32c3"

[build]
rustflags = ["-C", "force-frame-pointers"]
target = "riscv32imc-unknown-none-elf"

[unstable]
build-std = ["core"]
```

Key points:
- **runner**: `cargo run` will flash the chip and open a serial monitor
- **target**: ESP32-C3 uses the RISC-V `riscv32imc-unknown-none-elf` target
- **build-std**: Rebuilds `core` from source (needed for `no_std` targets)

### `rust-toolchain.toml`

```toml
[toolchain]
channel    = "stable"
components = ["rust-src"]
targets    = ["riscv32imc-unknown-none-elf"]
```

We need `rust-src` because `build-std` compiles `core` from source.

### `Cargo.toml`

```toml
[dependencies]
esp-hal = { version = "1.0.0", features = ["esp32c3"] }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"] }
critical-section = "1.2.0"
```

- **esp-hal**: Hardware Abstraction Layer — gives us GPIO, timers, peripherals
- **esp-bootloader-esp-idf**: Provides the bootloader app descriptor
- **critical-section**: Required for interrupt-safe shared state

### `src/bin/main.rs` (generated)

```rust
#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }
}
```

Let's break this down:

| Line | Purpose |
|------|---------|
| `#![no_std]` | No standard library — we're on bare metal |
| `#![no_main]` | No normal `main` — the HAL provides the entry point |
| `#[panic_handler]` | Required in `no_std` — what to do on panic (loop forever) |
| `esp_app_desc!()` | App descriptor required by the ESP-IDF bootloader |
| `#[main]` | ESP-HAL's entry point macro |
| `-> !` | Never returns — embedded programs run forever |
| `esp_hal::init()` | Initializes all hardware, returns peripheral handles |

## Add Hello World

The generated code just loops doing nothing. Let's add serial output.

Add the `esp-println` dependency to `Cargo.toml`:

```toml
[dependencies]
esp-hal = { version = "1.0.0", features = ["esp32c3"] }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"] }
critical-section = "1.2.0"
esp-println = { version = "0.14.0", features = ["esp32c3"] }
```

Now modify `src/bin/main.rs`:

```rust
#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    let mut count = 0u32;

    loop {
        println!("Hello, world! (count: {})", count);
        count += 1;

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_secs(1) {}
    }
}
```

`esp_println::println!` works just like the standard `println!` but sends output over the USB serial connection.

## Build & Flash

```bash
cargo run --release
```

This does three things:
1. **Builds** the firmware (cross-compiling to RISC-V)
2. **Flashes** it to the ESP32-C3 via USB
3. **Opens a serial monitor** so you see the output

You should see:

```
Hello, world! (count: 0)
Hello, world! (count: 1)
Hello, world! (count: 2)
...
```

Press `Ctrl+R` to reset the chip. Press `Ctrl+C` to exit the monitor.

> **Tip**: Use `--release` for faster builds and smaller binaries. The `[profile.dev]` already sets `opt-level = "s"`, but release mode enables LTO and other optimizations.

## How It Works

Unlike a normal Rust program, our code runs directly on the hardware with no operating system:

```
┌──────────────────────────┐
│     Your Application     │
├──────────────────────────┤
│    esp-hal (HAL layer)   │
├──────────────────────────┤
│   ESP32-C3 Hardware      │
│  (CPU, GPIO, UART, ...)  │
└──────────────────────────┘
```

- **No OS**: No threads, no filesystem, no heap (unless you add one)
- **Direct hardware access**: `esp_hal::init()` gives you handles to all peripherals
- **Never returns**: `-> !` means `main` loops forever — there's nowhere to return to

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `Permission denied` on serial port | `sudo usermod -a -G dialout $USER` and re-login |
| `espflash` can't find the chip | Try a different USB cable (some are charge-only) |
| Build fails with linker errors | Make sure `build.rs` and `.cargo/config.toml` are present |
| `rust-src` component missing | Run `rustup component add rust-src` |

## Exercise

Modify the Hello World program to also print the loop iteration time. Use `Instant::now()` and `elapsed()` to measure how long each iteration takes. Does it match your delay?
