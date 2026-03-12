#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::tsens::{Config, TemperatureSensor};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

// Shared state: latest temperature reading
static LATEST_TEMP: Mutex<CriticalSectionRawMutex, Option<f32>> = Mutex::new(None);

#[embassy_executor::task]
async fn display_task() {
    loop {
        Timer::after(Duration::from_secs(5)).await;

        let temp = {
            let guard = LATEST_TEMP.lock().await;
            *guard
        };

        match temp {
            Some(t) => println!("[display] Latest die temp: {}C", t),
            None => println!("[display] No reading yet"),
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let temp_sensor = TemperatureSensor::new(peripherals.TSENS, Config::default()).unwrap();

    // Spawn the display consumer task
    spawner.spawn(display_task()).unwrap();

    println!("Temperature monitor started");
    println!("  Main task: reads sensor every 2s");
    println!("  Display task: prints latest reading every 5s");

    // Main task: read sensor and update shared state
    loop {
        let celsius = temp_sensor.get_temperature().to_celsius();

        {
            let mut guard = LATEST_TEMP.lock().await;
            *guard = Some(celsius);
        }

        println!("[sensor] Read: {}C", celsius);
        Timer::after(Duration::from_secs(2)).await;
    }
}
