# Chapter 8: Generics & Type Safety

## Learning Objectives
- Master generic functions, structs, and methods
- Understand trait bounds and where clauses
- Learn const generics for compile-time parameters
- Apply type-driven design patterns
- Compare with C++ templates and .NET generics

## Introduction

Generics allow you to write flexible, reusable code that works with multiple types while maintaining type safety. Coming from C++ or .NET, you'll find Rust's generics familiar but more constrained—in a good way.

## Generic Functions

### Basic Generic Functions

```rust
// Generic function that works with any type T
fn swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}

// Multiple generic parameters
fn pair<T, U>(first: T, second: U) -> (T, U) {
    (first, second)
}

// Usage
fn main() {
    let mut x = 5;
    let mut y = 10;
    swap(&mut x, &mut y);
    println!("x: {}, y: {}", x, y); // x: 10, y: 5
    
    let p = pair("hello", 42);
    println!("{:?}", p); // ("hello", 42)
}
```

### Comparison with C++ and .NET

| Feature | Rust | C++ Templates | .NET Generics |
|---------|------|---------------|---------------|
| Compilation | Monomorphization | Template instantiation | Runtime generics |
| Type checking | At definition | At instantiation | At definition |
| Constraints | Trait bounds | Concepts (C++20) | Where clauses |
| Code bloat | Yes (like C++) | Yes | No |
| Performance | Zero-cost | Zero-cost | Small overhead |

## Generic Structs

```rust
// Generic struct
struct Point<T> {
    x: T,
    y: T,
}

// Different types for each field
struct Pair<T, U> {
    first: T,
    second: U,
}

// Implementation for generic struct
impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Implementation for specific type
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

fn main() {
    let integer_point = Point::new(5, 10);
    let float_point = Point::new(1.0, 4.0);
    
    // Only available for Point<f64>
    println!("Distance: {}", float_point.distance_from_origin());
}
```

## Trait Bounds

Trait bounds specify what functionality a generic type must have.

```rust
use std::fmt::Display;

// T must implement Display
fn print_it<T: Display>(value: T) {
    println!("{}", value);
}

// Multiple bounds with +
fn print_and_clone<T: Display + Clone>(value: T) -> T {
    println!("{}", value);
    value.clone()
}

// Trait bounds on structs
struct Wrapper<T: Display> {
    value: T,
}

// Complex bounds
fn complex_function<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: Display + Debug,
{
    format!("{} and {:?}", t.clone(), u)
}
```

## Where Clauses

Where clauses make complex bounds more readable:

```rust
use std::fmt::Debug;

// Instead of this...
fn ugly<T: Display + Clone, U: Debug + Display>(t: T, u: U) {
    // ...
}

// Write this...
fn pretty<T, U>(t: T, u: U)
where
    T: Display + Clone,
    U: Debug + Display,
{
    // Much cleaner!
}

// Particularly useful with associated types
fn process<I>(iter: I)
where
    I: Iterator,
    I::Item: Display,
{
    for item in iter {
        println!("{}", item);
    }
}
```

## Generic Enums

The most common generic enums you'll use:

```rust
// Option<T> - Rust's null replacement
enum Option<T> {
    Some(T),
    None,
}

// Result<T, E> - For error handling
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Custom generic enum
enum BinaryTree<T> {
    Empty,
    Node {
        value: T,
        left: Box<BinaryTree<T>>,
        right: Box<BinaryTree<T>>,
    },
}

impl<T> BinaryTree<T> {
    fn new() -> Self {
        BinaryTree::Empty
    }
    
    fn insert(&mut self, value: T) 
    where 
        T: Ord,
    {
        // Implementation here
    }
}
```

## Const Generics

Const generics allow you to parameterize types with constant values:

```rust
// Array wrapper with compile-time size
struct ArrayWrapper<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> ArrayWrapper<T, N> {
    fn new(value: T) -> Self
    where
        T: Copy,
    {
        ArrayWrapper {
            data: [value; N],
        }
    }
}

// Matrix type with compile-time dimensions
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

fn main() {
    let arr: ArrayWrapper<i32, 5> = ArrayWrapper::new(0);
    let matrix: Matrix<f64, 3, 4> = Matrix {
        data: [[0.0; 4]; 3],
    };
}
```

## Type Aliases and Newtype Pattern

