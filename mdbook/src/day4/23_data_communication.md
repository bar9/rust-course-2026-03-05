# Chapter 23: Data & Communication

## Learning Objectives
This chapter covers:
- Use Serde for serialization in no_std embedded environments
- Send structured temperature data as JSON over USB Serial
- Implement efficient binary protocols with postcard
- Create command/response interfaces for embedded systems
- Handle communication errors gracefully in resource-constrained environments
- Design protocols optimized for IoT and embedded applications

## Task: Send Structured Temperature Data via JSON

Building on chapters 20-22, where we created temperature monitoring with testing, now we need to enable communication with external systems.

**Your Mission:**
1. **Add serialization support** to temperature data structures using Serde
2. **Send JSON data** over USB Serial for monitoring dashboards
3. **Implement command/response protocol** for remote control
4. **Use fixed-size strings** and heapless collections for efficiency
5. **Handle communication errors** gracefully in resource-constrained environment

**Why This Matters:**
- **Remote monitoring**: Send data to dashboards and cloud services
- **Remote control**: Change settings without reflashing firmware
- **Interoperability**: JSON works with any programming language
- **Debugging**: Structured data makes debugging easier than raw values

**The Challenge:**
- No heap allocation for JSON serialization
- Fixed-size buffers for serial communication
- Error handling without panicking

## Serde in no_std: Serialization for Embedded

Serde is Rust's premier serialization framework, and it works great in no_std environments:

```toml
[package]
name = "chapter23_communication"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"

[[bin]]
name = "chapter23_communication"
path = "./src/bin/main.rs"

[lib]
name = "chapter23_communication"
path = "src/lib.rs"

[dependencies]
# Only include ESP dependencies when not testing
esp-hal = { version = "1.0.0", features = ["esp32c3", "unstable"], optional = true }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"], optional = true }
esp-println = { version = "0.16", features = ["esp32c3"], optional = true }

# Core dependencies
critical-section = "1.2.0"
heapless = "0.8"

# Serialization
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"

[features]
default = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]
embedded = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]
```

### Making Temperature Data Serializable

Let's update our temperature types to support serialization:

```rust
// src/temperature.rs - Updated with serde support
#![cfg_attr(not(test), no_std)]

use serde::{Deserialize, Serialize};
use core::fmt;

#[cfg(test)]
use std::vec::Vec;
#[cfg(not(test))]
use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Temperature {
    celsius_tenths: i16,
}

impl Temperature {
    pub const fn from_celsius(celsius: f32) -> Self {
        Self {
            celsius_tenths: (celsius * 10.0) as i16,
        }
    }

    pub fn celsius(&self) -> f32 {
        self.celsius_tenths as f32 / 10.0
    }

    pub fn fahrenheit(&self) -> f32 {
        self.celsius() * 9.0 / 5.0 + 32.0
    }

    pub const fn is_overheating(&self) -> bool {
        self.celsius_tenths > 500  // > 50°C
    }

    // Helper for JSON serialization with nice format
    pub fn to_celsius_rounded(&self) -> f32 {
        (self.celsius() * 10.0).round() / 10.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub temperature: Temperature,
    pub timestamp_ms: u32,
    pub sensor_id: u8,  // Compact sensor identifier
}

impl TemperatureReading {
    pub fn new(temperature: Temperature, timestamp_ms: u32, sensor_id: u8) -> Self {
        Self {
            temperature,
            timestamp_ms,
            sensor_id,
        }
    }

    pub fn current_time(temperature: Temperature) -> Self {
        // In real implementation, this would get actual timestamp
        // For now, use a simple counter
        static mut TIMESTAMP: u32 = 0;
        unsafe {
            TIMESTAMP += 1000; // Simulate 1-second intervals
            Self::new(temperature, TIMESTAMP, 0)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperatureStats {
    pub count: u16,          // Use u16 to save space
    pub total_count: u32,
    pub min_celsius: f32,    // Store as f32 for JSON compatibility
    pub max_celsius: f32,
    pub avg_celsius: f32,
    pub timestamp_ms: u32,
}

impl TemperatureStats {
    pub fn from_buffer<const N: usize>(
        buffer: &TemperatureBuffer<N>,
        timestamp_ms: u32
    ) -> Option<Self> {
        if buffer.len() == 0 {
            return None;
        }

        let min = buffer.min()?.celsius();
        let max = buffer.max()?.celsius();
        let avg = buffer.average()?.celsius();

        Some(Self {
            count: buffer.len() as u16,
            total_count: buffer.total_readings(),
            min_celsius: min,
            max_celsius: max,
            avg_celsius: avg,
            timestamp_ms,
        })
    }
}
```

