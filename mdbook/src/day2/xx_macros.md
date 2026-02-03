# Chapter xx: Macros & Code Generation
--skipped, is in transfer day--

## Learning Objectives
- Understand declarative macros with `macro_rules!`
- Master pattern matching syntax variants
- Learn procedural macros basics
- Generate code at compile time

## What are Macros?

Macros are code that writes other code (metaprogramming). They run at compile time, generating Rust code that gets compiled normally.

```rust
// This macro call
println!("Hello, {}!", "world");

// Expands to something like this
::std::io::_print(::std::fmt::Arguments::new_v1(
    &["Hello, ", "!\n"],
    &[::std::fmt::ArgumentV1::new("world", ::std::fmt::Display::fmt)]
));
```

## Declarative Macros with `macro_rules!`

### Basic Syntax

```rust
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}

say_hello!(); // Prints: Hello!
```

### Pattern Matching Types

#### 1. `item` - Items like functions, structs, modules

```rust
macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() {
            println!("You called {}!", stringify!($func_name));
        }
    };
}

create_function!(foo);
foo(); // Prints: You called foo!
```

#### 2. `block` - Code blocks

```rust
macro_rules! time_it {
    ($block:block) => {
        let start = std::time::Instant::now();
        $block
        println!("Took: {:?}", start.elapsed());
    };
}

time_it!({
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!("Work done!");
});
```

#### 3. `stmt` - Statements

```rust
macro_rules! debug_stmt {
    ($stmt:stmt) => {
        println!("Executing: {}", stringify!($stmt));
        $stmt
    };
}

debug_stmt!(let x = 42;);
```

#### 4. `expr` - Expressions

```rust
macro_rules! double {
    ($e:expr) => {
        $e * 2
    };
}

let result = double!(5 + 3); // 16
```

#### 5. `ty` - Types

```rust
macro_rules! create_struct {
    ($name:ident, $field_type:ty) => {
        struct $name {
            value: $field_type,
        }
    };
}

create_struct!(MyStruct, i32);
```

#### 6. `ident` - Identifiers

```rust
macro_rules! getter {
    ($field:ident) => {
        fn $field(&self) -> &str {
            &self.$field
        }
    };
}
```

#### 7. `path` - Paths like `std::vec::Vec`

```rust
macro_rules! use_type {
    ($path:path) => {
        let _instance: $path = Default::default();
    };
}

use_type!(std::collections::HashMap<String, i32>);
```

#### 8. `literal` - Literal values

```rust
macro_rules! print_literal {
    ($lit:literal) => {
        println!("Literal: {}", $lit);
    };
}

print_literal!("hello");
print_literal!(42);
```

#### 9. `tt` - Token trees (any valid tokens)

```rust
macro_rules! capture_tokens {
    ($($tt:tt)*) => {
        println!("Tokens: {}", stringify!($($tt)*));
    };
}

capture_tokens!(fn main() { println!("anything"); });
```

### Repetition Patterns

#### `*` - Zero or more repetitions

```rust
macro_rules! print_all {
    ($($item:expr),*) => {
        $(
            println!("{}", $item);
        )*
    };
}

print_all!(1, 2, 3, "hello");
```

#### `+` - One or more repetitions

```rust
macro_rules! sum {
    ($first:expr $(, $rest:expr)+) => {
        $first $(+ $rest)+
    };
}

let result = sum!(1, 2, 3, 4); // 10
```

#### `?` - Zero or one repetition

```rust
macro_rules! optional_print {
    ($msg:expr $(, $extra:expr)?) => {
        print!("{}", $msg);
        $(print!(" {}", $extra);)?
        println!();
    };
}

optional_print!("Hello");           // Hello
optional_print!("Hello", "World");  // Hello World
```

### Multiple Pattern Arms

```rust
macro_rules! calculate {
    (add $a:expr, $b:expr) => {
        $a + $b
    };
    (multiply $a:expr, $b:expr) => {
        $a * $b
    };
    (power $base:expr, $exp:expr) => {
        $base.pow($exp)
    };
}

let sum = calculate!(add 5, 3);        // 8
let product = calculate!(multiply 4, 7); // 28
let power = calculate!(power 2, 3);    // 8
```

### Practical Example: HashMap Creation

```rust
macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

let ages = hashmap! {
    "Alice" => 30,
    "Bob" => 25,
    "Carol" => 35,
};
```

## Procedural Macros

Procedural macros are more powerful but complex. They operate on token streams.

### Function-like Macros

```rust
// In Cargo.toml:
// [lib]
// proc-macro = true

use proc_macro::TokenStream;

#[proc_macro]
pub fn make_answer(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

// Usage:
make_answer!();
println!("{}", answer()); // 42
```

### Derive Macros

```rust
#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = syn::parse(input).unwrap();
    impl_hello_macro(&ast)
}

// Usage:
#[derive(HelloMacro)]
struct Pancakes;

Pancakes::hello_macro(); // Hello, Macro! My name is Pancakes!
```

