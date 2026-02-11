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
└── day4/                    # Day 4: imgforge — Image Processing Service
    └── imgforge/            # Single incremental project built across chapters 20-25
        ├── src/             # CLI, web server, FFI, batch processing
        └── tests/           # Unit + integration tests
```

Day 3 has 3 exercise projects (testing, serde, concurrency). The remaining Day 3 chapters (Cargo, macros, unsafe/FFI, design patterns) are theory-only.

### Day 4 — imgforge

The `imgforge` project is a single Cargo crate that evolves across 6 chapters:

| Chapter | What's added | Key files |
|---------|-------------|-----------|
| Ch20 | CLI tool, project structure | main.rs, lib.rs, error.rs, config.rs, transform.rs |
| Ch21 | FFI backend (optional) | transform_turbojpeg.rs |
| Ch22 | Axum HTTP server | server.rs |
| Ch23 | Concurrent job processing | server.rs (jobs), spawn_blocking |
| Ch24 | Tests, batch mode | tests/, batch.rs |
| Ch25 | Optional GUI | (stretch goal) |

```bash
# Build and test
cargo build --manifest-path solutions/day4/imgforge/Cargo.toml
cargo test --manifest-path solutions/day4/imgforge/Cargo.toml
```

## Source Information

- **Day 1 & 2 solutions**: Cloned from the `solutions` branch of https://github.com/bar9/rust-course-2025-09-18

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