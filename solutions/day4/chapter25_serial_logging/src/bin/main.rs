#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::tsens::{Config, TemperatureSensor};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_println::println;

use chapter25_serial_logging::command::{self, Command};
use chapter25_serial_logging::TemperatureStore;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

static STORE: Mutex<CriticalSectionRawMutex, TemperatureStore> =
    Mutex::new(TemperatureStore::new(30.0));

/// Reads lines from USB serial and executes commands
#[embassy_executor::task]
async fn command_task(
    mut rx: esp_hal::usb_serial_jtag::UsbSerialJtagRx<'static, esp_hal::Async>,
) {
    let mut buf = [0u8; 64];
    let mut pos = 0usize;

    loop {
        let mut byte = [0u8; 1];
        let n = embedded_io_async::Read::read(&mut rx, &mut byte).await;
        if n == Ok(0) {
            continue;
        }

        let b = byte[0];

        // Enter or newline — process the line
        if b == b'\r' || b == b'\n' {
            if pos > 0 {
                if let Ok(line) = core::str::from_utf8(&buf[..pos]) {
                    handle_command(line).await;
                }
                pos = 0;
            }
            continue;
        }

        // Accumulate into buffer
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }
}

async fn handle_command(line: &str) {
    match command::parse(line) {
        Command::Status => {
            let guard = STORE.lock().await;
            match guard.reading() {
                Some(reading) => {
                    let mut json_buf = [0u8; 128];
                    match serde_json_core::to_slice(&reading, &mut json_buf) {
                        Ok(len) => {
                            if let Ok(json) = core::str::from_utf8(&json_buf[..len]) {
                                println!("{}", json);
                            }
                        }
                        Err(_) => println!("{{\"error\":\"serialization failed\"}}"),
                    }
                }
                None => println!("{{\"error\":\"no reading yet\"}}"),
            }
        }
        Command::SetOffset(val) => {
            let mut guard = STORE.lock().await;
            guard.set_offset(val);
            println!("OK offset={}", val);
        }
        Command::Help => {
            println!("Commands:");
            println!("  status         — print current reading as JSON");
            println!("  offset <value> — set die-to-ambient offset");
            println!("  help           — show this message");
        }
        Command::Unknown(input) => {
            println!("Unknown command: '{}'. Type 'help' for usage.", input);
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

    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();
    let (rx, _tx) = usb_serial.split();
    spawner.spawn(command_task(rx)).unwrap();

    println!("=== Temperature Monitor ===");
    println!("Type 'help' for commands");
    println!();

    loop {
        let celsius = temp_sensor.get_temperature().to_celsius();

        {
            let mut guard = STORE.lock().await;
            guard.update(celsius);
        }

        Timer::after(Duration::from_secs(2)).await;
    }
}
