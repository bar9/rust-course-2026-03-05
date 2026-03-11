# Chapter 22: Testing Embedded Code

## Learning Objectives
This chapter covers:
- Test no_std code on your desktop using conditional compilation
- Create hardware abstraction layers (HAL) for testable embedded code
- Write unit tests for temperature data structures and algorithms
- Mock hardware dependencies for isolated testing
- Use integration tests to validate ESP32-C3 behavior
- Debug embedded code efficiently using both tests and hardware

## Task: Test Embedded Code on Desktop

Building on chapters 20-21, where we created temperature monitoring with data structures, now we need to ensure our code is robust and correct.

**Your Mission:**
1. **Test no_std code** on desktop using conditional compilation
2. **Mock hardware dependencies** (temperature sensor, GPIO) for isolated testing
3. **Validate algorithms** (circular buffer, statistics) without hardware
4. **Create testable abstractions** that work both embedded and on desktop
5. **Add comprehensive test coverage** including edge cases and error conditions

**Why This Matters:**
- **Faster development**: Test business logic without flashing hardware
- **Better reliability**: Catch bugs before they reach embedded systems
- **Easier debugging**: Desktop tools are more powerful than embedded debuggers
- **Continuous Integration**: Automated testing in CI/CD pipelines

**The Challenge:**
- Code runs on ESP32-C3 (RISC-V), but tests run on desktop (x86/ARM)
- No access to GPIO, sensors, or timers in test environment
- Need to test `no_std` code using `std` tools

## Conditional Compilation Strategy

The key insight: **Your business logic doesn't need hardware to be tested.**

```rust
// This works in both embedded and test environments
#[cfg(test)]
use std::vec::Vec;  // Tests can use std

#[cfg(not(test))]
use heapless::Vec;  // Embedded uses heapless

// The rest of your code works with either Vec!
fn calculate_average(readings: &[f32]) -> Option<f32> {
    if readings.is_empty() {
        return None;
    }
    let sum: f32 = readings.iter().sum();
    Some(sum / readings.len() as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_average_calculation() {
        let readings = vec![20.0, 25.0, 30.0];  // std::vec in tests
        let avg = calculate_average(&readings).unwrap();
        assert!((avg - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_readings() {
        let readings = vec![];
        assert_eq!(calculate_average(&readings), None);
    }
}
```

## Project Setup for Testable Embedded Code

First, let's set up our project to support both embedded and testing targets:

```toml
[package]
name = "chapter22_testing"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"

[[bin]]
name = "chapter22_testing"
path = "./src/bin/main.rs"

[lib]
name = "chapter22_testing"
path = "src/lib.rs"

[dependencies]
# Only include ESP dependencies when not testing
esp-hal = { version = "1.0.0", features = ["esp32c3", "unstable"], optional = true }
heapless = "0.8"
esp-println = { version = "0.16", features = ["esp32c3"], optional = true }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"], optional = true }
critical-section = "1.2.0"

[features]
default = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]
embedded = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]

[profile.dev]
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

**Key Setup Details:**
- **Optional ESP dependencies**: Only included when building for embedded target
- **Feature flags**: Control when ESP-specific code is compiled
- **Library + Binary**: Allows testing the library separately from main embedded binary

## Testing the Temperature Types from Chapter 21

Let's add comprehensive tests to our embedded temperature code:

```rust
// src/lib.rs - Testable embedded temperature library
#![cfg_attr(not(test), no_std)]

use core::fmt;

// Conditional imports for testing
#[cfg(test)]
use std::vec::Vec;
#[cfg(not(test))]
use heapless::Vec;

/// Temperature reading optimized for embedded systems
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    // Store as i16 to save memory (16-bit vs 32-bit f32)
    // Resolution: 0.1°C, Range: -3276.8°C to +3276.7°C
    pub(crate) celsius_tenths: i16,
}

impl Temperature {
    /// Create temperature from Celsius value
    pub const fn from_celsius(celsius: f32) -> Self {
        Self {
            celsius_tenths: (celsius * 10.0) as i16,
        }
    }

    /// Get temperature as Celsius f32
    pub fn celsius(&self) -> f32 {
        self.celsius_tenths as f32 / 10.0
    }

    pub fn fahrenheit(&self) -> f32 {
        self.celsius() * 9.0 / 5.0 + 32.0
    }

