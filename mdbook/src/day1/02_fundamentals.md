# Chapter 2: Rust Fundamentals
## Type System, Variables, Functions, and Basic Collections

### Learning Objectives
By the end of this chapter, you'll be able to:
- Understand Rust's type system and its relationship to C++/.NET
- Work with variables, mutability, and type inference
- Write and call functions with proper parameter passing
- Handle strings effectively (String vs &str)
- Use basic collections (Vec, HashMap, etc.)
- Apply pattern matching with match expressions

---

## Rust's Type System: Safety First

Rust's type system is designed around two core principles:
1. **Memory Safety**: Prevent segfaults, buffer overflows, and memory leaks
2. **Thread Safety**: Eliminate data races at compile time

### Comparison with Familiar Languages

| Concept | C++ | C#/.NET | Rust |
|---------|-----|---------|------|
| Null checking | Runtime (segfaults) | Runtime (NullReferenceException) | Compile-time (Option<T>) |
| Memory management | Manual (new/delete) | GC | Compile-time (ownership) |
| Thread safety | Runtime (mutexes) | Runtime (locks) | Compile-time (Send/Sync) |
| Type inference | `auto` (C++11+) | `var` | Extensive |

---

## Variables and Mutability

### The Default: Immutable

In Rust, variables are **immutable by default** - a key philosophical difference:

```rust
// Immutable by default
let x = 5;
x = 6; // ❌ Compile error!

// Must explicitly opt into mutability
let mut y = 5;
y = 6; // ✅ This works
```

**Why This Matters:**
- Prevents accidental modifications
- Enables compiler optimizations
- Makes concurrent code safer
- Forces you to think about what should change

### Comparison to C++/.NET

```cpp
// C++: Mutable by default
int x = 5;        // Mutable
const int y = 5;  // Immutable
```

```csharp
// C#: Mutable by default  
int x = 5;              // Mutable
readonly int y = 5;     // Immutable (field-level)
```

```rust
// Rust: Immutable by default
let x = 5;         // Immutable
let mut y = 5;     // Mutable
```

### Type Annotations and Inference

Rust has excellent type inference, but you can be explicit when needed:

```rust
// Type inference (preferred when obvious)
let x = 42;                    // inferred as i32
let name = "Alice";            // inferred as &str
let numbers = vec![1, 2, 3];   // inferred as Vec<i32>

// Explicit types (when needed for clarity or disambiguation)
let x: i64 = 42;
let pi: f64 = 3.14159;
let is_ready: bool = true;
```

### Variable Shadowing

Rust allows "shadowing" - reusing variable names with different types:

```rust
let x = 5;           // x is i32
let x = "hello";     // x is now &str (different variable!)
let x = x.len();     // x is now usize
```

This is different from mutation and is often used for transformations.

---

## Basic Types

### Integer Types

Rust is explicit about integer sizes to prevent overflow issues:

```rust
// Signed integers
let a: i8 = -128;      // 8-bit signed (-128 to 127)
let b: i16 = 32_000;   // 16-bit signed  
let c: i32 = 2_000_000_000;  // 32-bit signed (default)
let d: i64 = 9_223_372_036_854_775_807; // 64-bit signed
let e: i128 = 1;       // 128-bit signed

// Unsigned integers  
let f: u8 = 255;       // 8-bit unsigned (0 to 255)
let g: u32 = 4_000_000_000; // 32-bit unsigned
let h: u64 = 18_446_744_073_709_551_615; // 64-bit unsigned

// Architecture-dependent
let size: usize = 64;  // Pointer-sized (32 or 64 bit)
let diff: isize = -32; // Signed pointer-sized
```

**Note:** Underscores in numbers are just for readability (like `1'000'000` in C++14+).

### Floating Point Types

```rust
let pi: f32 = 3.14159;    // Single precision
let e: f64 = 2.718281828; // Double precision (default)
```

### Boolean and Character Types

```rust
let is_rust_awesome: bool = true;
let emoji: char = '🦀';  // 4-byte Unicode scalar value

// Note: char is different from u8!
let byte_value: u8 = b'A';    // ASCII byte
let unicode_char: char = 'A'; // Unicode character
```

### Tuples: Fixed-Size Heterogeneous Collections