### JSON Serialization with serde-json-core

For IoT integration, JSON is widely supported but needs special handling in no_std:

```rust
// src/communication.rs - JSON communication module
#![cfg_attr(not(test), no_std)]

use heapless::{String, Vec};
use serde::{Deserialize, Serialize};
use serde_json_core;

use crate::temperature::{Temperature, TemperatureReading, TemperatureStats};

/// Commands that can be sent to the temperature monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    GetStatus,
    GetLatestReading,
    GetStats,
    SetSampleRate { rate_hz: u8 },
    SetThreshold { threshold_celsius: f32 },
    Reset,
}

/// Responses from the temperature monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Status {
        uptime_ms: u32,
        sample_rate_hz: u8,
        threshold_celsius: f32,
        buffer_usage: u8,  // Percentage full
    },
    Reading(TemperatureReading),
    Stats(TemperatureStats),
    SampleRateSet(u8),
    ThresholdSet(f32),
    ResetComplete,
    Error { code: u8, message: String<32> },
}

impl Response {
    pub fn error(code: u8, message: &str) -> Self {
        let mut error_message = String::new();
        error_message.push_str(message).ok();
        Self::Error {
            code,
            message: error_message,
        }
    }
}

/// Communication handler for temperature monitor
pub struct TemperatureComm {
    sample_rate_hz: u8,
    threshold_celsius: f32,
    start_time_ms: u32,
}

impl TemperatureComm {
    pub const fn new() -> Self {
        Self {
            sample_rate_hz: 1,  // 1 Hz default
            threshold_celsius: 35.0,
            start_time_ms: 0,
        }
    }

    pub fn init(&mut self, start_time_ms: u32) {
        self.start_time_ms = start_time_ms;
    }

    /// Process a command and return appropriate response
    pub fn process_command<const N: usize>(
        &mut self,
        command: Command,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> Response {
        match command {
            Command::GetStatus => {
                let uptime = current_time_ms.saturating_sub(self.start_time_ms);
                let buffer_usage = if buffer.capacity() > 0 {
                    ((buffer.len() * 100) / buffer.capacity()) as u8
                } else {
                    0
                };

                Response::Status {
                    uptime_ms: uptime,
                    sample_rate_hz: self.sample_rate_hz,
                    threshold_celsius: self.threshold_celsius,
                    buffer_usage,
                }
            }

            Command::GetLatestReading => {
                if let Some(temp) = buffer.latest() {
                    let reading = TemperatureReading::new(temp, current_time_ms, 0);
                    Response::Reading(reading)
                } else {
                    Response::error(1, "No readings available")
                }
            }

            Command::GetStats => {
                if let Some(stats) = TemperatureStats::from_buffer(buffer, current_time_ms) {
                    Response::Stats(stats)
                } else {
                    Response::error(2, "No data for statistics")
                }
            }

            Command::SetSampleRate { rate_hz } => {
                if rate_hz > 0 && rate_hz <= 10 {
                    self.sample_rate_hz = rate_hz;
                    Response::SampleRateSet(rate_hz)
                } else {
                    Response::error(3, "Rate must be 1-10 Hz")
                }
            }

            Command::SetThreshold { threshold_celsius } => {
                if threshold_celsius > 0.0 && threshold_celsius < 100.0 {
                    self.threshold_celsius = threshold_celsius;
                    Response::ThresholdSet(threshold_celsius)
                } else {
                    Response::error(4, "Threshold must be 0-100°C")
                }
            }

            Command::Reset => {
                self.start_time_ms = current_time_ms;
                self.sample_rate_hz = 1;
                self.threshold_celsius = 35.0;
                Response::ResetComplete
            }
        }
    }

    /// Serialize response to JSON string for transmission
    pub fn response_to_json(&self, response: &Response) -> Result<String<512>, ()> {
        // Use heapless String with fixed capacity
        match serde_json_core::to_string::<_, 512>(response) {
            Ok(json) => Ok(json),
            Err(_) => Err(()),
        }
    }

    /// Deserialize command from JSON string
    pub fn json_to_command(&self, json: &str) -> Result<Command, ()> {
        match serde_json_core::from_str(json) {
            Ok(command) => Ok(command),
            Err(_) => Err(()),
        }
    }

    /// Create a status response as JSON
    pub fn status_json<const N: usize>(
        &self,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> String<256> {
        let status = self.process_command(
            Command::GetStatus,
            buffer,
            current_time_ms
        );

        self.response_to_json(&status)
            .unwrap_or_else(|_| {
                let mut error = String::new();
                error.push_str("{\"error\":\"serialization_failed\"}").ok();
                error
            })
    }

    /// Create latest reading as JSON
    pub fn reading_json<const N: usize>(
        &self,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> String<256> {
        let reading = self.process_command(
            Command::GetLatestReading,
            buffer,
            current_time_ms
        );

        self.response_to_json(&reading)
            .unwrap_or_else(|_| {
                let mut error = String::new();
                error.push_str("{\"error\":\"no_reading\"}").ok();
                error
            })
    }

    pub fn sample_rate(&self) -> u8 {
        self.sample_rate_hz
    }

    pub fn threshold(&self) -> f32 {
        self.threshold_celsius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temperature::TemperatureBuffer;

    #[test]
    fn test_json_serialization() {
        let temp = Temperature::from_celsius(23.5);
        let reading = TemperatureReading::new(temp, 1000, 0);

        // Test command serialization
        let command = Command::GetStatus;
        let json = serde_json_core::to_string::<_, 64>(&command).unwrap();
        assert_eq!(json, "\"GetStatus\"");

        // Test response serialization
        let response = Response::Reading(reading);
        let json = serde_json_core::to_string::<_, 256>(&response).unwrap();
        assert!(json.contains("Reading"));
        assert!(json.contains("23.5"));
    }

    #[test]
    fn test_command_processing() {
        let mut comm = TemperatureComm::new();
        comm.init(0);
        let buffer = TemperatureBuffer::<5>::new();

        // Test status command
        let status_resp = comm.process_command(Command::GetStatus, &buffer, 5000);
        if let Response::Status { uptime_ms, .. } = status_resp {
            assert_eq!(uptime_ms, 5000);
        } else {
            panic!("Expected status response");
        }

        // Test rate setting
        let rate_resp = comm.process_command(
            Command::SetSampleRate { rate_hz: 5 },
            &buffer,
            5000
        );
        assert!(matches!(rate_resp, Response::SampleRateSet(5)));
        assert_eq!(comm.sample_rate(), 5);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut comm = TemperatureComm::new();

        // Test command deserialization
        let json_cmd = "\"GetStatus\"";
        let command = comm.json_to_command(json_cmd).unwrap();
        assert!(matches!(command, Command::GetStatus));

        // Test response serialization
        let response = Response::ResetComplete;
        let json_resp = comm.response_to_json(&response).unwrap();
        assert_eq!(json_resp, "\"ResetComplete\"");
    }

    #[test]
    fn test_error_handling() {
        let mut comm = TemperatureComm::new();
        let buffer = TemperatureBuffer::<5>::new();

        // Test invalid sample rate
        let response = comm.process_command(
            Command::SetSampleRate { rate_hz: 20 },  // Invalid: too high
            &buffer,
            0
        );

        if let Response::Error { code, message } = response {
            assert_eq!(code, 3);
            assert!(message.contains("Rate must be"));
        } else {
            panic!("Expected error response");
        }
    }
}
```

