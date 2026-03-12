# Chapter 24: Managed Temperature Store with Tests

## Learning Objectives
- Build a testable `no_std` library for embedded projects
- Use `#![cfg_attr(not(test), no_std)]` for dual-target code
- Write and run unit tests on the host machine
- Integrate tested library code with Embassy firmware

## From Raw Readings to Useful Data

The on-die temperature sensor from Chapter 23 gives us raw die temperature — typically ~30°C above ambient. A raw reading of 52°C probably means the room is about 22°C.

Let's build a `TemperatureStore` that:
- Stores the latest raw reading
- Computes estimated ambient temperature (raw minus an offset)
- Converts to Fahrenheit
- Is **fully testable** on the host — no ESP32 needed

## The TemperatureStore

This is pure Rust — no hardware dependencies, no `unsafe`, just math:

```rust
pub struct TemperatureStore {
    raw_celsius: Option<f32>,
    offset: f32,
}

impl TemperatureStore {
    pub fn new(offset: f32) -> Self {
        Self {
            raw_celsius: None,
            offset,
        }
    }

    pub fn update(&mut self, raw_celsius: f32) {
        self.raw_celsius = Some(raw_celsius);
    }

    pub fn raw_celsius(&self) -> Option<f32> {
        self.raw_celsius
    }

    pub fn ambient_celsius(&self) -> Option<f32> {
        self.raw_celsius.map(|raw| raw - self.offset)
    }

    pub fn ambient_fahrenheit(&self) -> Option<f32> {
        self.ambient_celsius().map(|c| c * 9.0 / 5.0 + 32.0)
    }
}
```

This is a simple struct — and that's the point. By keeping the logic free of hardware dependencies, we can test it anywhere.

## Project Structure for Testing

Here's the key challenge: our project needs to compile for **two targets**:
1. `riscv32imc-unknown-none-elf` — for the ESP32-C3 (no_std)
2. Host machine — for running tests (std available)

```
chapter24_temperature_store/
├── .cargo/config.toml      # Sets default target to riscv32imc
├── Cargo.toml              # Feature-gated dependencies
├── build.rs                # Linker setup (only for embedded)
├── rust-toolchain.toml
├── test.sh                 # Runs tests on host
└── src/
    ├── lib.rs              # #![cfg_attr(not(test), no_std)]
    ├── store.rs            # TemperatureStore + tests
    └── bin/main.rs         # Embassy firmware
```

### The `cfg_attr` Trick

```rust
// src/lib.rs
#![cfg_attr(not(test), no_std)]

mod store;
pub use store::TemperatureStore;
```

- When building for the ESP32: `no_std` is active — no standard library
- When running `cargo test`: `no_std` is **not** set — tests get the full standard library

### Feature-Gated Dependencies

```toml
[features]
default = ["embedded"]
embedded = [
    "dep:esp-hal",
    "dep:esp-rtos",
    "dep:esp-bootloader-esp-idf",
    "dep:embassy-executor",
    "dep:embassy-time",
    "dep:embassy-sync",
    "dep:critical-section",
    "dep:esp-println",
]

[dependencies]
esp-hal = { version = "1.0.0", features = ["esp32c3", "unstable"], optional = true }
esp-rtos = { version = "0.2.0", features = ["embassy", "esp32c3"], optional = true }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"], optional = true }
embassy-executor = { version = "0.9.1", optional = true }
embassy-time = { version = "0.5.0", optional = true }
embassy-sync = { version = "0.7.2", optional = true }
critical-section = { version = "1.2.0", optional = true }
esp-println = { version = "0.14.0", features = ["esp32c3"], optional = true }
```

All ESP/Embassy dependencies are `optional = true` and gated behind the `embedded` feature. When running tests with `--no-default-features`, none of them are compiled.

## Writing Tests

The `store.rs` module contains both the implementation and its tests:

```rust
// src/store.rs

pub struct TemperatureStore {
    raw_celsius: Option<f32>,
    offset: f32,
}

impl TemperatureStore {
    pub const fn new(offset: f32) -> Self {
        Self {
            raw_celsius: None,
            offset,
        }
    }

    pub fn update(&mut self, raw_celsius: f32) {
        self.raw_celsius = Some(raw_celsius);
    }

    pub fn raw_celsius(&self) -> Option<f32> {
        self.raw_celsius
    }

    pub fn ambient_celsius(&self) -> Option<f32> {
        self.raw_celsius.map(|raw| raw - self.offset)
    }

    pub fn ambient_fahrenheit(&self) -> Option<f32> {
        self.ambient_celsius().map(|c| c * 9.0 / 5.0 + 32.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_before_first_update() {
        let store = TemperatureStore::new(30.0);
        assert_eq!(store.raw_celsius(), None);
        assert_eq!(store.ambient_celsius(), None);
        assert_eq!(store.ambient_fahrenheit(), None);
    }

    #[test]
    fn update_stores_raw_value() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        assert_eq!(store.raw_celsius(), Some(52.0));
    }

    #[test]
    fn ambient_celsius_subtracts_offset() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        // 52.0 - 30.0 = 22.0
        let ambient = store.ambient_celsius().unwrap();
        assert!((ambient - 22.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ambient_fahrenheit_converts_correctly() {
        let mut store = TemperatureStore::new(30.0);
        store.update(52.0);
        // ambient = 22.0C → 22 * 9/5 + 32 = 71.6F
        let fahrenheit = store.ambient_fahrenheit().unwrap();
        assert!((fahrenheit - 71.6).abs() < 0.1);
    }

    #[test]
    fn negative_ambient_temperature() {
        let mut store = TemperatureStore::new(30.0);
        store.update(20.0);
        // 20.0 - 30.0 = -10.0C
        let ambient = store.ambient_celsius().unwrap();
        assert!((ambient - (-10.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn update_overwrites_previous() {
        let mut store = TemperatureStore::new(30.0);
        store.update(50.0);
        store.update(55.0);
        assert_eq!(store.raw_celsius(), Some(55.0));
    }
}
```