Tuples group values of different types into a compound type. They have a fixed size once declared:

```rust
// Creating tuples
let tup: (i32, f64, u8) = (500, 6.4, 1);
let tup = (500, 6.4, 1);  // Type inference works too

// Destructuring
let (x, y, z) = tup;
println!("The value of y is: {}", y);

// Direct access using dot notation
let five_hundred = tup.0;
let six_point_four = tup.1;
let one = tup.2;

// Empty tuple (unit type)
let unit = ();  // Type () - represents no meaningful value

// Common use: returning multiple values from functions
fn get_coordinates() -> (f64, f64) {
    (37.7749, -122.4194)  // San Francisco coordinates
}

let (lat, lon) = get_coordinates();
```

**Comparison with C++/C#:**
- C++: `std::tuple<int, double, char>` or `std::pair<T1, T2>`
- C#: `(int, double, byte)` value tuples or `Tuple<int, double, byte>`
- Rust: `(i32, f64, u8)` - simpler syntax, built into the language

### Arrays: Fixed-Size Homogeneous Collections

Arrays in Rust have a fixed size known at compile time and store elements of the same type:

```rust
// Creating arrays
let months = ["January", "February", "March", "April", "May", "June",
              "July", "August", "September", "October", "November", "December"];

let a: [i32; 5] = [1, 2, 3, 4, 5];  // Type annotation: [type; length]
let a = [1, 2, 3, 4, 5];            // Type inference

// Initialize with same value
let zeros = [0; 100];  // Creates array with 100 zeros

// Accessing elements
let first = months[0];   // "January"
let second = months[1];  // "February"

// Array slicing
let slice = &months[0..3];  // ["January", "February", "March"]

// Iterating over arrays
for month in &months {
    println!("{}", month);
}

// Arrays vs Vectors comparison
let arr = [1, 2, 3];        // Stack-allocated, fixed size
let vec = vec![1, 2, 3];    // Heap-allocated, growable
```

**Key Differences from Vectors:**
| Feature | Array `[T; N]` | Vector `Vec<T>` |
|---------|----------------|-----------------|
| Size | Fixed at compile time | Growable at runtime |
| Memory | Stack-allocated | Heap-allocated |
| Performance | Faster for small, fixed data | Better for dynamic data |
| Use case | Known size, performance critical | Unknown or changing size |

**Comparison with C++/C#:**
- C++: `int arr[5]` or `std::array<int, 5>`
- C#: `int[] arr = new int[5]` (heap) or `Span<int>` (stack)
- Rust: `let arr: [i32; 5]` - size is part of the type

---

## Functions: The Building Blocks

### Function Syntax

```rust
// Basic function
fn greet() {
    println!("Hello, world!");
}

// Function with parameters
fn add(x: i32, y: i32) -> i32 {
    x + y  // No semicolon = return value
}

// Alternative explicit return
fn subtract(x: i32, y: i32) -> i32 {
    return x - y;  // Explicit return with semicolon
}
```

### Key Differences from C++/.NET

| Aspect | C++ | C#/.NET | Rust |
|--------|-----|---------|------|
| Return syntax | `return x;` | `return x;` | `x` (no semicolon) |
| Parameter types | `int x` | `int x` | `x: i32` |
| Return type | `int func()` | `int Func()` | `fn func() -> i32` |

### Parameters: By Value vs By Reference

```rust
// By value (default) - ownership transferred
fn take_ownership(s: String) {
    println!("{}", s);
    // s is dropped here
}

// By immutable reference - borrowing
fn borrow_immutable(s: &String) {
    println!("{}", s);
    // s reference is dropped, original still valid
}

// By mutable reference - mutable borrowing  
fn borrow_mutable(s: &mut String) {
    s.push_str(" world");
}

// Example usage
fn main() {
    let mut message = String::from("Hello");
    
    borrow_immutable(&message);    // ✅ Can borrow immutably
    borrow_mutable(&mut message);  // ✅ Can borrow mutably
    take_ownership(message);       // ✅ Transfers ownership
    
    // println!("{}", message);    // ❌ Error: value moved
}
```

---

## Control Flow: Making Decisions and Repeating

Rust provides familiar control flow constructs with some unique features that enhance safety and expressiveness.