### Binary Serialization with postcard

For bandwidth-constrained applications, binary serialization is more efficient:

```rust
// src/binary_comm.rs - Binary communication with postcard
#![cfg_attr(not(test), no_std)]

use heapless::Vec;
use serde::{Deserialize, Serialize};
use postcard;

use crate::communication::{Command, Response};

/// Binary communication handler
pub struct BinaryComm;

impl BinaryComm {
    /// Serialize command to binary format
    pub fn command_to_binary(command: &Command) -> Result<Vec<u8, 64>, postcard::Error> {
        postcard::to_vec(command)
    }

    /// Deserialize command from binary format
    pub fn binary_to_command(data: &[u8]) -> Result<Command, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Serialize response to binary format
    pub fn response_to_binary(response: &Response) -> Result<Vec<u8, 256>, postcard::Error> {
        postcard::to_vec(response)
    }

    /// Deserialize response from binary format
    pub fn binary_to_response(data: &[u8]) -> Result<Response, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Get size of serialized command
    pub fn command_size(command: &Command) -> usize {
        Self::command_to_binary(command)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get size of serialized response
    pub fn response_size(response: &Response) -> usize {
        Self::response_to_binary(response)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temperature::{Temperature, TemperatureReading};

    #[test]
    fn test_binary_command_serialization() {
        let command = Command::SetSampleRate { rate_hz: 5 };

        // Serialize to binary
        let binary = BinaryComm::command_to_binary(&command).unwrap();

        // Deserialize back
        let deserialized = BinaryComm::binary_to_command(&binary).unwrap();

        if let Command::SetSampleRate { rate_hz } = deserialized {
            assert_eq!(rate_hz, 5);
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    fn test_binary_response_serialization() {
        let temp = Temperature::from_celsius(25.0);
        let reading = TemperatureReading::new(temp, 1000, 0);
        let response = Response::Reading(reading);

        // Serialize to binary
        let binary = BinaryComm::response_to_binary(&response).unwrap();

        // Should be much smaller than JSON
        println!("Binary size: {} bytes", binary.len());
        assert!(binary.len() < 20); // Much smaller than JSON

        // Deserialize back
        let deserialized = BinaryComm::binary_to_response(&binary).unwrap();

        if let Response::Reading(r) = deserialized {
            assert!((r.temperature.celsius() - 25.0).abs() < 0.1);
            assert_eq!(r.timestamp_ms, 1000);
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    fn test_size_comparison() {
        let temp = Temperature::from_celsius(23.5);
        let reading = TemperatureReading::new(temp, 1000, 0);
        let response = Response::Reading(reading);

        // Binary size
        let binary_size = BinaryComm::response_size(&response);

        // JSON size (approximate)
        let json = serde_json_core::to_string::<_, 256>(&response).unwrap();
        let json_size = json.len();

        println!("Binary: {} bytes, JSON: {} bytes", binary_size, json_size);
        println!("Binary is {}% smaller", ((json_size - binary_size) * 100) / json_size);

        assert!(binary_size < json_size);
        assert!(binary_size < 16); // Binary should be very compact
    }
}
```

