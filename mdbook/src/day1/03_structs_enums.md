# Chapter 3: Structs and Enums
## Data Modeling and Methods in Rust

### Learning Objectives
By the end of this chapter, you'll be able to:
- Define and use structs effectively for data modeling
- Understand when and how to implement methods and associated functions
- Master enums for type-safe state representation
- Apply pattern matching with complex data structures
- Choose between structs and enums for different scenarios
- Implement common patterns from OOP languages in Rust

---

## Structs: Structured Data

Structs in Rust are similar to structs in C++ or classes in C#, but with some key differences around memory layout and method definition.

### Basic Struct Definition

```rust
// Similar to C++ struct or C# class
struct Person {
    name: String,
    age: u32,
    email: String,
}

// Creating instances
let person = Person {
    name: String::from("Alice"),
    age: 30,
    email: String::from("alice@example.com"),
};

// Accessing fields
println!("Name: {}", person.name);
println!("Age: {}", person.age);
```

### Comparison with C++/.NET

| Feature | C++ | C#/.NET | Rust |
|---------|-----|---------|------|
| Definition | `struct Person { std::string name; };` | `class Person { public string Name; }` | `struct Person { name: String }` |
| Instantiation | `Person p{"Alice"};` | `var p = new Person { Name = "Alice" };` | `Person { name: "Alice".to_string() }` |
| Field Access | `p.name` | `p.Name` | `p.name` |
| Methods | Inside struct | Inside class | Separate `impl` block |

### Struct Update Syntax

```rust
let person1 = Person {
    name: String::from("Alice"),
    age: 30,
    email: String::from("alice@example.com"),
};

// Create a new instance based on existing one
let person2 = Person {
    name: String::from("Bob"),
    ..person1  // Use remaining fields from person1
};

// Note: person1 is no longer usable if any non-Copy fields were moved!
```

### Tuple Structs

When you don't need named fields:

```rust
// Tuple struct - like std::pair in C++ or Tuple in C#
struct Point(f64, f64);
struct Color(u8, u8, u8);

let origin = Point(0.0, 0.0);
let red = Color(255, 0, 0);

// Access by index
println!("X: {}, Y: {}", origin.0, origin.1);
```

### Unit Structs

Structs with no data - useful for type safety:

```rust
// Unit struct - zero size
struct Marker;

// Useful for phantom types and markers
let marker = Marker;
```

---

## Methods and Associated Functions

In Rust, methods are defined separately from the struct definition in `impl` blocks.

### Instance Methods

```rust
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    // Method that takes &self (immutable borrow)
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    // Method that takes &mut self (mutable borrow)
    fn scale(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
    }
    
    // Method that takes self (takes ownership)
    fn into_square(self) -> Rectangle {
        let size = (self.width + self.height) / 2.0;
        Rectangle {
            width: size,
            height: size,
        }
    }
}

// Usage
let mut rect = Rectangle { width: 10.0, height: 5.0 };
println!("Area: {}", rect.area());      // Borrows immutably
rect.scale(2.0);                        // Borrows mutably
let square = rect.into_square();        // Takes ownership
// rect is no longer usable here!
```

### Associated Functions (Static Methods)

```rust
impl Rectangle {
    // Associated function (like static method in C#)
    fn new(width: f64, height: f64) -> Rectangle {
        Rectangle { width, height }
    }
    
    // Constructor-like function
    fn square(size: f64) -> Rectangle {
        Rectangle {
            width: size,
            height: size,
        }
    }
}

// Usage - called on the type, not an instance
let rect = Rectangle::new(10.0, 5.0);
let square = Rectangle::square(7.0);
```

### Multiple impl Blocks

You can have multiple `impl` blocks for organization:

```rust
impl Rectangle {
    // Construction methods
    fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

impl Rectangle {
    // Calculation methods
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}
```

---

## Enums: More Powerful Than You Think

Rust enums are much more powerful than C++ enums or C# enums. They're similar to discriminated unions or algebraic data types.

### Basic Enums

```rust
// Simple enum - like C++ enum class
#[derive(Debug)]  // Allows printing with {:?}
enum Direction {
    North,
    South,
    East,
    West,
}

let dir = Direction::North;
println!("{:?}", dir);  // Prints: North
```

### Enums with Data

This is where Rust enums shine - each variant can hold different types of data:

```rust
enum IpAddr {
    V4(u8, u8, u8, u8),           // IPv4 with 4 bytes
    V6(String),                   // IPv6 as string
}

let home = IpAddr::V4(127, 0, 0, 1);
let loopback = IpAddr::V6(String::from("::1"));

// More complex example
enum Message {
    Quit,                         // No data
    Move { x: i32, y: i32 },     // Anonymous struct
    Write(String),                // Single value
    ChangeColor(i32, i32, i32),  // Tuple
}
```

### Pattern Matching with Enums

