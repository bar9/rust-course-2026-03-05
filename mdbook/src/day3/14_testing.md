# Chapter 14: Testing in Rust

Rust has first-class testing support built directly into the language and toolchain. There's no need for external test frameworks like xUnit, NUnit, or Google Test — `cargo test` works out of the box on every Rust project. This chapter covers how to write, organize, and run tests effectively.

## 1. Unit Tests — The Basics

### The `#[test]` Attribute

Any function annotated with `#[test]` becomes a test case. A test passes if it runs without panicking.

```rust
#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn greeting_contains_name() {
    let greeting = format!("Hello, {}!", "Alice");
    assert!(greeting.contains("Alice"));
}
```

### Assertion Macros

Rust provides three core assertion macros:

| Macro | Purpose | Panics when |
|-------|---------|-------------|
| `assert!(expr)` | Boolean check | `expr` is `false` |
| `assert_eq!(left, right)` | Equality check | `left != right` |
| `assert_ne!(left, right)` | Inequality check | `left == right` |

All three accept an optional custom message as additional arguments:

```rust
#[test]
fn test_with_messages() {
    let age = 17;
    assert!(age >= 18, "Expected adult, got age {}", age);

    let expected = 42;
    let actual = compute_answer();
    assert_eq!(actual, expected, "compute_answer() returned wrong value");
}

fn compute_answer() -> i32 { 42 }
```

> **Note:** `assert_eq!` and `assert_ne!` require the compared types to implement both `PartialEq` and `Debug`. Most standard types do; for your own types, add `#[derive(Debug, PartialEq)]`.

### Comparison with C#/.NET

| C# / .NET | Rust | Notes |
|-----------|------|-------|
| `[TestClass]` | `#[cfg(test)] mod tests` | Test module, compiled only during testing |
| `[TestMethod]` | `#[test]` | Marks a test function |
| `Assert.AreEqual(a, b)` | `assert_eq!(a, b)` | Prints both values on failure |
| `Assert.IsTrue(x)` | `assert!(x)` | Boolean assertion |
| `Assert.ThrowsException<T>` | `#[should_panic]` | Expects a panic |
| `[ExpectedException]` | `#[should_panic(expected = "msg")]` | Checks panic message |
| `[Ignore]` | `#[ignore]` | Skips test unless explicitly requested |

### The `#[cfg(test)]` Module

By convention, unit tests live in a `tests` module at the bottom of the same file, gated by `#[cfg(test)]`:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("Division by zero".to_string());
    }
    Ok(a / b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10.0, 2.0), Ok(5.0));
    }

    #[test]
    fn test_divide_by_zero() {
        assert!(divide(5.0, 0.0).is_err());
    }
}
```

The `#[cfg(test)]` attribute means this module is only compiled when running `cargo test` — it won't bloat your release binary. The `use super::*;` import brings the parent module's items into scope.

### Testing `Result<T, E>` Returns

Test functions can return `Result<(), E>`, which lets you use `?` instead of `unwrap()`:

```rust
#[cfg(test)]
mod tests {
    use std::num::ParseIntError;

    #[test]
    fn test_parsing() -> Result<(), ParseIntError> {
        let value: i32 = "42".parse()?;
        assert_eq!(value, 42);
        Ok(())
    }
}
```

The test fails if the function returns `Err`. This is especially useful when your test involves multiple fallible operations.

### `#[should_panic]`

Use `#[should_panic]` when you expect a function to panic. You can optionally check the panic message:

```rust
pub fn validate_age(age: i32) -> i32 {
    if age < 0 || age > 150 {
        panic!("Invalid age: {age}. Must be between 0 and 150.");
    }
    age
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_negative_age_panics() {
        validate_age(-1);
    }

    #[test]
    #[should_panic(expected = "Invalid age")]
    fn test_panic_message() {
        validate_age(200);
    }
}
```

## 2. Test Organization

Rust supports three kinds of tests, each with a different scope:

| Kind | Location | Compiles as | Tests... |
|------|----------|-------------|----------|
| **Unit tests** | `src/*.rs` inside `#[cfg(test)]` | Part of the crate | Private + public API |
| **Integration tests** | `tests/*.rs` directory | Separate crate | Public API only |
| **Doc tests** | `///` comments in source | Separate compilation | Examples in documentation |

### Unit Tests: Colocated with Code

Unit tests live inside the module they test. This is different from C# where test projects are always separate — in Rust, tests sit right next to the code they exercise:

