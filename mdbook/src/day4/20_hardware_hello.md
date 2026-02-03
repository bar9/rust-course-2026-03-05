# Chapter 20: Hardware Hello - ESP32-C3 Basics

## Learning Objectives
This chapter covers:
- Set up ESP32-C3 development environment
- Understand the ESP32-C3 hardware capabilities and built-in sensors
- Create your first embedded Rust program that blinks an LED
- Read temperature from the ESP32-C3's built-in temperature sensor
- Send data over USB Serial for monitoring
- Understand the basics of embedded program structure and entry points

## Welcome to Embedded Rust!

After learning Rust fundamentals, it's time to apply that knowledge to real hardware. The ESP32-C3 is perfect for learning embedded Rust because it has:

- **Built-in temperature sensor** - No external components needed!
- **USB Serial support** - Easy debugging and communication
- **WiFi capability** - For IoT projects
- **Rust-first tooling** - Good `esp-hal` and ecosystem support
- **RISC-V architecture** - Modern, open-source instruction set

**Why Start with Hardware?**

Many embedded courses start with theory, but we're jumping straight into practical work - making real hardware do real things. This approach helps you:
- See immediate results (LED blinking, temperature readings)
- Understand constraints early (memory, power, timing)
- Build intuition for embedded programming patterns
- Stay motivated with tangible progress

## ESP32-C3 Hardware Overview

The ESP32-C3 is a system-on-chip (SoC) that includes:

```
┌─────────────────────────────────────┐
│            ESP32-C3 SoC             │
│                                     │
│  ┌─────────────┐  ┌─────────────┐   │
│  │ RISC-V Core │  │    WiFi     │   │
│  │  160 MHz    │  │ 802.11 b/g/n│   │
│  └─────────────┘  └─────────────┘   │
│                                     │
│  ┌─────────────┐  ┌─────────────┐   │
│  │   320KB     │  │    GPIO     │   │
│  │    RAM      │  │   Pins      │   │
│  └─────────────┘  └─────────────┘   │
│                                     │
│  ┌─────────────┐  ┌─────────────┐   │
│  │   4MB       │  │Temperature  │   │
│  │   Flash     │  │   Sensor    │   │ ← We'll use this!
│  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────┘
```

**Key Features for Our Project:**
- **Built-in Temperature Sensor**: Returns readings in digital format
- **USB Serial**: Built-in USB-to-serial conversion for easy debugging
- **GPIO Pin 8**: Usually connected to an LED on development boards
- **Low Power**: Can run on batteries for IoT applications

## Development Environment Setup

### Prerequisites

```bash
# Install Rust targets for ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Install cargo-espflash for flashing ESP32-C3
cargo install cargo-espflash

# Install probe-rs for debugging (optional, works best on Linux/macOS)
cargo install probe-rs --features cli

# Install serial monitoring tool (optional, for serial communication)
cargo install serialport-rs
```

### Hardware Requirements
- ESP32-C3 development board (like ESP32-C3-DevKitM-1)
- USB-C cable for programming and power
- Computer with USB port

**No external sensors or components needed** - we'll use the built-in temperature sensor!

## Your First ESP32 Program: LED Blink

Let's start with the embedded equivalent of "Hello, World!" - blinking an LED:

```rust
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types"
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Required by the ESP-IDF bootloader
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Configure GPIO 8 as LED output
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // Main loop - blink LED every second
    loop {
        led.set_high();
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}

        led.set_low();
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}
```

### Understanding the Code

**Key Differences from Regular Rust:**
- `#![no_std]` - No standard library (no heap, no OS services)
- `#![no_main]` - No traditional main function (embedded entry point)
- `#[main]` - ESP-HAL's main macro for embedded programs
- `#[panic_handler]` - Required to handle panics in no_std
- `-> !` - Function never returns (embedded programs run forever)

**Hardware Abstraction:**
- `esp_hal::init()` - Initialize hardware with configuration
- `gpio::Output` - Type-safe GPIO pin configuration
- `Instant::now()` and `Duration` - Hardware timer-based timing

**Why These Patterns?**
- **Singleton Pattern**: Hardware can only have one owner
- **Type Safety**: GPIO configuration enforced at compile time
- **Zero Cost**: Abstractions compile to direct hardware access

