# Chapter 21: Embedded Foundations - no_std from the Start

## Learning Objectives
This chapter covers:
- Understand the difference between `core`, `alloc`, and `std` libraries
- Create temperature data structures that work in embedded environments
- Use heapless collections for fixed-capacity storage
- Implement const functions for compile-time configuration
- Build a circular buffer for continuous sensor data collection
- Calculate statistics without dynamic allocation

## Task: Build Memory-Efficient Temperature Storage

In Chapter 13, we successfully read temperature values from the ESP32-C3's built-in sensor. Now we need to build a system that can:

**Your Mission:**
1. **Store multiple readings** in a fixed-size circular buffer
2. **Calculate statistics** (average, min, max) without heap allocation
3. **Use only 2 bytes per temperature** reading (vs 4 bytes for f32)
4. **Handle buffer overflow** gracefully with circular behavior
5. **Monitor memory usage** and system performance

**Why This Matters:**
This chapter teaches essential embedded patterns:
- Memory-efficient data structures
- Fixed-capacity collections with `heapless`
- Const generics for compile-time configuration
- Statistics without dynamic allocation

## Understanding no_std: The Embedded Reality

### Why no_std?

Desktop programs can use:
- **Unlimited memory** (well, gigabytes via virtual memory)
- **Dynamic allocation** (`Vec`, `HashMap`, `String`)
- **Operating system services** (files, network, threads)

Embedded programs must work with:
- **Fixed memory** (320KB RAM total on ESP32-C3)
- **No heap allocator** (or very limited heap)
- **No operating system** (we *are* the operating system!)

```rust
// ❌ This won't work in no_std embedded
use std::collections::HashMap;
use std::vec::Vec;

fn desktop_approach() {
    let mut readings = Vec::new();           // Heap allocation
    let mut sensors = HashMap::new();        // Dynamic sizing
    readings.push(23.5);                     // Can grow infinitely
    sensors.insert("temp1", 24.1);          // Hash table overhead
}

// ✅ This is the embedded way
use heapless::Vec;
use heapless::FnvIndexMap;

fn embedded_approach() {
    let mut readings: Vec<f32, 32> = Vec::new();              // Fixed capacity
    let mut sensors: FnvIndexMap<&str, f32, 8> = FnvIndexMap::new(); // Known limits
    readings.push(23.5).ok();               // Handles full buffer
    sensors.insert("temp1", 24.1).ok();     // Graceful failure
}
```

### The Three-Layer Architecture

Rust's libraries are organized in layers:

```
┌─────────────────────────────────────┐
│               std                   │
│   File I/O, networking, threads,    │  ← Desktop applications
│   HashMap, process management       │
├─────────────────────────────────────┤
│               alloc                 │
│   Vec, String, Box, Rc,             │  ← Embedded with heap
│   heap-allocated collections       │
├─────────────────────────────────────┤
│               core                  │
│   Option, Result, Iterator,         │  ← Minimal embedded
│   basic traits, no allocation      │
└─────────────────────────────────────┘
```

**For our ESP32-C3 project, we'll use `core` + `heapless` collections.**

## Creating an Embedded Temperature Type

Let's build a temperature type designed for embedded use:

```rust
#![no_std]

use core::fmt;

/// Temperature reading optimized for embedded systems
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    // Store as i16 to save memory (16-bit vs 32-bit f32)
    // Resolution: 0.1°C, Range: -3276.8°C to +3276.7°C
    // More than enough for ESP32-C3's typical -40°C to +125°C range
    celsius_tenths: i16,
}

impl Temperature {
    /// Create temperature from Celsius value
    pub const fn from_celsius(celsius: f32) -> Self {
        Self {
            celsius_tenths: (celsius * 10.0) as i16,
        }
    }

    /// Create temperature from raw ESP32 sensor reading
    pub const fn from_sensor_raw(raw_value: u16) -> Self {
        // ESP32-C3 temperature sensor specific conversion
        // This is a simplified conversion - real implementation depends on calibration
        let celsius = (raw_value as f32 - 1000.0) / 10.0;
        Self::from_celsius(celsius)
    }

    /// Get temperature as Celsius f32
    pub fn celsius(&self) -> f32 {
        self.celsius_tenths as f32 / 10.0
    }

    /// Get temperature as Fahrenheit f32
    pub fn fahrenheit(&self) -> f32 {
        self.celsius() * 9.0 / 5.0 + 32.0
    }

    /// Check if temperature is within normal range
    pub const fn is_normal_range(&self) -> bool {
        // Normal room temperature: 15-35°C
        self.celsius_tenths >= 150 && self.celsius_tenths <= 350
    }

    /// Check if temperature is too high (potential overheating)
    pub const fn is_overheating(&self) -> bool {
        self.celsius_tenths > 1000  // > 100°C
    }
}

// Implement Display for serial output
impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius())
    }
}

// Example usage in embedded code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_creation() {
        let temp = Temperature::from_celsius(23.5);
        assert_eq!(temp.celsius(), 23.5);
        assert_eq!(temp.fahrenheit(), 74.3);
        assert!(temp.is_normal_range());
        assert!(!temp.is_overheating());
    }

    #[test]
    fn test_memory_efficiency() {
        // Temperature struct should be small
        assert_eq!(core::mem::size_of::<Temperature>(), 2); // Just 2 bytes!
    }
}
```