    pub const fn is_overheating(&self) -> bool {
        self.celsius_tenths > 500  // > 50°C
    }

    pub const fn is_normal_range(&self) -> bool {
        self.celsius_tenths >= 150 && self.celsius_tenths <= 350  // 15-35°C
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius())
    }
}

pub struct TemperatureBuffer<const N: usize> {
    #[cfg(test)]
    readings: Vec<Temperature>,  // std::vec for tests

    #[cfg(not(test))]
    readings: Vec<Temperature, N>,  // heapless::vec for embedded

    total_readings: u32,
}

impl<const N: usize> TemperatureBuffer<N> {
    pub const fn new() -> Self {
        Self {
            readings: Vec::new(),
            total_readings: 0,
        }
    }

    pub fn push(&mut self, temperature: Temperature) {
        #[cfg(test)]
        {
            // In tests, we can grow unlimited
            if self.readings.len() >= N {
                self.readings.remove(0);
            }
            self.readings.push(temperature);
        }

        #[cfg(not(test))]
        {
            // In embedded, handle fixed capacity with circular buffer
            if self.readings.len() < N {
                self.readings.push(temperature).ok();
            } else {
                // Use circular indexing (O(1) vs remove(0) which is O(n))
                let oldest_index = (self.total_readings as usize) % N;
                self.readings[oldest_index] = temperature;
            }
        }

        self.total_readings += 1;
    }

    pub fn len(&self) -> usize {
        self.readings.len()
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn latest(&self) -> Option<Temperature> {
        self.readings.last().copied()
    }

    pub fn average(&self) -> Option<Temperature> {
        if self.readings.is_empty() {
            return None;
        }

        let sum: i32 = self.readings.iter()
            .map(|t| t.celsius_tenths as i32)
            .sum();

        let avg_tenths = sum / self.readings.len() as i32;
        Some(Temperature { celsius_tenths: avg_tenths as i16 })
    }

    pub fn min(&self) -> Option<Temperature> {
        self.readings.iter()
            .min_by_key(|t| t.celsius_tenths)
            .copied()
    }

    pub fn max(&self) -> Option<Temperature> {
        self.readings.iter()
            .max_by_key(|t| t.celsius_tenths)
            .copied()
    }

    pub fn total_readings(&self) -> u32 {
        self.total_readings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_creation_and_conversion() {
        let temp = Temperature::from_celsius(23.5);

        // Test precision
        assert!((temp.celsius() - 23.5).abs() < 0.1);

        // Test Fahrenheit conversion
        let fahrenheit = temp.fahrenheit();
        assert!((fahrenheit - 74.3).abs() < 0.1);

        // Test memory efficiency
        assert_eq!(core::mem::size_of::<Temperature>(), 2);
    }

    #[test]
    fn test_temperature_ranges() {
        let normal = Temperature::from_celsius(25.0);
        assert!(normal.is_normal_range());
        assert!(!normal.is_overheating());

        let hot = Temperature::from_celsius(55.0);
        assert!(!hot.is_normal_range());
        assert!(hot.is_overheating());

        let cold = Temperature::from_celsius(5.0);
        assert!(!cold.is_normal_range());
        assert!(!cold.is_overheating());
    }

    #[test]
    fn test_temperature_edge_cases() {
        // Test extreme values
        let extreme_hot = Temperature::from_celsius(3276.0);
        let extreme_cold = Temperature::from_celsius(-3276.0);

        assert!(extreme_hot.celsius() > 3000.0);
        assert!(extreme_cold.celsius() < -3000.0);
    }

    #[test]
    fn test_buffer_basic_operations() {
        let mut buffer = TemperatureBuffer::<5>::new();

        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 5);
        assert_eq!(buffer.latest(), None);

        // Add some readings
        buffer.push(Temperature::from_celsius(20.0));
        buffer.push(Temperature::from_celsius(25.0));
        buffer.push(Temperature::from_celsius(30.0));

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.total_readings(), 3);
        assert_eq!(buffer.latest().unwrap().celsius(), 30.0);
    }