## Reading the Built-in Temperature Sensor

Now let's read the ESP32-C3's built-in temperature sensor:

```rust
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types"
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::tsens::{Config, TemperatureSensor};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Required by the ESP-IDF bootloader
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize GPIO for LED on GPIO8
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // Initialize the built-in temperature sensor
    let temp_sensor = TemperatureSensor::new(peripherals.TSENS, Config::default()).unwrap();

    // Track reading count
    let mut _reading_count = 0u32;

    // Main monitoring loop
    loop {
        // Small stabilization delay (recommended by ESP-HAL)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_micros(200) {}

        // Read temperature from built-in sensor
        let temperature = temp_sensor.get_temperature();
        let temp_celsius = temperature.to_celsius();
        _reading_count += 1;

        // LED feedback based on temperature threshold (52°C)
        if temp_celsius > 52.0 {
            // Fast blink pattern for high temperature
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}

            led.set_low();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}

            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}

            led.set_low();
        } else {
            // Slow single blink for normal temperature
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(200) {}

            led.set_low();
        }

        // Wait for remainder of 2-second interval
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(1500) {}
    }
}
```

### Understanding Temperature Sensor Code

**New Concepts:**
- `tsens::TemperatureSensor` - Hardware abstraction for built-in sensor (requires `unstable` feature)
- `get_temperature()` - Returns Temperature struct
- `to_celsius()` - Converts to Celsius value
- **No external wiring** - Sensor is built into the chip!
- **Temperature threshold** - We use 52°C to trigger fast blinking (you can trigger this by touching the chip)

**Data Flow:**
```
Hardware Sensor → ADC → Digital Value → Celsius Conversion → Your Code
```

**LED Status Patterns:**
- Normal temp (≤52°C): Single slow blink (200ms)
- High temp (>52°C): Fast double blink pattern (3x100ms blinks)

## Building and Running on Hardware

### Project Structure

Create a new embedded project:

```bash
cargo new --bin temp_monitor
cd temp_monitor
```

Update `Cargo.toml`:

```toml
[package]
name = "temp_monitor"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"

[[bin]]
name = "temp_monitor"
path = "./src/bin/main.rs"

[dependencies]
esp-hal = { version = "1.0.0", features = ["esp32c3", "unstable"] }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"] }
critical-section = "1.2.0"

[profile.dev]
# Rust debug is too slow for embedded
opt-level = "s"

[profile.release]
codegen-units = 1
debug = 2
debug-assertions = false
incremental = false
lto = 'fat'
opt-level = 's'
overflow-checks = false
```

### Building and Flashing

```bash
# Build and flash to ESP32-C3 (recommended method)
cargo run --release

# Alternative: Build first, then flash
cargo build --release
cargo espflash flash --monitor target/riscv32imc-unknown-none-elf/release/temp_monitor
```

### Serial Monitoring

Connect to see output:

```bash
# Using cargo-espflash (flashes and shows serial output)
cargo run --release

# Or just monitor serial output (after flashing)
cargo espflash monitor

# Alternative: Using screen (macOS/Linux)
screen /dev/cu.usbmodem* 115200
```

**Expected Output:**
```
ESP32-C3 Temperature Monitor
Built-in sensor initialized
Reading temperature every 2 seconds...

Reading #1: Temperature = 24.3°C
Reading #2: Temperature = 24.5°C
Reading #3: Temperature = 24.1°C
...
Reading #10: Temperature = 24.7°C
Status: 10 readings completed
```

## Understanding Embedded Program Structure

### Program Lifecycle

```rust
// 1. Hardware initialization
let peripherals = Peripherals::take();  // Get hardware ownership
let clocks = ClockControl::max(...);    // Configure clocks
let delay = Delay::new(&clocks);        // Set up timing

// 2. Peripheral configuration
let io = Io::new(...);                  // Initialize GPIO system
let mut led = Output::new(...);         // Configure specific pins
let mut temp_sensor = TemperatureSensor::new(...); // Set up sensors

// 3. Main application loop
loop {
    // Read sensors
    // Process data
    // Control outputs
    // Timing/delays
}
```

### Memory and Resource Management

