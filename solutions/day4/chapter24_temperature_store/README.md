# Chapter 24: Temperature Store

## Running tests

Since this is an embedded (`no_std`) project targeting `riscv32imc-unknown-none-elf`, tests cannot run on the default target. The pure-Rust library tests can be run on the host with:

```sh
cargo test --no-default-features --lib --target x86_64-unknown-linux-gnu
```

Adapt the `--target` to your host platform if needed (e.g. `aarch64-apple-darwin` on Apple Silicon).

- `--no-default-features` disables the `embedded` feature, skipping ESP-HAL and other hardware dependencies
- `--target <host>` overrides the RISC-V default set in `.cargo/config.toml`
- `--lib` runs only the library unit tests
