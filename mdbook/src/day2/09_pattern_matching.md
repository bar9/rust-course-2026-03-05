# Chapter 9: Pattern Matching - Exhaustive Control Flow
## Advanced Pattern Matching, Option/Result Handling, and Match Guards

### Learning Objectives
By the end of this chapter, you'll be able to:
- Use exhaustive pattern matching to handle all possible cases
- Apply advanced patterns with destructuring and guards
- Handle Option and Result types idiomatically
- Use if let, while let for conditional pattern matching
- Understand when to use match vs if let vs pattern matching in function parameters
- Write robust error handling with pattern matching
- Apply match guards for complex conditional logic

---

## Pattern Matching vs Switch Statements

### Comparison with Other Languages

| Feature | C/C++ switch | C# switch | Rust match |
|---------|--------------|-----------|------------|
| Exhaustiveness | No | Partial (warnings) | Yes (enforced) |
| Complex patterns | No | Limited | Full destructuring |
| Guards | No | Limited (when) | Yes |
| Return values | No | Expression (C# 8+) | Always expression |
| Fall-through | Default (dangerous) | No | Not possible |

### Basic Match Expression

```rust
enum TrafficLight {
    Red,
    Yellow,
    Green,
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn handle_traffic_light(light: TrafficLight) -> &'static str {
    match light {
        TrafficLight::Red => "Stop",
        TrafficLight::Yellow => "Prepare to stop",
        TrafficLight::Green => "Go",
        // Compiler ensures all variants are handled!
    }
}

fn handle_message(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quit message received");
            std::process::exit(0);
        },
        Message::Move { x, y } => {
            println!("Move to coordinates: ({}, {})", x, y);
        },
        Message::Write(text) => {
            println!("Text message: {}", text);
        },
        Message::ChangeColor(r, g, b) => {
            println!("Change color to RGB({}, {}, {})", r, g, b);
        },
    }
}
```

---

## Option and Result Pattern Matching

### Handling Option<T>

```rust
fn divide(x: f64, y: f64) -> Option<f64> {
    if y != 0.0 {
        Some(x / y)
    } else {
        None
    }
}

fn process_division(x: f64, y: f64) {
    match divide(x, y) {
        Some(result) => println!("Result: {}", result),
        None => println!("Cannot divide by zero"),
    }
}

// Nested Option handling
fn parse_config(input: Option<&str>) -> Option<u32> {
    match input {
        Some(s) => match s.parse::<u32>() {
            Ok(num) => Some(num),
            Err(_) => None,
        },
        None => None,
    }
}

// Better with combinators (covered later)
fn parse_config_better(input: Option<&str>) -> Option<u32> {
    input?.parse().ok()
}
```

### Handling Result<T, E>

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_file_contents(filename: &str) -> Result<String, io::Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn process_file(filename: &str) {
    match read_file_contents(filename) {
        Ok(contents) => {
            println!("File contents ({} bytes):", contents.len());
            println!("{}", contents);
        },
        Err(error) => {
            match error.kind() {
                io::ErrorKind::NotFound => {
                    println!("File '{}' not found", filename);
                },
                io::ErrorKind::PermissionDenied => {
                    println!("Permission denied for file '{}'", filename);
                },
                _ => {
                    println!("Error reading file '{}': {}", filename, error);
                },
            }
        }
    }
}

// Custom error types
#[derive(Debug)]
enum ConfigError {
    MissingFile,
    ParseError(String),
    ValidationError(String),
}

fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|_| ConfigError::MissingFile)?;
    
    let config: Config = serde_json::from_str(&contents)
        .map_err(|e| ConfigError::ParseError(e.to_string()))?;
    
    validate_config(&config)
        .map_err(|msg| ConfigError::ValidationError(msg))?;
    
    Ok(config)
}

#[derive(Debug)]
struct Config {
    port: u16,
    host: String,
}

fn validate_config(config: &Config) -> Result<(), String> {
    if config.port == 0 {
        return Err("Port cannot be zero".to_string());
    }
    if config.host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }
    Ok(())
}
```

---

## Advanced Patterns

### Destructuring and Nested Patterns

```rust
struct Point {
    x: i32,
    y: i32,
}

enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, bottom_right: Point },
    Triangle(Point, Point, Point),
}