    #[test]
    fn test_buffer_circular_behavior() {
        let mut buffer = TemperatureBuffer::<3>::new();

        // Fill buffer exactly
        buffer.push(Temperature::from_celsius(10.0));
        buffer.push(Temperature::from_celsius(20.0));
        buffer.push(Temperature::from_celsius(30.0));
        assert_eq!(buffer.len(), 3);

        // Add one more - should overwrite oldest
        buffer.push(Temperature::from_celsius(40.0));

        assert_eq!(buffer.len(), 3);  // Still full
        assert_eq!(buffer.total_readings(), 4);  // But total increased

        // First reading (10.0) should be gone
        assert_eq!(buffer.min().unwrap().celsius(), 20.0);  // Min is now 20
        assert_eq!(buffer.max().unwrap().celsius(), 40.0);  // Max is 40
    }

    #[test]
    fn test_buffer_statistics() {
        let mut buffer = TemperatureBuffer::<10>::new();

        // Add test data: 20, 21, 22, 23, 24
        for i in 0..5 {
            buffer.push(Temperature::from_celsius(20.0 + i as f32));
        }

        let avg = buffer.average().unwrap();
        assert!((avg.celsius() - 22.0).abs() < 0.1);

        assert_eq!(buffer.min().unwrap().celsius(), 20.0);
        assert_eq!(buffer.max().unwrap().celsius(), 24.0);
    }

    #[test]
    fn test_buffer_empty_statistics() {
        let buffer = TemperatureBuffer::<5>::new();

        assert_eq!(buffer.average(), None);
        assert_eq!(buffer.min(), None);
        assert_eq!(buffer.max(), None);
    }

    #[test]
    fn test_buffer_single_reading() {
        let mut buffer = TemperatureBuffer::<5>::new();
        buffer.push(Temperature::from_celsius(25.0));

        let avg = buffer.average().unwrap();
        assert_eq!(avg.celsius(), 25.0);
        assert_eq!(buffer.min().unwrap().celsius(), 25.0);
        assert_eq!(buffer.max().unwrap().celsius(), 25.0);
    }

    #[test]
    fn test_temperature_display() {
        let temp = Temperature::from_celsius(23.7);
        let display_str = format!("{}", temp);
        assert_eq!(display_str, "23.7°C");
    }

    #[test]
    fn test_memory_usage() {
        // Verify our types are memory efficient
        let temp_size = core::mem::size_of::<Temperature>();
        let buffer_size = core::mem::size_of::<TemperatureBuffer<20>>();

        println!("Temperature size: {} bytes", temp_size);
        println!("Buffer size (20 readings): {} bytes", buffer_size);

        assert_eq!(temp_size, 2);  // Should be exactly 2 bytes
        // Buffer size will be larger in tests due to std::Vec
    }
}
```

## Hardware Abstraction Layer (HAL) for Testing

To test hardware-dependent code, create an abstraction layer:

```rust
// src/hal.rs - Hardware abstraction layer
#[cfg(test)]
use std::cell::RefCell;

/// Trait for reading temperature from any source
pub trait TemperatureSensorHal {
    type Error;

    fn read_celsius(&mut self) -> Result<f32, Self::Error>;
    fn sensor_id(&self) -> &str;
}

/// Real ESP32 temperature sensor implementation
#[cfg(not(test))]
pub struct Esp32TemperatureSensor {
    sensor: esp_hal::temperature_sensor::TemperatureSensor,
}

#[cfg(not(test))]
impl Esp32TemperatureSensor {
    pub fn new(sensor: esp_hal::temperature_sensor::TemperatureSensor) -> Self {
        Self { sensor }
    }
}

#[cfg(not(test))]
impl TemperatureSensorHal for Esp32TemperatureSensor {
    type Error = ();

    fn read_celsius(&mut self) -> Result<f32, Self::Error> {
        Ok(self.sensor.read_celsius())
    }

    fn sensor_id(&self) -> &str {
        "ESP32-C3 Built-in"
    }
}

/// Mock sensor for testing
#[cfg(test)]
pub struct MockTemperatureSensor {
    temperatures: RefCell<Vec<f32>>,
    current_index: RefCell<usize>,
    id: String,
}

#[cfg(test)]
impl MockTemperatureSensor {
    pub fn new(id: String) -> Self {
        Self {
            temperatures: RefCell::new(vec![25.0]), // Default temperature
            current_index: RefCell::new(0),
            id,
        }
    }

    pub fn set_temperatures(&self, temps: Vec<f32>) {
        *self.temperatures.borrow_mut() = temps;
        *self.current_index.borrow_mut() = 0;
    }

