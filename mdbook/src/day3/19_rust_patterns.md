# Chapter 19: Rust Design Patterns

## Learning Objectives

- Apply library-first project structure with a thin `main.rs`
- Use trait-based backends for testability and runtime flexibility
- Define crate-level error enums with `derive_more::From` and a `Result<T>` alias
- Use feature flags to conditionally compile modules and dependencies
- Recognize the Builder, Newtype, and Type-State patterns
- Understand RAII via the `Drop` trait and zero-copy techniques with `Cow`

This chapter covers design patterns that come up repeatedly in production Rust code. Several of them are applied directly in the Day 4 imgforge project.

---

## 1. Thin `main.rs` -- Library-First Structure

In idiomatic Rust, `main.rs` does as little as possible. It parses configuration (or CLI arguments), then delegates to the library crate defined in `lib.rs`.

```rust,ignore
fn main() -> myapp::Result<()> {
    let config = myapp::Config::parse();
    myapp::run(config)
}
```

All application logic lives in the library. This has concrete benefits:

- **Testability** -- integration tests (`tests/*.rs`) can only access the library crate, not `main.rs`. If your logic is in `main`, it cannot be tested from integration tests.
- **Reusability** -- other binaries in the same crate (e.g., a CLI and a server) share the library without duplication.
- **Benchmarks** -- criterion benchmarks import from the library crate the same way tests do.

A typical project layout:

```text
myapp/
  Cargo.toml
  src/
    main.rs        # 3-5 lines: parse config, call lib
    lib.rs         # declares modules, re-exports public API
    config.rs      # CLI argument parsing (clap)
    error.rs       # crate-level Error enum + Result alias
    transform.rs   # core logic
  tests/
    integration.rs # imports myapp as a library
```

Production Rust projects like ripgrep and cargo itself follow this pattern. The Day 4 imgforge project (Chapter 20) uses it from the start.

---

## 2. Trait-Based Backends

Define behavior as a trait, then provide multiple implementations. Consumers depend on the trait, not a concrete type.

```rust,ignore
pub trait Transform: Send + Sync {
    fn apply(&self, input: &[u8], op: &Operation) -> Result<Vec<u8>>;
    fn name(&self) -> &str;
}
```

Two backends can implement this trait:

```rust,ignore
pub struct ImageRsBackend;

impl Transform for ImageRsBackend {
    fn apply(&self, input: &[u8], op: &Operation) -> Result<Vec<u8>> {
        // Use the pure-Rust `image` crate
        todo!()
    }
    fn name(&self) -> &str { "image-rs" }
}
```

```rust,ignore
pub struct MockBackend;

impl Transform for MockBackend {
    fn apply(&self, input: &[u8], _op: &Operation) -> Result<Vec<u8>> {
        // Return input unchanged -- useful for tests
        Ok(input.to_vec())
    }
    fn name(&self) -> &str { "mock" }
}
```

### Selecting the backend

At **compile time** (static dispatch via generics):

```rust,ignore
fn process<T: Transform>(backend: &T, data: &[u8], op: &Operation) -> Result<Vec<u8>> {
    backend.apply(data, op)
}
```

At **runtime** (dynamic dispatch via trait objects):

```rust,ignore
fn create_backend(use_turbo: bool) -> Box<dyn Transform> {
    if use_turbo {
        Box::new(TurboJpegBackend::new())
    } else {
        Box::new(ImageRsBackend)
    }
}
```

The `Send + Sync` bounds on the trait allow the boxed backend to be shared across threads (e.g., stored in `Arc` and passed to async handlers).

This pattern enables testing without modifying production code -- pass `MockBackend` in tests, `ImageRsBackend` in production. It is central to the Day 4 imgforge architecture (Chapters 20-24).

---

## 3. Crate-Level Error Enum + `Result<T>` Alias

A single `Error` enum per crate (or architectural layer) collects every error kind the crate can produce. A type alias shortens function signatures.

