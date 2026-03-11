# Chapter 16: Unsafe Rust & FFI

This chapter covers unsafe Rust operations and Foreign Function Interface (FFI) for interfacing with C/C++ code. Unsafe Rust provides low-level control when needed while FFI enables integration with existing system libraries and codebases.

**Edition 2024 Note**: Starting with Rust 1.85 and Edition 2024, all `extern` blocks must be marked as `unsafe extern` to make the unsafety of FFI calls explicit. This change improves clarity about where unsafe operations occur.

## Part 1: Unsafe Rust Foundations

### The Five Unsafe Superpowers

Unsafe Rust enables five specific operations that bypass Rust's safety guarantees:

1. **Dereference raw pointers** - Direct memory access
2. **Call unsafe functions/methods** - Including FFI functions
3. **Access/modify mutable statics** - Global state management
4. **Implement unsafe traits** - Like `Send` and `Sync`
5. **Access union fields** - Memory reinterpretation

### Raw Pointers

```rust
use std::ptr;

// Creating raw pointers
let mut num = 5;
let r1 = &num as *const i32;        // Immutable raw pointer
let r2 = &mut num as *mut i32;      // Mutable raw pointer

// Dereferencing requires unsafe
unsafe {
    println!("r1: {}", *r1);
    *r2 = 10;
    println!("r2: {}", *r2);
}

// Pointer arithmetic
unsafe {
    let array = [1, 2, 3, 4, 5];
    let ptr = array.as_ptr();

    for i in 0..5 {
        println!("Value at offset {}: {}", i, *ptr.add(i));
    }
}
```

### Unsafe Functions and Methods

```rust
unsafe fn dangerous() {
    // Function body can perform unsafe operations
}

// Calling unsafe functions
unsafe {
    dangerous();
}

// Safe abstraction over unsafe code
fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();

    assert!(mid <= len);

    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
```

### Mutable Static Variables

```rust
// Note: In Edition 2024, references to `static mut` are a hard error
// (`static_mut_refs` lint). Use atomics or raw pointers instead.

// Better alternative: use atomic types
use std::sync::atomic::{AtomicU32, Ordering};

static ATOMIC_COUNTER: AtomicU32 = AtomicU32::new(0);

fn safe_increment() {
    ATOMIC_COUNTER.fetch_add(1, Ordering::SeqCst);
}
```

### Unsafe Traits

```rust
unsafe trait Zeroable {
    // Trait is unsafe because implementor must guarantee safety
}

unsafe impl Zeroable for i32 {
    // We guarantee i32 can be safely zeroed
}

// Send and Sync are unsafe traits
struct RawPointer(*const u8);

unsafe impl Send for RawPointer {}
unsafe impl Sync for RawPointer {}
```

### Unions

```rust
#[repr(C)]
union IntOrFloat {
    i: i32,
    f: f32,
}

let mut u = IntOrFloat { i: 42 };

unsafe {
    // Accessing union fields is unsafe
    u.f = 3.14;
    println!("Float: {}", u.f);

    // Type punning (reinterpreting bits)
    println!("As int: {}", u.i);  // Type punning: reinterprets float bits as int (well-defined for repr(C))
}
```

## Part 2: Calling C/C++ from Rust

### Manual FFI Bindings

```rust
use std::ffi::{c_char, c_int, c_void, CString, CStr};

// Link to system libraries
// Edition 2024 (Rust 1.85+): extern blocks must be marked `unsafe extern`
#[link(name = "m")]  // Math library
unsafe extern "C" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exponent: f64) -> f64;
}

// Safe wrapper
pub fn safe_sqrt(x: f64) -> f64 {
    if x < 0.0 {
        panic!("Cannot take square root of negative number");
    }
    unsafe { sqrt(x) }
}

// Working with strings
unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

pub fn string_length(s: &str) -> usize {
    let c_string = CString::new(s).expect("CString creation failed");
    unsafe {
        strlen(c_string.as_ptr())
    }
}
```

### Complex C Structures

```rust
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

#[repr(C)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

unsafe extern "C" {
    fn calculate_area(rect: *const Rectangle) -> f64;
}

pub fn rect_area(rect: &Rectangle) -> f64 {
    unsafe {
        calculate_area(rect as *const Rectangle)
    }
}
```

### Using Bindgen

```toml
# Cargo.toml
[build-dependencies]
bindgen = "0.70"
cc = "1.1"
```

```rust,ignore
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // Compile C code
    cc::Build::new()
        .file("src/native.c")
        .compile("native");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

```rust,ignore
// src/lib.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Use generated bindings
pub fn use_native_function() {
    unsafe {
        let result = native_function(42);
        println!("Result: {}", result);
    }
}
```

## Part 3: Exposing Rust to C/C++

### Using cbindgen

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "staticlib"]

[build-dependencies]
cbindgen = "0.29"
```

