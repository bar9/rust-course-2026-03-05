# Solutions Directory

This directory contains the complete solutions for the Rust Course, organized by day and lesson.

## Structure

```
solutions/
├── day1/                    # Day 1: Foundations & Ownership
│   ├── 02_fundamentals/     # Rust Fundamentals exercises
│   ├── 03_structs_enums/    # Structs, Enums, and Methods
│   ├── 04_ownership/        # Memory Model & Ownership
│   └── 05_smart_pointers/   # Ownership Patterns & Smart Pointers
├── day2/                    # Day 2: Type System & Error Handling
│   └── (comprehensive exercises covering collections, traits, generics, etc.)
├── day3/                    # Day 3: Ecosystem & Advanced Topics
│   ├── 14_testing/          # Testing exercises (MarkdownProcessor)
│   ├── 17_serde/            # Serde serialization exercises
│   └── 18_concurrency/      # Async/concurrency exercises (Parallel WordCounter)
└── day4/                    # Day 4: ESP32-C3 Embedded Systems
    ├── chapter20_hardware_hello/    # LED blink, temperature sensor basics
    ├── chapter21_data_structures/   # no_std data structures, circular buffer
    ├── chapter22_testing/           # Testing embedded code on desktop
    ├── chapter23_communication/     # JSON serialization over serial
    ├── chapter24_integration/       # Production integration & deployment
    ├── chapter25_extensions/        # Power management & optimization
    └── esp_generate_output/         # Helper for generating expected output
```

Day 3 has 3 exercise projects (testing, serde, concurrency). The remaining Day 3 chapters (Cargo, macros, unsafe/FFI, design patterns) are theory-only.

### Day 4 — ESP32-C3 Embedded Systems

Day 4 consists of 6 incremental chapters building an embedded temperature monitoring system on the ESP32-C3 microcontroller:

| Chapter | Topic | Solution directory |
|---------|-------|--------------------|
| Ch 20 | Hardware hello — LED blink, temperature sensor | `chapter20_hardware_hello` |
| Ch 21 | Embedded foundations — no_std, heapless collections | `chapter21_data_structures` |
| Ch 22 | Testing — desktop testing of embedded code | `chapter22_testing` |
| Ch 23 | Communication — JSON over serial with serde | `chapter23_communication` |
| Ch 24 | Integration — production deployment | `chapter24_integration` |
| Ch 25 | Performance — power management & optimization | `chapter25_extensions` |

```bash
# Run tests (desktop, no hardware required)
cargo test --manifest-path solutions/day4/chapter22_testing/Cargo.toml --no-default-features
```

## Source Information

- **Day 1 & 2 solutions**: Cloned from the `solutions` branch of https://github.com/bar9/rust-course-2025-09-18
- **Day 4 solutions**: Ported from https://github.com/bar9/rust-course-2025-11-20 (old Day 3)

## Usage

Each directory contains a complete Rust project with `Cargo.toml` and source files. To run any solution:

```bash
cd solutions/day1/02_fundamentals
cargo run
```

To run tests (where available):

```bash
cargo test
```
