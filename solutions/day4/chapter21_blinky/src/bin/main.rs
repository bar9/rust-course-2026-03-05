#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Configure GPIO8 as a digital output, starting LOW (LED off)
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    loop {
        led.set_high(); // LED on

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}

        led.set_low(); // LED off

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }
}
