#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn heartbeat() {
    let mut count = 0u32;
    loop {
        println!("heartbeat: {}", count);
        count += 1;
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize the async runtime
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // Spawn the heartbeat task — runs concurrently
    spawner.spawn(heartbeat()).unwrap();

    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    loop {
        led.toggle();
        Timer::after(Duration::from_millis(200)).await;
    }
}
