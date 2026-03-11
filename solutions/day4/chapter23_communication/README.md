# Chapter 16: Data & Communication Solution

ESP32-C3 temperature monitoring with JSON communication capabilities.

## Features

- **Temperature Monitoring**: Uses ESP32-C3 built-in temperature sensor
- **JSON Communication**: Structured data output over USB Serial
- **Command Processing**: Handle status, readings, and stats requests
- **Serde Integration**: Serialize temperature data and commands
- **Memory Efficient**: Uses heapless collections, no heap allocation
- **LED Indicators**: Visual feedback for temperature ranges
- **Circular Buffer**: Fixed-capacity storage for temperature history

## Building and Running

### Flash to ESP32-C3

```bash
cargo run --release
```

This will build and flash the firmware to your ESP32-C3 device.

### Running Tests

```bash
./test.sh
```

The test script temporarily moves the embedded cargo config and build script to allow testing on the host machine.

### Manual Test Commands

```bash
# Run tests manually (requires moving .cargo and build.rs)
cargo test --lib --no-default-features

# Check compilation for ESP32-C3
cargo check --release
```

## Output Format

The device outputs structured JSON data:

```json
{"Reading":{"temperature":{"celsius_tenths":245},"timestamp_ms":1000,"sensor_id":0}}
{"Stats":{"count":5,"total_count":5,"min_celsius":24.1,"max_celsius":24.9,"avg_celsius":24.5,"timestamp_ms":5000}}
{"Status":{"uptime_ms":5000,"sample_rate_hz":1,"threshold_celsius":35.0,"buffer_usage":25}}
```

## Command System

The communication system supports these commands:

- `GetStatus` - System status and uptime
- `GetLatestReading` - Most recent temperature reading
- `GetStats` - Statistical summary of readings
- `SetSampleRate` - Change sampling rate (1-10 Hz)
- `SetThreshold` - Set temperature threshold
- `Reset` - Reset system configuration

## Architecture

- **src/temperature.rs**: Temperature types with serde support
- **src/communication.rs**: Command/response handling and JSON serialization
- **src/bin/main.rs**: ESP32-C3 application with sensor integration
- **test.sh**: Host-based testing script

## Dependencies

- `esp-hal`: ESP32-C3 hardware abstraction
- `serde`: Serialization framework (no_std)
- `serde-json-core`: JSON serialization for embedded
- `heapless`: Collections without allocation
- `esp-println`: Console output for ESP32

## Testing

The solution includes comprehensive tests covering:

- Temperature conversion and ranges
- Buffer operations and circular behavior
- JSON serialization/deserialization
- Command processing and error handling
- Serde round-trip validation