```rust
// src/temperature.rs
pub struct Celsius(pub f64);
pub struct Fahrenheit(pub f64);

impl Celsius {
    pub fn to_fahrenheit(&self) -> Fahrenheit {
        Fahrenheit(self.0 * 9.0 / 5.0 + 32.0)
    }

    fn is_valid(&self) -> bool {
        self.0 >= -273.15
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boiling_point() {
        let c = Celsius(100.0);
        let f = c.to_fahrenheit();
        assert!((f.0 - 212.0).abs() < f64::EPSILON);
    }

    #[test]
    fn can_test_private_functions() {
        // Rust allows unit tests to access private items!
        assert!(Celsius(20.0).is_valid());
        assert!(!Celsius(-300.0).is_valid());
    }
}
```

### Integration Tests: The `tests/` Directory

Files in a top-level `tests/` directory are compiled as separate crates. They can only access your crate's **public** API:

```text
my_crate/
├── src/
│   └── lib.rs
├── tests/
│   ├── basic_operations.rs
│   └── edge_cases.rs
└── Cargo.toml
```

```rust,ignore
// tests/basic_operations.rs
use my_crate::{add, divide};

#[test]
fn test_add_from_outside() {
    assert_eq!(add(10, 20), 30);
}

#[test]
fn test_divide_from_outside() {
    assert!(divide(1.0, 0.0).is_err());
}
```

Each file in `tests/` is a separate test binary. To share helper code between integration tests, put it in `tests/common/mod.rs` (the `mod.rs` naming prevents Cargo from treating the helper file itself as a test suite).

### Doc Tests: Executable Examples

Code blocks inside `///` documentation comments are compiled and run as tests:

```rust
/// Adds two numbers.
///
/// # Examples
///
/// ```
/// use my_crate::add;
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Doc tests serve double duty: they verify your examples are correct **and** provide documentation for users. If the example compiles but shouldn't be run, use `no_run`. If it shouldn't even compile (to show error examples), use `compile_fail`:

````rust
/// ```no_run
/// // This compiles but we don't want to run it in tests
/// std::process::exit(0);
/// ```
///
/// ```compile_fail
/// // This demonstrates a compile error
/// let x: i32 = "not a number";
/// ```
````

### When to Use Which

| Question | Answer |
|----------|--------|
| Testing private helper functions? | **Unit test** — only they have access |
| Testing your public API as a consumer would? | **Integration test** |
| Ensuring documentation examples stay correct? | **Doc test** |
| Quick check of a single function? | **Unit test** |
| End-to-end workflow across modules? | **Integration test** |

## 3. Running Tests

### Basic Commands

```bash
# Run all tests (unit, integration, doc)
cargo test

# Run only unit tests (lib)
cargo test --lib

# Run only unit tests (bin)
cargo test --bins
cargo test --bin hello

# Run only integration tests
cargo test --test basic_operations
cargo test --test '*'

# Run only doc tests
cargo test --doc

# Run tests in a specific package (workspace)
cargo test -p my_crate
```

### Filtering by Name

```bash
# Run tests whose name contains "divide"
cargo test divide

# Run tests in a specific module
cargo test temperature::tests

# Run a single, exact test
cargo test -- --exact test_add
```

### Useful Flags

```bash
# Show println! output (normally captured on success)
cargo test -- --nocapture

# Run tests sequentially (default is parallel)
cargo test -- --test-threads=1

# Run only ignored tests
cargo test -- --ignored

# Run all tests including ignored
cargo test -- --include-ignored

# Show which tests are running (without running them)
cargo test -- --list
```

### `#[ignore]` for Slow Tests

Mark expensive tests with `#[ignore]` so they don't slow down your normal test runs:

```rust
#[test]
#[ignore = "requires network access"]
fn test_api_integration() {
    // This test calls an external API and takes seconds
    // Only runs with: cargo test -- --ignored
}
```

### Cargo Nextest — A Faster Test Runner

[cargo-nextest](https://nexte.st/) is a drop-in replacement for `cargo test` with better performance and output:

```bash
# Install
cargo install cargo-nextest

# Run all tests (same as cargo test, but faster and prettier)
cargo nextest run

# Filter by name
cargo nextest run divide

# List tests
cargo nextest list
```

Nextest runs each test as a separate process, providing better isolation and parallel performance. It also gives clearer output on failures.

## 4. Testing Patterns

### Testing Private Functions

Unlike C# where you'd need `[InternalsVisibleTo]` or reflection to test private methods, Rust unit tests can test private functions directly — because they're inside the same module:

```rust
fn internal_hash(data: &[u8]) -> u64 {
    // private implementation detail
    data.iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

pub fn is_valid_hash(data: &[u8], expected: u64) -> bool {
    internal_hash(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_hash_directly() {
        let hash = internal_hash(b"hello");
        assert_ne!(hash, 0);
        // Same input always produces same output
        assert_eq!(internal_hash(b"hello"), hash);
    }
}
```

### Test Helpers and Setup

Rust doesn't have `[SetUp]` / `[TearDown]` attributes. Instead, use regular helper functions or builder patterns:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function — the Rust equivalent of [SetUp]
    fn setup_calculator() -> Calculator {
        Calculator::with_precision(2)
    }

    #[test]
    fn test_add() {
        let calc = setup_calculator();
        assert_eq!(calc.calculate(Operation::Add, 1.0, 2.0), Ok(3.0));
    }

    #[test]
    fn test_subtract() {
        let calc = setup_calculator();
        assert_eq!(calc.calculate(Operation::Subtract, 5.0, 3.0), Ok(2.0));
    }
}
```

### Mocking with Traits

Rust doesn't have a built-in mocking framework like Moq or Mockito. Instead, use **trait-based dependency injection** — define behavior behind a trait, then provide a mock implementation in tests:

```rust
// Define behavior as a trait
trait WeatherService {
    fn get_temperature(&self, city: &str) -> Result<f64, String>;
}

