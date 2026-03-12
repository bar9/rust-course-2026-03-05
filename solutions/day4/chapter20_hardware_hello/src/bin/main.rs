#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    let mut count = 0u32;

    loop {
        println!("Hello, world! (count: {})", count);
        count += 1;

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_secs(1) {}
    }
}
