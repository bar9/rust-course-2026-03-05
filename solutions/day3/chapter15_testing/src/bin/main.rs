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

// Use the tested library types
use chapter15_testing::{Temperature, TemperatureBuffer};

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