fn analyze_shape(shape: &Shape) {
    match shape {
        // Destructure nested structures
        Shape::Circle { center: Point { x, y }, radius } => {
            println!("Circle at ({}, {}) with radius {}", x, y, radius);
        },
        
        // Partial destructuring with ..
        Shape::Rectangle { top_left: Point { x: x1, y: y1 }, .. } => {
            println!("Rectangle starting at ({}, {})", x1, y1);
        },
        
        // Destructure tuple variants
        Shape::Triangle(p1, p2, p3) => {
            println!("Triangle with vertices: ({}, {}), ({}, {}), ({}, {})", 
                     p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
        },
    }
}

// Pattern matching with references and dereferencing
fn process_optional_point(point: &Option<Point>) {
    match point {
        Some(Point { x, y }) => println!("Point at ({}, {})", x, y),
        None => println!("No point"),
    }
}

// Multiple patterns
fn classify_number(n: i32) -> &'static str {
    match n {
        1 | 2 | 3 => "small",
        4..=10 => "medium",
        11..=100 => "large",
        _ => "very large",
    }
}

// Binding values in patterns
fn process_message_advanced(msg: Message) {
    match msg {
        Message::Move { x: 0, y } => {
            println!("Move vertically to y: {}", y);
        },
        Message::Move { x, y: 0 } => {
            println!("Move horizontally to x: {}", x);
        },
        Message::Move { x, y } if x == y => {
            println!("Move diagonally to ({}, {})", x, y);
        },
        Message::Move { x, y } => {
            println!("Move to ({}, {})", x, y);
        },
        msg @ Message::Write(_) => {
            println!("Received write message: {:?}", msg);
        },
        _ => println!("Other message"),
    }
}
```

### Match Guards

```rust
fn categorize_temperature(temp: f64, is_celsius: bool) -> &'static str {
    match temp {
        t if is_celsius && t < 0.0 => "freezing (Celsius)",
        t if is_celsius && t > 100.0 => "boiling (Celsius)",
        t if !is_celsius && t < 32.0 => "freezing (Fahrenheit)",
        t if !is_celsius && t > 212.0 => "boiling (Fahrenheit)",
        t if t > 0.0 => "positive temperature",
        0.0 => "exactly zero",
        _ => "negative temperature",
    }
}

// Complex guards with destructuring
#[derive(Debug)]
enum Request {
    Get { path: String, authenticated: bool },
    Post { path: String, data: Vec<u8> },
}

fn handle_request(req: Request) -> &'static str {
    match req {
        Request::Get { path, authenticated: true } if path.starts_with("/admin") => {
            "Admin access granted"
        },
        Request::Get { path, authenticated: false } if path.starts_with("/admin") => {
            "Admin access denied"
        },
        Request::Get { .. } => "Regular GET request",
        Request::Post { data, .. } if data.len() > 1024 => {
            "Large POST request"
        },
        Request::Post { .. } => "Regular POST request",
    }
}
```

---

## if let and while let

### if let for Simple Cases

```rust
// Instead of verbose match
fn process_option_verbose(opt: Option<i32>) {
    match opt {
        Some(value) => println!("Got value: {}", value),
        None => {}, // Do nothing
    }
}

// Use if let for cleaner code
fn process_option_clean(opt: Option<i32>) {
    if let Some(value) = opt {
        println!("Got value: {}", value);
    }
}

// if let with else
fn process_result(result: Result<String, &str>) {
    if let Ok(value) = result {
        println!("Success: {}", value);
    } else {
        println!("Something went wrong");
    }
}

// Chaining if let
fn process_nested(opt: Option<Result<i32, &str>>) {
    if let Some(result) = opt {
        if let Ok(value) = result {
            println!("Got nested value: {}", value);
        }
    }
}
```

### while let for Loops

```rust
fn process_iterator() {
    let mut stack = vec![1, 2, 3, 4, 5];
    
    // Pop elements while they exist
    while let Some(value) = stack.pop() {
        println!("Processing: {}", value);
    }
}

fn process_lines() {
    use std::io::{self, BufRead};
    
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    // Process lines until EOF or error
    while let Ok(line) = lines.next().unwrap_or(Err(io::Error::new(
        io::ErrorKind::UnexpectedEof, "EOF"
    ))) {
        if line.trim() == "quit" {
            break;
        }
        println!("You entered: {}", line);
    }
}
```

---

## Pattern Matching in Function Parameters

### Destructuring in Parameters

```rust
// Destructure tuples in parameters
fn print_coordinates((x, y): (i32, i32)) {
    println!("Coordinates: ({}, {})", x, y);
}

