# Chapter 11: Iterators and Functional Programming
## Efficient Data Processing with Rust's Iterator Pattern

### Learning Objectives
By the end of this chapter, you'll be able to:
- Use iterator adaptors like map, filter, fold effectively
- Understand lazy evaluation and its performance benefits
- Write closures with proper capture semantics
- Choose between loops and iterator chains
- Convert between collections using collect()
- Handle iterator errors gracefully

---

## The Iterator Trait

```rust
trait Iterator {
    type Item;
    
    fn next(&mut self) -> Option<Self::Item>;
    
    // 70+ provided methods like map, filter, fold, etc.
}
```

### Key Concepts
- **Lazy evaluation**: Operations don't execute until consumed
- **Zero-cost abstraction**: Compiles to same code as hand-written loops
- **Composable**: Chain multiple operations cleanly

---

## Creating Iterators

```rust
fn iterator_sources() {
    // From collections
    let vec = vec![1, 2, 3];
    vec.iter();       // &T - borrows
    vec.into_iter();  // T - takes ownership
    vec.iter_mut();   // &mut T - mutable borrow
    
    // From ranges
    (0..10)           // 0 to 9
    (0..=10)          // 0 to 10 inclusive
    
    // Infinite iterators
    std::iter::repeat(5)      // 5, 5, 5, ...
    (0..).step_by(2)          // 0, 2, 4, 6, ...
    
    // From functions
    std::iter::from_fn(|| Some(rand::random::<u32>()))
}
```

---

## Essential Iterator Adaptors

### Transform: map, flat_map

```rust
fn transformations() {
    let numbers = vec![1, 2, 3, 4];
    
    // Simple transformation
    let doubled: Vec<i32> = numbers.iter()
        .map(|x| x * 2)
        .collect();  // [2, 4, 6, 8]
    
    // Parse strings to numbers, handling errors
    let strings = vec!["1", "2", "3"];
    let parsed: Result<Vec<i32>, _> = strings
        .iter()
        .map(|s| s.parse::<i32>())
        .collect();  // Collects into Result<Vec<_>, _>
    
    // Flatten nested structures
    let nested = vec![vec![1, 2], vec![3, 4]];
    let flat: Vec<i32> = nested
        .into_iter()
        .flat_map(|v| v.into_iter())
        .collect();  // [1, 2, 3, 4]
}
```

### Filter and Search

```rust
fn filtering() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    
    // Keep only even numbers
    let evens: Vec<_> = numbers.iter()
        .filter(|&&x| x % 2 == 0)
        .cloned()
        .collect();  // [2, 4, 6]
    
    // Find first match
    let first_even = numbers.iter()
        .find(|&&x| x % 2 == 0);  // Some(&2)
    
    // Check conditions
    let all_positive = numbers.iter().all(|&x| x > 0);  // true
    let has_seven = numbers.iter().any(|&x| x == 7);    // false
    
    // Position of element
    let pos = numbers.iter().position(|&x| x == 4);  // Some(3)
}
```

### Reduce: fold, reduce, sum

```rust
fn reductions() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // Sum all elements
    let sum: i32 = numbers.iter().sum();  // 15
    
    // Product of all elements
    let product: i32 = numbers.iter().product();  // 120
    
    // Custom reduction with fold
    let result = numbers.iter()
        .fold(0, |acc, x| acc + x * x);  // Sum of squares: 55
    
    // Build a string
    let words = vec!["Hello", "World"];
    let sentence = words.iter()
        .fold(String::new(), |mut acc, word| {
            if !acc.is_empty() { acc.push(' '); }
            acc.push_str(word);
            acc
        });  // "Hello World"
}
```

### Take and Skip

```rust
fn slicing_iterators() {
    let numbers = 0..100;
    
    // Take first n elements
    let first_five: Vec<_> = numbers.clone()
        .take(5)
        .collect();  // [0, 1, 2, 3, 4]
    
    // Skip first n elements
    let after_ten: Vec<_> = numbers.clone()
        .skip(10)
        .take(5)
        .collect();  // [10, 11, 12, 13, 14]
    
    // Take while condition is true
    let until_ten: Vec<_> = numbers.clone()
        .take_while(|&x| x < 10)
        .collect();  // [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
}
```

---

## Closures: Anonymous Functions

### Closure Syntax and Captures

```rust
fn closure_basics() {
    let x = 10;
    
    // Closure that borrows
    let add_x = |y| x + y;
    println!("{}", add_x(5));  // 15
    
    // Closure that mutates
    let mut count = 0;
    let mut increment = || {
        count += 1;
        count
    };
    println!("{}", increment());  // 1
    println!("{}", increment());  // 2
    
    // Move closure - takes ownership
    let message = String::from("Hello");
    let print_message = move || println!("{}", message);
    print_message();
    // message is no longer accessible here
}
```

### Fn, FnMut, FnOnce Traits

```rust
// Fn: Can be called multiple times, borrows values
fn apply_twice<F>(f: F) -> i32 
where F: Fn(i32) -> i32 
{
    f(f(5))
}

// FnMut: Can be called multiple times, mutates values
fn apply_mut<F>(mut f: F) 
where F: FnMut() 
{
    f();
    f();
}

// FnOnce: Can only be called once, consumes values
fn apply_once<F>(f: F) 
where F: FnOnce() 
{
    f();
    // f(); // Error: f was consumed
}
```

---

## Common Patterns

### Processing Collections

```rust
use std::collections::HashMap;

fn collection_processing() {
    let text = "hello world hello rust";
    
    // Word frequency counter
    let word_counts: HashMap<&str, usize> = text
        .split_whitespace()
        .fold(HashMap::new(), |mut map, word| {
            *map.entry(word).or_insert(0) += 1;
            map
        });
    
    // Find most common word
    let most_common = word_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&word, _)| word);
    
    println!("Most common: {:?}", most_common);  // Some("hello")
}
```

