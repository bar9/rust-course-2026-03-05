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

use chapter24_temperature_store::TemperatureStore;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

static STORE: Mutex<CriticalSectionRawMutex, TemperatureStore> =
    Mutex::new(TemperatureStore::new(30.0));

#[embassy_executor::task]
async fn display_task() {
    loop {
        Timer::after(Duration::from_secs(5)).await;

        let guard = STORE.lock().await;
        match (guard.raw_celsius(), guard.ambient_celsius(), guard.ambient_fahrenheit()) {
            (Some(raw), Some(amb_c), Some(amb_f)) => {
                println!("[display] Raw: {}C | Ambient: {}C / {}F", raw, amb_c, amb_f);
            }
            _ => {
                println!("[display] No reading yet");
            }
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

    spawner.spawn(display_task()).unwrap();

    println!("Temperature store monitor started (offset: 30.0C)");

    loop {
        let celsius = temp_sensor.get_temperature().to_celsius();

        {
            let mut guard = STORE.lock().await;
            guard.update(celsius);
        }

        println!("[sensor] Raw: {}C", celsius);
        Timer::after(Duration::from_secs(2)).await;
    }
}
