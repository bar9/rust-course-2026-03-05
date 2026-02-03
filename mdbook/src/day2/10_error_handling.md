# Chapter 10: Error Handling - Result, ?, and Custom Errors
## Robust Error Management in Rust

### Learning Objectives
By the end of this chapter, you'll be able to:
- Use Result<T, E> for recoverable error handling
- Master the ? operator for error propagation
- Create custom error types with proper error handling
- Understand when to use Result vs panic!
- Work with popular error handling crates (anyhow, thiserror)
- Implement error conversion and chaining
- Handle multiple error types gracefully

---

## Rust's Error Handling Philosophy

### Error Categories

| Type | Examples | Rust Approach |
|------|----------|---------------|
| **Recoverable** | File not found, network timeout | `Result<T, E>` |
| **Unrecoverable** | Array out of bounds, null pointer | `panic!` |

### Comparison with Other Languages

| Language | Approach | Pros | Cons |
|----------|----------|------|------|
| **C++** | Exceptions, error codes | Familiar | Runtime overhead, can be ignored |
| **C#/.NET** | Exceptions | Clean syntax | Performance cost, hidden control flow |
| **Go** | Explicit error returns | Explicit, fast | Verbose |
| **Rust** | Result<T, E> | Explicit, zero-cost | Must be handled |

---

## Result<T, E>: The Foundation

### Basic Result Usage

```rust
use std::fs::File;
use std::io::ErrorKind;

fn open_file(filename: &str) -> Result<File, std::io::Error> {
    File::open(filename)
}

fn main() {
    // Pattern matching
    match open_file("test.txt") {
        Ok(file) => println!("File opened successfully"),
        Err(error) => match error.kind() {
            ErrorKind::NotFound => println!("File not found"),
            ErrorKind::PermissionDenied => println!("Permission denied"),
            other_error => println!("Other error: {:?}", other_error),
        },
    }
    
    // Using if let
    if let Ok(file) = open_file("test.txt") {
        println!("File opened with if let");
    }
    
    // Unwrap variants (use carefully!)
    // let file1 = open_file("test.txt").unwrap();                    // Panics on error
    // let file2 = open_file("test.txt").expect("Failed to open");    // Panics with message
}
```

---

## The ? Operator: Error Propagation Made Easy

### Basic ? Usage

```rust
use std::fs::File;
use std::io::{self, Read};

// Without ? operator (verbose)
fn read_file_old_way(filename: &str) -> Result<String, io::Error> {
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e),
    }
}

// With ? operator (concise)
fn read_file_new_way(filename: &str) -> Result<String, io::Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Even more concise
fn read_file_shortest(filename: &str) -> Result<String, io::Error> {
    std::fs::read_to_string(filename)
}
```

---

## Custom Error Types

### Simple Custom Errors

```rust
use std::fmt;

#[derive(Debug)]
enum MathError {
    DivisionByZero,
    NegativeSquareRoot,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Cannot divide by zero"),
            MathError::NegativeSquareRoot => write!(f, "Cannot take square root of negative number"),
        }
    }
}

impl std::error::Error for MathError {}

fn divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

fn square_root(x: f64) -> Result<f64, MathError> {
    if x < 0.0 {
        Err(MathError::NegativeSquareRoot)
    } else {
        Ok(x.sqrt())
    }
}
```

---

## Error Conversion and Chaining

### The From Trait for Error Conversion

```rust
use std::fs::File;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
    Custom(String),
}

// Automatic conversion from io::Error
impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::Io(error)
    }
}

// Automatic conversion from ParseIntError
impl From<ParseIntError> for AppError {
    fn from(error: ParseIntError) -> Self {
        AppError::Parse(error)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO error: {}", e),
            AppError::Parse(e) => write!(f, "Parse error: {}", e),
            AppError::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

// Now ? operator works seamlessly
fn read_number_from_file(filename: &str) -> Result<i32, AppError> {
    let contents = std::fs::read_to_string(filename)?; // io::Error -> AppError
    let number = contents.trim().parse::<i32>()?;       // ParseIntError -> AppError
    
    if number < 0 {
        return Err(AppError::Custom("Number must be positive".to_string()));
    }
    
    Ok(number)
}
```

### Chaining Multiple Operations

```rust
use std::path::Path;

fn process_config_file(path: &Path) -> Result<Config, AppError> {
    std::fs::read_to_string(path)?
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| parse_config_line(line))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(Config::default(), |mut cfg, (key, value)| {
            cfg.set(&key, value);
            cfg
        })
        .validate()
        .map_err(|e| AppError::Custom(e))
}

struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    fn default() -> Self {
        Config { settings: HashMap::new() }
    }
    
    fn set(&mut self, key: &str, value: String) {
        self.settings.insert(key.to_string(), value);
    }
    
    fn validate(self) -> Result<Config, String> {
        if self.settings.is_empty() {
            Err("Configuration is empty".to_string())
        } else {
            Ok(self)
        }
    }
}

fn parse_config_line(line: &str) -> Result<(String, String), AppError> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(AppError::Custom(format!("Invalid config line: {}", line)));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
```