```rust,ignore
use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    // Domain errors -- constructed manually at call sites
    UnsupportedFormat,
    DimensionTooLarge { width: u32, height: u32 },

    // External errors -- auto-converted via `?`
    #[from]
    Io(std::io::Error),
    #[from]
    Image(image::ImageError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
```

### How it works

- **`#[from]` on a variant** auto-generates a `From<std::io::Error> for Error` impl (and similarly for `image::ImageError`). This is what makes `?` propagation work: when a function returns `std::io::Error` and the caller returns `Result<T, Error>`, the compiler uses the `From` impl to convert automatically.
- **Domain errors** like `UnsupportedFormat` have no `#[from]` attribute. They are constructed explicitly at the call site: `return Err(Error::UnsupportedFormat)`. This is intentional -- domain errors represent decisions, not mechanical conversions.
- **`Display` as `Debug`** (`write!(f, "{self:?}")`) is a pragmatic shortcut. For CLI and server error output, the Debug representation is often sufficient. If you later need user-friendly messages, implement `Display` properly per variant.
- **`core::result::Result`** on the right-hand side of the type alias makes it visually clear that we refer to the standard library's `Result`, not recursively referencing the alias being defined. Both `core::result::Result` and `std::result::Result` are the same type.

### Comparison with alternatives

| Approach | Pros | Cons |
|----------|------|------|
| `derive_more::From` | Lightweight, selective `#[from]`, minimal proc-macro | Manual `Display` impl |
| `thiserror` | Generates `Display` from `#[error("...")]` attributes | Heavier proc-macro for the same `From` generation |
| `anyhow` | Minimal boilerplate, good for scripts and prototypes | Erases error type -- callers cannot match on variants |

For libraries and applications where callers need to handle specific error variants, `derive_more::From` or `thiserror` are appropriate. `anyhow` is suited for top-level binaries where you only need to print the error and exit.

The Day 4 imgforge project (Chapter 20) uses this exact pattern with `derive_more::From`.

---

## 4. Feature-Gated Modules

Cargo feature flags enable conditional compilation of entire modules and their dependencies.

### Declaring features in `Cargo.toml`

```toml
[features]
default = []
server = ["dep:axum", "dep:tokio"]
gui    = ["dep:eframe", "dep:egui"]

[dependencies]
axum   = { version = "0.8", optional = true }
tokio  = { version = "1", features = ["full"], optional = true }
eframe = { version = "0.30", optional = true }
egui   = { version = "0.30", optional = true }
```

The `dep:` prefix (stabilized in Rust 1.60) makes the dependency optional without implicitly creating a feature of the same name. Before `dep:`, writing `axum = { optional = true }` would create both a dependency *and* a feature named `axum`, which led to confusion.

### Gating modules in `lib.rs`

```rust,ignore
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "gui")]
pub mod gui;
```

When a feature is not enabled, the module is not compiled at all -- its optional dependencies are not compiled, its code is not checked, and it does not appear in the binary.

### Gating within a function

```rust,ignore
pub fn create_backend() -> Box<dyn Transform> {
    #[cfg(feature = "turbojpeg")]
    {
        return Box::new(TurboJpegBackend::new());
    }

    #[cfg(not(feature = "turbojpeg"))]
    {
        Box::new(ImageRsBackend)
    }
}
```

### Benefits

- **Smaller binaries** -- users who only need the CLI do not carry the HTTP server stack.
- **Faster compile times** -- fewer dependencies to download and build.
- **No unused code** -- the compiler does not process gated modules unless requested.

Feature flags are used throughout Day 4: Chapter 21 (TurboJPEG backend), Chapter 22 (Axum server), and Chapter 25 (egui GUI).

---

## 5. Builder Pattern

The Builder pattern is useful when a struct has many optional fields or requires validation before construction. Rust has no function overloading or default parameter values, so builders fill that role.

