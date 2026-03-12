# Chapter 22: Blinky with Embassy

## Learning Objectives
- Understand what Embassy is and why it matters
- Convert a blocking blink to an async blink
- Run multiple async tasks concurrently
- Compare blocking vs async approaches

## What is Embassy?

[Embassy](https://embassy.dev/) is an async runtime for embedded systems — think of it as **tokio for bare metal**. Instead of busy-waiting (spinning the CPU doing nothing), Embassy lets you `await` timers, I/O, and other events while the CPU sleeps or runs other tasks.

```
Blocking (Chapter 21)          Async (Embassy)
┌──────────────┐               ┌──────────────┐
│ set_high()   │               │ set_high()   │
│ SPIN 500ms   │ ← CPU busy   │ await 500ms  │ ← CPU sleeps
│ set_low()    │               │ set_low()    │
│ SPIN 500ms   │ ← CPU busy   │ await 500ms  │ ← CPU sleeps
└──────────────┘               └──────────────┘
```

With Embassy you can also run **multiple tasks concurrently** on a single core — no threads, no OS needed.

## Generate a New Project

```bash
esp-generate --chip esp32c3 -o embassy -o unstable-hal blinky-embassy
```

Select the defaults in the TUI. The `-o` flag pre-selects TUI options so you skip the interactive prompts for those choices: `-o embassy` adds Embassy support via `esp-rtos`, and `-o unstable-hal` enables the unstable HAL features that Embassy requires.

## Dependencies

```toml
[dependencies]
esp-hal = { version = "1.0.0", features = ["esp32c3", "unstable"] }
esp-rtos = { version = "0.2.0", features = ["embassy", "esp32c3"] }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"] }
embassy-executor = { version = "0.9.1", features = [] }
embassy-time = "0.5.0"
critical-section = "1.2.0"
```

The new dependencies compared to Chapter 21:
- **esp-rtos**: Lightweight RTOS layer that integrates Embassy with esp-hal (sets up timers and the executor)
- **embassy-executor**: The async task executor (scheduler)
- **embassy-time**: Async timers (`Timer::after()`)
- **unstable feature on esp-hal**: Required for Embassy integration

## Async Blink

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize the async runtime
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
```

### What Changed

| Blocking (Ch 21) | Async (Embassy) |
|-------------------|-----------------|
| `#[esp_hal::main]` | `#[esp_rtos::main]` |
| `fn main() -> !` | `async fn main(_spawner: Spawner) -> !` |
| `use esp_hal::time::{Duration, Instant}` | `use embassy_time::{Duration, Timer}` |
| Busy-wait spin loop | `Timer::after(...).await` |
| `set_high()` + `set_low()` | `led.toggle()` |
| — | `esp_rtos::start(...)` to initialize the runtime |

Key differences:
- **`#[esp_rtos::main]`**: Entry point macro that sets up the Embassy executor
- **`esp_rtos::start()`**: Connects Embassy to a hardware timer and software interrupt for task scheduling
- **`async fn main`**: Our `main` is now an async task
- **`Spawner`**: Used to spawn additional async tasks (we'll use it below)
- **`Timer::after().await`**: Suspends the task — the CPU can sleep instead of spinning
- **`led.toggle()`**: Flips the pin state — simpler than separate `set_high`/`set_low`

## Multiple Tasks

The real power of Embassy is running concurrent tasks. Let's add a second task that prints a message periodically:

```rust
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

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // Spawn the heartbeat task — runs concurrently
    spawner.spawn(heartbeat()).unwrap();

    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
```

Both tasks run on the **same core**, interleaving at `await` points. No threads, no mutexes, no data races.

```
Time ──────────────────────────────────────────►
LED task:    [on]  await  [off]  await  [on]  await  [off]  await
Heartbeat:                [print] await              ...          [print] await
CPU:         run   sleep  run    sleep  run   sleep  run   sleep
```

### Task Rules

- Tasks are defined with `#[embassy_executor::task]`
- Tasks must be `async fn` with no return value (or `-> !`)
- Tasks are spawned with `spawner.spawn(task_name()).unwrap()`
- Each task function can only have **one instance** running at a time (Embassy limitation)
- Tasks cannot borrow local data from `main` — they must own their data or use `static`s

## Blocking vs Async: When to Use What

| | Blocking | Async (Embassy) |
|---|---------|-----------------|
| **Complexity** | Simpler | Slightly more setup |
| **CPU usage** | 100% during delays | Near 0% during delays |
| **Multiple activities** | Hard (manual state machines) | Easy (spawn tasks) |
| **Best for** | Simple single-purpose code | Anything with concurrency |

For most real projects, Embassy is worth the small added complexity.

## Exercise

Starting from the single-task async blink example (not the multi-task version):

1. Add `esp-println` to the dependencies
2. Create a second task that prints "tick" every 3 seconds
3. Observe how both the LED and the serial output run concurrently
4. **Challenge**: Create a task that blinks the LED in a pattern (e.g., SOS in Morse code: `··· −−− ···`)