### Attribute Macros

```rust
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    // Transform the function based on arguments
    input // Simplified
}

// Usage:
#[route(GET, "/")]
fn index() -> String {
    "Hello World".to_string()
}
```

## Debugging Macros

### `cargo expand`

```bash
# Install cargo-expand
cargo install cargo-expand

# Expand macros in your code
cargo expand
```

### Debug Printing

```rust
macro_rules! debug_macro {
    ($($tt:tt)*) => {
        println!("Macro input: {}", stringify!($($tt)*));
        // Your actual macro logic here
    };
}
```

## Common Patterns

### Creating DSLs (Domain Specific Languages)

```rust
macro_rules! html {
    ($tag:ident { $($content:tt)* }) => {
        format!("<{0}>{1}</{0}>", stringify!($tag), html!($($content)*))
    };
    ($text:literal) => {
        $text.to_string()
    };
    ($($content:tt)*) => {
        format!("{}", stringify!($($content)*))
    };
}

let page = html! {
    html {
        body {
            h1 { "Welcome" }
            p { "Hello World" }
        }
    }
};
```

### Configuration Macros

```rust
macro_rules! config {
    ($($key:ident: $value:expr),*) => {
        struct Config {
            $(pub $key: String,)*
        }

        impl Default for Config {
            fn default() -> Self {
                Config {
                    $($key: $value.to_string(),)*
                }
            }
        }
    };
}

config! {
    host: "localhost",
    port: "8080",
    debug: "true"
}
```

## Exercise: Create Useful Macros

Create macros that demonstrate different macro patterns and use cases:

### Part 1: Math Operations Macro

Create a macro that handles basic math operations:

```rust
// TODO: Implement a math! macro that can handle different operations
// Usage examples:
// math!(5 + 3) -> 8
// math!(10 - 2) -> 8
// math!(4 * 6) -> 24
// math!(15 / 3) -> 5

macro_rules! math {
    // TODO: Add patterns for different operations
}

fn main() {
    println!("5 + 3 = {}", math!(5 + 3));
    println!("10 - 2 = {}", math!(10 - 2));
    println!("4 * 6 = {}", math!(4 * 6));
    println!("15 / 3 = {}", math!(15 / 3));
}
```

### Part 2: HashMap Creation Macro

Create a macro for easy HashMap creation:

```rust
use std::collections::HashMap;

// TODO: Implement a hashmap! macro for easy HashMap creation
// Usage: hashmap!("key1" => "value1", "key2" => "value2")

macro_rules! hashmap {
    // TODO: Add pattern for key => value pairs
}

fn main() {
    let ages = hashmap!(
        "Alice" => 30,
        "Bob" => 25,
        "Carol" => 35
    );

    println!("Alice is {} years old", ages["Alice"]);
}
```

### Part 3: Struct Generation Macro

Create a macro that generates simple structs:

```rust
// TODO: Implement a make_struct! macro that generates structs
// Usage: make_struct!(Person, name: String, age: u32);

macro_rules! make_struct {
    // TODO: Add pattern for struct name and fields
}

// This should generate:
// struct Person {
//     name: String,
//     age: u32,
// }
//
// impl Person {
//     fn new(name: String, age: u32) -> Self {
//         Person { name, age }
//     }
// }

fn main() {
    make_struct!(Person, name: String, age: u32);

    let person = Person::new("Alice".to_string(), 25);
    println!("{} is {} years old", person.name, person.age);
}
```

**Implementation Hints:**

1. **Math macro:**
   - Use patterns like `($a:expr + $b:expr) => { $a + $b }`
   - Handle different operators with separate arms

2. **HashMap macro:**
   - Use repetition: `($($key:expr => $value:expr),* $(,)?)`
   - Create HashMap and insert each key-value pair

3. **Struct macro:**
   - Use `($name:ident, $($field:ident: $type:ty),*)` pattern
   - Generate both struct and impl with new() method

**What you'll learn:**
- **Pattern matching** in macros with different fragment types
- **Repetition patterns** with `*` and `+`
- **Code generation** for common Rust patterns
- **Macro hygiene** and identifier creation

## Best Practices

- **Use declarative macros** for simple code generation
- **Prefer functions** when macros aren't necessary
- **Test macro expansions** with `cargo expand`
- **Document macro usage** clearly
- **Handle edge cases** in pattern matching
- **Use meaningful names** for macro parameters

## When to Use Macros

✅ **Good for:**
- Reducing boilerplate code
- Creating DSLs
- Code that needs to run at compile time
- When you need the exact tokens/identifiers

❌ **Avoid for:**
- Simple calculations (use functions)
- Type conversions (use traits)
- Logic that can be runtime

---

Next: [Chapter 24: Unsafe Rust & FFI with Bindgen](./24_unsafe_ffi.md)