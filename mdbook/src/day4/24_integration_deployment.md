# Chapter 24: Integration & Deployment

## Learning Objectives
This chapter covers:
- Integrate all components into a complete temperature monitoring system
- Configure build optimization for embedded deployment
- Flash and debug applications on ESP32-C3 hardware
- Implement basic error handling and recovery

## Task: Build Production-Ready Temperature Monitor

Over chapters 20-23, we've built individual components. Now it's time to integrate everything into a robust, production-ready system.

**Your Mission:**
1. **Integrate all components** into a single working system
2. **Add error handling** and recovery mechanisms
3. **Optimize build configuration** for production deployment
4. **Add deployment scripts** for easy flashing and monitoring
5. **Create production monitoring** with structured output

**What We're Combining:**
- **Chapter 20**: Hardware interaction with ESP32-C3 and temperature sensor
- **Chapter 21**: Embedded data structures with no_std foundations
- **Chapter 22**: Comprehensive testing strategy for embedded code
- **Chapter 23**: JSON communication and structured data protocols

**Production Requirements:**
- Graceful error handling (no panics in production)
- Optimized binary size and performance
- Reliable sensor reading with fallback
- Structured logging for monitoring
- Easy deployment and debugging

### Simplified System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   ESP32-C3 System                      │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Main Loop                          │   │
│  │                                                 │   │
│  │  1. Read Temperature                            │   │
│  │  2. Store in Buffer                             │   │
│  │  3. Update LED Status                           │   │
│  │  4. Output JSON (every 5 readings)             │   │
│  │  5. Delay 1 second                              │   │
│  │  6. Repeat                                      │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │Temperature  │  │   LED       │  │    JSON     │     │
│  │Buffer       │  │ Controller  │  │  Output     │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │              USB Serial Output                  │   │
│  │  Status Messages | Readings | Statistics        │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Complete Temperature Monitor Implementation

### Project Setup

First, let's create the complete `Cargo.toml`:

```toml
[package]
name = "chapter24_integration"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"

[[bin]]
name = "chapter24_integration"
path = "./src/bin/main.rs"

[lib]
name = "chapter24_integration"
path = "src/lib.rs"

[dependencies]
# Only include ESP dependencies when not testing
esp-hal = { version = "1.0.0", features = ["esp32c3", "unstable"], optional = true }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"], optional = true }
esp-println = { version = "0.16", features = ["esp32c3"], optional = true }

# Core dependencies
heapless = "0.8"

# Serialization
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"

[features]
default = ["hardware"]
hardware = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]
simulation = []     # Use mock sensors instead of hardware
verbose = []        # Enable detailed debug logging
telemetry = []      # Extended monitoring capabilities

[profile.dev]
# Rust debug is too slow for embedded
opt-level = "s"

[profile.release]
# Production optimizations
codegen-units = 1     # LLVM can perform better optimizations using a single thread
debug = 2
debug-assertions = false
incremental = false
lto = 'fat'
opt-level = 's'
overflow-checks = false
```

### Main System Implementation

