# Chapter 23: Temperature Sensor & Shared State

## Learning Objectives
- Read the ESP32-C3 on-die temperature sensor
- Share state between Embassy async tasks
- Use `embassy_sync::mutex::Mutex` for safe concurrent access
- Build a two-task architecture with producer/consumer pattern

## The On-Die Temperature Sensor

The ESP32-C3 has a built-in temperature sensor (tsens) that measures the die temperature. It's not precise for ambient readings (the die runs ~30°C above room temperature), but it's perfect for learning sensor patterns without extra hardware.

```
┌─────────────────────────────┐
│         ESP32-C3            │
│                             │
│  ┌───────────────────────┐  │
│  │   Temperature Sensor  │  │
│  │   (on-die, ~50-60°C)  │  │
│  └───────────────────────┘  │
│                             │
│  ┌───────────────────────┐  │
│  │   GPIO8 — LED         │  │
│  └───────────────────────┘  │
│                             │
│  ┌───────────────────────┐  │
│  │   USB Serial Output   │  │
│  └───────────────────────┘  │
└─────────────────────────────┘
```

## Single-Task Version

Let's start simple — read the sensor and print every 2 seconds:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
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

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let temp_sensor = TemperatureSensor::new(peripherals.TSENS, Config::default()).unwrap();

    println!("Temperature sensor ready");

    loop {
        let temp = temp_sensor.get_temperature();
        println!("Die temperature: {}C", temp.to_celsius());
        Timer::after(Duration::from_secs(2)).await;
    }
}
```

Key points:
- `TemperatureSensor::new()` takes the `TSENS` peripheral
- `get_temperature()` returns a temperature value; `.to_celsius()` converts it to `f32`
- The sensor measures **die temperature**, not ambient — expect values around 50–60°C

## Sharing State Between Tasks

In Chapter 22 we spawned independent tasks. But what if one task produces data and another consumes it? We need **shared state**.

In Embassy, tasks cannot borrow from each other — they're independent async functions. The solution is a `static` variable protected by a mutex:

```
┌──────────────┐     ┌───────────────────────┐     ┌──────────────┐
│  main task   │────►│  static LATEST_TEMP   │◄────│ display_task │
│  (producer)  │     │  Mutex<Option<f32>>   │     │  (consumer)  │
│  writes temp │     └───────────────────────┘     │  reads temp  │
│  every 2s    │                                   │  every 5s    │
└──────────────┘                                   └──────────────┘
```

### The Mutex Type

In `no_std` there's no `std::sync::Mutex`. Embassy provides its own [`Mutex`](https://docs.embassy.dev/embassy-sync/git/default/mutex/struct.Mutex.html) that takes a raw mutex implementation as a type parameter. For single-core microcontrollers like ESP32-C3, [`CriticalSectionRawMutex`](https://docs.embassy.dev/embassy-sync/git/default/blocking_mutex/raw/struct.CriticalSectionRawMutex.html) is the right choice — it briefly disables interrupts to guarantee exclusive access:

```rust
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

// Shared state: latest temperature reading
static LATEST_TEMP: Mutex<CriticalSectionRawMutex, Option<f32>> =
    Mutex::new(None);
```

- `CriticalSectionRawMutex` — the locking strategy (disable interrupts, safe on single-core)
- `Option<f32>` — `None` until the first reading arrives
- **`static`** — lives for the entire program, accessible from any task

#### Why `static` instead of `Arc<Mutex<T>>`?

On day 3, we shared state between threads using `Arc<Mutex<T>>` — reference-counted smart pointers that track ownership at runtime. On a `no_std` microcontroller, there's no heap allocator (so no `Arc`), and Embassy tasks aren't OS threads. Instead, we use a **`static`** variable: it has a fixed memory address known at compile time, so any task can access it without heap allocation or reference counting. The mutex still ensures only one task accesses the data at a time.

## Two-Task Architecture

Now let's split reading and displaying into separate tasks:

```rust
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
```

### Why Does the Sensor Stay in Main?

The `TemperatureSensor` type is not `Send` — it can't be moved across task boundaries. Since Embassy tasks are like independent async functions, the sensor must live in the task that created it. We keep it in `main` and share only the `f32` result through the mutex.

### Lock Scope

Notice the braces around the lock:

```rust
{
    let mut guard = LATEST_TEMP.lock().await;
    *guard = Some(celsius);
}  // lock released here
```

The mutex guard is dropped at the closing brace, releasing the lock immediately. This minimizes the time the lock is held, which is good practice even on single-core systems.

## How It Works

```
Time ──────────────────────────────────────────────────────►
Sensor:  [read]  await 2s  [read]  await 2s  [read]  await 2s
Display:              await 5s              [print]  await 5s
Shared:  None → Some(52.1)  →  Some(52.3)  →  Some(51.8)
```

The display task always sees the **most recent** reading, even though the sensor reads more frequently than the display prints.

## Exercise

Add a third task: an LED that blinks based on temperature.

1. Create a `led_task` that reads `LATEST_TEMP` every second
2. If the temperature is above 55°C, blink fast (200ms)
3. If below 55°C, blink slow (1000ms)
4. **Hint**: You'll need to pass the LED pin. Since Embassy tasks can't take non-`Send` borrows, create the `Output` inside the task by passing the GPIO pin, or use another `static` for the threshold

**Challenge**: Instead of a fixed threshold, add a second `static Mutex` for the threshold value. Have the display task update the threshold based on the average of the last few readings.