### if Expressions

In Rust, `if` is an expression, not just a statement - it returns a value:

```rust
// Basic if/else
let number = 7;
if number < 5 {
    println!("Less than 5");
} else if number == 5 {
    println!("Equal to 5");
} else {
    println!("Greater than 5");
}

// if as an expression returning values
let condition = true;
let number = if condition { 5 } else { 10 };  // number = 5

// Must have same type in both branches
// let value = if condition { 5 } else { "ten" }; // ❌ Type mismatch!
```

### Loops: Three Flavors

Rust offers three loop constructs, each with specific use cases:

#### loop - Infinite Loop with Break

```rust
// Infinite loop - must break explicitly
let mut counter = 0;
let result = loop {
    counter += 1;
    
    if counter == 10 {
        break counter * 2;  // loop can return a value!
    }
};
println!("Result: {}", result);  // Prints: Result: 20

// Loop labels for nested loops
'outer: loop {
    println!("Entered outer loop");
    
    'inner: loop {
        println!("Entered inner loop");
        break 'outer;  // Break the outer loop
    }
    
    println!("This won't execute");
}
```

#### while - Conditional Loop

```rust
// Standard while loop
let mut number = 3;
while number != 0 {
    println!("{}!", number);
    number -= 1;
}
println!("LIFTOFF!!!");

// Common pattern: checking conditions
let mut stack = vec![1, 2, 3];
while !stack.is_empty() {
    let value = stack.pop();
    println!("Popped: {:?}", value);
}
```

#### for - Iterator Loop

The `for` loop is the most idiomatic way to iterate in Rust:

```rust
// Iterate over a collection
let numbers = vec![1, 2, 3, 4, 5];
for num in &numbers {
    println!("{}", num);
}

// Range syntax (exclusive end)
for i in 0..5 {
    println!("{}", i);  // Prints 0, 1, 2, 3, 4
}

// Inclusive range
for i in 1..=5 {
    println!("{}", i);  // Prints 1, 2, 3, 4, 5
}

// Enumerate for index and value
let items = vec!["a", "b", "c"];
for (index, value) in items.iter().enumerate() {
    println!("{}: {}", index, value);
}

// Reverse iteration
for i in (1..=3).rev() {
    println!("{}", i);  // Prints 3, 2, 1
}
```

### Comparison with C++/.NET

| Feature | C++ | C#/.NET | Rust |
|---------|-----|---------|------|
| for-each | `for (auto& x : vec)` | `foreach (var x in list)` | `for x in &vec` |
| Index loop | `for (int i = 0; i < n; i++)` | `for (int i = 0; i < n; i++)` | `for i in 0..n` |
| Infinite | `while (true)` | `while (true)` | `loop` |
| Break with value | Not supported | Not supported | `break value` |

### Control Flow Best Practices

```rust
// Prefer iterators over index loops
// ❌ Not idiomatic
let vec = vec![1, 2, 3];
let mut i = 0;
while i < vec.len() {
    println!("{}", vec[i]);
    i += 1;
}

// ✅ Idiomatic
for item in &vec {
    println!("{}", item);
}

// Use if-let for simple pattern matching
let optional = Some(5);

// Verbose match
match optional {
    Some(value) => println!("Got: {}", value),
    None => {},
}

// Cleaner if-let
if let Some(value) = optional {
    println!("Got: {}", value);
}

// while-let for repeated pattern matching
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("Popped: {}", top);
}
```

---

## Strings: The Complex Topic

Strings in Rust are more complex than C++/.NET due to UTF-8 handling and ownership.

### String vs &str: The Key Distinction

```rust
// String: Owned, growable, heap-allocated
let mut owned_string = String::from("Hello");
owned_string.push_str(" world");

// &str: String slice, borrowed, usually stack-allocated  
let string_slice: &str = "Hello world";
let slice_of_string: &str = &owned_string;
```

### Comparison Table

| Type | C++ Equivalent | C#/.NET Equivalent | Rust |
|------|----------------|-------------------|------|
| Owned | `std::string` | `string` | `String` |
| View/Slice | `std::string_view` | `ReadOnlySpan<char>` | `&str` |

### Common String Operations