### Why This Design?

**Memory Efficiency:**
- `i16` (2 bytes) instead of `f32` (4 bytes) saves 50% memory
- 0.1°C resolution is more than adequate for most applications
- Fits in CPU registers for fast operations

**Const Functions:**
- `const fn from_celsius()` - Computed at compile time
- `const fn is_normal_range()` - Zero runtime cost
- Perfect for configuration and thresholds

**No Heap Usage:**
- Copy trait means values are stack-allocated
- No hidden allocations or indirection

## Heapless Collections for Sensor Data

Now let's store multiple temperature readings efficiently:

```rust
use heapless::Vec;
use heapless::pool::{Pool, Node};

/// Fixed-capacity temperature buffer for embedded systems
pub struct TemperatureBuffer<const N: usize> {
    readings: Vec<Temperature, N>,
    total_readings: u32,  // Track total for statistics
}

impl<const N: usize> TemperatureBuffer<N> {
    /// Create new buffer with compile-time capacity
    pub const fn new() -> Self {
        Self {
            readings: Vec::new(),
            total_readings: 0,
        }
    }

    /// Add a temperature reading (circular buffer behavior)
    pub fn push(&mut self, temperature: Temperature) {
        if self.readings.len() < N {
            // Buffer not full yet - just add
            self.readings.push(temperature).ok();
        } else {
            // Buffer full - use circular indexing (more efficient than remove(0))
            let oldest_index = (self.total_readings as usize) % N;
            self.readings[oldest_index] = temperature;
        }
        self.total_readings += 1;
    }

    /// Get current number of readings
    pub fn len(&self) -> usize {
        self.readings.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.readings.is_empty()
    }

    /// Get buffer capacity
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get the latest reading
    pub fn latest(&self) -> Option<Temperature> {
        self.readings.last().copied()
    }

    /// Get the oldest reading in buffer
    pub fn oldest(&self) -> Option<Temperature> {
        self.readings.first().copied()
    }

    /// Calculate average temperature
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

    /// Find minimum temperature in buffer
    pub fn min(&self) -> Option<Temperature> {
        self.readings.iter()
            .min_by_key(|t| t.celsius_tenths)
            .copied()
    }

    /// Find maximum temperature in buffer
    pub fn max(&self) -> Option<Temperature> {
        self.readings.iter()
            .max_by_key(|t| t.celsius_tenths)
            .copied()
    }

    /// Get total readings processed (including overwritten ones)
    pub fn total_readings(&self) -> u32 {
        self.total_readings
    }

    /// Clear all readings
    pub fn clear(&mut self) {
        self.readings.clear();
        self.total_readings = 0;
    }

    /// Get statistics summary
    pub fn stats(&self) -> Option<TemperatureStats> {
        if self.readings.is_empty() {
            return None;
        }

        Some(TemperatureStats {
            count: self.readings.len(),
            total_count: self.total_readings,
            average: self.average()?,
            min: self.min()?,
            max: self.max()?,
        })
    }
}

/// Statistics summary for temperature readings
#[derive(Debug, Clone, Copy)]
pub struct TemperatureStats {
    pub count: usize,           // Current readings in buffer
    pub total_count: u32,       // Total readings ever processed
    pub average: Temperature,
    pub min: Temperature,
    pub max: Temperature,
}

impl fmt::Display for TemperatureStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "Stats: {} readings (total: {}), Avg: {}, Min: {}, Max: {}",
            self.count, self.total_count, self.average, self.min, self.max
        )
    }
}
```

### Understanding Heapless Collections

**Key Differences from std:**

| Feature | std::Vec | heapless::Vec |
|---------|----------|---------------|
| Capacity | Dynamic (grows) | Fixed at compile time |
| Memory | Heap allocated | Stack or static |
| Failure | Panic on OOM | Returns Result |
| Performance | Allocation overhead | Zero allocation |