## Integrating Communication with ESP32-C3

Let's update our main application to use these communication capabilities:

```rust
// src/bin/main.rs - ESP32 temperature monitor with communication
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

// Use the communication library types
use chapter23_communication::{Temperature, TemperatureBuffer, Command, TemperatureComm};

const BUFFER_SIZE: usize = 20;
const SAMPLE_INTERVAL_MS: u64 = 1000; // 1 second

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("💥 SYSTEM PANIC: {}", info);
    loop {}
}

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

    // Create fixed-capacity temperature buffer
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();

    // Initialize communication handler
    let mut comm = TemperatureComm::new();
    comm.init(0);

    // Startup messages with JSON communication
    esp_println::println!("🌡️ ESP32-C3 Temperature Monitor with Communication");
    esp_println::println!("📊 Buffer capacity: {} readings", temp_buffer.capacity());
    esp_println::println!("📡 JSON communication enabled");
    esp_println::println!("🔧 Send commands: status, reading, stats, reset");
    esp_println::println!();

    // Demonstrate initial JSON output
    let status_json = comm.status_json(&temp_buffer, 0);
    esp_println::println!("INITIAL_STATUS: {}", status_json);
    esp_println::println!();

    let mut reading_count = 0u32;

    // Main monitoring loop
    loop {
        // Get current timestamp (simplified)
        let current_time = reading_count * SAMPLE_INTERVAL_MS as u32;

        // Small stabilization delay (recommended by ESP-HAL)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_micros(200) {}

        // Read temperature from built-in sensor
        let esp_temperature = temp_sensor.get_temperature();
        let temp_celsius = esp_temperature.to_celsius();
        let temperature = Temperature::from_celsius(temp_celsius);

        // Store in buffer
        temp_buffer.push(temperature);
        reading_count += 1;

        // LED status based on temperature
        if temperature.is_overheating() {
            // Rapid triple blink for overheating (>50°C)
            for _ in 0..3 {
                led.set_high();
                let blink_start = Instant::now();
                while blink_start.elapsed() < Duration::from_millis(100) {}
                led.set_low();
                let blink_start = Instant::now();
                while blink_start.elapsed() < Duration::from_millis(100) {}
            }
        } else if !temperature.is_normal_range() {
            // Double blink for out of normal range (not 15-35°C)
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
            // Single blink for normal temperature
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(200) {}
            led.set_low();
        }

        // Output structured JSON data
        let reading_json = comm.latest_reading_json(&temp_buffer, current_time);
        esp_println::println!("READING: {}", reading_json);

        // Print statistics every 5 readings
        if reading_count % 5 == 0 {
            let stats_json = comm.stats_json(&temp_buffer, current_time);
            esp_println::println!("STATS: {}", stats_json);

            let status_json = comm.status_json(&temp_buffer, current_time);
            esp_println::println!("STATUS: {}", status_json);
            esp_println::println!();
        }

        // Wait for next sample
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(SAMPLE_INTERVAL_MS) {}
    }
}
```

