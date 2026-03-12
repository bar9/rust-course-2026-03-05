# Chapter 21: Blinky

## Learning Objectives
- Control GPIO pins to drive an LED
- Understand digital output (high/low)
- Implement a busy-wait delay loop
- Build the classic "Hello World" of embedded systems

## The Classic Blinky

Blinking an LED is the embedded equivalent of "Hello World" — it proves you can control hardware. Most ESP32-C3 development boards have a built-in LED connected to **GPIO8**.

## Starting Point

We start from the same `esp-generate` project structure as Chapter 20. The only dependency we need is `esp-hal` — no `esp-println` required.

```toml
[dependencies]
esp-hal = { version = "1.0.0", features = ["esp32c3"] }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"] }
critical-section = "1.2.0"
```

## The Code

```rust
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
        led.set_high();  // LED on

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}

        led.set_low();   // LED off

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }
}
```

## How GPIO Works

GPIO stands for **General Purpose Input/Output**. Each pin can be configured as either input or output:

```
                    ESP32-C3
                  ┌───────────┐
                  │           │
  GPIO8 ──────── │  Output   │ ──── LED
                  │  Register │
                  │           │
                  └───────────┘

  set_high() → pin outputs 3.3V → LED turns on
  set_low()  → pin outputs 0V   → LED turns off
```

### Output::new()

```rust
let mut led = Output::new(
    peripherals.GPIO8,          // Which pin
    Level::Low,                 // Initial state
    OutputConfig::default(),    // Default drive strength, no pull-up/down
);
```

The three arguments:
- **Pin**: `peripherals.GPIO8` — Rust's ownership system ensures only one part of your code controls this pin
- **Initial level**: `Level::Low` — start with LED off
- **Config**: Drive strength and other electrical settings (defaults are fine)

### Ownership

Notice that `peripherals.GPIO8` is *moved* into `Output::new()`. This is Rust's ownership system at work — you can't accidentally configure the same pin twice from different parts of your code. This is a compile-time guarantee that prevents a whole class of hardware bugs.

## The Busy-Wait Delay

```rust
let delay_start = Instant::now();
while delay_start.elapsed() < Duration::from_millis(500) {}
```

This is a **busy-wait** (also called spin-loop): the CPU continuously checks the clock until 500ms have passed. It's simple but wasteful — the CPU can't do anything else during the wait.

> In Chapter 22 we'll see how Embassy's async runtime solves this — `Timer::after(Duration::from_millis(500)).await` lets the CPU sleep or do other work while waiting.

## Build & Flash

```bash
cargo run --release
```

You should see the LED blinking at 1 Hz (on for 500ms, off for 500ms).

## Exercise

1. Change the blink rate to 200ms on, 800ms off (asymmetric blink)
2. Add `esp-println` and print "ON" / "OFF" each time the LED changes state
3. Create a pattern: 3 fast blinks (100ms), then a 1-second pause, repeat