These tests exercise:
- Initial state (all `None`)
- Basic update and retrieval
- Ambient calculation (raw minus offset)
- Fahrenheit conversion
- Negative temperatures
- Overwriting previous values

## Running Tests

There's a catch: `.cargo/config.toml` sets the default target to `riscv32imc-unknown-none-elf`, and `build.rs` adds embedded linker scripts. Both interfere with host compilation.

The `test.sh` script moves them aside temporarily:

```bash
#!/bin/bash
set -e

echo "Running tests on host..."

# Move embedded-specific files aside
mv .cargo/config.toml .cargo/config.toml.bak
mv build.rs build.rs.bak

# Restore on exit (success or failure)
trap 'mv .cargo/config.toml.bak .cargo/config.toml; mv build.rs.bak build.rs' EXIT

# Run tests without embedded features
cargo test --lib --no-default-features

echo "Tests passed!"
```

```bash
$ chmod +x test.sh
$ ./test.sh
Running tests on host...
running 6 tests
test store::tests::none_before_first_update ... ok
test store::tests::update_stores_raw_value ... ok
test store::tests::ambient_celsius_subtracts_offset ... ok
test store::tests::ambient_fahrenheit_converts_correctly ... ok
test store::tests::negative_ambient_temperature ... ok
test store::tests::update_overwrites_previous ... ok

test result: ok. 6 passed; 0 failed
Tests passed!
```

### Why This Works

| Step | What happens |
|------|-------------|
| Move `.cargo/config.toml` | Cargo no longer defaults to riscv32imc target — uses host |
| Move `build.rs` | No embedded linker scripts injected |
| `--no-default-features` | Disables `embedded` feature — all ESP deps are skipped |
| `--lib` | Only tests the library, not the binary (which needs ESP deps) |

## Integrating with Embassy

The `bin/main.rs` uses the store wrapped in a `static Mutex`, same pattern as Chapter 23:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::tsens::{Config, TemperatureSensor};
use esp_println::println;

use chapter24_temperature_store::TemperatureStore;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

static STORE: Mutex<critical_section::RawMutex, TemperatureStore> =
    Mutex::new(TemperatureStore::new(30.0));

#[embassy_executor::task]
async fn display_task() {
    loop {
        Timer::after(Duration::from_secs(5)).await;

        let guard = STORE.lock().await;
        match (guard.raw_celsius(), guard.ambient_celsius(), guard.ambient_fahrenheit()) {
            (Some(raw), Some(amb_c), Some(amb_f)) => {
                println!("[display] Raw: {}C | Ambient: {}C / {}F", raw, amb_c, amb_f);
            }
            _ => {
                println!("[display] No reading yet");
            }
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let temp_sensor = TemperatureSensor::new(peripherals.TSENS, Config::default()).unwrap();

    spawner.spawn(display_task()).unwrap();

    println!("Temperature store monitor started (offset: 30.0C)");

    loop {
        let celsius = temp_sensor.get_temperature().to_celsius();

        {
            let mut guard = STORE.lock().await;
            guard.update(celsius);
        }

        println!("[sensor] Raw: {}C", celsius);
        Timer::after(Duration::from_secs(2)).await;
    }
}
```

The only difference from Chapter 23: instead of storing a raw `f32`, we store a `TemperatureStore` that computes ambient and Fahrenheit for us.

### Note on `const fn new()`

For the `static STORE` to work, `TemperatureStore::new()` must be `const`:

```rust
pub const fn new(offset: f32) -> Self {
    Self {
        raw_celsius: None,
        offset,
    }
}
```

This lets us initialize the store at compile time inside the `Mutex::new()` call.

## Exercise

Extend `TemperatureStore` with a `TemperatureHistory` that keeps the last N readings:

1. Add a fixed-size array (e.g., `[f32; 8]`) and a count/index to `TemperatureStore`
2. Each `update()` stores the reading in the ring buffer
3. Add methods: `average() -> Option<f32>`, `min() -> Option<f32>`, `max() -> Option<f32>`
4. Write tests for all three methods, including edge cases (empty, single reading, full buffer, wrap-around)
5. Run the tests with `./test.sh`

**Hint**: A ring buffer with a fixed array works well in `no_std` — no heap allocation needed.
