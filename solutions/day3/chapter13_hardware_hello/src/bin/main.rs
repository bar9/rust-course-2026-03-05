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

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
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

    // Track reading count
    let mut _reading_count = 0u32;

    // Main monitoring loop
    loop {
        // Small stabilization delay (recommended by ESP-HAL)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_micros(200) {}

        // Read temperature from built-in sensor
        let temperature = temp_sensor.get_temperature();
        let temp_celsius = temperature.to_celsius();
        _reading_count += 1;

        // LED feedback based on temperature threshold (52°C)
        if temp_celsius > 52.0 {
            // Fast blink pattern for temperature > 52°C
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}

            led.set_low();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}

            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}

            led.set_low();
        } else {
            // Slow single blink for temperature <= 52°C
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(200) {}

            led.set_low();
        }

        // Wait for remainder of 2-second interval
        // Account for LED blinking time
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(1500) {}
    }
}
