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
use esp_hal::tsens::{Config, TemperatureSensor};

// Use the communication library types
use chapter16_communication::{Temperature, TemperatureBuffer, Command, TemperatureComm};

const BUFFER_SIZE: usize = 20;
const SAMPLE_INTERVAL_MS: u64 = 1000; // 1 second

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("💥 SYSTEM PANIC: {}", info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
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

    // Startup messages
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

        // Output current reading as JSON
        let reading_json = comm.reading_json(&temp_buffer, current_time);
        esp_println::println!("READING: {}", reading_json);

        // Print current reading (human-readable)
        esp_println::println!("Reading #{}: {:.1}°C ({:.1}°F)",
            temp_buffer.total_readings(),
            temperature.celsius(),
            temperature.fahrenheit()
        );

        // Output statistics every 5 readings
        if reading_count % 5 == 0 {
            let stats_resp = comm.process_command(
                Command::GetStats,
                &temp_buffer,
                current_time
            );

            if let Ok(stats_json) = comm.response_to_json(&stats_resp) {
                esp_println::println!("STATS: {}", stats_json);
            }

            let status_json = comm.status_json(&temp_buffer, current_time);
            esp_println::println!("STATUS: {}", status_json);

            // Memory usage info
            let buffer_size = core::mem::size_of::<TemperatureBuffer<BUFFER_SIZE>>();
            esp_println::println!("Memory: Buffer using {} of {} slots ({} bytes total)",
                temp_buffer.len(), BUFFER_SIZE, buffer_size
            );

            // Buffer status
            if temp_buffer.len() >= BUFFER_SIZE {
                esp_println::println!("Buffer full - circular mode active (overwriting oldest data)");
            }
            esp_println::println!();
        }

        // Simulate command processing every 10 readings
        if reading_count % 10 == 0 {
            demonstrate_command_processing(&mut comm, &temp_buffer, current_time);
        }

        // Wait for next sample
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(SAMPLE_INTERVAL_MS) {}
    }
}

fn demonstrate_command_processing(
    comm: &mut TemperatureComm,
    buffer: &TemperatureBuffer<BUFFER_SIZE>,
    current_time: u32
) {
    esp_println::println!("--- Command Processing Demo ---");

    // Simulate received commands
    let commands = [
        Command::GetStatus,
        Command::SetSampleRate { rate_hz: 2 },
        Command::SetThreshold { threshold_celsius: 40.0 },
    ];

    for command in commands {
        esp_println::println!("Processing command: {:?}", command);

        let response = comm.process_command(command, buffer, current_time);

        if let Ok(json) = comm.response_to_json(&response) {
            esp_println::println!("Response: {}", json);
        } else {
            esp_println::println!("Failed to serialize response");
        }
    }

    esp_println::println!("--- End Demo ---");
    esp_println::println!();
}