```rust
// src/bin/main.rs - Production-ready integrated system
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

// Use the integrated system components from previous chapters
use chapter24_integration::{Temperature, TemperatureBuffer, Command, TemperatureComm};

// Production system configuration
const BUFFER_SIZE: usize = 32;
const SAMPLE_RATE_MS: u32 = 1000;
const JSON_OUTPUT_INTERVAL: u32 = 5;
const HEALTH_REPORT_INTERVAL: u32 = 20;

// System state tracking for production monitoring
struct SystemState {
    reading_count: u32,
    system_time_ms: u32,
    overheating_count: u32,
    sensor_error_count: u32,
    last_temp: f32,
}

impl SystemState {
    fn new() -> Self {
        Self {
            reading_count: 0,
            system_time_ms: 0,
            overheating_count: 0,
            sensor_error_count: 0,
            last_temp: 0.0,
        }
    }

    fn advance_time(&mut self) {
        self.reading_count += 1;
        self.system_time_ms += SAMPLE_RATE_MS;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // In production, we want graceful error handling
    esp_println::println!("SYSTEM_ERROR: Panic occurred, attempting recovery...");
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Initialize hardware with error handling
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize components
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    let temp_sensor = TemperatureSensor::new(peripherals.TSENS, Config::default()).unwrap();
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let mut comm = TemperatureComm::new();
    let mut state = SystemState::new();

    // System startup
    esp_println::println!("🚀 ESP32-C3 Production Temperature Monitor v1.0");
    esp_println::println!("📊 Buffer: {} readings | Sample rate: {}ms", BUFFER_SIZE, SAMPLE_RATE_MS);
    esp_println::println!("📡 JSON output every {} readings", JSON_OUTPUT_INTERVAL);
    esp_println::println!("🏥 Health reports every {} readings", HEALTH_REPORT_INTERVAL);
    esp_println::println!("✅ System initialized successfully");
    esp_println::println!();

    comm.init(0);

    // Main production loop with error handling
    loop {
        // Read temperature with error handling
        let esp_temperature = temp_sensor.get_temperature();
        let temp_celsius = esp_temperature.to_celsius();
        let temperature = Temperature::from_celsius(temp_celsius);

        // Update system state
        state.last_temp = temp_celsius;
        temp_buffer.push(temperature);
        state.advance_time();

        // LED status indication
        if temperature.is_overheating() {
            state.overheating_count += 1;
            // Rapid triple blink for overheating
            for _ in 0..3 {
                led.set_high();
                let blink_start = Instant::now();
                while blink_start.elapsed() < Duration::from_millis(100) {}
                led.set_low();
                let blink_start = Instant::now();
                while blink_start.elapsed() < Duration::from_millis(100) {}
            }
        } else if !temperature.is_normal_range() {
            // Double blink for abnormal range
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(150) {}
            led.set_low();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(150) {}
            led.set_low();
        } else {
            // Normal single blink
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(200) {}
            led.set_low();
        }

        // JSON output every N readings
        if state.reading_count % JSON_OUTPUT_INTERVAL == 0 {
            let reading_json = comm.latest_reading_json(&temp_buffer, state.system_time_ms);
            esp_println::println!("READING: {}", reading_json);

            let stats_json = comm.stats_json(&temp_buffer, state.system_time_ms);
            esp_println::println!("STATS: {}", stats_json);
        }

        // Health report every N readings
        if state.reading_count % HEALTH_REPORT_INTERVAL == 0 {
            esp_println::println!("HEALTH: readings={} overheating={} errors={} uptime={}ms",
                state.reading_count,
                state.overheating_count,
                state.sensor_error_count,
                state.system_time_ms
            );
        }

        // Wait for next sample
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(SAMPLE_RATE_MS as u64) {}
    }
}
```

## Production Deployment

Build and deploy the production system:

```bash
# Run tests
cargo test

# Build and deploy to ESP32-C3 (recommended)
cargo run --release

# Alternative: Build then flash separately
cargo build --release --target riscv32imc-unknown-none-elf
cargo espflash flash target/riscv32imc-unknown-none-elf/release/chapter24_integration

# Monitor production logs
cargo espflash monitor
```

## Understanding Cargo Features

Cargo features are a powerful mechanism for conditional compilation in Rust projects. In embedded systems, they're especially useful for managing different build configurations.

### What Are Cargo Features?

Features allow you to:
- Enable/disable functionality at compile time
- Support multiple hardware platforms from one codebase
- Create development vs production builds
- Reduce binary size by excluding unused code

### Our Feature Configuration

```toml
[features]
default = ["hardware"]              # Default features enabled
hardware = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]  # Real ESP32-C3 hardware
simulation = []                     # Mock sensors for testing
verbose = []                        # Detailed debug logging
telemetry = []                      # Extended monitoring capabilities
```

### Conditional Compilation

Use `#[cfg(feature = "...")]` to conditionally compile code:

```rust
// Different sensor implementations based on features
#[cfg(feature = "hardware")]
use esp_hal::tsens::{Config, TemperatureSensor};

#[cfg(feature = "simulation")]
mod mock_sensor {
    pub struct MockTemperatureSensor {
        temperature: f32,
    }

    impl MockTemperatureSensor {
        pub fn new() -> Self {
            Self { temperature: 25.0 }
        }

        pub fn get_temperature(&mut self) -> f32 {
            // Simulate varying temperature
            self.temperature += (rand() % 5) as f32 - 2.0;
            self.temperature
        }
    }
}

// Optional verbose logging
#[cfg(feature = "verbose")]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        esp_println::println!("DEBUG: {}", format!($($arg)*));
    };
}

#[cfg(not(feature = "verbose"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

// Extended telemetry
#[cfg(feature = "telemetry")]
fn output_telemetry_data(state: &SystemState) {
    esp_println::println!("TELEMETRY: {{");
    esp_println::println!("  \"uptime_ms\": {},", state.system_time_ms);
    esp_println::println!("  \"free_heap\": {},", get_free_heap());
    esp_println::println!("  \"cpu_usage\": {},", get_cpu_usage());
    esp_println::println!("}}");
}
```