```rust
use std::time::Duration;

#[derive(Debug)]
pub struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
    timeout: Duration,
}

impl ServerConfig {
    fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<usize>,
    timeout: Option<Duration>,
}

impl ServerConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn max_connections(mut self, n: usize) -> Self {
        self.max_connections = Some(n);
        self
    }

    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = Some(t);
        self
    }

    pub fn build(self) -> Result<ServerConfig, &'static str> {
        Ok(ServerConfig {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(8080),
            max_connections: self.max_connections.unwrap_or(100),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}

fn main() -> Result<(), &'static str> {
    let config = ServerConfig::builder()
        .host("localhost")
        .port(3000)
        .build()?;

    println!("{:?}", config);
    Ok(())
}
```

Each setter method takes `self` by value (not `&mut self`), which enables method chaining. The `build` method can enforce invariants and return a `Result` if required fields are missing.

The `derive_builder` crate can generate builder implementations automatically, but writing them by hand is straightforward and avoids a proc-macro dependency.

---

## 6. Newtype Pattern

Wrap a primitive type in a single-field struct to give it a distinct type. The compiler prevents accidental mixing of values that share the same underlying representation.

```rust
struct Kilometers(f64);
struct Miles(f64);
struct Liters(f64);
struct KmPerLiter(f64);

impl Kilometers {
    fn to_miles(&self) -> Miles {
        Miles(self.0 * 0.621371)
    }
}

fn calculate_fuel_efficiency(distance: Kilometers, fuel: Liters) -> KmPerLiter {
    KmPerLiter(distance.0 / fuel.0)
}

fn main() {
    let dist = Kilometers(100.0);
    let fuel = Liters(8.5);
    let efficiency = calculate_fuel_efficiency(dist, fuel);
    println!("{:.1} km/L", efficiency.0);
}
```

Calling `calculate_fuel_efficiency(fuel, dist)` with swapped arguments is a compile-time error, not a silent bug. The newtype has zero runtime cost -- the wrapper is erased during compilation.

Newtypes are also useful for implementing external traits on external types (the orphan rule requires that either the trait or the type is defined in the current crate).

---

## 7. Type-State Pattern

Encode state transitions in the type system so that invalid sequences are compile-time errors. The key ingredient is `PhantomData<State>` — a zero-sized marker type that exists only at compile time and tells the compiler which state the struct is in, without using any runtime memory.

```rust
struct Draft;
struct PendingReview;
struct Published;

struct Post<State> {
    content: String,
    _state: std::marker::PhantomData<State>,
}

impl Post<Draft> {
    fn new(content: impl Into<String>) -> Self {
        Post {
            content: content.into(),
            _state: std::marker::PhantomData,
        }
    }

    fn submit(self) -> Post<PendingReview> {
        Post {
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

impl Post<PendingReview> {
    fn approve(self) -> Post<Published> {
        Post {
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }

    fn reject(self) -> Post<Draft> {
        Post {
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

impl Post<Published> {
    fn content(&self) -> &str {
        &self.content
    }
}

fn main() {
    let post = Post::new("Hello, world!")
        .submit()
        .approve();

    println!("{}", post.content());

    // This would not compile -- cannot call .content() on a Draft:
    // let draft = Post::new("draft");
    // println!("{}", draft.content());
}
```

Each state is a zero-sized type. The generic parameter `State` controls which methods are available. Calling `.content()` on a `Post<Draft>` is a compile-time error -- the method simply does not exist for that type. The state types carry no runtime data, so the pattern has zero cost.

This pattern appears in libraries like `hyper` (request builders) and `tower` (service layers).

---

## 8. RAII and Drop Pattern

Rust's `Drop` trait provides deterministic resource cleanup. When a value goes out of scope, its `drop` method runs automatically. This is Rust's version of C++ RAII -- but enforced by the ownership system, so there is no risk of use-after-free.

```rust,ignore
use std::path::PathBuf;

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(name: &str, content: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, content)?;
        Ok(TempFile { path })
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
```

Usage:

```rust,ignore
fn process() -> std::io::Result<()> {
    let temp = TempFile::new("work.tmp", "temporary data")?;
    // ... use temp.path() ...
    Ok(())
}   // temp is dropped here -- file is deleted automatically
```

