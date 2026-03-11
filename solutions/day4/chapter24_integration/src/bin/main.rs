#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

// Conditional imports based on features
#[cfg(all(feature = "hardware", not(feature = "simulation")))]
use esp_hal::tsens::{Config, TemperatureSensor};

// Use the integrated system components
use chapter24_integration::{Temperature, TemperatureBuffer, Command, TemperatureComm};

// Mock sensor for simulation feature
#[cfg(feature = "simulation")]
struct MockTemperatureSensor {
    base_temp: f32,
    reading_count: u32,
}

#[cfg(feature = "simulation")]
impl MockTemperatureSensor {
    fn new() -> Self {
        Self {
            base_temp: 25.0,
            reading_count: 0,
        }
    }

    fn get_temperature(&mut self) -> MockTemperature {
        self.reading_count += 1;
        // Simulate varying temperature with a simple pattern (no_std compatible)
        let cycle = self.reading_count % 20;
        let variation = if cycle < 10 {
            (cycle as f32 * 0.2) - 1.0  // -1.0 to +1.0
        } else {
            1.0 - ((cycle - 10) as f32 * 0.2) // +1.0 to -1.0
        };
        let temp = self.base_temp + variation;
        MockTemperature { celsius: temp }
    }
}

#[cfg(feature = "simulation")]
struct MockTemperature {
    celsius: f32,
}

#[cfg(feature = "simulation")]
impl MockTemperature {
    fn to_celsius(&self) -> f32 {
        self.celsius
    }
}

// Verbose logging macros
#[cfg(feature = "verbose")]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        esp_println::println!("🔍 DEBUG: {}", format_args!($($arg)*));
    };
}

#[cfg(not(feature = "verbose"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

// System configuration constants
const BUFFER_SIZE: usize = 32;
const SAMPLE_RATE_MS: u32 = 1000;
const JSON_OUTPUT_INTERVAL: u32 = 5;
const HEALTH_REPORT_INTERVAL: u32 = 20;
const OVERHEATING_THRESHOLD: f32 = 100.0;

// System state tracking
struct SystemState {
    reading_count: u32,
    system_time_ms: u32,
    overheating_count: u32,
    sensor_error_count: u32,
    last_temp: f32,
    start_time: Instant,
}

impl SystemState {
    fn new() -> Self {
        Self {
            reading_count: 0,
            system_time_ms: 0,
            overheating_count: 0,
            sensor_error_count: 0,
            last_temp: 0.0,
            start_time: Instant::now(),
        }
    }

    fn advance_time(&mut self) {
        self.reading_count += 1;
        self.system_time_ms += SAMPLE_RATE_MS;
    }

    fn record_overheating(&mut self) {
        self.overheating_count += 1;
    }

    fn record_sensor_error(&mut self) {
        self.sensor_error_count += 1;
    }

