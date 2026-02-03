# Chapter 12: Modules and Visibility
## Organizing Rust Projects at Scale

### Learning Objectives
By the end of this chapter, you'll be able to:
- Structure Rust projects with modules and submodules
- Control visibility with `pub` and privacy rules
- Use the `use` keyword effectively for imports
- Organize code across multiple files
- Design clean module APIs with proper encapsulation
- Apply the module system to build maintainable projects
- Understand path resolution and the module tree

---

## Module Basics

### Defining Modules

```rust
// Modules can be defined inline
mod network {
    pub fn connect() {
        println!("Connecting to network...");
    }
    
    fn internal_function() {
        // Private by default - not accessible outside this module
        println!("Internal network operation");
    }
}

mod database {
    pub struct Connection {
        // Fields are private by default
        host: String,
        port: u16,
    }
    
    impl Connection {
        // Public constructor
        pub fn new(host: String, port: u16) -> Self {
            Connection { host, port }
        }
        
        // Public method
        pub fn execute(&self, query: &str) {
            println!("Executing: {}", query);
        }
        
        // Private method
        fn validate_query(&self, query: &str) -> bool {
            !query.is_empty()
        }
    }
}

fn main() {
    network::connect();
    // network::internal_function(); // Error: private function
    
    let conn = database::Connection::new("localhost".to_string(), 5432);
    conn.execute("SELECT * FROM users");
    // println!("{}", conn.host); // Error: private field
}
```

### Module Hierarchy

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {
            println!("Added to waitlist");
        }
        
        fn seat_at_table() {
            println!("Seated at table");
        }
    }
    
    mod serving {
        fn take_order() {}
        fn serve_order() {}
        fn take_payment() {}
    }
}

// Using paths to access nested modules
pub fn eat_at_restaurant() {
    // Absolute path
    crate::front_of_house::hosting::add_to_waitlist();
    
    // Relative path
    front_of_house::hosting::add_to_waitlist();
}
```

---

## The `use` Keyword

### Basic Imports

```rust
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    
    pub fn multiply(a: i32, b: i32) -> i32 {
        a * b
    }
    
    pub mod advanced {
        pub fn power(base: i32, exp: u32) -> i32 {
            base.pow(exp)
        }
    }
}

// Bring functions into scope
use math::add;
use math::multiply;
use math::advanced::power;

// Group imports
use math::{add, multiply};

// Import everything from a module
use math::advanced::*;

fn main() {
    let sum = add(2, 3);        // No need for math:: prefix
    let product = multiply(4, 5);
    let result = power(2, 10);
}
```

### Re-exporting with `pub use`

```rust
mod shapes {
    pub mod circle {
        pub struct Circle {
            pub radius: f64,
        }
        
        impl Circle {
            pub fn area(&self) -> f64 {
                std::f64::consts::PI * self.radius * self.radius
            }
        }
    }
    
    pub mod rectangle {
        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }
        
        impl Rectangle {
            pub fn area(&self) -> f64 {
                self.width * self.height
            }
        }
    }
}

// Re-export to flatten the hierarchy
pub use shapes::circle::Circle;
pub use shapes::rectangle::Rectangle;

// Now users can do:
// use your_crate::{Circle, Rectangle};
// Instead of:
// use your_crate::shapes::circle::Circle;
```

---

## File-based Modules

### Project Structure

```
src/
├── main.rs
├── lib.rs
├── network/
│   ├── mod.rs
│   ├── client.rs
│   └── server.rs
└── utils.rs
```

### Main Module File (src/main.rs or src/lib.rs)

```rust
// src/lib.rs
pub mod network;  // Looks for network/mod.rs or network.rs
pub mod utils;    // Looks for utils.rs

// Re-export commonly used items
pub use network::client::Client;
pub use network::server::Server;
```

### Module Directory (src/network/mod.rs)

```rust
// src/network/mod.rs
pub mod client;
pub mod server;

