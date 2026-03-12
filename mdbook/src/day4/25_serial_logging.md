# Chapter 25: Serial Commands & JSON Output

## Learning Objectives
- Read serial input using `UsbSerialJtag` with async
- Parse text commands in `no_std`
- Serialize data to JSON with `serde` and `serde-json-core`
- Build a testable command parser

## What's New

Chapters 20–24 only *wrote* to serial via `println!`. Now we'll also *read* from it — turning the ESP32-C3 into an interactive device that responds to typed commands with JSON output.

| Chapter | What we learned |
|---------|----------------|
| 20 | Hello World — serial output, project setup |
| 21 | Blinky — GPIO, blocking delays |
| 22 | Embassy — async tasks, concurrency |
| 23 | Temperature sensor — shared state with Mutex |
| 24 | Temperature store — testable no_std library |
| **25** | **Serial input, command parsing, serde + JSON** |

## Architecture

```
             USB Serial (same cable as flashing)
                 │
      ┌──────────┴──────────┐
      │ UsbSerialJtag       │
      │   TX ──► println!   │  (output — we've been using this)
      │   RX ◄── terminal   │  (input — NEW)
      └─────────────────────┘
                 │
      ┌──────────┴──────────┐
      │   command_task       │
      │   reads lines,       │
      │   parses commands    │
      └──────────┬──────────┘
                 │ locks
                 ▼
      ┌──────────────────────┐
      │  static STORE        │
      │  Mutex<TempStore>    │
      └──────────────────────┘
                 ▲ locks
                 │
      ┌──────────┴──────────┐
      │   main task          │
      │   reads sensor,      │
      │   updates store      │
      └─────────────────────┘
```

The user types commands in their terminal (e.g., `picocom`, `minicom`, or the VS Code serial monitor). The `command_task` reads bytes asynchronously, assembles lines, and responds.

## Reading Serial Input

### The `UsbSerialJtag` Peripheral

The ESP32-C3's USB port is a **USB Serial/JTAG** peripheral. Until now, `esp_println` wrote to it directly using raw register access. To *read* from it, we use the `esp_hal::usb_serial_jtag::UsbSerialJtag` HAL driver:

```rust
use esp_hal::usb_serial_jtag::UsbSerialJtag;

// Take the peripheral, convert to async mode, split into RX and TX
let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();
let (rx, _tx) = usb_serial.split();
```

- `into_async()` — enables interrupt-driven async I/O (no busy-waiting)
- `split()` — separates into independent RX and TX halves so they can live in different tasks
- We keep `_tx` unused because `esp_println` handles output via direct register writes

### Async Byte Reading

The RX half implements [`embedded_io_async::Read`](https://docs.rs/embedded-io-async/0.6/embedded_io_async/trait.Read.html), which gives us an `.await`-able read:

```rust
use embedded_io_async::Read;

let mut byte = [0u8; 1];
let n = Read::read(&mut rx, &mut byte).await;
```

This call **yields** to the executor until a byte arrives — no CPU cycles wasted spinning. Compare this to the blocking `read_byte()` from `nb` which would monopolize the core.

### Building Lines from Bytes

Serial terminals send one byte at a time. We accumulate them into a buffer until we see `\r` or `\n`:

```rust
#[embassy_executor::task]
async fn command_task(
    mut rx: UsbSerialJtagRx<'static, Async>,
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

        if b == b'\r' || b == b'\n' {
            if pos > 0 {
                if let Ok(line) = core::str::from_utf8(&buf[..pos]) {
                    handle_command(line).await;
                }
                pos = 0;
            }
            continue;
        }

        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }
}
```

Key points:
- Fixed `[u8; 64]` buffer — no heap allocation needed
- `core::str::from_utf8` validates that the bytes are valid UTF-8 before parsing
- The `pos` index resets after each command

## The Command Parser

This is pure Rust — no hardware, fully testable on the host:

```rust
// src/command.rs

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Status,
    SetOffset(f32),
    Help,
    Unknown(&'a str),
}

pub fn parse(input: &str) -> Command<'_> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Command::Unknown(trimmed);
    }

    let (cmd, arg) = match trimmed.find(' ') {
        Some(pos) => (&trimmed[..pos], trimmed[pos + 1..].trim()),
        None => (trimmed, ""),
    };

    match cmd {
        "status" => Command::Status,
        "help" => Command::Help,
        "offset" => match arg.parse::<f32>() {
            Ok(val) => Command::SetOffset(val),
            Err(_) => Command::Unknown(trimmed),
        },
        _ => Command::Unknown(trimmed),
    }
}
```

Note the `Command<'a>` lifetime — `Unknown` borrows from the input string rather than allocating a copy. This is idiomatic `no_std` Rust: avoid allocation by borrowing.

### Testing the Parser

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status() {
        assert_eq!(parse("status"), Command::Status);
        assert_eq!(parse("  status  "), Command::Status);
    }

    #[test]
    fn parse_offset_valid() {
        assert_eq!(parse("offset 25.5"), Command::SetOffset(25.5));
        assert_eq!(parse("offset -10"), Command::SetOffset(-10.0));
    }

    #[test]
    fn parse_offset_missing_value() {
        assert_eq!(parse("offset"), Command::Unknown("offset"));
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(parse("reboot"), Command::Unknown("reboot"));
    }
}
```

These run on your laptop with `./test.sh` — no ESP32 needed.

## JSON Output with Serde

### Why Serde in `no_std`?

Standard `serde_json` needs an allocator. For embedded, [`serde-json-core`](https://docs.rs/serde-json-core) serializes directly into a fixed-size `[u8]` buffer — zero heap allocation.

### A Serializable Reading

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct Reading {
    pub raw_celsius: f32,
    pub ambient_celsius: f32,
    pub ambient_fahrenheit: f32,
    pub offset: f32,
}
```