    pub fn set_single_temperature(&self, temp: f32) {
        *self.temperatures.borrow_mut() = vec![temp];
        *self.current_index.borrow_mut() = 0;
    }
}

#[cfg(test)]
impl TemperatureSensorHal for MockTemperatureSensor {
    type Error = &'static str;

    fn read_celsius(&mut self) -> Result<f32, Self::Error> {
        let temps = self.temperatures.borrow();
        let mut index = self.current_index.borrow_mut();

        if temps.is_empty() {
            return Err("No temperature data configured");
        }

        let temp = temps[*index];
        *index = (*index + 1) % temps.len();  // Cycle through temperatures

        Ok(temp)
    }

    fn sensor_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_sensor_single_value() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string());
        sensor.set_single_temperature(23.5);

        let temp1 = sensor.read_celsius().unwrap();
        let temp2 = sensor.read_celsius().unwrap();

        assert_eq!(temp1, 23.5);
        assert_eq!(temp2, 23.5);  // Should repeat same value
        assert_eq!(sensor.sensor_id(), "test-sensor");
    }

    #[test]
    fn test_mock_sensor_cycling_values() {
        let mut sensor = MockTemperatureSensor::new("cycle-test".to_string());
        sensor.set_temperatures(vec![20.0, 25.0, 30.0]);

        assert_eq!(sensor.read_celsius().unwrap(), 20.0);
        assert_eq!(sensor.read_celsius().unwrap(), 25.0);
        assert_eq!(sensor.read_celsius().unwrap(), 30.0);
        assert_eq!(sensor.read_celsius().unwrap(), 20.0);  // Cycles back
    }

    #[test]
    fn test_mock_sensor_empty_data() {
        let mut sensor = MockTemperatureSensor::new("empty-test".to_string());
        sensor.set_temperatures(vec![]);

        assert!(sensor.read_celsius().is_err());
    }
}
```

## Integration Testing on Hardware

For testing actual hardware behavior, create integration tests:

```rust
// tests/integration_tests.rs - Hardware integration tests

use temp_monitor::{Temperature, TemperatureBuffer};

#[cfg(target_arch = "riscv32")]  // Only run on ESP32
#[test]
fn test_hardware_sensor_reading() {
    // This test would run on actual ESP32 hardware
    // (Implementation depends on test framework like defmt-test)
}

// Cross-platform integration tests
#[test]
fn test_temperature_monitor_workflow() {
    // Test the complete workflow without hardware
    let mut buffer = TemperatureBuffer::<5>::new();

    // Simulate sensor readings
    let readings = vec![22.0, 23.0, 24.0, 25.0, 26.0, 27.0];

    for temp_celsius in readings {
        let temp = Temperature::from_celsius(temp_celsius);
        buffer.push(temp);
    }

    // Verify circular buffer behavior
    assert_eq!(buffer.len(), 5);
    assert_eq!(buffer.total_readings(), 6);

    // Verify statistics
    let stats = buffer.average().unwrap();
    assert!((stats.celsius() - 25.0).abs() < 0.1);  // Should be ~25°C average

    assert_eq!(buffer.min().unwrap().celsius(), 23.0);  // Oldest (22.0) was overwritten
    assert_eq!(buffer.max().unwrap().celsius(), 27.0);
}

#[test]
fn test_overheating_detection() {
    let normal_temp = Temperature::from_celsius(25.0);
    let hot_temp = Temperature::from_celsius(55.0);
    let very_hot_temp = Temperature::from_celsius(75.0);

    assert!(!normal_temp.is_overheating());
    assert!(hot_temp.is_overheating());
    assert!(very_hot_temp.is_overheating());

    // Test with buffer
    let mut buffer = TemperatureBuffer::<3>::new();
    buffer.push(normal_temp);
    buffer.push(hot_temp);
    buffer.push(very_hot_temp);

    // Should average to overheating territory
    let avg = buffer.average().unwrap();
    assert!(avg.is_overheating());
}
```

## Running Tests

### Desktop Tests

```bash
# Run all tests on desktop
cargo test

# Run specific test module
cargo test temperature::tests

# Run with output
cargo test -- --nocapture

# Run tests in verbose mode
cargo test --verbose
```

### Test Output Example

```
$ cargo test
   Compiling temp_monitor v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 1.23s
     Running unittests src/lib.rs