// Common network functionality
pub struct Config {
    pub timeout: u64,
    pub retry_count: u32,
}

impl Config {
    pub fn default() -> Self {
        Config {
            timeout: 30,
            retry_count: 3,
        }
    }
}
```

### Submodule Files

```rust
// src/network/client.rs
use super::Config;  // Access parent module

pub struct Client {
    config: Config,
    connected: bool,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            config,
            connected: false,
        }
    }
    
    pub fn connect(&mut self) -> Result<(), String> {
        // Connection logic
        self.connected = true;
        Ok(())
    }
}
```

```rust
// src/network/server.rs
use super::Config;

pub struct Server {
    config: Config,
    listening: bool,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Server {
            config,
            listening: false,
        }
    }
    
    pub fn listen(&mut self, port: u16) -> Result<(), String> {
        println!("Listening on port {}", port);
        self.listening = true;
        Ok(())
    }
}
```

---

## Visibility Rules

### Privacy Boundaries

```rust
mod outer {
    pub fn public_function() {
        println!("Public function");
    }
    
    fn private_function() {
        println!("Private function");
    }
    
    pub mod inner {
        pub fn inner_public() {
            // Can access parent's private items
            super::private_function();
        }
        
        pub(super) fn visible_to_parent() {
            println!("Only visible to parent module");
        }
        
        pub(crate) fn visible_in_crate() {
            println!("Visible throughout the crate");
        }
    }
}

fn main() {
    outer::public_function();
    outer::inner::inner_public();
    // outer::inner::visible_to_parent(); // Error: not visible here
    outer::inner::visible_in_crate(); // OK: we're in the same crate
}
```

### Struct Field Visibility

```rust
mod back_of_house {
    pub struct Breakfast {
        pub toast: String,      // Public field
        seasonal_fruit: String, // Private field
    }
    
    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }
    }
    
    // All fields must be public for tuple struct to be constructable
    pub struct Color(pub u8, pub u8, pub u8);
}

fn main() {
    let mut meal = back_of_house::Breakfast::summer("Rye");
    meal.toast = String::from("Wheat");  // OK: public field
    // meal.seasonal_fruit = String::from("strawberries"); // Error: private
    
    let color = back_of_house::Color(255, 0, 0);  // OK: all fields public
}
```

---

## Module Design Patterns

### API Design with Modules

```rust
// A well-designed module API
pub mod database {
    // Re-export the main types users need
    pub use self::connection::Connection;
    pub use self::error::{Error, Result};
    
    mod connection {
        use super::error::Result;
        
        pub struct Connection {
            // Implementation details hidden
            url: String,
        }
        
        impl Connection {
            pub fn open(url: &str) -> Result<Self> {
                Ok(Connection {
                    url: url.to_string(),
                })
            }
            
            pub fn execute(&self, query: &str) -> Result<()> {
                // Implementation
                Ok(())
            }
        }
    }
    
    mod error {
        use std::fmt;
        
        #[derive(Debug)]
        pub struct Error {
            message: String,
        }
        
        impl fmt::Display for Error {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Database error: {}", self.message)
            }
        }
        
        impl std::error::Error for Error {}
        
        pub type Result<T> = std::result::Result<T, Error>;
    }
}

// Clean usage
use database::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open("postgres://localhost/mydb")?;
    conn.execute("SELECT * FROM users")?;
    Ok(())
}
```

### Builder Pattern with Modules

```rust
pub mod request {
    pub struct Request {
        url: String,
        method: Method,
        headers: Vec<(String, String)>,
    }
    
    #[derive(Clone)]
    pub enum Method {
        GET,
        POST,
        PUT,
        DELETE,
    }
    
    pub struct RequestBuilder {
        url: Option<String>,
        method: Method,
        headers: Vec<(String, String)>,
    }
    
    impl RequestBuilder {
        pub fn new() -> Self {
            RequestBuilder {
                url: None,
                method: Method::GET,
                headers: Vec::new(),
            }
        }
        