    fn uptime_seconds(&self) -> u32 {
        self.system_time_ms / 1000
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("💥 SYSTEM PANIC: {}", info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // === SYSTEM INITIALIZATION ===
    esp_println::println!("🌡️ ESP32-C3 Complete Temperature Monitor System");
    esp_println::println!("=================================================");

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // GPIO setup
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // Initialize temperature sensor with feature-dependent implementation
    #[cfg(all(feature = "hardware", not(feature = "simulation")))]
    let mut temp_sensor = match TemperatureSensor::new(peripherals.TSENS, Config::default()) {
        Ok(sensor) => sensor,
        Err(_) => {
            esp_println::println!("❌ Failed to initialize temperature sensor");
            loop {}
        }
    };

    #[cfg(feature = "simulation")]
    let mut temp_sensor = MockTemperatureSensor::new();

    #[cfg(all(feature = "hardware", not(feature = "simulation")))]
    esp_println::println!("🔧 Hardware: ESP32-C3 @ max frequency");

    #[cfg(feature = "simulation")]
    esp_println::println!("🔧 Simulation: Mock sensor enabled");

    // System components
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let mut comm = TemperatureComm::new();
    let mut system_state = SystemState::new();

    // Initialize communication handler
    comm.init(0);

    // System startup information
    esp_println::println!("🔧 Hardware: ESP32-C3 @ max frequency");
    esp_println::println!("📊 Buffer capacity: {} readings", BUFFER_SIZE);
    esp_println::println!("⏱️  Sample rate: {} Hz", 1000 / SAMPLE_RATE_MS);
    esp_println::println!("🌡️ Overheating threshold: {:.1}°C", OVERHEATING_THRESHOLD);
    esp_println::println!("📡 JSON output every {} readings", JSON_OUTPUT_INTERVAL);
    esp_println::println!("💓 Health reports every {} readings", HEALTH_REPORT_INTERVAL);
    esp_println::println!("🚀 System starting...");
    esp_println::println!();

    // Initial system status
    let initial_status = comm.status_json(&temp_buffer, 0);
    esp_println::println!("INITIAL_STATUS: {}", initial_status);
    esp_println::println!();

    // === MAIN SYSTEM LOOP ===
    loop {
        // STEP 1: Read temperature with error handling (feature-dependent)
        #[cfg(all(feature = "hardware", not(feature = "simulation")))]
        let celsius = match read_temperature_safe_hw(&mut temp_sensor) {
            Ok(temp) => {
                debug_log!("Temperature read successfully: {:.2}°C", temp);
                system_state.last_temp = temp;
                temp
            }
            Err(_) => {
                system_state.record_sensor_error();
                debug_log!("Sensor error #{}, fallback to last value", system_state.sensor_error_count);
                esp_println::println!("❌ Sensor error #{}, using last value: {:.1}°C",
                    system_state.sensor_error_count, system_state.last_temp);
                system_state.last_temp // Use last known good value
            }
        };

        #[cfg(feature = "simulation")]
        let celsius = match read_temperature_safe_sim(&mut temp_sensor) {
            Ok(temp) => {
                debug_log!("Temperature read successfully: {:.2}°C", temp);
                system_state.last_temp = temp;
                temp
            }
            Err(_) => {
                system_state.record_sensor_error();
                debug_log!("Sensor error #{}, fallback to last value", system_state.sensor_error_count);
                esp_println::println!("❌ Sensor error #{}, using last value: {:.1}°C",
                    system_state.sensor_error_count, system_state.last_temp);
                system_state.last_temp // Use last known good value
            }
        };

        let temperature = Temperature::from_celsius(celsius);
        temp_buffer.push(temperature);
        system_state.advance_time();

        debug_log!("Buffer status: {}/{}, Total: {}", temp_buffer.len(), BUFFER_SIZE, temp_buffer.total_readings());

        // STEP 2: LED feedback with enhanced patterns
        update_led_status(&mut led, &temperature, &system_state);

        // STEP 3: Track overheating events
        if temperature.is_overheating() {
            system_state.record_overheating();
        }

        // STEP 4: Console output with status indicators
        let status_icon = get_status_icon(&temperature, &system_state);
        esp_println::println!("{}📊 #{:03} | {:.1}°C | Buffer: {}/{}",
            status_icon,
            system_state.reading_count,
            celsius,
            temp_buffer.len(),
            BUFFER_SIZE
        );

        // STEP 5: JSON data output at regular intervals
        if system_state.reading_count % JSON_OUTPUT_INTERVAL == 0 {
            output_json_data(&mut comm, &temp_buffer, &system_state);
        }

        // STEP 6: Health monitoring and reporting
        if system_state.reading_count % HEALTH_REPORT_INTERVAL == 0 {
            debug_log!("Health report interval reached");
            output_health_report(&system_state, &temp_buffer);

            // Output extended telemetry if feature is enabled
            #[cfg(feature = "telemetry")]
            output_telemetry_data(&system_state, &temp_buffer);
        }

        // STEP 7: Error condition management
        handle_error_conditions(&system_state);

        // STEP 8: System demonstration features
        if system_state.reading_count % 50 == 0 {
            demonstrate_system_integration(&mut comm, &temp_buffer, &system_state);
        }

        // STEP 9: Precise timing control
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(SAMPLE_RATE_MS as u64) {}
    }
}

// Feature-dependent temperature reading implementations
#[cfg(all(feature = "hardware", not(feature = "simulation")))]
fn read_temperature_safe_hw(sensor: &mut TemperatureSensor) -> Result<f32, ()> {
    // Small stabilization delay
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_micros(200) {}

    let esp_temperature = sensor.get_temperature();
    Ok(esp_temperature.to_celsius())
}

#[cfg(feature = "simulation")]
fn read_temperature_safe_sim(sensor: &mut MockTemperatureSensor) -> Result<f32, ()> {
    let mock_temp = sensor.get_temperature();
    Ok(mock_temp.to_celsius())
}

fn update_led_status(led: &mut Output, temperature: &Temperature, state: &SystemState) {
    if temperature.is_overheating() {
        // Rapid blink for overheating
        led.set_high();
    } else if state.sensor_error_count > 0 && state.reading_count % 4 == 0 {
        // Fast blink pattern for sensor errors
        led.toggle();
    } else if state.reading_count % 10 == 0 {
        // Slow heartbeat blink for normal operation
        led.toggle();
    }
}

fn get_status_icon(temperature: &Temperature, state: &SystemState) -> &'static str {
    if temperature.is_overheating() {
        "🔴"
    } else if state.sensor_error_count > 0 {
        "🟡"
    } else if temperature.is_normal_range() {
        "🟢"
    } else {
        "🔵"
    }
}

fn output_json_data(comm: &mut TemperatureComm, buffer: &TemperatureBuffer<BUFFER_SIZE>, state: &SystemState) {
    esp_println::println!("\n--- JSON OUTPUT ---");

    // Current reading
    let reading_json = comm.reading_json(buffer, state.system_time_ms);
    esp_println::println!("READING: {}", reading_json);

    // Statistics
    let stats_command = comm.process_command(Command::GetStats, buffer, state.system_time_ms);
    if let Ok(stats_resp) = comm.response_to_json(&stats_command) {
        esp_println::println!("STATS: {}", stats_resp);
    }

    // System status
    let status_json = comm.status_json(buffer, state.system_time_ms);
    esp_println::println!("STATUS: {}", status_json);

    esp_println::println!("--- END JSON ---\n");
}

fn output_health_report(state: &SystemState, buffer: &TemperatureBuffer<BUFFER_SIZE>) {
    let uptime = state.uptime_seconds();
    let buffer_usage_pct = (buffer.len() * 100) / BUFFER_SIZE;
    let memory_usage = core::mem::size_of::<TemperatureBuffer<BUFFER_SIZE>>();

    esp_println::println!("💓 HEALTH REPORT");
    esp_println::println!("  Uptime: {}s | Readings: {}", uptime, state.reading_count);
    esp_println::println!("  Buffer: {}% ({}/{}) | Memory: {} bytes",
        buffer_usage_pct, buffer.len(), BUFFER_SIZE, memory_usage);
    esp_println::println!("  Errors: {} sensor, {} overheating events",
        state.sensor_error_count, state.overheating_count);
    esp_println::println!("  Current temp: {:.1}°C", state.last_temp);

    if buffer.len() >= BUFFER_SIZE {
        esp_println::println!("  ℹ️  Buffer full - circular mode active");
    }
    esp_println::println!();
}

fn handle_error_conditions(state: &SystemState) {
    if state.overheating_count >= 5 {
        esp_println::println!("🚨 WARNING: {} overheating events detected! System requires attention.",
            state.overheating_count);
    }

    if state.sensor_error_count >= 3 {
        esp_println::println!("⚠️  ALERT: {} sensor errors detected. Check sensor connection.",
            state.sensor_error_count);
    }
}

fn demonstrate_system_integration(
    comm: &mut TemperatureComm,
    buffer: &TemperatureBuffer<BUFFER_SIZE>,
    state: &SystemState
) {
    esp_println::println!("🔧 SYSTEM INTEGRATION DEMO");
    esp_println::println!("  Testing complete system functionality...");

    // Test all communication features
    let commands = [
        Command::GetStatus,
        Command::GetLatestReading,
        Command::GetStats,
    ];

    for command in commands {
        let response = comm.process_command(command, buffer, state.system_time_ms);
        if let Ok(json) = comm.response_to_json(&response) {
            esp_println::println!("  ✅ Command processed: {}", json.len());
        }
    }

    // System performance info
    esp_println::println!("  📈 Performance: {} Hz stable, {} total readings",
        1000 / SAMPLE_RATE_MS, state.reading_count);
    esp_println::println!("  🔧 Integration test complete\n");
}

// Telemetry output (only included with telemetry feature)
#[cfg(feature = "telemetry")]
fn output_telemetry_data(state: &SystemState, buffer: &TemperatureBuffer<BUFFER_SIZE>) {
    esp_println::println!("📊 TELEMETRY DATA:");
    esp_println::println!("{{");
    esp_println::println!("  \"system\": {{");
    esp_println::println!("    \"uptime_ms\": {},", state.system_time_ms);
    esp_println::println!("    \"total_readings\": {},", state.reading_count);
    esp_println::println!("    \"sample_rate_hz\": {},", 1000 / SAMPLE_RATE_MS);
    esp_println::println!("    \"free_stack\": \"unknown\",");
    esp_println::println!("    \"cpu_freq_mhz\": 160");
    esp_println::println!("  }},");
    esp_println::println!("  \"errors\": {{");
    esp_println::println!("    \"sensor_errors\": {},", state.sensor_error_count);
    esp_println::println!("    \"overheating_events\": {}", state.overheating_count);
    esp_println::println!("  }},");
    esp_println::println!("  \"buffer\": {{");
    esp_println::println!("    \"capacity\": {},", BUFFER_SIZE);
    esp_println::println!("    \"current_size\": {},", buffer.len());
    esp_println::println!("    \"total_processed\": {},", buffer.total_readings());
    esp_println::println!("    \"usage_percent\": {}", (buffer.len() * 100) / BUFFER_SIZE);
    esp_println::println!("  }}");
    esp_println::println!("}}");
}