**Key Constraints:**
- **320KB RAM** - All variables must fit in memory
- **No heap allocation** - Only stack and static allocation
- **No garbage collector** - Manual memory management
- **Real-time constraints** - Delays must be predictable

**Best Practices:**
- Use `delay.delay_millis()` instead of `std::thread::sleep()`
- Prefer fixed-size arrays over dynamic vectors
- Initialize all peripherals before main loop
- Keep critical timing sections short

### Error Handling in Embedded

Embedded Rust uses `Result<T, E>` even more extensively:

```rust
// Temperature sensor can fail
match temp_sensor.read_celsius() {
    Ok(temperature) => {
        esp_println::println!("Temperature: {:.1}°C", temperature);
    }
    Err(e) => {
        esp_println::println!("Sensor error: {:?}", e);
        // Could enter error state, reset, or retry
    }
}

// Alternative: Use expect() for prototype code
let temperature = temp_sensor.read_celsius()
    .expect("Temperature sensor failed");
```

## Exercise: Your First Temperature Monitor


Build a basic temperature monitoring system with the ESP32-C3's built-in sensor.

### Requirements

1. **Hardware Setup**: ESP32-C3 development board connected via USB
2. **Temperature Reading**: Use built-in temperature sensor
3. **LED Status**: Visual feedback based on temperature
4. **Serial Output**: Temperature readings every 2 seconds
5. **Status Reporting**: Progress summary every 10 readings

### Starting Code

Create `src/main.rs` with this foundation:

```rust
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    temperature_sensor::{TemperatureSensor, TempSensorConfig},
};

#[entry]
fn main() -> ! {
    // TODO: Initialize hardware

    // TODO: Set up temperature sensor

    // TODO: Main monitoring loop

    loop {
        // TODO: Read temperature

        // TODO: Control LED based on temperature

        // TODO: Print reading with status

        // TODO: Wait for next reading
    }
}
```

### Implementation Tasks

1. **Initialize Hardware**:
   ```rust
   let peripherals = Peripherals::take();
   let system = SystemControl::new(peripherals.SYSTEM);
   let clocks = ClockControl::max(system.clock_control).freeze();
   let delay = Delay::new(&clocks);

   let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
   let mut led = Output::new(io.pins.gpio8, Level::Low);
   ```

2. **Configure Temperature Sensor**:
   ```rust
   let temp_config = TempSensorConfig::default();
   let mut temp_sensor = TemperatureSensor::new(
       peripherals.TEMP_SENSOR,
       temp_config
   );
   ```

3. **Main Monitoring Loop**:
   - Read temperature with `temp_sensor.read_celsius()`
   - Control LED: fast blink if >25°C, slow if ≤25°C
   - Print "Reading #N: Temperature = X.X°C"
   - Status summary every 10 readings
   - 2-second intervals between readings

4. **Test on Hardware**:
   - Build and flash to ESP32-C3
   - Verify temperature readings and LED behavior
   - Try warming the chip with your finger

### Success Criteria

- [ ] Program compiles without warnings
- [ ] ESP32-C3 boots and shows startup message
- [ ] Temperature readings displayed every 2 seconds
- [ ] LED blinks with different patterns based on temperature
- [ ] Status summary appears every 10 readings
- [ ] Temperature values are reasonable (20-40°C typically)

### Expected Serial Output

```
ESP32-C3 Temperature Monitor
Built-in sensor initialized
Reading temperature every 2 seconds...

Reading #1: Temperature = 24.3°C
Reading #2: Temperature = 24.5°C
Reading #3: Temperature = 24.1°C
Reading #4: Temperature = 24.8°C
Reading #5: Temperature = 25.2°C  ← LED should blink faster now
...
Reading #10: Temperature = 24.7°C
Status: 10 readings completed

Reading #11: Temperature = 24.9°C
...
```

### Extension Challenges

1. **Temperature Threshold**: Make threshold adjustable via const
2. **LED Patterns**: Different patterns for different temperature ranges
3. **Statistics**: Track min/max temperatures
4. **Timing**: More precise 2-second intervals
5. **Error Handling**: Handle sensor reading failures gracefully

### Troubleshooting Tips

**Build Errors:**
- Ensure `rustup target add riscv32imc-unknown-none-elf` is installed
- Check that feature flags match your ESP32-C3 variant