---

## Working with External Error Libraries

### Using anyhow for Applications

```rust
use anyhow::{Context, Result, bail};

// anyhow::Result is Result<T, anyhow::Error>
fn load_config(path: &str) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .context("Failed to read config file")?;
    
    let config: Config = serde_json::from_str(&contents)
        .context("Failed to parse JSON config")?;
    
    if config.port == 0 {
        bail!("Invalid port: 0");
    }
    
    Ok(config)
}

fn main() -> Result<()> {
    let config = load_config("app.json")?;
    
    // Chain multiple operations with context
    let server = create_server(&config)
        .context("Failed to create server")?;
    
    server.run()
        .context("Server failed during execution")?;
    
    Ok(())
}
```

### Using thiserror for Libraries

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum DataStoreError {
    #[error("data not found")]
    NotFound,
    
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("invalid input: {msg}")]
    InvalidInput { msg: String },
    
    #[error("database error")]
    Database(#[from] sqlx::Error),
    
    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Use in library code
fn get_user(id: u64) -> Result<User, DataStoreError> {
    if id == 0 {
        return Err(DataStoreError::InvalidInput { 
            msg: "ID cannot be 0".to_string() 
        });
    }
    
    let user = db::query_user(id)?; // Automatic conversion from sqlx::Error
    Ok(user)
}
```

---

## Error Handling Patterns

### Early Returns with ?

```rust
fn process_data(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parsed = parse_input(input)?;
    let validated = validate(parsed)?;
    let processed = transform(validated)?;
    Ok(format_output(processed))
}

// Compare with nested match statements (avoid this!)
fn process_data_verbose(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    match parse_input(input) {
        Ok(parsed) => {
            match validate(parsed) {
                Ok(validated) => {
                    match transform(validated) {
                        Ok(processed) => Ok(format_output(processed)),
                        Err(e) => Err(e.into()),
                    }
                },
                Err(e) => Err(e.into()),
            }
        },
        Err(e) => Err(e.into()),
    }
}
```

### Collecting Results

```rust
fn process_files(paths: &[&str]) -> Result<Vec<String>, io::Error> {
    paths.iter()
        .map(|path| std::fs::read_to_string(path))
        .collect::<Result<Vec<_>, _>>()
}

// Handle partial success
fn process_files_partial(paths: &[&str]) -> (Vec<String>, Vec<io::Error>) {
    let results: Vec<Result<String, io::Error>> = paths.iter()
        .map(|path| std::fs::read_to_string(path))
        .collect();
    
    let mut successes = Vec::new();
    let mut failures = Vec::new();
    
    for result in results {
        match result {
            Ok(content) => successes.push(content),
            Err(e) => failures.push(e),
        }
    }
    
    (successes, failures)
}
```

---

## Testing Error Cases

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_division_by_zero() {
        let result = divide(10.0, 0.0);
        assert!(result.is_err());
        
        match result {
            Err(MathError::DivisionByZero) => (),
            _ => panic!("Expected DivisionByZero error"),
        }
    }
    
    #[test]
    fn test_file_not_found() {
        let result = read_file_contents("nonexistent.txt");
        assert!(result.is_err());
    }
    
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_panic_condition() {
        assert!(false, "assertion failed");
    }
}
```

---

## Exercise: Build a Configuration Parser

Create a robust configuration parser with proper error handling:

```rust
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
enum ConfigError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

// TODO: Implement Display and Error traits for ConfigError
// TODO: Implement From<std::io::Error> for automatic conversion

struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        // TODO: Read file, parse lines, handle comments (#)
        // TODO: Parse key=value pairs
        // TODO: Validate required keys exist
        todo!()
    }
    
    fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
    
    fn get_required(&self, key: &str) -> Result<&String, ConfigError> {
        // TODO: Return error if key doesn't exist
        todo!()
    }
    
    fn get_int(&self, key: &str) -> Result<i32, ConfigError> {
        // TODO: Get value and parse as integer
        todo!()
    }
}

fn main() -> Result<(), ConfigError> {
    let config = Config::from_file("app.conf")?;
    let port = config.get_int("port")?;
    let host = config.get_required("host")?;
    
    println!("Starting server on {}:{}", host, port);
    Ok(())
}
```


---

## Key Takeaways

1. **Use Result<T, E>** for recoverable errors, panic! for unrecoverable ones
2. **The ? operator** makes error propagation clean and efficient
3. **Custom error types** should implement Display and Error traits
4. **Error conversion** with From trait enables seamless ? usage
5. **anyhow** is great for applications, **thiserror** for libraries
6. **Chain operations** with Result for clean error handling
7. **Test error cases** as thoroughly as success cases
8. **Collect multiple errors** when appropriate instead of failing fast

**Next Up:** In Chapter 11, we'll explore iterators and closures - Rust's functional programming features that make data processing both efficient and expressive.