**When to Use Each Pattern:**

```rust
// ✅ Use const generics for compile-time capacity
type SmallBuffer = TemperatureBuffer<16>;   // 16 readings max
type LargeBuffer = TemperatureBuffer<128>;  // 128 readings max

// ✅ Handle full buffer gracefully
let mut buffer = TemperatureBuffer::<10>::new();
for i in 0..20 {
    let temp = Temperature::from_celsius(20.0 + i as f32);
    buffer.push(temp); // Automatically overwrites oldest when full
}

// ✅ Check capacity and adjust behavior
if buffer.len() >= buffer.capacity() {
    esp_println::println!("Buffer full, overwriting oldest data");
}
```

## Const Configuration for Embedded Systems

Embedded systems benefit from compile-time configuration:

```rust
/// System configuration computed at compile time
pub struct SystemConfig;

impl SystemConfig {
    /// ESP32-C3 system clock frequency
    pub const CLOCK_HZ: u32 = 160_000_000; // 160 MHz

    /// Temperature monitoring configuration
    pub const TEMP_SAMPLE_RATE_HZ: u32 = 1;  // 1 reading per second
    pub const TEMP_BUFFER_SIZE: usize = 60;  // 1 minute of readings
    pub const TEMP_WARNING_THRESHOLD: f32 = 52.0; // 52°C warning threshold
    pub const TEMP_CRITICAL_THRESHOLD: f32 = 100.0; // 100°C critical threshold

    /// Calculate timer interval for sampling rate
    pub const fn sample_interval_ms() -> u32 {
        1000 / Self::TEMP_SAMPLE_RATE_HZ
    }

    /// Create temperature thresholds at compile time
    pub const fn warning_threshold() -> Temperature {
        Temperature::from_celsius(Self::TEMP_WARNING_THRESHOLD)
    }

    pub const fn critical_threshold() -> Temperature {
        Temperature::from_celsius(Self::TEMP_CRITICAL_THRESHOLD)
    }

    /// Validate buffer size is reasonable
    pub const fn validate_buffer_size() -> bool {
        // Buffer should hold 1-300 seconds of data
        Self::TEMP_BUFFER_SIZE >= Self::TEMP_SAMPLE_RATE_HZ as usize &&
        Self::TEMP_BUFFER_SIZE <= (Self::TEMP_SAMPLE_RATE_HZ * 300) as usize
    }
}

// Compile-time assertions (will fail at compile time if invalid)
const _: () = assert!(SystemConfig::validate_buffer_size());
const _: () = assert!(SystemConfig::TEMP_SAMPLE_RATE_HZ > 0);
const _: () = assert!(SystemConfig::TEMP_BUFFER_SIZE > 0);

// Pre-computed constants (zero runtime cost)
pub const SAMPLE_INTERVAL: u32 = SystemConfig::sample_interval_ms();
pub const WARNING_TEMP: Temperature = SystemConfig::warning_threshold();
pub const CRITICAL_TEMP: Temperature = SystemConfig::critical_threshold();
```

## Integrating with ESP32-C3 Hardware

Let's update our temperature monitor to use these new data structures:

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
use heapless::Vec;

// Temperature types from earlier in this chapter
/// Temperature reading optimized for embedded systems
#[derive(Debug, Clone, Copy, PartialEq)]
struct Temperature {
    celsius_tenths: i16,
}

impl Temperature {
    const fn from_celsius(celsius: f32) -> Self {
        Self {
            celsius_tenths: (celsius * 10.0) as i16,
        }
    }

    fn celsius(&self) -> f32 {
        self.celsius_tenths as f32 / 10.0
    }

    fn fahrenheit(&self) -> f32 {
        self.celsius() * 9.0 / 5.0 + 32.0
    }

    const fn is_normal_range(&self) -> bool {
        // Normal room temperature: 15-35°C
        self.celsius_tenths >= 150 && self.celsius_tenths <= 350
    }

    const fn is_overheating(&self) -> bool {
        self.celsius_tenths > 1000  // > 100°C
    }
}

/// Fixed-capacity temperature buffer
struct TemperatureBuffer<const N: usize> {
    readings: Vec<Temperature, N>,
    total_readings: u32,
}

impl<const N: usize> TemperatureBuffer<N> {
    const fn new() -> Self {
        Self {
            readings: Vec::new(),
            total_readings: 0,
        }
    }

    fn push(&mut self, temperature: Temperature) {
        if self.readings.len() < N {
            self.readings.push(temperature).ok();
        } else {
            // Circular buffer - overwrite oldest
            let oldest_index = (self.total_readings as usize) % N;
            self.readings[oldest_index] = temperature;
        }
        self.total_readings += 1;
    }