        pub fn url(mut self, url: &str) -> Self {
            self.url = Some(url.to_string());
            self
        }
        
        pub fn method(mut self, method: Method) -> Self {
            self.method = method;
            self
        }
        
        pub fn header(mut self, key: &str, value: &str) -> Self {
            self.headers.push((key.to_string(), value.to_string()));
            self
        }
        
        pub fn build(self) -> Result<Request, &'static str> {
            let url = self.url.ok_or("URL is required")?;
            Ok(Request {
                url,
                method: self.method,
                headers: self.headers,
            })
        }
    }
    
    impl Request {
        pub fn builder() -> RequestBuilder {
            RequestBuilder::new()
        }
        
        pub fn send(&self) -> Result<Response, &'static str> {
            // Send request logic
            Ok(Response { status: 200 })
        }
    }
    
    pub struct Response {
        pub status: u16,
    }
}

use request::{Request, Method};

fn main() {
    let response = Request::builder()
        .url("https://api.example.com/data")
        .method(Method::POST)
        .header("Content-Type", "application/json")
        .build()
        .unwrap()
        .send()
        .unwrap();
    
    println!("Response status: {}", response.status);
}
```

---

## Common Patterns and Best Practices

### Prelude Pattern

```rust
// Create a prelude module for commonly used items
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::config::Config;
    pub use crate::client::Client;
    pub use crate::server::Server;
}

// Users can import everything they need with one line:
// use your_crate::prelude::*;
```

### Internal Module Pattern

```rust
pub mod parser {
    // Public API
    pub fn parse(input: &str) -> Result<Expression, Error> {
        let tokens = internal::tokenize(input)?;
        internal::build_ast(tokens)
    }
    
    pub struct Expression {
        // ...
    }
    
    pub struct Error {
        // ...
    }
    
    // Implementation details in internal module
    mod internal {
        use super::*;
        
        pub(super) fn tokenize(input: &str) -> Result<Vec<Token>, Error> {
            // ...
        }
        
        pub(super) fn build_ast(tokens: Vec<Token>) -> Result<Expression, Error> {
            // ...
        }
        
        struct Token {
            // Private implementation detail
        }
    }
}
```

---

## Exercise: Create a Library Management System

Design a module structure for a library system:

```rust
// TODO: Create the following module structure:
// - books module with Book struct and methods
// - members module with Member struct  
// - loans module for managing book loans
// - Use proper visibility modifiers

mod books {
    pub struct Book {
        // TODO: Add fields (some public, some private)
    }
    
    impl Book {
        // TODO: Add constructor and methods
    }
}

mod members {
    pub struct Member {
        // TODO: Add fields
    }
    
    impl Member {
        // TODO: Add methods
    }
}

mod loans {
    use super::books::Book;
    use super::members::Member;
    
    pub struct Loan {
        // TODO: Reference a Book and Member
    }
    
    impl Loan {
        // TODO: Implement loan management
    }
}

pub mod library {
    // TODO: Create a public API that uses the above modules
    // Re-export necessary types
}

fn main() {
    // TODO: Use the library module to:
    // 1. Create some books
    // 2. Register members
    // 3. Create loans
    // 4. Return books
}
```


---

## Key Takeaways

1. **Modules organize code** into logical units with clear boundaries
2. **Privacy by default** - items are private unless marked `pub`
3. **The `use` keyword** brings items into scope for convenience
4. **File structure mirrors module structure** for large projects
5. **`pub use` for re-exports** creates clean public APIs
6. **Visibility modifiers** (`pub(crate)`, `pub(super)`) provide fine-grained control
7. **Module design** should hide implementation details and expose minimal APIs
8. **Prelude pattern** simplifies imports for users of your crate

**Congratulations!** You've completed Day 2 of the Rust course. You now have a solid understanding of Rust's advanced features including traits, generics, error handling, iterators, and module organization. These concepts form the foundation for building robust, maintainable Rust applications.