```rust
// Creation
let s1 = String::from("Hello");
let s2 = "World".to_string();
let s3 = String::new();

// Concatenation
let combined = format!("{} {}", s1, s2);  // Like printf/String.Format
let mut s4 = String::from("Hello");
s4.push_str(" world");                    // Append string
s4.push('!');                            // Append character

// Length and iteration
println!("Length: {}", s4.len());        // Byte length!
println!("Chars: {}", s4.chars().count()); // Character count

// Iterating over characters (proper Unicode handling)
for c in s4.chars() {
    println!("{}", c);
}

// Iterating over bytes
for byte in s4.bytes() {
    println!("{}", byte);
}
```

### String Slicing

```rust
let s = String::from("hello world");

let hello = &s[0..5];   // "hello" - byte indices!
let world = &s[6..11];  // "world"
let full = &s[..];      // Entire string

// ⚠️ Warning: Slicing can panic with Unicode!
let unicode = "🦀🔥";
// let bad = &unicode[0..1]; // ❌ Panics! Cuts through emoji
let good = &unicode[0..4];   // ✅ One emoji (4 bytes)
```

---

## Collections: Vectors and Hash Maps

### Vec<T>: The Workhorse Collection

Vectors are Rust's equivalent to `std::vector` or `List<T>`:

```rust
// Creation
let mut numbers = Vec::new();           // Empty vector
let mut numbers: Vec<i32> = Vec::new(); // With type annotation
let numbers = vec![1, 2, 3, 4, 5];     // vec! macro

// Adding elements
let mut v = Vec::new();
v.push(1);
v.push(2);
v.push(3);

// Accessing elements
let first = &v[0];                      // Panics if out of bounds
let first_safe = v.get(0);              // Returns Option<&T>

match v.get(0) {
    Some(value) => println!("First: {}", value),
    None => println!("Vector is empty"),
}

// Iteration
for item in &v {                        // Borrow each element
    println!("{}", item);
}

for item in &mut v {                    // Mutable borrow
    *item *= 2;
}

for item in v {                         // Take ownership (consumes v)
    println!("{}", item);
}
```

### HashMap<K, V>: Key-Value Storage

```rust
use std::collections::HashMap;

// Creation
let mut scores = HashMap::new();
scores.insert("Alice".to_string(), 100);
scores.insert("Bob".to_string(), 85);

// Or with collect
let teams = vec!["Blue", "Yellow"];
let initial_scores = vec![10, 50];
let scores: HashMap<_, _> = teams
    .iter()
    .zip(initial_scores.iter())
    .collect();

// Accessing values
let alice_score = scores.get("Alice");
match alice_score {
    Some(score) => println!("Alice: {}", score),
    None => println!("Alice not found"),
}

// Iteration
for (key, value) in &scores {
    println!("{}: {}", key, value);
}

// Entry API for complex operations
scores.entry("Charlie".to_string()).or_insert(0);
*scores.entry("Alice".to_string()).or_insert(0) += 10;
```

---

## Pattern Matching with match

The `match` expression is Rust's powerful control flow construct:

### Basic Matching

```rust
let number = 7;

match number {
    1 => println!("One"),
    2 | 3 => println!("Two or three"),
    4..=6 => println!("Four to six"),
    _ => println!("Something else"),  // Default case
}
```

### Matching with Option<T>

```rust
let maybe_number: Option<i32> = Some(5);

match maybe_number {
    Some(value) => println!("Got: {}", value),
    None => println!("Nothing here"),
}

// Or use if let for simple cases
if let Some(value) = maybe_number {
    println!("Got: {}", value);
}
```

### Destructuring

```rust
let point = (3, 4);

match point {
    (0, 0) => println!("Origin"),
    (x, 0) => println!("On x-axis at {}", x),
    (0, y) => println!("On y-axis at {}", y),
    (x, y) => println!("Point at ({}, {})", x, y),
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: String vs &str Confusion
```rust
// ❌ Common mistake
fn greet(name: String) {  // Takes ownership
    println!("Hello, {}", name);
}

let name = String::from("Alice");
greet(name);
// greet(name); // ❌ Error: value moved

// ✅ Better approach
fn greet(name: &str) {    // Borrows
    println!("Hello, {}", name);
}