    fn total_readings(&self) -> u32 {
        self.total_readings
    }

    fn stats(&self) -> Option<TemperatureStats> {
        if self.readings.is_empty() {
            return None;
        }

        let sum: i32 = self.readings.iter()
            .map(|t| t.celsius_tenths as i32)
            .sum();
        let avg_tenths = sum / self.readings.len() as i32;
        let average = Temperature { celsius_tenths: avg_tenths as i16 };

        let min = *self.readings.iter()
            .min_by_key(|t| t.celsius_tenths)?;
        let max = *self.readings.iter()
            .max_by_key(|t| t.celsius_tenths)?;

        Some(TemperatureStats {
            count: self.readings.len(),
            total_count: self.total_readings,
            average,
            min,
            max,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct TemperatureStats {
    count: usize,
    total_count: u32,
    average: Temperature,
    min: Temperature,
    max: Temperature,
}

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

    // Startup messages
    esp_println::println!("ESP32-C3 Temperature Monitor with Data Storage");
    esp_println::println!("Buffer capacity: {} readings", BUFFER_SIZE);
    esp_println::println!("Sample rate: {} second intervals", SAMPLE_INTERVAL_MS / 1000);
    esp_println::println!("Temperature stored as {} bytes per reading", core::mem::size_of::<Temperature>());
    esp_println::println!();

    // Main monitoring loop
    loop {
        // Small stabilization delay (recommended by ESP-HAL)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_micros(200) {}

        // Read temperature from built-in sensor
        let esp_temperature = temp_sensor.get_temperature();
        let temp_celsius = esp_temperature.to_celsius();
        let temperature = Temperature::from_celsius(temp_celsius);

        // Store in buffer
        temp_buffer.push(temperature);

        // LED status based on temperature
        if temperature.is_overheating() {
            // Rapid triple blink for overheating (>100°C)
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

        // Print current reading
        esp_println::println!("Reading #{}: {:.1}°C ({:.1}°F)",
            temp_buffer.total_readings(),
            temperature.celsius(),
            temperature.fahrenheit()
        );

        // Print statistics every 5 readings
        if temp_buffer.total_readings() % 5 == 0 {
            if let Some(stats) = temp_buffer.stats() {
                esp_println::println!("Stats: {} readings (total: {}), Avg: {:.1}°C, Min: {:.1}°C, Max: {:.1}°C",
                    stats.count,
                    stats.total_count,
                    stats.average.celsius(),
                    stats.min.celsius(),
                    stats.max.celsius()
                );

                // Memory usage info
                let buffer_size = core::mem::size_of::<TemperatureBuffer<BUFFER_SIZE>>();
                esp_println::println!("Memory: Buffer using {} of {} slots ({} bytes total)",
                    stats.count, BUFFER_SIZE, buffer_size
                );

                // Buffer status
                if stats.count >= BUFFER_SIZE {
                    esp_println::println!("Buffer full - circular mode active (overwriting oldest data)");
                }
                esp_println::println!();
            }
        }

        // Wait for next sample
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(SAMPLE_INTERVAL_MS) {}
    }
}

## Exercise: Temperature Data Collection System


Build an embedded data collection system that stores and analyzes temperature readings.

### Requirements

1. **Temperature Type**: Create efficient embedded temperature representation
2. **Circular Buffer**: Fixed-capacity storage with automatic oldest-data replacement
3. **Statistics**: Real-time calculation of min, max, average
4. **Configuration**: Compile-time system parameters
5. **Memory Efficiency**: Minimize RAM usage while maintaining functionality

### Starting Project Structure

Create new module files:

```rust
// src/temperature.rs
#![no_std]

use core::fmt;
use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    // TODO: Implement memory-efficient temperature storage
}

impl Temperature {
    pub const fn from_celsius(celsius: f32) -> Self {
        // TODO: Convert f32 to efficient internal representation
        unimplemented!()
    }

    pub fn celsius(&self) -> f32 {
        // TODO: Convert back to f32
        unimplemented!()
    }

    pub const fn is_overheating(&self) -> bool {
        // TODO: Check if temperature > 100°C
        unimplemented!()
    }
}

pub struct TemperatureBuffer<const N: usize> {
    // TODO: Implement fixed-capacity circular buffer
}

impl<const N: usize> TemperatureBuffer<N> {
    pub const fn new() -> Self {
        // TODO: Initialize empty buffer
        unimplemented!()
    }

    pub fn push(&mut self, temperature: Temperature) {
        // TODO: Add reading with circular buffer behavior
        unimplemented!()
    }

    pub fn stats(&self) -> Option<TemperatureStats> {
        // TODO: Calculate min, max, average
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TemperatureStats {
    pub count: usize,
    pub average: Temperature,
    pub min: Temperature,
    pub max: Temperature,
}
```

```rust
// src/main.rs
#![no_std]
#![no_main]

mod temperature;
use temperature::{Temperature, TemperatureBuffer};

#[entry]
fn main() -> ! {
    // TODO: Initialize hardware (from Chapter 13)

    // TODO: Create temperature buffer with capacity 20

    loop {
        // TODO: Read temperature sensor

        // TODO: Store in buffer

        // TODO: Display statistics every 5 readings

        // TODO: LED status based on temperature

        // TODO: Wait 2 seconds between readings
    }
}
```

### Implementation Tasks

1. **Efficient Temperature Type**:
   - Use `i16` to store temperature * 10 (0.1°C resolution)
   - Implement `from_celsius()` and `celsius()` conversion
   - Add `is_overheating()` check for > 100°C
   - Implement `Display` trait for printing

2. **Circular Buffer Implementation**:
   - Use `heapless::Vec<Temperature, N>` for storage
   - Implement `push()` with oldest-data replacement when full
   - Track total readings processed
   - Add `len()`, `capacity()`, `latest()` methods

3. **Statistics Calculation**:
   - Implement `min()`, `max()`, `average()` functions
   - Create `TemperatureStats` struct
   - Handle empty buffer case gracefully
   - Efficient integer-based calculations

4. **Integration Testing**:
   - Build and flash to ESP32-C3
   - Verify buffer behavior and statistics
   - Test with temperature changes

### Expected Output

```
ESP32-C3 Temperature Monitor with Data Storage
Sample rate: 1 Hz
Buffer capacity: 20 readings

Reading #1: 24.3°C
Reading #2: 24.5°C
Reading #3: 24.1°C
Reading #4: 24.8°C
Reading #5: 25.2°C
Stats: 5 readings, Avg: 24.6°C, Min: 24.1°C, Max: 25.2°C
Memory: Buffer using 5 of 20 slots

...

Reading #25: 24.7°C
Stats: 20 readings, Avg: 24.4°C, Min: 23.8°C, Max: 25.3°C
Memory: Buffer using 20 of 20 slots (circular mode active)
```

### Success Criteria

- [ ] Temperature stored efficiently in 2 bytes per reading
- [ ] Buffer correctly implements circular behavior when full
- [ ] Statistics calculated accurately without floating-point overhead
- [ ] LED indicates overheating condition
- [ ] Memory usage is predictable and bounded
- [ ] No heap allocation or dynamic memory

### Extension Challenges

1. **Compile-time Configuration**: Move buffer size and thresholds to const
2. **Temperature Trends**: Track if temperature is rising or falling
3. **Alarm Conditions**: Multiple threshold levels with different LED patterns
4. **Data Persistence**: Retain readings across ESP32 resets (use RTC memory)
5. **Memory Analysis**: Measure actual RAM usage of data structures

### Understanding Memory Usage

```rust
// Check memory footprint of your types
const TEMP_SIZE: usize = core::mem::size_of::<Temperature>();
const BUFFER_SIZE: usize = core::mem::size_of::<TemperatureBuffer<20>>();
const STATS_SIZE: usize = core::mem::size_of::<TemperatureStats>();

esp_println::println!("Memory usage:");
esp_println::println!("  Temperature: {} bytes", TEMP_SIZE);
esp_println::println!("  Buffer (20 readings): {} bytes", BUFFER_SIZE);
esp_println::println!("  Stats: {} bytes", STATS_SIZE);
esp_println::println!("  Total: {} bytes", BUFFER_SIZE + STATS_SIZE);
```

Target: Less than 100 bytes total for 20 temperature readings + metadata.

## Key Takeaways

✅ **Memory Efficiency**: Using `i16` instead of `f32` saves 50% memory without losing precision

✅ **Fixed Allocation**: `heapless::Vec` provides dynamic behavior with static memory

✅ **Const Configuration**: Compile-time parameters eliminate runtime overhead

✅ **Circular Buffers**: Essential pattern for continuous data collection in embedded systems

✅ **Statistical Processing**: Can calculate aggregates efficiently without external libraries

✅ **Type Safety**: Rust's type system prevents common embedded errors like buffer overflows

**Next**: In Chapter 15, we'll add proper testing strategies for embedded code, including how to test no_std code on desktop and validate hardware behavior.