## Example Output

When you run this on the ESP32-C3, you'll see structured JSON output like:

```json
🌡️ ESP32-C3 Temperature Monitor with Communication
📊 Buffer capacity: 20 readings
📡 JSON communication enabled
🔧 Send commands: status, reading, stats, reset

INITIAL_STATUS: {"Status":{"uptime_ms":0,"sample_rate_hz":1,"threshold_celsius":35.0,"buffer_usage":0}}

READING: {"Reading":{"temperature":{"celsius_tenths":523},"timestamp_ms":1000,"sensor_id":0}}
READING: {"Reading":{"temperature":{"celsius_tenths":524},"timestamp_ms":2000,"sensor_id":0}}
READING: {"Reading":{"temperature":{"celsius_tenths":521},"timestamp_ms":3000,"sensor_id":0}}
READING: {"Reading":{"temperature":{"celsius_tenths":522},"timestamp_ms":4000,"sensor_id":0}}
READING: {"Reading":{"temperature":{"celsius_tenths":523},"timestamp_ms":5000,"sensor_id":0}}

STATS: {"Stats":{"count":5,"total_count":5,"average":{"celsius_tenths":523},"min":{"celsius_tenths":521},"max":{"celsius_tenths":524},"timestamp_ms":5000}}
STATUS: {"Status":{"uptime_ms":5000,"sample_rate_hz":1,"threshold_celsius":35.0,"buffer_usage":25}}
```

## Building and Testing

```bash
# Run tests on desktop
cargo test

# Build and flash to ESP32-C3 (recommended)
cargo run --release --features embedded

# Alternative: Build then flash separately
cargo build --release --target riscv32imc-unknown-none-elf --features embedded
cargo espflash flash target/riscv32imc-unknown-none-elf/release/chapter23_communication
```

## Exercise: JSON Temperature Communication System


Build a complete JSON communication system for your temperature monitor.

