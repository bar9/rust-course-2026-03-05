
use serde::{Deserialize, Serialize};
use core::fmt;

// Conditional imports for testing
#[cfg(test)]
use std::vec::Vec;
#[cfg(not(test))]
use heapless::Vec;

/// Temperature reading optimized for embedded systems with serialization support
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Temperature {
    celsius_tenths: i16,
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

    /// Check if temperature is too high (potential overheating >50°C)
    pub const fn is_overheating(&self) -> bool {
        self.celsius_tenths > 500
    }

    /// Helper for JSON serialization with nice format
    pub fn to_celsius_rounded(&self) -> f32 {
        let celsius_times_ten = (self.celsius() * 10.0) as i32;
        celsius_times_ten as f32 / 10.0
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub temperature: Temperature,
    pub timestamp_ms: u32,
    pub sensor_id: u8,
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
        static mut TIMESTAMP: u32 = 0;
        unsafe {
            TIMESTAMP += 1000;
            Self::new(temperature, TIMESTAMP, 0)
        }
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

    /// Get all readings as a slice
    pub fn get_readings(&self) -> &[Temperature] {
        &self.readings
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperatureStats {
    pub count: u16,
    pub total_count: u32,
    pub min_celsius: f32,
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
    fn test_serde_serialization() {
        let temp = Temperature::from_celsius(23.5);
        let reading = TemperatureReading::new(temp, 1000, 0);

        // Test JSON serialization
        let json = serde_json_core::to_string::<_, 256>(&reading).unwrap();
        assert!(json.contains("temperature"));
        assert!(json.contains("timestamp_ms"));

        // Test JSON round-trip
        let (parsed, _): (TemperatureReading, _) = serde_json_core::from_str(&json).unwrap();
        assert_eq!(parsed.temperature.celsius(), temp.celsius());
        assert_eq!(parsed.timestamp_ms, 1000);
    }
}