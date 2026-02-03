# Chapter 1: Course Introduction & Setup

## Development Environment Setup

Let's get your Rust development environment ready. Rust's tooling is excellent - you'll find it more unified than C++ and more performant than .NET.

### Installing Rust

The recommended way to install Rust is through `rustup`, Rust's official toolchain manager.

#### On Unix-like systems (Linux/macOS):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### On Windows:
Download and run the installer from [rustup.rs](https://rustup.rs/)

After installation, verify:
```bash
rustc --version
cargo --version
```

### Understanding the Rust Toolchain

| Tool | Purpose | C++ Equivalent | .NET Equivalent |
|------|---------|----------------|-----------------|
| `rustc` | Compiler | `g++`, `clang++` | `csc`, `dotnet build` |
| `cargo` | Build system & package manager | `cmake` + `conan`/`vcpkg` | `dotnet` CLI + NuGet |
| `rustup` | Toolchain manager | - | .NET SDK manager |
| `clippy` | Linter | `clang-tidy` | Code analyzers |
| `rustfmt` | Formatter | `clang-format` | `dotnet format` |


### Your First Rust Project

Let's create a Hello World project to verify everything works:

```bash
cargo new hello_rust
cd hello_rust
```

This creates:
```
hello_rust/
├── Cargo.toml    # Like CMakeLists.txt or .csproj
└── src/
    └── main.rs   # Entry point
```

Look at `src/main.rs`:
```rust
fn main() {
    println!("Hello, world!");
}
```

Run it:
```bash
cargo run
```

### Understanding Cargo

Cargo is Rust's build system and package manager. Coming from C++ or .NET, you'll love its simplicity.

#### Key Cargo Commands

| Command | Purpose | Similar to |
|---------|---------|------------|
| `cargo new` | Create new project | `dotnet new`, `cmake init` |
| `cargo build` | Compile project | `make`, `dotnet build` |
| `cargo run` | Build & run | `./a.out`, `dotnet run` |
| `cargo test` | Run tests | `ctest`, `dotnet test` |
| `cargo doc` | Generate documentation | `doxygen` |
| `cargo check` | Fast syntax/type check | Incremental compilation |

#### Debug vs Release Builds

```bash
cargo build          # Debug build (./target/debug/)
cargo build --release # Optimized build (./target/release/)
```

**Performance difference is significant!** Debug builds include:
- Overflow checks
- Debug symbols
- No optimizations

### Project Structure Best Practices

A typical Rust project structure:

```
my_project/
├── Cargo.toml           # Project manifest
├── Cargo.lock          # Dependency lock file (like package-lock.json)
├── src/
│   ├── main.rs         # Binary entry point
│   ├── lib.rs          # Library entry point
│   └── module.rs       # Additional modules
├── tests/              # Integration tests
│   └── integration_test.rs
├── benches/            # Benchmarks
│   └── benchmark.rs
├── examples/           # Example programs
│   └── example.rs
└── target/             # Build artifacts (gitignored)
```

### Comparing with C++/.NET

#### C++ Developers
- No header files! Modules are automatically resolved
- No makefiles to write - Cargo handles everything
- Dependencies are downloaded automatically (like vcpkg/conan)
- No undefined behavior in safe Rust

#### .NET Developers
- Similar project structure to .NET Core
- `Cargo.toml` is like `.csproj`
- crates.io is like NuGet
- No garbage collector - deterministic destruction

### Quick Wins: Why You'll Love Rust's Tooling

1. **Unified tooling**: Everything works together seamlessly
2. **Excellent error messages**: The compiler teaches you Rust
3. **Fast incremental compilation**: cargo check is lightning fast
4. **Built-in testing**: No need for external test frameworks
5. **Documentation generation**: Automatic API docs from comments

### Setting Up for Success

#### Enable Useful Rustup Components

```bash
rustup component add clippy       # Linter
rustup component add rustfmt      # Formatter
rustup component add rust-src     # Source code for std library
```

#### Create a Learning Workspace

Let's set up a workspace for this course:

```bash
mkdir rust-course-workspace
cd rust-course-workspace
cargo new --bin day1_exercises
cargo new --lib day1_library
```

### Common Setup Issues and Solutions

| Issue | Solution |
|-------|----------|
| "rustc not found" | Restart terminal after installation |
| Slow compilation | Enable sccache: `cargo install sccache` |
| Can't debug | Zed has built-in debugging support |
| Windows linker errors | Install Visual Studio Build Tools |

## Exercises

### Exercise 1.1: Toolchain Exploration
Create a new project and explore these cargo commands:
- `cargo tree` - View dependency tree
- `cargo doc --open` - Generate and view documentation
- `cargo clippy` - Run the linter

### Exercise 1.2: Build Configurations
1. Create a simple program that prints the numbers 1 to 1_000_000
2. Time the difference between debug and release builds
3. Compare binary sizes

### Exercise 1.3: First Debugging Session
1. Create a program with an intentional panic
2. Set a breakpoint in Zed
3. Step through the code with the debugger

## Key Takeaways

✅ Rust's tooling is unified and modern - no need for complex build systems

✅ Cargo handles dependencies, building, testing, and documentation

✅ Debug vs Release builds have significant performance differences

✅ The development experience is similar to modern .NET, better than typical C++

✅ Zed with built-in rust-analyzer provides excellent IDE support

---

Next up: [Chapter 2: Rust Fundamentals](./02_fundamentals.md) - Let's write some Rust!