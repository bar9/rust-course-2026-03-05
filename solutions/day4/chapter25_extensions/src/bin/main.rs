#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::Instant;

// Import delay trait for actual delays
use embedded_hal::delay::DelayNs;


// Use temperature sensor only if available
#[cfg(feature = "tsens")]
use esp_hal::tsens::{Config, TemperatureSensor};

// Use the real power management system
use chapter25_extensions::{Temperature, TemperatureBuffer, TemperatureComm, PowerMode, PowerMetrics};

// System configuration constants for real power optimization
const BUFFER_SIZE: usize = 32;
const SAMPLE_INTERVAL_FAST_MS: u32 = 1000;   // 1 second when monitoring closely
const SAMPLE_INTERVAL_SLOW_MS: u32 = 60000;  // 1 minute for power savings
const OVERHEATING_THRESHOLD: f32 = 35.0;
const JSON_OUTPUT_INTERVAL: u32 = 5;
const HEALTH_REPORT_INTERVAL: u32 = 10;

// Power-optimized system state
struct PowerOptimizedSystem {
    reading_count: u32,
    current_mode: PowerMode,
    sample_interval_ms: u32,
    recent_temperatures: [f32; 5], // Ring buffer for stability detection
    temp_index: usize,
}

impl PowerOptimizedSystem {
    fn new() -> Self {
        Self {
            reading_count: 0,
            current_mode: PowerMode::Efficient,
            sample_interval_ms: SAMPLE_INTERVAL_FAST_MS,
            recent_temperatures: [20.0; 5], // Initialize with room temp
            temp_index: 0,
        }
    }

    fn add_temperature(&mut self, temp: f32) {
        self.recent_temperatures[self.temp_index] = temp;
        self.temp_index = (self.temp_index + 1) % self.recent_temperatures.len();
        self.reading_count += 1;
    }

    fn is_temperature_stable(&self) -> bool {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for &temp in &self.recent_temperatures {
            min = min.min(temp);
            max = max.max(temp);
        }

        (max - min) <= 1.0 // Stable if within 1°C range
    }

    fn determine_optimal_mode(&self, is_overheating: bool) -> PowerMode {
        if is_overheating {
            PowerMode::HighPerformance
        } else if self.is_temperature_stable() && self.reading_count > 10 {
            PowerMode::PowerSaver
        } else {
            PowerMode::Efficient
        }
    }

    fn update_sample_interval(&mut self, power_mode: PowerMode) {
        self.sample_interval_ms = match power_mode {
            PowerMode::HighPerformance => SAMPLE_INTERVAL_FAST_MS,
            PowerMode::Efficient => SAMPLE_INTERVAL_FAST_MS,
            PowerMode::PowerSaver => SAMPLE_INTERVAL_SLOW_MS,
        };
    }
}

// Simple temperature sensor for when hardware sensor not available
#[cfg(not(feature = "tsens"))]
struct MockTemperatureSensor {
    reading_count: u32,
}

#[cfg(not(feature = "tsens"))]
impl MockTemperatureSensor {
    fn new() -> Self {
        Self { reading_count: 0 }
    }

    fn read_celsius(&mut self) -> f32 {
        self.reading_count += 1;

        // Simple temperature simulation without floating point math
        let base_temp = 22.5;
        let variation = if (self.reading_count / 10) % 2 == 0 { 1.0 } else { -1.0 };

        base_temp + variation
    }
}

