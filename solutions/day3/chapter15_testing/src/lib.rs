//! Chapter 15: Testing Embedded Code
//!
//! This library provides temperature monitoring functionality with comprehensive testing.
//! Uses conditional compilation to test no_std code on desktop environments.

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

    /// Get temperature as Fahrenheit f32
    pub fn fahrenheit(&self) -> f32 {
        self.celsius() * 9.0 / 5.0 + 32.0
    }

    /// Check if temperature is within normal range (15-35°C)
    pub const fn is_normal_range(&self) -> bool {
        self.celsius_tenths >= 150 && self.celsius_tenths <= 350
    }

    /// Check if temperature is too high (potential overheating >100°C)
    pub const fn is_overheating(&self) -> bool {
        self.celsius_tenths > 1000
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius())
    }
}

/// Fixed-capacity temperature buffer for embedded systems
pub struct TemperatureBuffer<const N: usize> {
    #[cfg(test)]
    readings: Vec<Temperature>,  // std::vec for tests

    #[cfg(not(test))]
    readings: Vec<Temperature, N>,  // heapless::vec for embedded

    total_readings: u32,
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
        #[cfg(test)]
        {
            // In tests, simulate circular buffer with std::Vec
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
                let oldest_index = (self.total_readings as usize) % N;
                self.readings[oldest_index] = temperature;
            }
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

// Hardware abstraction layer for testing
pub trait TemperatureSensorHal {
    type Error;

    fn read_celsius(&mut self) -> Result<f32, Self::Error>;
    fn sensor_id(&self) -> &str;
}

// Mock sensor for testing
#[cfg(test)]
pub struct MockTemperatureSensor {
    temperatures: std::cell::RefCell<Vec<f32>>,
    current_index: std::cell::RefCell<usize>,
    id: String,
}

#[cfg(test)]
impl MockTemperatureSensor {
    pub fn new(id: String) -> Self {
        Self {
            temperatures: std::cell::RefCell::new(vec![25.0]), // Default temperature
            current_index: std::cell::RefCell::new(0),
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

        let hot = Temperature::from_celsius(105.0);
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
    fn test_temperature_precision() {
        // Test 0.1°C precision
        let temp1 = Temperature::from_celsius(23.1);
        let temp2 = Temperature::from_celsius(23.2);

        assert_ne!(temp1.celsius_tenths, temp2.celsius_tenths);
        assert!((temp1.celsius() - 23.1).abs() < 0.05);
        assert!((temp2.celsius() - 23.2).abs() < 0.05);
    }

    #[test]
    fn test_conversion_roundtrip() {
        // celsius -> internal -> celsius should be stable
        let original = 23.7;
        let temp = Temperature::from_celsius(original);
        let roundtrip = temp.celsius();

        assert!((original - roundtrip).abs() < 0.1);
    }

    #[test]
    fn test_buffer_basic_operations() {
        let mut buffer = TemperatureBuffer::<5>::new();

        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 5);
        assert_eq!(buffer.latest(), None);
        assert!(buffer.is_empty());

        // Add some readings
        buffer.push(Temperature::from_celsius(20.0));
        buffer.push(Temperature::from_celsius(25.0));
        buffer.push(Temperature::from_celsius(30.0));

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.total_readings(), 3);
        assert_eq!(buffer.latest().unwrap().celsius(), 30.0);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_buffer_circular_behavior() {
        let mut buffer = TemperatureBuffer::<3>::new();

        // Fill buffer exactly
        buffer.push(Temperature::from_celsius(10.0));
        buffer.push(Temperature::from_celsius(20.0));
        buffer.push(Temperature::from_celsius(30.0));
        assert_eq!(buffer.len(), 3);

        // Add one more - should overwrite oldest in test environment
        buffer.push(Temperature::from_celsius(40.0));

        assert_eq!(buffer.len(), 3);  // Still full
        assert_eq!(buffer.total_readings(), 4);  // But total increased

        // First reading (10.0) should be gone in test environment
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
        assert_eq!(buffer.stats(), None);
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

        println!("Temperature size: {} bytes", temp_size);

        assert_eq!(temp_size, 2);  // Should be exactly 2 bytes
    }

    #[test]
    fn test_buffer_capacity_limits() {
        let mut buffer = TemperatureBuffer::<2>::new();

        // Test buffer behavior at exact capacity
        buffer.push(Temperature::from_celsius(20.0));
        assert_eq!(buffer.len(), 1);

        buffer.push(Temperature::from_celsius(25.0));
        assert_eq!(buffer.len(), 2);

        // Should now be at capacity
        buffer.push(Temperature::from_celsius(30.0));
        assert_eq!(buffer.len(), 2);  // Still at capacity
        assert_eq!(buffer.total_readings(), 3);  // But total increased
    }

    #[test]
    fn test_statistics_accuracy() {
        let mut buffer = TemperatureBuffer::<4>::new();

        // Add known values: 10, 20, 30, 40
        buffer.push(Temperature::from_celsius(10.0));
        buffer.push(Temperature::from_celsius(20.0));
        buffer.push(Temperature::from_celsius(30.0));
        buffer.push(Temperature::from_celsius(40.0));

        // Average should be 25.0
        let avg = buffer.average().unwrap();
        assert_eq!(avg.celsius(), 25.0);

        // Min/max should be correct
        assert_eq!(buffer.min().unwrap().celsius(), 10.0);
        assert_eq!(buffer.max().unwrap().celsius(), 40.0);

        // Test stats structure
        let stats = buffer.stats().unwrap();
        assert_eq!(stats.count, 4);
        assert_eq!(stats.total_count, 4);
        assert_eq!(stats.average.celsius(), 25.0);
        assert_eq!(stats.min.celsius(), 10.0);
        assert_eq!(stats.max.celsius(), 40.0);
    }

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

// Integration tests module
#[cfg(test)]
mod integration_tests {
    use super::*;

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
        let hot_temp = Temperature::from_celsius(150.0);
        let very_hot_temp = Temperature::from_celsius(175.0);

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

    #[test]
    fn test_sensor_hal_integration() {
        let mut sensor = MockTemperatureSensor::new("integration-test".to_string());
        sensor.set_temperatures(vec![20.0, 25.0, 30.0, 35.0, 40.0]);

        let mut buffer = TemperatureBuffer::<3>::new();

        // Simulate reading from sensor and storing in buffer
        for _ in 0..5 {
            let reading = sensor.read_celsius().unwrap();
            let temp = Temperature::from_celsius(reading);
            buffer.push(temp);
        }

        // Buffer should contain last 3 readings: 30, 35, 40
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.total_readings(), 5);

        let stats = buffer.stats().unwrap();
        assert_eq!(stats.average.celsius(), 35.0);  // (30+35+40)/3
        assert_eq!(stats.min.celsius(), 30.0);
        assert_eq!(stats.max.celsius(), 40.0);
    }
}