```rust
// src/lib.rs
use std::ffi::{c_char, c_int, CStr};

#[no_mangle]
pub extern "C" fn rust_add(a: c_int, b: c_int) -> c_int {
    a + b
}

#[no_mangle]
pub extern "C" fn rust_greet(name: *const c_char) -> *mut c_char {
    let name = unsafe {
        assert!(!name.is_null());
        CStr::from_ptr(name)
    };

    let greeting = format!("Hello, {}!", name.to_string_lossy());
    let c_string = std::ffi::CString::new(greeting).unwrap();
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn rust_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = std::ffi::CString::from_raw(s);
    }
}
```

```rust,ignore
// build.rs
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/rust_lib.h");
}
```

## Part 4: C++ Integration with cxx

### Using cxx for Safe C++ FFI

```toml
# Cargo.toml
[dependencies]
cxx = "1.0"

[build-dependencies]
cxx-build = "1.0"
```

```rust,ignore
// src/lib.rs
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("cpp/include/blobstore.h");

        type BlobstoreClient;

        fn new_blobstore_client() -> UniquePtr<BlobstoreClient>;
        fn put(&self, key: &str, value: &[u8]) -> Result<()>;
        fn get(&self, key: &str) -> Vec<u8>;
    }

    extern "Rust" {
        fn process_blob(data: &[u8]) -> Vec<u8>;
    }
}

pub fn process_blob(data: &[u8]) -> Vec<u8> {
    // Rust implementation
    data.iter().map(|&b| b.wrapping_add(1)).collect()
}

pub fn use_blobstore() -> Result<(), Box<dyn std::error::Error>> {
    let client = ffi::new_blobstore_client();
    let key = "test_key";
    let data = b"hello world";

    client.put(key, data)?;
    let retrieved = client.get(key);

    Ok(())
}
```

```rust,ignore
// build.rs
fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("cpp/src/blobstore.cc")
        .std("c++17")
        .compile("cxx-demo");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp/include/blobstore.h");
    println!("cargo:rerun-if-changed=cpp/src/blobstore.cc");
}
```

## Part 5: Platform-Specific Code & Conditional Compilation

```rust
#[cfg(target_os = "windows")]
mod windows {
    use winapi::um::fileapi::GetFileAttributesW;
    use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;

    pub fn is_hidden(path: &std::path::Path) -> bool {
        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(Some(0))
            .collect();

        unsafe {
            let attrs = GetFileAttributesW(wide.as_ptr());
            attrs != u32::MAX && (attrs & FILE_ATTRIBUTE_HIDDEN) != 0
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    pub fn is_hidden(path: &std::path::Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }
}
```

## Part 6: Safety Patterns and Best Practices

### Safe Abstraction Pattern

```rust,ignore
pub struct SafeWrapper {
    ptr: *mut SomeFFIType,
}

impl SafeWrapper {
    pub fn new() -> Option<Self> {
        unsafe {
            let ptr = ffi_create_object();
            if ptr.is_null() {
                None
            } else {
                Some(SafeWrapper { ptr })
            }
        }
    }

    pub fn do_something(&self) -> Result<i32, String> {
        unsafe {
            let result = ffi_do_something(self.ptr);
            if result < 0 {
                Err("Operation failed".to_string())
            } else {
                Ok(result)
            }
        }
    }
}

impl Drop for SafeWrapper {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ffi_destroy_object(self.ptr);
            }
        }
    }
}

// Only implement these if the underlying C library is truly thread-safe!
// unsafe impl Send for SafeWrapper {}
// unsafe impl Sync for SafeWrapper {}
```

### Error Handling Across FFI

The key principle: convert Rust's `Result`/`panic` into C-compatible error codes at the FFI boundary. Common patterns:

- Return error codes (`0` = success, negative = error) with an out-parameter for the result
- Use a `*mut ErrorInfo` struct to pass error details (code + message)
- Catch panics with `std::panic::catch_unwind` to prevent unwinding across FFI boundaries

## Part 6: Testing FFI Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_wrapper() {
        // Mock the FFI functions in tests
        struct MockFFI;

        impl MockFFI {
            fn mock_function(&self, input: i32) -> i32 {
                input * 2
            }
        }

        let mock = MockFFI;
        assert_eq!(mock.mock_function(21), 42);
    }

    #[test]
    fn test_error_handling() {
        let mut error = ErrorInfo {
            code: 0,
            message: ptr::null_mut(),
        };

        let result = rust_operation(
            ptr::null(),
            &mut error as *mut ErrorInfo,
        );

        assert!(result.is_null());
        assert_eq!(unsafe { error.code }, 1);
    }
}
```

## Part 7: Volatile Memory Access & HAL Patterns

In embedded systems, hardware registers are mapped to specific memory addresses. The compiler must not optimize away reads or writes to these addresses, even if the values appear unused — because the hardware side-effects matter.

### Volatile Reads and Writes

`core::ptr::read_volatile` and `core::ptr::write_volatile` guarantee that every access reaches memory, preventing the compiler from eliding or reordering them:

```rust
use core::ptr;