The `TemperatureStore` produces a `Reading` snapshot:

```rust
impl TemperatureStore {
    pub fn reading(&self) -> Option<Reading> {
        let raw = self.raw_celsius?;
        let amb_c = raw - self.offset;
        Some(Reading {
            raw_celsius: raw,
            ambient_celsius: amb_c,
            ambient_fahrenheit: amb_c * 9.0 / 5.0 + 32.0,
            offset: self.offset,
        })
    }
}
```

### Serializing to a Buffer

```rust
let mut json_buf = [0u8; 128];
let len = serde_json_core::to_slice(&reading, &mut json_buf).unwrap();
let json = core::str::from_utf8(&json_buf[..len]).unwrap();
println!("{}", json);
```

Output:
```json
{"raw_celsius":52.3,"ambient_celsius":22.3,"ambient_fahrenheit":72.14,"offset":30.0}
```

### Testing Serialization

```rust
#[test]
fn reading_serializes_to_json() {
    let mut store = TemperatureStore::new(30.0);
    store.update(52.0);
    let reading = store.reading().unwrap();

    let mut buf = [0u8; 128];
    let len = serde_json_core::to_slice(&reading, &mut buf).unwrap();
    let json = core::str::from_utf8(&buf[..len]).unwrap();

    assert!(json.contains("\"raw_celsius\""));
    assert!(json.contains("\"ambient_celsius\""));
    assert!(json.contains("\"offset\""));
}
```

## Handling Commands

The `handle_command` function connects the parser to the store:

```rust
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
```

Note the `{{` in the error messages — that's Rust's escape for literal braces inside `println!` format strings.

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"

# ... embedded deps unchanged from chapter 24, plus:
embedded-io-async = { version = "0.6", optional = true }
```

- `serde` with `default-features = false` — disables std, keeps `#[derive(Serialize)]`
- `serde-json-core` — no_std JSON serializer using fixed buffers
- `embedded-io-async` — the `Read` trait for async serial I/O

## Expected Session

```
=== Temperature Monitor ===
Type 'help' for commands

> help
Commands:
  status         — print current reading as JSON
  offset <value> — set die-to-ambient offset
  help           — show this message

> status
{"raw_celsius":52.3,"ambient_celsius":22.3,"ambient_fahrenheit":72.14,"offset":30.0}

> offset 28
OK offset=28

> status
{"raw_celsius":52.1,"ambient_celsius":24.1,"ambient_fahrenheit":75.38,"offset":28.0}
```

## Day 4 Recap

Over six chapters, we went from zero to an interactive embedded system:

| Chapter | Concept | Key takeaway |
|---------|---------|-------------|
| 20 | Hello World | `no_std`, `no_main`, serial output |
| 21 | Blinky | GPIO, blocking delays, hardware control |
| 22 | Embassy | Async tasks, `await` instead of spin |
| 23 | Temperature sensor | Shared state with `Mutex`, producer/consumer |
| 24 | Temperature store | Testable `no_std` library, `cfg_attr`, feature flags |
| 25 | Serial commands | Serial input, command parsing, serde + JSON |

Each chapter built on the previous one. The same Rust concepts from Days 1–3 (ownership, traits, modules, testing) apply directly to embedded — just with `no_std` constraints.

## Exercise

Add a `history` command that returns the last N readings as a JSON array:

1. Extend `TemperatureStore` with a ring buffer (fixed-size array, e.g., `[Option<f32>; 8]`)
2. Each `update()` stores the reading in the next slot
3. Add a `history()` method that returns a serializable struct containing the buffered readings
4. Parse `"history"` as a new `Command` variant
5. Respond with JSON like: `{"readings":[52.1,51.8,52.3],"count":3}`
6. Write tests for the ring buffer (empty, partial, full, wrap-around) and run them with `./test.sh`

**Hint**: `heapless::Vec<f32, 8>` (already in your dependency tree via `embassy`) is a stack-allocated `Vec` that implements `Serialize` — it's a convenient alternative to a raw array + index.
