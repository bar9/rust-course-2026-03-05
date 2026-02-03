# Chapter 25: Rust Patterns

## Learning Objectives
- Master memory management patterns from C++/.NET to Rust
- Understand Option<T> for null safety
- Apply type system patterns and explicit conversions
- Use traits for composition over inheritance
- Write idiomatic Rust code

## Memory Management Patterns

### From RAII to Ownership

The transition from C++ RAII or .NET garbage collection to Rust ownership requires a fundamental mindset shift:

| Aspect | C++ | .NET | Rust |
|--------|-----|------|------|
| Memory control | Manual/RAII | Garbage collector | Ownership system |
| Safety guarantees | Runtime checks | Runtime managed | Compile-time |
| Performance | Predictable | GC pauses | Zero-cost |
| Resource cleanup | Destructors | Finalizers (unreliable) | Drop trait |

### Resource Management Pattern

**C++ RAII:**
```cpp
class FileHandler {
    std::unique_ptr<FILE, decltype(&fclose)> file;
public:
    FileHandler(const char* path)
        : file(fopen(path, "r"), fclose) {
        if (!file) throw std::runtime_error("Failed to open");
    }
    // Manual destructor, copy prevention, etc.
};
```

**Rust Ownership:**
```rust
use std::fs::File;
use std::io::{BufReader, BufRead, Result};

struct FileHandler {
    reader: BufReader<File>,
}

impl FileHandler {
    fn new(path: &str) -> Result<Self> {
        Ok(FileHandler {
            reader: BufReader::new(File::open(path)?),
        })
    }

    fn read_lines(&mut self) -> Result<Vec<String>> {
        self.reader.by_ref().lines().collect()
    }
    // Drop automatically implemented - no manual cleanup needed
}
```

### Shared State Patterns

**C++ Shared Pointer:**
```cpp
std::shared_ptr<Data> data = std::make_shared<Data>();
auto data2 = data;  // Reference counted
```

**Rust Arc (Atomic Reference Counting):**
```rust
use std::sync::Arc;

let data = Arc::new(Data::new());
let data2 = Arc::clone(&data);  // Explicit clone for clarity
```

### Interior Mutability

When you need to mutate data behind a shared reference:

```rust
use std::cell::RefCell;
use std::rc::Rc;

// Single-threaded interior mutability
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
data.borrow_mut().push(4);

// Multi-threaded interior mutability
use std::sync::{Arc, Mutex};
let shared = Arc::new(Mutex::new(vec![1, 2, 3]));
shared.lock().unwrap().push(4);
```

## Null Safety with Option<T>

### Eliminating Null Pointer Exceptions

Tony Hoare's "billion-dollar mistake" is eliminated in Rust:

**C++/C# Nullable:**
```cpp
std::string* find_user(int id) {
    if (id == 1) return new std::string("Alice");
    return nullptr;  // Potential crash
}
```

**Rust Option:**
```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some("Alice".to_string())
    } else {
        None
    }
}

fn use_user() {
    match find_user(42) {
        Some(name) => println!("Found: {}", name),
        None => println!("Not found"),
    }

    // Or use combinators
    let name = find_user(1)
        .map(|n| n.to_uppercase())
        .unwrap_or_else(|| "ANONYMOUS".to_string());
}
```

### Option Combinators

```rust
fn process_optional_data(input: Option<i32>) -> i32 {
    input
        .map(|x| x * 2)           // Transform if Some
        .filter(|x| x > &10)      // Keep only if predicate true
        .unwrap_or(0)              // Provide default
}

// Chaining operations
fn get_config_value() -> Option<String> {
    std::env::var("CONFIG_PATH").ok()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|contents| contents.lines().next().map(String::from))
}
```

## Type System Patterns

### No Implicit Conversions

Rust requires explicit type conversions for safety:

```rust
fn process(value: f64) { }

fn main() {
    let x: i32 = 42;
    // process(x);           // ERROR: expected f64
    process(x as f64);       // Explicit cast
    process(f64::from(x));   // Type conversion

    // String conversions are explicit
    let s = String::from("hello");
    let slice: &str = &s;
    let owned = slice.to_string();
}
```