// Production implementation
struct RealWeatherService;

impl WeatherService for RealWeatherService {
    fn get_temperature(&self, city: &str) -> Result<f64, String> {
        // HTTP call to weather API...
        # Ok(20.0)
    }
}

// Code under test depends on the trait, not the implementation
fn should_wear_jacket(service: &dyn WeatherService, city: &str) -> bool {
    match service.get_temperature(city) {
        Ok(temp) => temp < 15.0,
        Err(_) => true, // When in doubt, bring a jacket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test mock — no framework needed
    struct MockWeatherService {
        temperature: Result<f64, String>,
    }

    impl WeatherService for MockWeatherService {
        fn get_temperature(&self, _city: &str) -> Result<f64, String> {
            self.temperature.clone()
        }
    }

    #[test]
    fn cold_weather_needs_jacket() {
        let service = MockWeatherService { temperature: Ok(5.0) };
        assert!(should_wear_jacket(&service, "Zurich"));
    }

    #[test]
    fn warm_weather_no_jacket() {
        let service = MockWeatherService { temperature: Ok(25.0) };
        assert!(!should_wear_jacket(&service, "Barcelona"));
    }

    #[test]
    fn error_means_jacket() {
        let service = MockWeatherService {
            temperature: Err("API down".to_string()),
        };
        assert!(should_wear_jacket(&service, "Unknown"));
    }
}
```

This pattern is idiomatic Rust and works without any external crate. In Day 4, the ESP32-C3 exercises use exactly this approach: a `TemperatureSensorHal` trait defines sensor behavior, with a real ESP32 implementation for hardware and a `MockTemperatureSensor` for desktop testing. For more complex mocking needs, crates like `mockall` can auto-generate mock implementations from traits.

### Testing Async Code

If you use async Rust, the `#[tokio::test]` attribute creates a runtime for your test:

```rust,ignore
#[tokio::test]
async fn test_async_fetch() {
    let result = fetch_data("https://example.com").await;
    assert!(result.is_ok());
}
```

This requires `tokio` as a dev-dependency with the `macros` and `rt` features:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt"] }
```

## 5. Code Coverage with `cargo llvm-cov`

Code coverage measures which lines, branches, and functions your tests actually execute. It's a useful tool for finding **untested** code — but high coverage numbers don't guarantee correctness. Focus on testing critical paths rather than chasing 100%.

### Setup

```bash
# Install
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

### Basic Usage

```bash
# Run tests and show coverage summary
cargo llvm-cov

# Generate HTML report and open in browser
cargo llvm-cov --open

# Coverage for the whole workspace
cargo llvm-cov --workspace
```

### Example Output

```text
Filename                Regions    Missed     Cover   Lines      Missed     Cover
---------------------------------------------------------------------------------------
src/calculator.rs            12         2    83.33%      45           3    93.33%
src/lib.rs                    8         0   100.00%      30           0   100.00%
---------------------------------------------------------------------------------------
TOTAL                        20         2    90.00%      75           3    96.00%
```

The HTML report highlights covered lines in green and uncovered lines in red — a quick way to spot gaps.

### CI/CD Integration

Add coverage to your GitHub Actions pipeline:

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - uses: codecov/codecov-action@v5
        with:
          files: lcov.info
```

### Coverage Best Practices

- **Coverage finds untested code, not bugs** — a covered line can still contain a bug
- **Focus on critical paths** — business logic, error handling, edge cases
- **Don't chase 100%** — some code (e.g., `Display` impls, CLI boilerplate) isn't worth testing exhaustively
- **Watch coverage trends** — a sudden drop often signals forgotten tests for new code

## 6. Property-Based Testing

Standard unit tests check specific examples: "does `add(2, 3)` return `5`?" Property-based testing takes a different approach: "does `add(a, b)` always equal `add(b, a)` for all `a` and `b`?" The framework generates hundreds of random inputs and checks that your invariants hold.

### Using `proptest`

```toml
[dev-dependencies]
proptest = "1"
```

```rust,ignore
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }

    proptest! {
        #[test]
        fn reversing_twice_gives_original(s in "\\PC*") {
            // For any string, reversing twice should return the original
            assert_eq!(reverse(&reverse(&s)), s);
        }

        #[test]
        fn reverse_preserves_length(s in "\\PC*") {
            // .len() returns byte length; reversing chars preserves total bytes
            assert_eq!(reverse(&s).len(), s.len());
        }

        #[test]
        fn addition_is_commutative(a in 0i64..1000, b in 0i64..1000) {
            assert_eq!(a + b, b + a);
        }
    }
}
```

When `proptest` finds a failing input, it **shrinks** it to the smallest reproducible case — making debugging much easier than staring at a random 200-character string.

### When to Use Property-Based Tests

| Situation | Approach |
|-----------|----------|
| Known specific inputs and expected outputs | Standard `#[test]` |
| Mathematical invariants (commutativity, associativity) | `proptest` |
| Parsers: `parse(format(x)) == x` | `proptest` |
| "This should never panic for any input" | `proptest` |
| Testing a function against a simpler reference implementation | `proptest` |