running 12 tests
test temperature::tests::test_temperature_creation_and_conversion ... ok
test temperature::tests::test_temperature_ranges ... ok
test temperature::tests::test_temperature_edge_cases ... ok
test temperature::tests::test_buffer_basic_operations ... ok
test temperature::tests::test_buffer_circular_behavior ... ok
test temperature::tests::test_buffer_statistics ... ok
test temperature::tests::test_buffer_empty_statistics ... ok
test temperature::tests::test_buffer_single_reading ... ok
test temperature::tests::test_temperature_display ... ok
test temperature::tests::test_memory_usage ... ok
test hal::tests::test_mock_sensor_single_value ... ok
test hal::tests::test_mock_sensor_cycling_values ... ok

     Running tests/integration_tests.rs

running 2 tests
test test_temperature_monitor_workflow ... ok
test test_overheating_detection ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Building for Embedded Target

When you're ready to test on hardware:

```bash
# Build and flash to ESP32-C3 (recommended)
cargo run --release --features embedded

# Alternative: Build then flash separately
cargo build --release --target riscv32imc-unknown-none-elf --features embedded
cargo espflash flash target/riscv32imc-unknown-none-elf/release/chapter22_testing
```

## Key Testing Patterns Learned

✅ **Conditional Compilation**: Use `#[cfg(test)]` and `#[cfg(not(test))]` to create testable embedded code
✅ **Hardware Abstraction**: Create traits that can be mocked for testing hardware dependencies
✅ **Memory Efficiency Testing**: Verify size and memory usage in unit tests
✅ **Edge Case Testing**: Test boundary conditions like buffer overflow, empty data, extreme values
✅ **Integration Testing**: Test complete workflows without hardware dependencies

**Next**: In Chapter 23, we'll add communication capabilities to send structured data like JSON over serial connections.

### Hardware Validation

```bash
# Build and flash test version (recommended)
cargo run --release --features test-on-hardware

# Alternative: Build then flash
cargo build --release --features test-on-hardware
cargo espflash flash target/riscv32imc-unknown-none-elf/release/temp_monitor

# Expected hardware output:
# Running hardware validation...
# ✅ Temperature sensor responding
# ✅ LED control working
# ✅ Buffer operations correct
# ✅ Statistics calculation accurate
# Hardware tests passed!
```

## Test-Driven Development for Embedded

Use TDD to develop new features:

```rust
// 1. Write failing test first
#[test]
fn test_temperature_trend_detection() {
    let mut buffer = TemperatureBuffer::<5>::new();

    // Rising temperature trend
    buffer.push(Temperature::from_celsius(20.0));
    buffer.push(Temperature::from_celsius(22.0));
    buffer.push(Temperature::from_celsius(24.0));

    // This will fail until we implement it
    assert_eq!(buffer.trend(), Some(TemperatureTrend::Rising));
}

// 2. Implement minimal code to make test pass
#[derive(Debug, PartialEq)]
pub enum TemperatureTrend {
    Rising,
    Falling,
    Stable,
}

impl<const N: usize> TemperatureBuffer<N> {
    pub fn trend(&self) -> Option<TemperatureTrend> {
        if self.readings.len() < 3 {
            return None;
        }

        // Simple trend detection - compare first and last
        let first = self.readings.first().unwrap().celsius_tenths;
        let last = self.readings.last().unwrap().celsius_tenths;

        if last > first + 20 {  // More than 2°C increase
            Some(TemperatureTrend::Rising)
        } else if last < first - 20 {  // More than 2°C decrease
            Some(TemperatureTrend::Falling)
        } else {
            Some(TemperatureTrend::Stable)
        }
    }
}

// 3. Refactor and add more test cases
```

## Exercise: Add Comprehensive Testing


Add a full test suite to your temperature monitoring code.

### Requirements

1. **Unit Tests**: Test all temperature and buffer functions
2. **Mock Hardware**: Create testable hardware abstraction
3. **Integration Tests**: Test complete workflows
4. **Error Cases**: Test edge cases and error conditions
5. **Performance**: Verify memory usage and efficiency

### Tasks

1. **Setup Test Environment**:
   - Add conditional compilation for tests
   - Create `src/lib.rs` to expose modules for testing
   - Update `Cargo.toml` with test dependencies