### Error Handling with Iterators

```rust
fn parse_numbers(input: &[&str]) -> Result<Vec<i32>, std::num::ParseIntError> {
    input.iter()
        .map(|s| s.parse::<i32>())
        .collect()  // Collects into Result<Vec<_>, _>
}

fn process_files(paths: &[&str]) -> Vec<Result<String, std::io::Error>> {
    paths.iter()
        .map(|path| std::fs::read_to_string(path))
        .collect()  // Collects all results, both Ok and Err
}

// Partition successes and failures
fn partition_results<T, E>(results: Vec<Result<T, E>>) -> (Vec<T>, Vec<E>) {
    let (oks, errs): (Vec<_>, Vec<_>) = results
        .into_iter()
        .partition(|r| r.is_ok());
    
    let values = oks.into_iter().map(|r| r.unwrap()).collect();
    let errors = errs.into_iter().map(|r| r.unwrap_err()).collect();
    
    (values, errors)
}
```

### Infinite Iterators and Lazy Evaluation

```rust
fn lazy_evaluation() {
    // Generate Fibonacci numbers lazily
    let mut fib = (0u64, 1u64);
    let fibonacci = std::iter::from_fn(move || {
        let next = fib.0;
        fib = (fib.1, fib.0 + fib.1);
        Some(next)
    });
    
    // Take only what we need
    let first_10: Vec<_> = fibonacci
        .take(10)
        .collect();
    
    println!("First 10 Fibonacci: {:?}", first_10);
    
    // Find first Fibonacci > 1000
    let mut fib2 = (0u64, 1u64);
    let first_large = std::iter::from_fn(move || {
        let next = fib2.0;
        fib2 = (fib2.1, fib2.0 + fib2.1);
        Some(next)
    })
    .find(|&n| n > 1000);
    
    println!("First > 1000: {:?}", first_large);
}
```

---

## Performance: Iterators vs Loops

```rust
// These compile to identical machine code!

fn sum_squares_loop(nums: &[i32]) -> i32 {
    let mut sum = 0;
    for &n in nums {
        sum += n * n;
    }
    sum
}

fn sum_squares_iter(nums: &[i32]) -> i32 {
    nums.iter()
        .map(|&n| n * n)
        .sum()
}

// Iterator version is:
// - More concise
// - Harder to introduce bugs
// - Easier to modify (add filter, take, etc.)
// - Same performance!
```

---

## Exercise: Data Pipeline

Build a log analysis pipeline using iterators:

```rust
#[derive(Debug)]
struct LogEntry {
    timestamp: u64,
    level: LogLevel,
    message: String,
}

#[derive(Debug, PartialEq)]
enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogEntry {
    fn parse(line: &str) -> Option<LogEntry> {
        // Format: "timestamp|level|message"
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 3 {
            return None;
        }
        
        let timestamp = parts[0].parse().ok()?;
        let level = match parts[1] {
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARNING" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            _ => return None,
        };
        
        Some(LogEntry {
            timestamp,
            level,
            message: parts[2].to_string(),
        })
    }
}

struct LogAnalyzer<'a> {
    lines: &'a [String],
}

impl<'a> LogAnalyzer<'a> {
    fn new(lines: &'a [String]) -> Self {
        LogAnalyzer { lines }
    }
    
    fn parse_entries(&self) -> impl Iterator<Item = LogEntry> + '_ {
        // TODO: Parse lines into LogEntry, skip invalid lines
        self.lines.iter()
            .filter_map(|line| LogEntry::parse(line))
    }
    
    fn errors_only(&self) -> impl Iterator<Item = LogEntry> + '_ {
        // TODO: Return only ERROR level entries
        todo!()
    }
    
    fn in_time_range(&self, start: u64, end: u64) -> impl Iterator<Item = LogEntry> + '_ {
        // TODO: Return entries within time range
        todo!()
    }
    
    fn count_by_level(&self) -> HashMap<LogLevel, usize> {
        // TODO: Count entries by log level
        todo!()
    }
    
    fn most_recent(&self, n: usize) -> Vec<LogEntry> {
        // TODO: Return n most recent entries (highest timestamps)
        todo!()
    }
}

fn main() {
    let log_lines = vec![
        "1000|INFO|Server started".to_string(),
        "1001|DEBUG|Connection received".to_string(),
        "1002|ERROR|Failed to connect to database".to_string(),
        "invalid line".to_string(),
        "1003|WARNING|High memory usage".to_string(),
        "1004|INFO|Request processed".to_string(),
        "1005|ERROR|Timeout error".to_string(),
    ];
    
    let analyzer = LogAnalyzer::new(&log_lines);
    
    // Test the methods
    println!("Valid entries: {}", analyzer.parse_entries().count());
    println!("Errors: {:?}", analyzer.errors_only().collect::<Vec<_>>());
    println!("Count by level: {:?}", analyzer.count_by_level());
    println!("Most recent 3: {:?}", analyzer.most_recent(3));
}
```

---

## Key Takeaways

1. **Iterators are lazy** - nothing happens until you consume them
2. **Zero-cost abstraction** - same performance as hand-written loops
3. **Composable** - chain operations for clean, readable code
4. **collect() is powerful** - converts to any collection type
5. **Closures capture environment** - be aware of borrowing vs moving
6. **Error handling** - Result<Vec<T>, E> vs Vec<Result<T, E>>
7. **Prefer iterators** over manual loops for clarity and safety

**Next Up:** In Chapter 12, we'll explore modules and visibility - essential for organizing larger Rust projects and creating clean APIs.