**Flash Errors:**
- Ensure cargo-espflash is installed: `cargo install cargo-espflash`
- Check USB cable and ESP32-C3 connection
- Try: `cargo espflash flash target/riscv32imc-unknown-none-elf/release/temp_monitor`

**No Serial Output:**
- Verify baud rate (115200)
- Try different serial monitor tools
- Check USB device enumeration

**Sensor Issues:**
- Temperature readings should be 20-40°C typically
- Values outside this range might indicate calibration issues
- Warm the chip gently with your finger to test responsiveness

## Key Takeaways

✅ **Hardware First**: Starting with real hardware creates immediate engagement and practical learning

✅ **Built-in Sensors**: ESP32-C3's temperature sensor eliminates external component complexity

✅ **Embedded Patterns**: `#[no_std]`, `#[no_main]`, and `loop` are fundamental embedded concepts

✅ **Real-time Constraints**: Understanding timing and resource limitations from the start

✅ **Type Safety**: Rust's ownership system prevents common embedded bugs even on bare metal

✅ **Immediate Feedback**: LED status and serial output provide instant verification of functionality

## ESP32-C3 Troubleshooting Guide

### Hardware Issues

**Device Not Found / Flashing Fails:**
- Check USB-C cable is properly connected
- Try a different USB-C cable (some are power-only)
- Press and hold BOOT button while connecting USB
- Check device enumeration: `ls /dev/cu.*` (macOS) or `ls /dev/ttyUSB*` (Linux)
- Install USB drivers if needed: `brew install --cask silicon-labs-vcp-driver` (macOS)

**No Serial Output:**
- Verify baud rate is 115200
- Try different terminal: `screen /dev/cu.usbmodem* 115200`
- Check if device is already open in another terminal
- Press RESET button on ESP32-C3 to restart program

**Sensor Readings Look Wrong:**
- Temperature should be 20-40°C typically for room temperature
- Very high values (>80°C) may indicate calibration issues
- Try warming chip gently with finger to test responsiveness
- Compare with room thermometer for validation

### Software Issues

**Build Errors:**
```bash
# Install required targets and tools
rustup target add riscv32imc-unknown-none-elf
cargo install cargo-espflash
cargo install probe-rs --features cli

# Update tools if outdated
cargo install-update -a
```

**Linker Errors:**
- Check Cargo.toml dependencies match examples exactly
- Verify feature flags: `features = ["esp32c3", "unstable"]`
- Clean and rebuild: `cargo clean && cargo build`

**Runtime Panics:**
- Check temperature sensor initialization succeeds
- Verify GPIO pin 8 is available (built-in LED)
- Add more delay if sensor readings fail intermittently

**Performance Issues:**
- Use `opt-level = "s"` in Cargo.toml for size optimization
- Debug builds are very slow - always test with `--release`
- Monitor memory usage if experiencing strange behavior

### Development Tips

**Faster Development Cycle:**
- Use `cargo run --release` for combined build + flash + monitor
- Keep one terminal open for monitoring, another for building
- Save modified code before flashing (auto-save recommended)

**Serial Monitoring:**
```bash
# Built-in monitoring
cargo espflash monitor

# External tools
screen /dev/cu.usbmodem* 115200    # macOS/Linux
picocom /dev/ttyUSB0 -b 115200     # Linux alternative

# Exit screen: Ctrl+A then K, then Y
```

**When Things Go Wrong:**
1. Try different USB cable/port
2. Power cycle ESP32-C3 (unplug + replug)
3. Press RESET button
4. Clean build: `cargo clean`
5. Check for conflicting cargo processes: `pkill cargo`

### Common Error Messages

**`espflash::connection_failed`:**
- Device not in bootloader mode
- Wrong serial port selected
- Driver issues

**`failed to parse elf`:**
- Build failed but cargo didn't catch it
- Run `cargo build` first to see actual error
- Check target architecture matches

**`timer not found`:**
- Old esp-hal version - update dependencies
- Feature flag mismatch in Cargo.toml

If problems persist, check the [ESP32-C3 documentation](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/) and [esp-rs community](https://github.com/esp-rs).

**Next**: In Chapter 14, we'll build proper data structures for storing and processing these temperature readings using embedded-friendly `no_std` patterns.