### Requirements

1. **JSON Output**: Send temperature readings as JSON over serial every second
2. **Command Processing**: Parse and respond to JSON commands
3. **Status Reporting**: Provide system status via JSON
4. **Statistics Export**: Export temperature statistics in JSON format
5. **Error Handling**: Handle serialization errors gracefully

### Starting Project Structure

Create these files:

```rust
// src/temperature.rs - Add Serde support to existing types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Temperature {
    celsius_tenths: i16,
}

// TODO: Add Serde derives to TemperatureBuffer
// TODO: Create TemperatureReading struct with timestamp
```

```rust
// src/communication.rs - Create command/response system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    GetStatus,
    GetLatestReading,
    GetStats,
    SetSampleRate { rate_hz: u8 },
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    // TODO: Define response types
}

pub struct TemperatureComm {
    // TODO: Implement communication handler
}
```

### Implementation Tasks

1. **Add Serde Support**:
   - Add `Serialize, Deserialize` to Temperature struct
   - Create TemperatureReading with timestamp
   - Update Cargo.toml with serde dependencies

2. **Create Command System**:
   - Define Command enum for incoming commands
   - Define Response enum for outgoing responses
   - Implement command processing logic

3. **JSON Communication**:
   - Serialize responses to JSON strings
   - Deserialize commands from JSON
   - Handle serialization errors gracefully

4. **Integration**:
   - Update main loop to output JSON readings
   - Add command demonstration
   - Test JSON format with serial monitor

### Success Criteria

- [ ] Program compiles without warnings
- [ ] Temperature readings output as valid JSON
- [ ] Commands processed and responses sent as JSON
- [ ] Statistics exported in JSON format
- [ ] Serial output shows structured data
- [ ] No panics on malformed input

### Expected JSON Output

```json
🌡️ ESP32-C3 Temperature Monitor with Communication

READING: {"Reading":{"temperature":{"celsius_tenths":523},"timestamp_ms":1000,"sensor_id":0}}
STATUS: {"Status":{"uptime_ms":1000,"sample_rate_hz":1,"threshold_celsius":52.0,"buffer_usage":5}}
STATS: {"Stats":{"count":5,"average":{"celsius_tenths":522},"min":{"celsius_tenths":520},"max":{"celsius_tenths":525}}}

Command Response: {"SampleRateSet":2}
```

### Testing Commands

```bash
# Run tests first
./test.sh

# Build and flash
cargo run --release

# Monitor output
cargo espflash monitor
```

You can test commands by sending JSON to the serial interface:
- `"GetStatus"`
- `{"SetSampleRate":{"rate_hz":2}}`
- `"Reset"`

### Extension Challenges

1. **Command Input**: Read commands from serial input
2. **Binary Protocol**: Compare JSON vs postcard serialization
3. **Compression**: Implement message compression for efficiency
4. **Authentication**: Add simple command authentication
5. **Batch Operations**: Send multiple readings in one JSON message

### Troubleshooting

**Serialization Errors:**
- Check that all types implement Serde traits
- Ensure fixed-size strings for heapless compatibility
- Use `serde-json-core` instead of `serde_json` for no_std

**JSON Format Issues:**
- Validate JSON with online tools
- Use pretty-printing for debugging
- Check string buffer sizes are sufficient

**Memory Errors:**
- Monitor stack usage during JSON operations
- Use smaller buffer sizes if memory is limited
- Consider streaming large responses

## Key Communication Patterns Learned

✅ **Serde Integration**: Add serialization support to embedded types with `#[derive(Serialize, Deserialize)]`
✅ **Fixed-size Collections**: Use `heapless::String` and `heapless::Vec` for JSON without heap allocation
✅ **Command/Response Protocol**: Design structured interfaces for remote control
✅ **Error Handling**: Handle serialization errors gracefully in resource-constrained environments
✅ **JSON vs Binary**: Understand trade-offs between readability and efficiency

**Next**: In Chapter 24, we'll integrate all these components into a production-ready system with proper error handling and deployment strategies.