### Building with Different Features

```bash
# Default build (hardware features enabled)
cargo build --release

# Build for simulation (no hardware needed)
cargo build --features simulation

# Build with verbose logging
cargo build --features "hardware,verbose"

# Build with all monitoring features
cargo build --features "hardware,verbose,telemetry"

# Build with only simulation and telemetry
cargo build --no-default-features --features "simulation,telemetry"
```

### Why Features Matter in Embedded

1. **Binary Size**: Exclude unused features to reduce Flash usage
2. **Testing**: Run tests without hardware using simulation features
3. **Development**: Enable verbose logging during development
4. **Production**: Strip debug features for production builds
5. **Portability**: Support multiple hardware platforms

### Real-World Examples

```rust
// Production vs Development builds
#[cfg(feature = "verbose")]
const LOG_LEVEL: LogLevel = LogLevel::Debug;

#[cfg(not(feature = "verbose"))]
const LOG_LEVEL: LogLevel = LogLevel::Error;

// Hardware-specific implementations
#[cfg(feature = "hardware")]
fn read_temperature() -> Result<f32, SensorError> {
    let sensor = TemperatureSensor::new(/* ... */)?;
    Ok(sensor.get_temperature().to_celsius())
}

#[cfg(feature = "simulation")]
fn read_temperature() -> Result<f32, SensorError> {
    // Return predictable test data
    Ok(23.5 + (system_time() % 10) as f32)
}
```

## Exercise: Production System Integration


Integrate all previous components into a production-ready temperature monitoring system.

### Requirements

1. **System Integration**: Combine hardware, data structures, testing, and communication
2. **Cargo Features**: Implement conditional compilation for different build configurations
3. **Error Recovery**: Handle sensor failures and system errors gracefully
4. **Production Monitoring**: Add health reporting and system metrics
5. **Build Optimization**: Configure release builds for optimal performance
6. **Deployment Ready**: Create scripts for easy flashing and monitoring

### Starting Structure

Based on previous chapters, create the integrated system:

```rust
// src/bin/main.rs - Production system main file
#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;

// Conditional imports based on features
#[cfg(feature = "hardware")]
use esp_hal::tsens::{Config, TemperatureSensor};

// Import from your integrated library
use chapter24_integration::{Temperature, TemperatureBuffer, TemperatureComm};

// Debug logging macro (only compiled with verbose feature)
#[cfg(feature = "verbose")]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        esp_println::println!("DEBUG: {}", format_args!($($arg)*));
    };
}

#[cfg(not(feature = "verbose"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

// Production configuration
const BUFFER_SIZE: usize = 32;
const SAMPLE_RATE_MS: u32 = 1000;
const HEALTH_REPORT_INTERVAL: u32 = 20;

// System state tracking
struct SystemState {
    reading_count: u32,
    system_time_ms: u32,
    overheating_count: u32,
    sensor_error_count: u32,
    // TODO: Add more state fields
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // TODO: Implement production panic handler with logging
    loop {}
}

#[main]
fn main() -> ! {
    // TODO: Initialize all components with feature-based configuration
    // TODO: Initialize sensor based on hardware vs simulation features
    // TODO: Add error handling for sensor initialization
    // TODO: Implement main monitoring loop with health checks
    // TODO: Add conditional telemetry and verbose logging
}

// TODO: Implement helper functions with feature gates:
// - read_temperature_safe() with hardware/simulation branches
// - update_led_status() with enhanced patterns
// - output_health_report() for system monitoring
// - handle_error_conditions() for error recovery
// - output_telemetry_data() (telemetry feature only)
// - debug logging (verbose feature only)
```

### Implementation Tasks

1. **Cargo Features Setup**:
   - Implement conditional sensor initialization (hardware vs simulation)
   - Add debug logging macros with verbose feature
   - Create feature-gated telemetry functions

