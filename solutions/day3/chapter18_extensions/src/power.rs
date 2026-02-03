// Real power management module for ESP32-C3

#[cfg(feature = "esp-hal")]
use esp_hal::clock::CpuClock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerMode {
    HighPerformance, // 160MHz - Maximum speed, higher power consumption
    Efficient,       // 80MHz - Balanced performance and power
    PowerSaver,      // 40MHz - Minimum power consumption
}

impl Default for PowerMode {
    fn default() -> Self {
        PowerMode::Efficient
    }
}

impl PowerMode {
    /// Get the corresponding CPU clock frequency for this power mode
    #[cfg(feature = "esp-hal")]
    pub fn cpu_clock(&self) -> CpuClock {
        match self {
            PowerMode::HighPerformance => CpuClock::max(),
            PowerMode::Efficient => CpuClock::default(),
            PowerMode::PowerSaver => CpuClock::default(), // Use default for now
        }
    }

    /// Get the frequency description for display
    pub fn frequency_description(&self) -> &'static str {
        match self {
            PowerMode::HighPerformance => "Max MHz",
            PowerMode::Efficient => "Default MHz",
            PowerMode::PowerSaver => "Default MHz (Power Saving)",
        }
    }

    /// Estimate relative power consumption (normalized to PowerSaver = 1.0)
    pub fn relative_power_consumption(&self) -> f32 {
        match self {
            PowerMode::HighPerformance => 4.0, // ~4x more power than PowerSaver
            PowerMode::Efficient => 2.0,       // ~2x more power than PowerSaver
            PowerMode::PowerSaver => 1.0,      // Baseline
        }
    }
}

pub struct PowerMetrics {
    cycle_count: u32,
    total_active_time_ms: u32,
    total_sleep_time_ms: u32,
    power_mode_changes: u32,
}

impl PowerMetrics {
    pub fn new() -> Self {
        Self {
            cycle_count: 0,
            total_active_time_ms: 0,
            total_sleep_time_ms: 0,
            power_mode_changes: 0,
        }
    }

    /// Record a measurement cycle
    pub fn record_cycle(&mut self, active_time_ms: u32, sleep_time_ms: u32) {
        self.cycle_count += 1;
        self.total_active_time_ms += active_time_ms;
        self.total_sleep_time_ms += sleep_time_ms;
    }

    /// Record a power mode change
    pub fn record_power_mode_change(&mut self) {
        self.power_mode_changes += 1;
    }

    /// Calculate duty cycle percentage (active time / total time * 100)
    pub fn duty_cycle_percentage(&self) -> f32 {
        let total_time = self.total_active_time_ms + self.total_sleep_time_ms;
        if total_time == 0 {
            return 100.0; // Assume 100% if no data
        }
        (self.total_active_time_ms as f32 / total_time as f32) * 100.0
    }

    /// Calculate power savings percentage compared to continuous operation
    pub fn power_savings_percentage(&self) -> f32 {
        100.0 - self.duty_cycle_percentage()
    }

    /// Get total uptime in seconds
    pub fn total_uptime_seconds(&self) -> u32 {
        (self.total_active_time_ms + self.total_sleep_time_ms) / 1000
    }

    /// Get number of completed cycles
    pub fn cycle_count(&self) -> u32 {
        self.cycle_count
    }

    /// Get number of power mode changes
    pub fn power_mode_changes(&self) -> u32 {
        self.power_mode_changes
    }

    /// Estimate memory usage of this structure
    pub fn memory_usage_bytes(&self) -> u32 {
        core::mem::size_of::<Self>() as u32
    }
}

/// Calculate optimal sample interval based on temperature stability
pub fn calculate_optimal_interval_ms(
    temperature_stable: bool,
    is_overheating: bool,
    power_mode: PowerMode,
) -> u32 {
    if is_overheating {
        return 1000; // 1 second for critical monitoring
    }

    if temperature_stable {
        // Stable temperature: longer intervals to save power
        match power_mode {
            PowerMode::HighPerformance => 5000,  // 5 seconds
            PowerMode::Efficient => 30000,       // 30 seconds
            PowerMode::PowerSaver => 60000,      // 1 minute
        }
    } else {
        // Unstable temperature: shorter intervals for monitoring
        match power_mode {
            PowerMode::HighPerformance => 1000,  // 1 second
            PowerMode::Efficient => 5000,        // 5 seconds
            PowerMode::PowerSaver => 10000,      // 10 seconds
        }
    }
}

/// Determine if temperature is stable based on recent readings
pub fn is_temperature_stable(recent_temps: &[f32], threshold: f32) -> bool {
    if recent_temps.len() < 3 {
        return false; // Need at least 3 readings
    }

    let min = recent_temps.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max = recent_temps.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

    (max - min) <= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_mode_descriptions() {
        assert_eq!(PowerMode::HighPerformance.frequency_description(), "Max MHz");
        assert_eq!(PowerMode::Efficient.frequency_description(), "Default MHz");
        assert_eq!(PowerMode::PowerSaver.frequency_description(), "Default MHz (Power Saving)");
    }

    #[test]
    fn test_power_mode_relative_consumption() {
        assert_eq!(PowerMode::PowerSaver.relative_power_consumption(), 1.0);
        assert_eq!(PowerMode::Efficient.relative_power_consumption(), 2.0);
        assert_eq!(PowerMode::HighPerformance.relative_power_consumption(), 4.0);
    }

    #[test]
    fn test_power_metrics_duty_cycle() {
        let mut metrics = PowerMetrics::new();

        // Record 1 second active, 9 seconds sleep = 10% duty cycle
        metrics.record_cycle(1000, 9000);

        assert!((metrics.duty_cycle_percentage() - 10.0).abs() < 0.1);
        assert!((metrics.power_savings_percentage() - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_temperature_stability() {
        let stable_temps = [22.0, 22.1, 21.9, 22.0];
        let unstable_temps = [22.0, 24.0, 20.0, 26.0];

        assert!(is_temperature_stable(&stable_temps, 0.5));
        assert!(!is_temperature_stable(&unstable_temps, 0.5));
    }

    #[test]
    fn test_optimal_interval_calculation() {
        // Overheating should always be fast
        assert_eq!(calculate_optimal_interval_ms(true, true, PowerMode::PowerSaver), 1000);

        // Stable temperature should use longer intervals
        let stable_interval = calculate_optimal_interval_ms(true, false, PowerMode::PowerSaver);
        let unstable_interval = calculate_optimal_interval_ms(false, false, PowerMode::PowerSaver);
        assert!(stable_interval > unstable_interval);
    }

    #[test]
    fn test_power_metrics_memory_usage() {
        let metrics = PowerMetrics::new();
        assert!(metrics.memory_usage_bytes() > 0);
        assert!(metrics.memory_usage_bytes() < 100); // Should be small
    }
}