```rust
// Type alias - just a synonym
type Kilometers = i32;
type Result<T> = std::result::Result<T, std::io::Error>;

// Newtype pattern - creates a distinct type
struct Meters(f64);
struct Seconds(f64);

impl Meters {
    fn to_feet(&self) -> f64 {
        self.0 * 3.28084
    }
}

// Prevents mixing units
fn calculate_speed(distance: Meters, time: Seconds) -> f64 {
    distance.0 / time.0
}

fn main() {
    let distance = Meters(100.0);
    let time = Seconds(9.58);
    
    // Type safety prevents this:
    // let wrong = calculate_speed(time, distance); // Error!
    
    let speed = calculate_speed(distance, time);
    println!("Speed: {} m/s", speed);
}
```

## Phantom Types

Phantom types provide compile-time guarantees without runtime cost:

```rust
use std::marker::PhantomData;

// States for a type-safe builder
struct Locked;
struct Unlocked;

struct Door<State> {
    name: String,
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new(name: String) -> Self {
        Door {
            name,
            _state: PhantomData,
        }
    }
    
    fn unlock(self) -> Door<Unlocked> {
        Door {
            name: self.name,
            _state: PhantomData,
        }
    }
}

impl Door<Unlocked> {
    fn open(&self) {
        println!("Opening door: {}", self.name);
    }
    
    fn lock(self) -> Door<Locked> {
        Door {
            name: self.name,
            _state: PhantomData,
        }
    }
}

fn main() {
    let door = Door::<Locked>::new("Front".to_string());
    // door.open(); // Error: method not found
    
    let door = door.unlock();
    door.open(); // OK
}
```

## Advanced Pattern: Type-Driven Design

```rust
// Email validation at compile time
struct Unvalidated;
struct Validated;

struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

impl Email<Unvalidated> {
    fn new(value: String) -> Self {
        Email {
            value,
            _state: PhantomData,
        }
    }
    
    fn validate(self) -> Result<Email<Validated>, String> {
        if self.value.contains('@') {
            Ok(Email {
                value: self.value,
                _state: PhantomData,
            })
        } else {
            Err("Invalid email".to_string())
        }
    }
}

impl Email<Validated> {
    fn send(&self) {
        println!("Sending email to: {}", self.value);
    }
}

// Function that only accepts validated emails
fn send_newsletter(email: &Email<Validated>) {
    email.send();
}
```

## Common Pitfalls

### 1. Over-constraining Generics
```rust
// Bad: unnecessary Clone bound
fn bad<T: Clone + Display>(value: &T) {
    println!("{}", value); // Clone not needed!
}

// Good: only required bounds
fn good<T: Display>(value: &T) {
    println!("{}", value);
}
```

### 2. Missing Lifetime Parameters
```rust
// Won't compile
// struct RefHolder<T> {
//     value: &T,
// }

// Correct
struct RefHolder<'a, T> {
    value: &'a T,
}
```

### 3. Monomorphization Bloat
```rust
// Each T creates a new function copy
fn generic<T>(value: T) -> T {
    value
}

// Consider using trait objects for large functions
fn with_trait_object(value: &dyn Display) {
    println!("{}", value);
}
```

## Exercise: Generic Priority Queue with Constraints

Create a priority queue system that demonstrates multiple generic programming concepts:

```rust
use std::fmt::{Debug, Display};
use std::cmp::Ord;
use std::marker::PhantomData;

// Part 1: Basic generic queue with trait bounds
#[derive(Debug)]
struct PriorityQueue<T>
where
    T: Ord + Debug,
{
    items: Vec<T>,
}

impl<T> PriorityQueue<T>
where
    T: Ord + Debug,
{
    fn new() -> Self {
        // TODO: Create a new empty priority queue
        todo!()
    }

    fn enqueue(&mut self, item: T) {
        // TODO: Add item and maintain sorted order (highest priority first)
        // Hint: Use Vec::push() then Vec::sort()
        todo!()
    }

    fn dequeue(&mut self) -> Option<T> {
        // TODO: Remove and return the highest priority item
        // Hint: Use Vec::pop() since we keep items sorted
        todo!()
    }

    fn peek(&self) -> Option<&T> {
        // TODO: Return reference to highest priority item without removing it
        todo!()
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// Part 2: Generic trait for items that can be prioritized
trait Prioritized {
    type Priority: Ord;

    fn priority(&self) -> Self::Priority;
}

// Part 3: Advanced queue that works with any Prioritized type
struct AdvancedQueue<T>
where
    T: Prioritized + Debug,
{
    items: Vec<T>,
}

impl<T> AdvancedQueue<T>
where
    T: Prioritized + Debug,
{
    fn new() -> Self {
        AdvancedQueue { items: Vec::new() }
    }

    fn enqueue(&mut self, item: T) {
        // TODO: Insert item in correct position based on priority
        // Use binary search for efficient insertion
        todo!()
    }

    fn dequeue(&mut self) -> Option<T> {
        // TODO: Remove highest priority item
        todo!()
    }
}

// Part 4: Example types implementing Prioritized
#[derive(Debug, Eq, PartialEq)]
struct Task {
    name: String,
    urgency: u32,
}

impl Prioritized for Task {
    type Priority = u32;

    fn priority(&self) -> Self::Priority {
        // TODO: Return the urgency level
        todo!()
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // TODO: Compare based on urgency (higher urgency = higher priority)
        todo!()
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Part 5: Generic function with multiple trait bounds
fn process_queue<T, Q>(queue: &mut Q, max_items: usize) -> Vec<T>
where
    T: Debug + Clone,
    Q: QueueOperations<T>,
{
    // TODO: Process up to max_items from the queue
    // Return a vector of processed items
    todo!()
}

// Part 6: Trait for queue operations (demonstrates trait design)
trait QueueOperations<T> {
    fn enqueue(&mut self, item: T);
    fn dequeue(&mut self) -> Option<T>;
    fn len(&self) -> usize;
}

// TODO: Implement QueueOperations for PriorityQueue<T>

fn main() {
    // Test basic priority queue with numbers
    let mut num_queue = PriorityQueue::new();
    num_queue.enqueue(5);
    num_queue.enqueue(1);
    num_queue.enqueue(10);
    num_queue.enqueue(3);

    println!("Number queue:");
    while let Some(num) = num_queue.dequeue() {
        println!("Processing: {}", num);
    }

    // Test with custom Task type
    let mut task_queue = PriorityQueue::new();
    task_queue.enqueue(Task { name: "Low".to_string(), urgency: 1 });
    task_queue.enqueue(Task { name: "High".to_string(), urgency: 5 });
    task_queue.enqueue(Task { name: "Medium".to_string(), urgency: 3 });

    println!("\nTask queue:");
    while let Some(task) = task_queue.dequeue() {
        println!("Processing: {:?}", task);
    }

    // Test advanced queue with Prioritized trait
    let mut advanced_queue = AdvancedQueue::new();
    advanced_queue.enqueue(Task { name: "First".to_string(), urgency: 2 });
    advanced_queue.enqueue(Task { name: "Second".to_string(), urgency: 4 });

    println!("\nAdvanced queue:");
    while let Some(task) = advanced_queue.dequeue() {
        println!("Processing: {:?}", task);
    }
}
```

**Implementation Guidelines:**

1. **PriorityQueue methods:**
   - `new()`: Return `PriorityQueue { items: Vec::new() }`
   - `enqueue()`: Push item then sort with `self.items.sort()`
   - `dequeue()`: Use `self.items.pop()` (gets highest after sorting)
   - `peek()`: Use `self.items.last()`

2. **Task::priority():**
   - Return `self.urgency`

3. **Task::cmp():**
   - Use `self.urgency.cmp(&other.urgency)`

4. **AdvancedQueue::enqueue():**
   - Use `binary_search_by_key()` to find insertion point
   - Use `insert()` to maintain sorted order

5. **QueueOperations trait implementation:**
   - Implement for `PriorityQueue<T>` by delegating to existing methods

**What this exercise teaches:**
- **Trait bounds** (`Ord + Debug`) restrict generic types
- **Associated types** in traits (`Priority`)
- **Complex where clauses** for readable constraints
- **Generic trait implementation** with multiple bounds
- **Real-world generic patterns** beyond simple containers
- **Trait design** for abstraction over different implementations

## Key Takeaways

✅ **Generics provide type safety without code duplication** - Write once, use with many types

✅ **Trait bounds specify required functionality** - More explicit than C++ templates

✅ **Monomorphization means zero runtime cost** - Like C++ templates, unlike .NET generics

✅ **Const generics enable compile-time computations** - Arrays and matrices with known sizes

✅ **Phantom types provide compile-time guarantees** - State machines in the type system

✅ **Type-driven design prevents bugs at compile time** - Invalid states are unrepresentable

---

Next: [Chapter 9: Enums & Pattern Matching](./09_pattern_matching.md)