2. **System Integration**:
   - Initialize all hardware components with error handling
   - Create SystemState struct to track system health
   - Set up production configuration constants

3. **Error Recovery**:
   - Implement safe temperature reading with fallback
   - Add sensor error counting and recovery
   - Create production panic handler with logging

4. **Health Monitoring**:
   - Add system state tracking (uptime, errors, performance)
   - Implement health report generation
   - Create status indicators and LED patterns

5. **Production Features**:
   - Configure optimized Cargo.toml profile
   - Add JSON health reporting
   - Test complete system integration with different features

### Success Criteria

- [ ] System integrates all previous chapter components
- [ ] Cargo features work correctly (hardware, simulation, verbose, telemetry)
- [ ] Handles sensor failures without crashing
- [ ] Provides health monitoring and error reporting
- [ ] Optimized build configuration for production
- [ ] Complete JSON communication system working
- [ ] LED status indicates system health
- [ ] Recovery from common error conditions
- [ ] Different features produce different build outputs

### Expected Health Report Output

```
🌡️ ESP32-C3 Complete Temperature Monitor System
=================================================
🔧 Hardware: ESP32-C3 @ max frequency
📊 Buffer capacity: 32 readings
⏱️  Sample rate: 1 Hz
🌡️ Overheating threshold: 52.0°C
📡 JSON output every 5 readings
💓 Health reports every 20 readings
🚀 System starting...

🟢📊 #001 | 24.3°C | Buffer: 1/32
🟢📊 #002 | 24.1°C | Buffer: 2/32
...
🟢📊 #020 | 24.8°C | Buffer: 20/32

💓 HEALTH REPORT
  Uptime: 20s | Readings: 20
  Buffer: 62% (20/32) | Memory: 128 bytes
  Errors: 0 sensor, 0 overheating events
  Current temp: 24.8°C
```

### Build Optimization

Update your `Cargo.toml` with production settings:

```toml
[profile.release]
codegen-units = 1         # Better optimization
debug = false            # Remove debug info
debug-assertions = false # Remove runtime checks
incremental = false      # Full rebuild for optimization
lto = 'fat'             # Link-time optimization
opt-level = 's'         # Optimize for size
overflow-checks = false # Remove overflow checks
panic = 'abort'         # Smaller panic handler
strip = true            # Remove symbols
```

### Deployment Script

Create `deploy.sh`:

```bash
#!/bin/bash
echo "🚀 Deploying Production Temperature Monitor..."
cargo build --release --features embedded
cargo espflash flash --monitor target/riscv32imc-unknown-none-elf/release/chapter24_integration
```

### Extension Challenges

1. **Watchdog Timer**: Add hardware watchdog for system recovery
2. **Flash Storage**: Persist configuration across resets
3. **Over-the-Air Updates**: Implement firmware update capability
4. **Network Integration**: Connect to WiFi for remote monitoring
5. **Power Management**: Add sleep modes for battery operation

### Error Recovery Strategies

- **Sensor Failure**: Use last known good value, count errors
- **Buffer Overflow**: Circular buffer handles automatically
- **Communication Error**: Continue operation, log errors
- **Memory Issues**: Monitor stack usage, implement safeguards
- **Timing Drift**: Use hardware timers for precision

### Testing Production System

```bash
# Run comprehensive tests
./test.sh

# Test different feature combinations
cargo check --features hardware
cargo check --features simulation
cargo check --features "hardware,verbose"
cargo check --features "hardware,telemetry"
cargo check --features "simulation,verbose,telemetry"

# Check build sizes with different features
cargo size --release                           # Default (hardware)
cargo size --release --features simulation     # Simulation only
cargo size --release --features "hardware,verbose,telemetry"  # Full featured

# Flash and monitor
chmod +x deploy.sh
./deploy.sh
```

## Production System Features

✅ **Error Handling**: Graceful panic handling with recovery attempts
✅ **Health Monitoring**: System metrics and error counting
✅ **Structured Logging**: JSON output for monitoring dashboards
✅ **Performance Optimization**: Optimized builds for production deployment
✅ **State Tracking**: Comprehensive system state monitoring
✅ **Production Configuration**: Configurable intervals and thresholds

**Next**: In Chapter 25, we'll explore advanced features and extensions to make the system even more capable.