2. **Unit Tests for Temperature**:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_temperature_precision() {
           // TODO: Test 0.1°C precision
       }

       #[test]
       fn test_conversion_roundtrip() {
           // TODO: celsius -> internal -> celsius should be stable
       }

       #[test]
       fn test_extreme_temperatures() {
           // TODO: Test very hot and cold values
       }
   }
   ```

3. **Unit Tests for Buffer**:
   ```rust
   #[test]
   fn test_buffer_capacity_limits() {
       // TODO: Test buffer behavior at capacity
   }

   #[test]
   fn test_statistics_accuracy() {
       // TODO: Verify min/max/average calculations
   }

   #[test]
   fn test_circular_replacement() {
       // TODO: Ensure oldest data is properly replaced
   }
   ```

4. **Hardware Abstraction Tests**:
   - Create mock sensor implementation
   - Test sensor trait with controlled data
   - Verify error handling

5. **Run and Validate**:
   - Execute test suite with `cargo test`
   - Verify all tests pass
   - Check test coverage

### Expected Test Results

```
running 15 tests
test temperature::tests::test_temperature_precision ... ok
test temperature::tests::test_conversion_roundtrip ... ok
test temperature::tests::test_extreme_temperatures ... ok
test temperature::tests::test_buffer_capacity_limits ... ok
test temperature::tests::test_statistics_accuracy ... ok
test temperature::tests::test_circular_replacement ... ok
test hal::tests::test_mock_sensor ... ok
test integration::test_complete_workflow ... ok
...

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Memory usage:
  Temperature: 2 bytes
  Buffer (20 readings): 86 bytes
  Total: 88 bytes ✅
```

### Success Criteria

- [ ] All unit tests pass on desktop
- [ ] Mock sensor provides controlled test data
- [ ] Integration tests verify complete workflows
- [ ] Edge cases are handled gracefully
- [ ] Memory usage is within expected bounds
- [ ] Tests run quickly (< 1 second total)

### Extension Challenges

1. **Property-Based Testing**: Use `quickcheck` to test with random data
2. **Benchmark Tests**: Measure performance of temperature calculations
3. **Hardware-in-the-Loop**: Run tests on actual ESP32 hardware
4. **Coverage Analysis**: Use `cargo tarpaulin` to measure test coverage
5. **Fuzzing**: Test with invalid input data

## Debugging Embedded Code

### Test-First Debugging

When hardware doesn't behave as expected:

1. **Write Test for Expected Behavior**:
   ```rust
   #[test]
   fn test_sensor_reading_should_be_realistic() {
       let reading = mock_esp32_reading(1500); // ADC value
       let temp = Temperature::from_sensor_raw(reading);
       assert!(temp.celsius() > 15.0 && temp.celsius() < 40.0);
   }
   ```

2. **Run Test on Desktop** to verify logic
3. **Compare with Hardware** output
4. **Identify Discrepancy** and fix

### Serial Debug Output

```rust
// Add debug output to embedded code
esp_println::println!("Debug: ADC raw = {}, converted = {}°C",
                      raw_value, temperature.celsius());

// Compare with test expectations
#[test]
fn test_debug_conversion() {
    let temp = Temperature::from_sensor_raw(1500);
    println!("Test: ADC raw = 1500, converted = {}°C", temp.celsius());
    // Should match hardware output
}
```

### Test-Driven Hardware Validation

```rust
#[cfg(feature = "hardware-test")]
pub fn validate_hardware() {
    // This function runs on hardware to validate assumptions
    let mut sensor = /* initialize real sensor */;

    for _ in 0..10 {
        let reading = sensor.read_celsius();
        esp_println::println!("Hardware reading: {:.1}°C", reading);

        // Sanity checks
        assert!(reading > -50.0 && reading < 100.0, "Reading out of range");
    }

    esp_println::println!("✅ Hardware validation passed");
}
```

## Key Takeaways

✅ **Conditional Compilation**: Use `#[cfg(test)]` to test no_std code on desktop

✅ **Hardware Abstraction**: Create traits to mock hardware dependencies

✅ **Test Structure**: Unit tests for logic, integration tests for workflows

✅ **TDD for Embedded**: Write tests first, even for hardware-dependent features

✅ **Debug Strategy**: Combine desktop tests with serial debugging on hardware

✅ **Performance Testing**: Verify memory usage and timing in tests

**Next**: In Chapter 23, we'll add communication capabilities to send our temperature data in structured formats like JSON and binary protocols.