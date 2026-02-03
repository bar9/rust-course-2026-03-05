# The `#![cfg_attr(not(test), no_std)]` Pattern

Minimal example demonstrating how to test `no_std` embedded code on a host machine.

## The Core Pattern

```rust
#![cfg_attr(not(test), no_std)]
```

This single line enables:
- **`cargo build`**: Compiles as `no_std` for ARM embedded target (thumbv7m-none-eabi)
- **`cargo test`**: Compiles with `std` for host machine (aarch64-apple-darwin or x86_64)

## Key Challenge Solved

The pattern addresses the fundamental problem that:
- Embedded code must target a different architecture without `std`
- Tests need to run on the host architecture with `std` available

## File Structure

```
src/lib.rs      - Library with conditional compilation
src/main.rs     - no_std binary (excluded from tests)
.cargo/config.toml - Configures default ARM target for builds
```

## The Implementation

```rust
// src/lib.rs
#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use std::vec::Vec;      // std::Vec in tests

#[cfg(not(test))]
use heapless::Vec;      // Fixed-size Vec for embedded

pub struct DataBuffer<const N: usize> {
    #[cfg(test)]
    data: Vec<u32>,     // Dynamic for tests

    #[cfg(not(test))]
    data: Vec<u32, N>,  // Fixed-size for embedded
}
```

## Cross-Architecture Configuration

```toml
# .cargo/config.toml
[build]
target = "thumbv7m-none-eabi"  # ARM Cortex-M3 (default for cargo build)
```

## Usage

```bash
# Build for ARM embedded target (no_std)
cargo build
# → Creates target/thumbv7m-none-eabi/debug/libtesting2.rlib

# Run tests on host machine (with std)
cargo test --target=aarch64-apple-darwin
# → Tests run with full std library support
```

## Why This Works

1. **Conditional Compilation**: `cfg_attr` applies attributes based on conditions
2. **Test Detection**: `cfg(test)` is true only when running tests
3. **Architecture Separation**: Different targets for build vs test
4. **Conditional Types**: Same API with different implementations

This pattern is essential for embedded Rust development, allowing rapid testing on development machines while maintaining `no_std` compatibility for deployment.