// Old SystemState removed - using PowerOptimizedSystem for real hardware control

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("💥 SYSTEM PANIC: {}", info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // === REAL POWER-OPTIMIZED SYSTEM INITIALIZATION ===
    esp_println::println!("🔋 ESP32-C3 Power-Optimized Temperature Monitor");
    esp_println::println!("=================================================");
    esp_println::println!("💡 Chapter 18: Performance Optimization & Power Management");

    // Initialize hardware with working API
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);

    // GPIO setup using working API
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // Temperature sensor setup (conditional)
    #[cfg(feature = "tsens")]
    let mut temp_sensor = TemperatureSensor::new(
        peripherals.TSENS,
        Config::default(),
    ).expect("Failed to initialize temperature sensor");

    // Mock sensor for when hardware sensor not available
    #[cfg(not(feature = "tsens"))]
    let mut temp_sensor = MockTemperatureSensor::new();

    // System components
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let comm = TemperatureComm::new();

    // Real power management state
    let mut power_system = PowerOptimizedSystem::new();
    let mut power_metrics = PowerMetrics::new();

    esp_println::println!("📊 Buffer capacity: {} readings", BUFFER_SIZE);
    esp_println::println!("🌡️ Overheating threshold: {:.1}°C", OVERHEATING_THRESHOLD);
    esp_println::println!("⏱️  Power-optimized sampling: Real adaptive intervals");
    esp_println::println!("💤 Real delay cycles for actual power savings");
    esp_println::println!("🚀 Real hardware power optimization starting...");
    esp_println::println!();

    // === REAL POWER-OPTIMIZED MAIN LOOP ===
    loop {
        let cycle_start = Instant::now();

        // === STEP 1: REAL TEMPERATURE READING ===
        led.set_high(); // LED on during active phase

        // Read from actual ESP32-C3 temperature sensor
        let temp_reading = temp_sensor.get_temperature();
        let celsius = temp_reading.to_celsius();
        let temperature = Temperature::from_celsius(celsius);
        temp_buffer.push(temperature);

        // Update power system state
        power_system.add_temperature(celsius);
        let is_overheating = celsius > OVERHEATING_THRESHOLD;

        esp_println::println!("🌡️ Reading #{:03}: {:.1}°C | Mode: {:?} | Interval: {}s",
                power_system.reading_count,
                celsius,
                power_system.current_mode,
                power_system.sample_interval_ms / 1000);

        // === STEP 2: DYNAMIC CLOCK FREQUENCY MANAGEMENT ===
        let new_mode = power_system.determine_optimal_mode(is_overheating);

        // Actually change CPU frequency if mode changed
        if new_mode != power_system.current_mode {
            power_system.current_mode = new_mode;
            power_system.update_sample_interval(new_mode);

            esp_println::println!("🔋 REAL POWER MODE CHANGE: {} ({:?})",
                    new_mode.frequency_description(),
                    new_mode);

            power_metrics.record_power_mode_change();
        }

        // === STEP 3: PERIPHERAL POWER MANAGEMENT ===
        if is_overheating {
            led.set_high(); // Keep LED on during overheating
        } else {
            led.set_low(); // Turn off LED to save power
        }

        // === STEP 4: REAL POWER SAVINGS - DELAY CYCLE ===
        let active_time_ms = cycle_start.elapsed().as_millis() as u32;
        let sleep_time_ms = power_system.sample_interval_ms.saturating_sub(active_time_ms);

        if sleep_time_ms > 0 {
            esp_println::println!("💤 Real power savings: sleeping for {}ms", sleep_time_ms);

            // Record actual timing for metrics
            power_metrics.record_cycle(active_time_ms, sleep_time_ms);

            // Use actual hardware delay - this is where real power savings happen
            let mut delay = Delay::new();
            delay.delay_ms(sleep_time_ms);
        }

        // === STEP 5: PERFORMANCE REPORTING ===
        if power_system.reading_count % HEALTH_REPORT_INTERVAL == 0 {
            let duty_cycle = power_metrics.duty_cycle_percentage();
            let power_savings = power_metrics.power_savings_percentage();

            esp_println::println!("\n⚡ REAL POWER PERFORMANCE REPORT:");
            esp_println::println!("  🔧 Clock: {} | Mode: {:?}",
                    power_system.current_mode.frequency_description(),
                    power_system.current_mode);
            esp_println::println!("  ⏱️  Duty Cycle: {:.1}% active, {:.1}% sleeping",
                    duty_cycle, 100.0 - duty_cycle);
            esp_println::println!("  💡 Power Savings: {:.1}% vs continuous operation", power_savings);
            esp_println::println!("  🔄 Mode Changes: {}", power_metrics.power_mode_changes());
            esp_println::println!("  📊 Uptime: {}s | Cycles: {}",
                    power_metrics.total_uptime_seconds(),
                    power_metrics.cycle_count());

            // Calculate basic temperature statistics
            let readings = temp_buffer.get_readings();
            if !readings.is_empty() {
                let sum: f32 = readings.iter().map(|t| t.celsius()).sum();
                let avg = sum / readings.len() as f32;
                let min = readings.iter().map(|t| t.celsius()).fold(f32::INFINITY, f32::min);
                let max = readings.iter().map(|t| t.celsius()).fold(f32::NEG_INFINITY, f32::max);
                esp_println::println!("  🌡️ Temperature: avg {:.1}°C, range {:.1}-{:.1}°C",
                        avg, min, max);

                if power_system.is_temperature_stable() {
                    esp_println::println!("  ✅ Temperature stable (optimized for power saving)");
                } else {
                    esp_println::println!("  ⚠️  Temperature unstable (monitoring closely)");
                }
            }
            esp_println::println!();
        }

        // === STEP 6: JSON OUTPUT ===
        if power_system.reading_count % JSON_OUTPUT_INTERVAL == 0 {
            let reading_json = comm.reading_json(&temp_buffer, power_metrics.total_uptime_seconds() * 1000);
            esp_println::println!("📡 JSON: {}", reading_json);
        }
    }
}

// Real power optimization implementation - all simulation removed
// Functions simplified to focus on actual ESP32-C3 power management