The file is cleaned up whether the function returns normally or propagates an error via `?`. This is the same guarantee that C++ destructors provide, but Rust additionally prevents accessing `temp` after it has been moved or dropped.

Common uses of `Drop` in practice: closing file handles, releasing locks, flushing buffers, cleaning up temporary directories, and disconnecting network connections.

---

## 9. Performance Patterns

### Zero-Copy with Borrowing

Passing `&[u8]` or `&str` instead of `Vec<u8>` or `String` avoids allocations when the caller already owns the data.

```rust
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn main() {
    let owned = String::from("hello world from Rust");
    let count = count_words(&owned);  // borrows, no allocation
    println!("{count} words");
}
```

### `Cow` -- Clone on Write

`Cow<'a, T>` holds either a borrowed reference or an owned value. It only allocates when modification is needed.

```rust
use std::borrow::Cow;

fn normalize_whitespace<'a>(input: &'a str) -> Cow<'a, str> {
    if input.contains('\n') {
        Cow::Owned(input.replace('\n', " "))
    } else {
        Cow::Borrowed(input)
    }
}

fn main() {
    let clean = "no newlines here";
    let dirty = "has\nnewlines\nin it";

    let result1 = normalize_whitespace(clean);   // Borrowed -- no allocation
    let result2 = normalize_whitespace(dirty);   // Owned -- allocated

    println!("{result1}");
    println!("{result2}");
}
```

`Cow` is particularly useful in functions where most inputs pass through unchanged but some need transformation. You avoid allocating in the common case while still supporting the uncommon one.

### Memory Layout Control

The `#[repr(C)]` attribute gives a struct C-compatible memory layout, which is required for FFI and sometimes useful for memory-mapped data.

```rust
#[repr(C)]
struct NetworkPacket {
    header: [u8; 4],
    length: u32,
    payload: [u8; 1024],
}

fn main() {
    println!("Packet size: {} bytes", std::mem::size_of::<NetworkPacket>());
}
```

Without `#[repr(C)]`, the Rust compiler is free to reorder and pad struct fields for optimal alignment. With it, fields appear in declaration order with C-standard padding rules.

---

## 10. Best Practices

1. **Keep `main.rs` thin** -- parse arguments, call into the library, print errors. Nothing else.
2. **Define a crate-level `Error` and `Result<T>`** -- propagation with `?` should work across your entire crate without manual conversions.
3. **Use traits to abstract behavior** -- this enables testing with mocks and swapping implementations via feature flags.
4. **Gate optional functionality behind feature flags** -- compile only what is needed.
5. **Make invalid states unrepresentable** -- use the type system (enums, newtypes, type-state) instead of runtime checks.
6. **Prefer borrowing over cloning** -- pass `&str` and `&[u8]` where ownership is not needed.
7. **Use `Cow` when most inputs pass through unchanged** -- avoid allocations in the common path.
8. **Run clippy** -- it catches unidiomatic patterns and common mistakes. Treat warnings as errors in CI.
9. **Start flat, nest when earned** -- do not create deep module hierarchies before they are needed.

---

## Summary

This chapter covered ten patterns that appear in real Rust projects:

| Pattern | Purpose |
|---------|---------|
| Thin `main.rs` | Testability and reuse via library-first design |
| Trait-based backends | Swappable implementations, mockable in tests |
| Error enum + `Result<T>` alias | Unified error handling with `?` propagation |
| Feature-gated modules | Conditional compilation of entire subsystems |
| Builder | Flexible construction with validation |
| Newtype | Type-safe wrappers over primitives |
| Type-State | Compile-time enforcement of state transitions |
| RAII / Drop | Deterministic resource cleanup |
| Zero-copy / Cow | Avoid unnecessary allocations |
| Best practices | Guidelines for idiomatic Rust project structure |

The first four patterns are applied directly in Day 4 when building the imgforge project. The remaining patterns are general techniques that appear across the Rust ecosystem.
