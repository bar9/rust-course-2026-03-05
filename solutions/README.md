# Solutions Directory

This directory contains the complete solutions for the Rust Course 2025, organized by day and lesson.

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
└── day3/                    # Day 3: Embedded Systems Programming
    └── capstone/            # Complete Temperature Monitoring System
        ├── temp_async/      # Async programming components
        ├── temp_core/       # Core functionality
        ├── temp_embedded/   # Embedded-specific code
        ├── temp_esp32/      # ESP32-C3 specific implementation
        ├── temp_protocol/   # Communication protocols
        └── temp_store/      # Data storage components
```

## Source Information

- **Day 1 & 2 solutions**: Cloned from the `solutions` branch of https://github.com/bar9/rust-course-2025-09-18
- **Day 3 solutions**: Cloned from the `main` branch of https://github.com/bar9/rust-course-2025-09-18

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