Property-based tests are particularly good at catching edge cases you'd never think to write manually — empty strings, integer overflow, Unicode boundary conditions, etc.

## Exercise: Test a `MarkdownProcessor`

In this exercise you'll practise the testing techniques covered above by implementing a small Markdown-to-text processor **and** its test suite. You get the type signatures and the tests — your job is to make every test pass.

### Setup

```bash
# Copy the starter into your workspace
cp -r solutions/day3/14_testing/ mysolutions/day3/14_testing/
cd mysolutions/day3/14_testing
cargo test          # all tests should fail initially
```

### What you implement

```rust,ignore
pub struct MarkdownProcessor;

impl MarkdownProcessor {
    pub fn new() -> Self;

    /// Strip all markdown formatting, return plain text.
    pub fn to_plain_text(&self, input: &str) -> String;

    /// Extract all links as (text, url) pairs from `[text](url)`.
    pub fn extract_links(&self, input: &str) -> Vec<(String, String)>;

    /// Count headings by level (1–6).
    pub fn count_headings(&self, input: &str) -> HashMap<u8, usize>;

    /// **bold** → UPPERCASE, *italic* → lowercase.
    /// Panics on unmatched `**` markers.
    pub fn transform_emphasis(&self, input: &str) -> String;
}
```

### What the tests cover

| # | Test | Technique |
|---|------|-----------|
| 1 | `plain_text_strips_headings` | `assert_eq!` |
| 2 | `plain_text_strips_bold_and_italic` | `assert_eq!` |
| 3 | `plain_text_converts_links_to_text` | `assert_eq!` |
| 4 | `extract_links_finds_all_links` | `assert_eq!` on `Vec` |
| 5 | `extract_links_returns_empty_for_no_links` | `assert!(_.is_empty())` |
| 6 | `count_headings_by_level` | `HashMap` assertions |
| 7 | `transform_emphasis_bold_to_uppercase` | `assert_eq!` |
| 8 | `transform_emphasis_italic_to_lowercase` | `assert_eq!` |
| 9 | `transform_emphasis_panics_on_unmatched_bold` | `#[should_panic(expected = "...")]` |
| 10 | `round_trip_plain_text_is_stable` | `Result<(), String>` return |
| 11 | `large_document_performance` | `#[ignore]` — run with `cargo test -- --ignored` |

### Tips

- Start with `to_plain_text` — most other functions build on the same parsing logic.
- Use `str::trim_start_matches` to strip leading `#` characters.
- For `extract_links`, search for `[` then `](` then `)` in sequence.
- The `#[should_panic]` test expects the exact substring `"Unmatched bold markers"`.
- Run `cargo test -- --nocapture` to see `println!` output from your code while debugging.

### Solution

The reference solution is in `solutions/day3/14_testing/src/lib.rs`.

## Summary

| What | How |
|------|-----|
| Write a test | `#[test] fn name() { ... }` |
| Check equality | `assert_eq!(actual, expected)` |
| Expect a panic | `#[should_panic(expected = "msg")]` |
| Return Result from test | `fn test() -> Result<(), E> { ... }` |
| Test private functions | Put tests in `#[cfg(test)] mod tests` inside the same file |
| Integration tests | `tests/*.rs` directory |
| Doc tests | Code blocks in `///` comments |
| Run all tests | `cargo test` |
| Filter tests | `cargo test name_filter` |
| See output | `cargo test -- --nocapture` |
| Skip slow tests | `#[ignore]`, run with `cargo test -- --ignored` |
| Code coverage | `cargo llvm-cov --open` |
| Property testing | `proptest!` macro with random input generators |