// Destructure structs
fn print_point(Point { x, y }: Point) {
    println!("Point: ({}, {})", x, y);
}

// Destructure with references
fn analyze_point_ref(&Point { x, y }: &Point) {
    println!("Analyzing point at ({}, {})", x, y);
}

// Closure patterns
fn main() {
    let points = vec![
        Point { x: 1, y: 2 },
        Point { x: 3, y: 4 },
        Point { x: 5, y: 6 },
    ];
    
    // Destructure in closure parameters
    points.iter().for_each(|&Point { x, y }| {
        println!("Point: ({}, {})", x, y);
    });
    
    // Filter with pattern matching
    let origin_points: Vec<_> = points
        .into_iter()
        .filter(|Point { x: 0, y: 0 }| true)  // Only points at origin
        .collect();
}
```

---

## Common Pitfalls and Best Practices

### Pitfall 1: Incomplete Patterns

```rust
// BAD: This won't compile - missing Some case
fn bad_option_handling(opt: Option<i32>) {
    match opt {
        None => println!("Nothing"),
        // Error: non-exhaustive patterns
    }
}

// GOOD: Handle all cases
fn good_option_handling(opt: Option<i32>) {
    match opt {
        Some(val) => println!("Value: {}", val),
        None => println!("Nothing"),
    }
}
```

### Pitfall 2: Unreachable Patterns

```rust
// BAD: Unreachable pattern
fn bad_range_matching(n: i32) {
    match n {
        1..=10 => println!("Small"),
        5 => println!("Five"), // This is unreachable!
        _ => println!("Other"),
    }
}

// GOOD: More specific patterns first
fn good_range_matching(n: i32) {
    match n {
        5 => println!("Five"),
        1..=10 => println!("Small (not five)"),
        _ => println!("Other"),
    }
}
```

### Best Practices

```rust
// 1. Use @ binding to capture while pattern matching
fn handle_special_ranges(value: i32) {
    match value {
        n @ 1..=5 => println!("Small number: {}", n),
        n @ 6..=10 => println!("Medium number: {}", n),
        n => println!("Large number: {}", n),
    }
}

// 2. Use .. to ignore fields you don't need
struct LargeStruct {
    important: i32,
    flag: bool,
    data1: String,
    data2: String,
    data3: Vec<u8>,
}

fn process_large_struct(s: LargeStruct) {
    match s {
        LargeStruct { important, flag: true, .. } => {
            println!("Important value with flag: {}", important);
        },
        LargeStruct { important, .. } => {
            println!("Important value without flag: {}", important);
        },
    }
}

// 3. Prefer early returns with guards
fn validate_user_input(input: &str) -> Result<i32, &'static str> {
    match input.parse::<i32>() {
        Ok(n) if n >= 0 => Ok(n),
        Ok(_) => Err("Number must be non-negative"),
        Err(_) => Err("Invalid number format"),
    }
}
```

---

## Exercise: HTTP Status Handler
Create a function that handles different HTTP status codes using pattern matching:

```rust
#[derive(Debug)]
enum HttpStatus {
    Ok,                    // 200
    NotFound,             // 404
    ServerError,          // 500
    Custom(u16),          // Any other code
}

#[derive(Debug)]
struct HttpResponse {
    status: HttpStatus,
    body: Option<String>,
    headers: Vec<(String, String)>,
}

// TODO: Implement this function
fn handle_response(response: HttpResponse) -> String {
    // Pattern match on the response to return appropriate messages:
    // - Ok with body: "Success: {body}"
    // - Ok without body: "Success: No content"
    // - NotFound: "Error: Resource not found"
    // - ServerError: "Error: Internal server error"
    // - Custom(code) where code < 400: "Info: Status {code}"
    // - Custom(code) where code >= 400: "Error: Status {code}"
    todo!()
}
```


---

## Key Takeaways

1. **Exhaustiveness** - Rust's compiler ensures you handle all possible cases
2. **Pattern matching is an expression** - Every match arm must return the same type
3. **Use if let** for simple Option/Result handling instead of verbose match
4. **Match guards** enable complex conditional logic within patterns
5. **Destructuring** allows you to extract values from complex data structures
6. **Order matters** - More specific patterns should come before general ones
7. **@ binding** lets you capture values while pattern matching
8. **Early returns** with guards can make code more readable

**Next Up:** In Chapter 10, we'll explore error handling - Rust's approach to robust error management with Result types and the ? operator.