### Newtype Pattern

Wrap primitive types for type safety:

```rust
struct Kilometers(f64);
struct Miles(f64);

impl Kilometers {
    fn to_miles(&self) -> Miles {
        Miles(self.0 * 0.621371)
    }
}

fn calculate_fuel_efficiency(distance: Kilometers, fuel: Liters) -> KmPerLiter {
    KmPerLiter(distance.0 / fuel.0)
}
```

### Builder Pattern

For complex object construction:

```rust
#[derive(Debug, Default)]
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

    pub fn build(self) -> Result<ServerConfig, &'static str> {
        Ok(ServerConfig {
            host: self.host.ok_or("host required")?,
            port: self.port.unwrap_or(8080),
            max_connections: self.max_connections.unwrap_or(100),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}

// Usage
let config = ServerConfig::builder()
    .host("localhost")
    .port(3000)
    .build()?;
```

## Traits vs Inheritance

### Composition Over Inheritance

**C++ Inheritance:**
```cpp
class Animal { virtual void make_sound() = 0; };
class Dog : public Animal {
    void make_sound() override { cout << "Woof"; }
};
```

**Rust Traits:**
```rust
trait Animal {
    fn make_sound(&self);
}

struct Dog {
    name: String,
}

impl Animal for Dog {
    fn make_sound(&self) {
        println!("{} says Woof", self.name);
    }
}

// Multiple trait implementation
trait Swimmer {
    fn swim(&self);
}

impl Swimmer for Dog {
    fn swim(&self) {
        println!("{} is swimming", self.name);
    }
}
```

### Trait Objects for Runtime Polymorphism

```rust
// Static dispatch (monomorphization)
fn feed_animal<T: Animal>(animal: &T) {
    animal.make_sound();
}

// Dynamic dispatch (trait objects)
fn feed_any_animal(animal: &dyn Animal) {
    animal.make_sound();
}

// Storing heterogeneous collections
let animals: Vec<Box<dyn Animal>> = vec![
    Box::new(Dog { name: "Rex".into() }),
    Box::new(Cat { name: "Whiskers".into() }),
];
```

### Extension Traits

Add methods to existing types:

```rust
trait StringExt {
    fn words(&self) -> Vec<&str>;
}

impl StringExt for str {
    fn words(&self) -> Vec<&str> {
        self.split_whitespace().collect()
    }
}

// Now available on all &str
let words = "hello world".words();
```

## Error Handling Patterns

### Result Type Pattern

Replace exceptions with explicit error handling:

```rust
#[derive(Debug)]
enum DataError {
    NotFound,
    ParseError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for DataError {
    fn from(err: std::io::Error) -> Self {
        DataError::IoError(err)
    }
}

fn load_data(path: &str) -> Result<Data, DataError> {
    let contents = std::fs::read_to_string(path)?;  // ? operator for propagation
    parse_data(&contents).ok_or(DataError::ParseError("Invalid format".into()))
}

// Error handling at call site
match load_data("config.json") {
    Ok(data) => process(data),
    Err(DataError::NotFound) => use_defaults(),
    Err(e) => eprintln!("Error: {:?}", e),
}
```

### Custom Error Types

```rust
use std::fmt;

#[derive(Debug)]
struct ValidationError {
    field: String,
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

// Result type alias for cleaner signatures
type ValidationResult<T> = Result<T, ValidationError>;

fn validate_email(email: &str) -> ValidationResult<()> {
    if !email.contains('@') {
        return Err(ValidationError {
            field: "email".into(),
            message: "Invalid email format".into(),
        });
    }
    Ok(())
}
```

## Functional Patterns

### Iterator Chains

Transform data without intermediate allocations:

```rust
let result: Vec<_> = data
    .iter()
    .filter(|x| x.is_valid())
    .map(|x| x.transform())
    .take(10)
    .collect();

// Lazy evaluation - no work done until collect()
let lazy_iter = (0..)
    .map(|x| x * x)
    .filter(|x| x % 2 == 0)
    .take(5);
```

### Closures and Higher-Order Functions

```rust
fn retry<F, T, E>(mut f: F, max_attempts: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    for _ in 0..max_attempts - 1 {
        if let Ok(result) = f() {
            return Ok(result);
        }
    }
    f()  // Last attempt
}

// Usage with closure
let result = retry(|| risky_operation(), 3)?;
```

## Smart Pointer Patterns

### Box for Heap Allocation

```rust
// Recursive types need Box
enum List<T> {
    Node(T, Box<List<T>>),
    Nil,
}

// Trait objects need Box
let drawable: Box<dyn Draw> = Box::new(Circle::new());
```

### Rc for Shared Ownership (Single-threaded)

```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);
let data2 = Rc::clone(&data);

println!("Reference count: {}", Rc::strong_count(&data));
```

## State Machine Pattern

Model state transitions at compile time:

```rust
struct Draft;
struct PendingReview;
struct Published;

struct Post<State> {
    content: String,
    state: State,
}

impl Post<Draft> {
    fn new() -> Self {
        Post {
            content: String::new(),
            state: Draft,
        }
    }

    fn submit(self) -> Post<PendingReview> {
        Post {
            content: self.content,
            state: PendingReview,
        }
    }
}

impl Post<PendingReview> {
    fn approve(self) -> Post<Published> {
        Post {
            content: self.content,
            state: Published,
        }
    }

    fn reject(self) -> Post<Draft> {
        Post {
            content: self.content,
            state: Draft,
        }
    }
}

impl Post<Published> {
    fn content(&self) -> &str {
        &self.content
    }
}

// Usage enforces correct state transitions at compile time
let post = Post::new()
    .submit()
    .approve();
println!("{}", post.content());
```

## RAII and Drop Pattern

Automatic resource management:

```rust
struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(content: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!("temp_{}.txt", uuid::Uuid::new_v4()));
        std::fs::write(&path, content)?;
        Ok(TempFile { path })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);  // Clean up automatically
    }
}

// File automatically deleted when temp_file goes out of scope
{
    let temp_file = TempFile::new("temporary data")?;
    // Use temp_file
}  // Deleted here
```

## Performance Patterns

### Zero-Copy Operations

```rust
// Borrowing instead of cloning
fn process(data: &[u8]) {
    // Work with borrowed data
}

// String slicing without allocation
let s = "hello world";
let hello = &s[0..5];  // No allocation

// Using Cow for conditional cloning
use std::borrow::Cow;

fn normalize<'a>(input: &'a str) -> Cow<'a, str> {
    if input.contains('\n') {
        Cow::Owned(input.replace('\n', " "))
    } else {
        Cow::Borrowed(input)  // No allocation if unchanged
    }
}
```

### Memory Layout Control

```rust
#[repr(C)]  // C-compatible layout
struct NetworkPacket {
    header: [u8; 4],
    length: u32,
    payload: [u8; 1024],
}

#[repr(C, packed)]  // Remove padding
struct CompactData {
    a: u8,
    b: u32,
    c: u8,
}
```

## Best Practices

1. **Prefer borrowing over owning** when possible
2. **Use iterators** instead of indexing loops
3. **Make invalid states unrepresentable** using the type system
4. **Fail fast** with Result instead of panicking
5. **Document ownership** in complex APIs
6. **Use clippy** to catch unidiomatic patterns
7. **Prefer composition** over inheritance-like patterns
8. **Be explicit** about type conversions and error handling

## Summary

Rust patterns emphasize:
- **Ownership** for automatic memory management
- **Option/Result** for explicit error handling
- **Traits** for polymorphism without inheritance
- **Zero-cost abstractions** for performance
- **Type safety** to catch errors at compile time

These patterns work together to create systems that are both safe and fast, catching entire categories of bugs at compile time while maintaining C++ level performance.