```rust
fn process_message(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quit received");
        }
        Message::Move { x, y } => {
            println!("Move to ({}, {})", x, y);
        }
        Message::Write(text) => {
            println!("Write: {}", text);
        }
        Message::ChangeColor(r, g, b) => {
            println!("Change color to RGB({}, {}, {})", r, g, b);
        }
    }
}
```

### Methods on Enums

Enums can have methods too:

```rust
impl Message {
    fn is_quit(&self) -> bool {
        matches!(self, Message::Quit)
    }
    
    fn process(&self) {
        match self {
            Message::Quit => std::process::exit(0),
            Message::Write(text) => println!("{}", text),
            _ => println!("Processing other message"),
        }
    }
}
```

---

## Option<T>: Null Safety

The most important enum in Rust is `Option<T>` - Rust's way of handling nullable values:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

### Comparison with Null Handling

| Language | Null Representation | Safety |
|----------|-------------------|---------|
| C++ | `nullptr`, raw pointers | Runtime crashes |
| C#/.NET | `null`, `Nullable<T>` | Runtime exceptions |
| Rust | `Option<T>` | Compile-time safety |

### Working with Option<T>

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Alice"))
    } else {
        None
    }
}

// Pattern matching
match find_user(1) {
    Some(name) => println!("Found user: {}", name),
    None => println!("User not found"),
}

// Using if let for simple cases
if let Some(name) = find_user(1) {
    println!("Hello, {}", name);
}

// Chaining operations
let user_name_length = find_user(1)
    .map(|name| name.len())      // Transform if Some
    .unwrap_or(0);               // Default value if None
```

### Common Option Methods

```rust
let maybe_number: Option<i32> = Some(5);

// Unwrapping (use carefully!)
let number = maybe_number.unwrap();           // Panics if None
let number = maybe_number.unwrap_or(0);       // Default value
let number = maybe_number.unwrap_or_else(|| compute_default());

// Safe checking
if maybe_number.is_some() {
    println!("Has value: {}", maybe_number.unwrap());
}

// Transformation
let doubled = maybe_number.map(|x| x * 2);    // Some(10) or None
let as_string = maybe_number.map(|x| x.to_string());

// Filtering
let even = maybe_number.filter(|&x| x % 2 == 0);
```

---

## Result<T, E>: Error Handling

Another crucial enum is `Result<T, E>` for error handling:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### Basic Usage

```rust
use std::fs::File;
use std::io::ErrorKind;

fn open_file(filename: &str) -> Result<File, std::io::Error> {
    File::open(filename)
}

// Pattern matching
match open_file("config.txt") {
    Ok(file) => println!("File opened successfully"),
    Err(error) => match error.kind() {
        ErrorKind::NotFound => println!("File not found"),
        ErrorKind::PermissionDenied => println!("Permission denied"),
        other_error => println!("Other error: {:?}", other_error),
    },
}
```

---

## When to Use Structs vs Enums

### Use Structs When:
- You need to group related data together
- All fields are always present and meaningful
- You're modeling "entities" or "things"

```rust
// Good use of struct - user profile
struct UserProfile {
    username: String,
    email: String,
    created_at: std::time::SystemTime,
    is_active: bool,
}
```

### Use Enums When:
- You have mutually exclusive states or variants  
- You need type-safe state machines
- You're modeling "choices" or "alternatives"

```rust
// Good use of enum - connection state
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { since: std::time::SystemTime },
    Error { message: String, retry_count: u32 },
}
```

### Combining Structs and Enums

```rust
struct GamePlayer {
    name: String,
    health: u32,
    state: PlayerState,
}

enum PlayerState {
    Idle,
    Moving { destination: Point },
    Fighting { target: String },
    Dead { respawn_time: u64 },
}

struct Point {
    x: f64,
    y: f64,
}
```

---

## Advanced Patterns

### Generic Structs

```rust
struct Pair<T> {
    first: T,
    second: T,
}

impl<T> Pair<T> {
    fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }
    
    fn get_first(&self) -> &T {
        &self.first
    }
}

// Usage
let int_pair = Pair::new(1, 2);
let string_pair = Pair::new("hello".to_string(), "world".to_string());
```

### Deriving Common Traits

```rust
#[derive(Debug, Clone, PartialEq)]  // Auto-implement common traits
struct Point {
    x: f64,
    y: f64,
}

let p1 = Point { x: 1.0, y: 2.0 };
let p2 = p1.clone();                // Clone trait
println!("{:?}", p1);               // Debug trait
println!("Equal: {}", p1 == p2);    // PartialEq trait
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting to Handle All Enum Variants

```rust
enum Status {
    Active,
    Inactive,
    Pending,
}

fn handle_status(status: Status) {
    match status {
        Status::Active => println!("Active"),
        Status::Inactive => println!("Inactive"),
        // ❌ Missing Status::Pending - won't compile!
    }
}

// ✅ Solution: Handle all variants or use default
fn handle_status_fixed(status: Status) {
    match status {
        Status::Active => println!("Active"),
        Status::Inactive => println!("Inactive"),
        Status::Pending => println!("Pending"),  // Handle all variants
    }
}
```