// Memory-mapped I/O: a hardware register at a fixed address
const GPIO_OUTPUT_REG: *mut u32 = 0x6000_4004 as *mut u32;
const GPIO_INPUT_REG: *const u32 = 0x6000_403C as *const u32;

/// Set a GPIO pin high by writing to the output register.
///
/// # Safety
/// Caller must ensure the address is a valid, mapped hardware register.
unsafe fn gpio_set_high(pin: u8) {
    let current = ptr::read_volatile(GPIO_OUTPUT_REG);
    ptr::write_volatile(GPIO_OUTPUT_REG, current | (1 << pin));
}

/// Read the current state of all GPIO input pins.
///
/// # Safety
/// Caller must ensure the address is a valid, mapped hardware register.
unsafe fn gpio_read_all() -> u32 {
    ptr::read_volatile(GPIO_INPUT_REG)
}
```

**Why not just use `*ptr`?** A normal dereference may be optimized away if the compiler decides the value is never "really" used, or it may be merged with adjacent accesses. Hardware registers have *side effects* on read (e.g. clearing an interrupt flag) or write (e.g. toggling a pin), so every access must be preserved.

### The Memory-Mapped I/O (MMIO) Pattern

Embedded Rust crates typically wrap raw register addresses in a typed struct:

```rust,ignore
/// A register block representing a peripheral's control registers.
#[repr(C)]
struct GpioRegisters {
    output:     u32,   // offset 0x00
    output_set: u32,   // offset 0x04
    output_clr: u32,   // offset 0x08
    input:      u32,   // offset 0x0C
}

impl GpioRegisters {
    /// # Safety
    /// The base address must point to a valid GPIO register block.
    unsafe fn from_base(base: usize) -> &'static mut Self {
        &mut *(base as *mut Self)
    }

    fn set_pin(&mut self, pin: u8) {
        // Safety: this struct is only constructed over valid MMIO memory
        unsafe {
            core::ptr::write_volatile(&mut self.output_set, 1 << pin);
        }
    }

    fn read_input(&self) -> u32 {
        unsafe { core::ptr::read_volatile(&self.input) }
    }
}
```

### The HAL Trait Pattern (`embedded-hal`)

The `embedded-hal` crate defines vendor-neutral traits that any microcontroller HAL can implement. This lets application code and drivers be portable across chips:

```rust,ignore
use embedded_hal::digital::OutputPin;

/// Blink an LED using any OutputPin — works on ESP32, STM32, nRF, etc.
fn blink<P: OutputPin>(led: &mut P, delay_ms: u32) {
    led.set_high().ok();
    // ... delay ...
    led.set_low().ok();
}
```

Chip vendors (like `esp-hal`, `stm32-hal`) provide concrete types that implement these traits, wrapping volatile register access in safe abstractions. This is the same pattern we saw in Part 6 — safe wrappers around unsafe internals — applied to hardware.

**Day 4 preview**: In the ESP32-C3 exercises, you will use `esp-hal` which builds on these exact patterns — `Output::new()` returns a type implementing `OutputPin`, hiding all volatile register manipulation behind a safe, type-checked API.

## Best Practices

1. **Minimize Unsafe Code**: Keep unsafe blocks small and isolated
2. **Document Safety Requirements**: Clearly state what callers must guarantee
3. **Use Safe Abstractions**: Wrap unsafe code in safe APIs
4. **Validate All Inputs**: Never trust data from FFI boundaries
5. **Handle Errors Gracefully**: Convert panics to error codes at FFI boundaries
6. **Test Thoroughly**: Include fuzzing and property-based testing
7. **Use Tools**: Run Miri, Valgrind, and sanitizers on FFI code

## Common Pitfalls

1. **Memory Management**: Ensure consistent allocation/deallocation across FFI
2. **String Encoding**: C uses null-terminated strings, Rust doesn't
3. **ABI Compatibility**: Always use `#[repr(C)]` for FFI structs
4. **Lifetime Management**: Raw pointers don't encode lifetimes
5. **Thread Safety**: Verify thread safety of external libraries

## Summary

Unsafe Rust and FFI provide powerful tools for systems programming:

- **Unsafe Rust** enables low-level operations with explicit opt-in
- **FFI** allows seamless integration with C/C++ codebases
- **Safe abstractions** wrap unsafe code in safe interfaces
- **Tools like bindgen and cbindgen** automate binding generation
- **cxx** provides safe C++ interop

Always prefer safe Rust, use unsafe only when necessary, and wrap it in safe abstractions.

## Additional Resources

- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [bindgen User Guide](https://rust-lang.github.io/rust-bindgen/)
- [cxx Documentation](https://cxx.rs/)
- [FFI Omnibus](http://jakegoulding.com/rust-ffi-omnibus/)