let name = String::from("Alice");
greet(&name);
greet(&name); // ✅ Still works
```

### Pitfall 2: Integer Overflow in Debug Mode
```rust
let mut x: u8 = 255;
x += 1;  // Panics in debug mode, wraps in release mode

// Use checked arithmetic for explicit handling
match x.checked_add(1) {
    Some(result) => x = result,
    None => println!("Overflow detected!"),
}
```

### Pitfall 3: Vec Index Out of Bounds
```rust
let v = vec![1, 2, 3];
// let x = v[10];  // ❌ Panics!

// ✅ Safe alternatives
let x = v.get(10);          // Returns Option<&T>
let x = v.get(0).unwrap();  // Explicit panic with better message
```

---

## Key Takeaways

1. **Immutability by default** encourages safer, more predictable code
2. **Type inference is powerful** but explicit types help with clarity
3. **String handling is more complex** but prevents many Unicode bugs
4. **Collections are memory-safe** with compile-time bounds checking
5. **Pattern matching is exhaustive** and catches errors at compile time

**Memory Insight:** Unlike C++ or .NET, Rust tracks ownership at compile time, preventing entire classes of bugs without runtime overhead.

---

## Exercises

### Exercise 1: Basic Types and Functions
Create a program that:
1. Defines a function `calculate_bmi(height: f64, weight: f64) -> f64`
2. Uses the function to calculate BMI for several people
3. Returns a string description ("Underweight", "Normal", "Overweight", "Obese")

```rust
// Starter code
fn calculate_bmi(height: f64, weight: f64) -> f64 {
    // Your implementation here
}

fn bmi_category(bmi: f64) -> &'static str {
    // Your implementation here
}

fn main() {
    let height = 1.75; // meters
    let weight = 70.0;  // kg
    
    let bmi = calculate_bmi(height, weight);
    let category = bmi_category(bmi);
    
    println!("BMI: {:.1}, Category: {}", bmi, category);
}
```

### Exercise 2: String Manipulation
Write a function that:
1. Takes a sentence as input
2. Returns the longest word in the sentence
3. Handle the case where multiple words have the same length

```rust
fn find_longest_word(sentence: &str) -> Option<&str> {
    // Your implementation here
    // Hint: Use split_whitespace() and max_by_key()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_word() {
        assert_eq!(find_longest_word("Hello world rust"), Some("Hello"));
        assert_eq!(find_longest_word(""), None);
        assert_eq!(find_longest_word("a bb ccc"), Some("ccc"));
    }
}
```

### Exercise 3: Collections and Pattern Matching
Build a simple inventory system:
1. Use HashMap to store item names and quantities
2. Implement functions to add, remove, and check items
3. Use pattern matching to handle different scenarios

```rust
use std::collections::HashMap;

struct Inventory {
    items: HashMap<String, u32>,
}

impl Inventory {
    fn new() -> Self {
        Inventory {
            items: HashMap::new(),
        }
    }
    
    fn add_item(&mut self, name: String, quantity: u32) {
        // Your implementation here
    }
    
    fn remove_item(&mut self, name: &str, quantity: u32) -> Result<(), String> {
        // Your implementation here
        // Return error if not enough items
    }
    
    fn check_stock(&self, name: &str) -> Option<u32> {
        // Your implementation here
    }
}

fn main() {
    let mut inventory = Inventory::new();
    
    inventory.add_item("Apples".to_string(), 10);
    inventory.add_item("Bananas".to_string(), 5);
    
    match inventory.remove_item("Apples", 3) {
        Ok(()) => println!("Removed 3 apples"),
        Err(e) => println!("Error: {}", e),
    }
    
    match inventory.check_stock("Apples") {
        Some(quantity) => println!("Apples in stock: {}", quantity),
        None => println!("Apples not found"),
    }
}
```

---

## Additional Resources

- [The Rust Book - Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
- [Rust by Example - Primitives](https://doc.rust-lang.org/rust-by-example/primitives.html)
- [String vs &str Guide](https://blog.mgattozzi.dev/how-do-i-str-string/)

**Next Up:** In Chapter 3, we'll explore structs and enums - Rust's powerful data modeling tools that go far beyond what you might expect from C++/.NET experience.