### Pitfall 2: Moving Out of Borrowed Content

```rust
struct Container {
    value: String,
}

fn bad_example(container: &Container) -> String {
    container.value  // ❌ Cannot move out of borrowed content
}

// ✅ Solutions:
fn return_reference(container: &Container) -> &str {
    &container.value  // Return a reference
}

fn return_clone(container: &Container) -> String {
    container.value.clone()  // Clone the value
}
```

### Pitfall 3: Unwrapping Options/Results in Production

```rust
// ❌ Dangerous in production code
fn bad_parse(input: &str) -> i32 {
    input.parse::<i32>().unwrap()  // Can panic!
}

// ✅ Better approaches
fn safe_parse(input: &str) -> Option<i32> {
    input.parse().ok()
}

fn parse_with_default(input: &str, default: i32) -> i32 {
    input.parse().unwrap_or(default)
}
```

---

## Key Takeaways

1. **Structs group related data** - similar to classes but with explicit memory layout
2. **Methods are separate** from data definition in `impl` blocks
3. **Enums are powerful** - they can hold data and represent complex state
4. **Pattern matching is exhaustive** - compiler ensures all cases are handled
5. **Option and Result** eliminate null pointer exceptions and improve error handling
6. **Choose the right tool**: structs for entities, enums for choices

---

## Exercises

### Exercise 1: Building a Library System

Create a library management system using structs and enums:

```rust
// Define the data structures
struct Book {
    title: String,
    author: String,
    isbn: String,
    status: BookStatus,
}

enum BookStatus {
    Available,
    CheckedOut { 
        borrower: String, 
        due_date: String 
    },
    Reserved { 
        reserver: String 
    },
}

impl Book {
    fn new(title: String, author: String, isbn: String) -> Self {
        // Your implementation
    }
    
    fn checkout(&mut self, borrower: String, due_date: String) -> Result<(), String> {
        // Your implementation - return error if not available
    }
    
    fn return_book(&mut self) -> Result<(), String> {
        // Your implementation
    }
    
    fn is_available(&self) -> bool {
        // Your implementation
    }
}

fn main() {
    let mut book = Book::new(
        "The Rust Programming Language".to_string(),
        "Steve Klabnik".to_string(),
        "978-1718500440".to_string(),
    );
    
    // Test the implementation
    println!("Available: {}", book.is_available());
    
    match book.checkout("Alice".to_string(), "2023-12-01".to_string()) {
        Ok(()) => println!("Book checked out successfully"),
        Err(e) => println!("Checkout failed: {}", e),
    }
}
```

### Exercise 2: Calculator with Different Number Types

Build a calculator that can handle different number types:

```rust
#[derive(Debug, Clone)]
enum Number {
    Integer(i64),
    Float(f64),
    Fraction { numerator: i64, denominator: i64 },
}

impl Number {
    fn add(self, other: Number) -> Number {
        // Your implementation
        // Convert everything to float for simplicity, or implement proper fraction math
    }
    
    fn to_float(&self) -> f64 {
        // Your implementation
    }
    
    fn display(&self) -> String {
        // Your implementation
    }
}

fn main() {
    let a = Number::Integer(5);
    let b = Number::Float(3.14);
    let c = Number::Fraction { numerator: 1, denominator: 2 };
    
    let result = a.add(b);
    println!("5 + 3.14 = {}", result.display());
}
```

### Exercise 3: State Machine for a Traffic Light

<!-- TODO: Fix this exercise - remove durations from enum variants (redundant with timer field).
     Better approach: use enum variants without data, external ticks_remaining field, 
     and demonstrate struct update syntax. Focus on working with enum variants as core concept. -->

Implement a traffic light state machine:

```rust
struct TrafficLight {
    current_state: LightState,
    timer: u32,
}

enum LightState {
    Red { duration: u32 },
    Yellow { duration: u32 },
    Green { duration: u32 },
}

impl TrafficLight {
    fn new() -> Self {
        // Start with Red for 30 seconds
    }
    
    fn tick(&mut self) {
        // Decrease timer and change state when timer reaches 0
        // Red(30) -> Green(25) -> Yellow(5) -> Red(30) -> ...
    }
    
    fn current_color(&self) -> &str {
        // Return the current color as a string
    }
    
    fn time_remaining(&self) -> u32 {
        // Return remaining time in current state
    }
}

fn main() {
    let mut light = TrafficLight::new();
    
    for _ in 0..100 {
        println!("Light: {}, Time remaining: {}", 
                light.current_color(), 
                light.time_remaining());
        light.tick();
        
        // Simulate 1 second delay
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

**Next Up:** In Chapter 4, we'll dive deep into ownership - Rust's unique approach to memory management that eliminates entire classes of bugs without garbage collection.