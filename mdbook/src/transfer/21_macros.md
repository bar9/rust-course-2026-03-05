# Chapter 21: Macros & Code Generation

Macros are Rust's metaprogramming feature - code that writes other code. They run at compile time, generating Rust code that gets compiled with the rest of your program. This chapter covers declarative macros with `macro_rules!` and introduces procedural macros.

## What are Macros?

Macros enable code generation at compile time, reducing boilerplate and enabling domain-specific languages (DSLs). Unlike functions, macros:

- Operate on syntax trees, not values
- Can take a variable number of arguments
- Generate code before type checking
- Can create new syntax patterns

```rust
// This macro call
println!("Hello, {}!", "world");

// Expands to something like this (simplified)
std::io::_print(format_args!("Hello, {}!\n", "world"));
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

Macros use pattern matching with specific fragment specifiers:

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

**Note: Edition 2024 Change**: The `expr` fragment now also matches `const` and `_` expressions. For backwards compatibility, use `expr_2021` if you need the old behavior that doesn't match these expressions.

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

capture_tokens!(fn main() { println!("hello"); });
```

#### 10. `pat` - Patterns

```rust
macro_rules! match_pattern {
    ($val:expr, $($pat:pat => $result:expr),+) => {
        match $val {
            $($pat => $result,)+
        }
    };
}

let x = match_pattern!(5,
    0..=3 => "low",
    4..=6 => "medium",
    _ => "high"
);
```

#### 11. `vis` - Visibility qualifiers

```rust
macro_rules! make_struct {
    ($vis:vis struct $name:ident) => {
        $vis struct $name {
            value: i32,
        }
    };
}

make_struct!(pub struct PublicStruct);
```

#### 12. `lifetime` - Lifetime parameters

```rust
macro_rules! with_lifetime {
    ($lt:lifetime) => {
        struct Ref<$lt> {
            data: &$lt str,
        }
    };
}

with_lifetime!('a);
```

#### 13. `meta` - Attributes

```rust
macro_rules! with_attributes {
    ($(#[$meta:meta])* struct $name:ident) => {
        $(#[$meta])*
        struct $name {
            value: i32,
        }
    };
}

with_attributes! {
    #[derive(Debug, Clone)]
    struct MyStruct
}
```

### Multiple Patterns

```rust
macro_rules! vec_shorthand {
    // Empty vector
    () => {
        Vec::new()
    };

    // Vector with elements
    ($($x:expr),+ $(,)?) => {
        {
            let mut vec = Vec::new();
            $(vec.push($x);)+
            vec
        }
    };
}

let v1 = vec_shorthand!();
let v2 = vec_shorthand![1, 2, 3];
let v3 = vec_shorthand![1, 2, 3,]; // Trailing comma ok
```

### Repetition Operators

- `*` - Zero or more repetitions
- `+` - One or more repetitions
- `?` - Zero or one (optional)

```rust
macro_rules! create_enum {
    ($name:ident { $($variant:ident),* }) => {
        enum $name {
            $($variant,)*
        }
    };
}

create_enum!(Color { Red, Green, Blue });

macro_rules! sum {
    ($x:expr) => ($x);
    ($x:expr, $($rest:expr),+) => {
        $x + sum!($($rest),+)
    };
}

let total = sum!(1, 2, 3, 4); // 10
```

## Advanced Macro Patterns

### Incremental TT Munching

```rust
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

macro_rules! count_tts {
    () => {0usize};
    ($_head:tt $($tail:tt)*) => {1usize + count_tts!($($tail)*)};
}

let count = count_tts!(a b c d); // 4
```

### Push-down Accumulation

```rust
macro_rules! reverse {
    ([] $($reversed:tt)*) => {
        ($($reversed)*)
    };
    ([$head:tt $($tail:tt)*] $($reversed:tt)*) => {
        reverse!([$($tail)*] $head $($reversed)*)
    };
}

let rev = reverse!([1 2 3 4]); // (4 3 2 1)
```

### Internal Rules

```rust
macro_rules! my_macro {
    // Public API
    ($($input:expr),*) => {
        my_macro!(@internal [$($input),*] [])
    };

    // Internal implementation
    (@internal [] [$($result:expr),*]) => {
        vec![$($result),*]
    };

    (@internal [$head:expr $(, $tail:expr)*] [$($result:expr),*]) => {
        my_macro!(@internal [$($tail),*] [$($result,)* $head * 2])
    };
}

let doubled = my_macro!(1, 2, 3); // vec![2, 4, 6]
```

## Hygienic Macros

Rust macros are hygienic - they don't accidentally capture or interfere with variables:

```rust
macro_rules! using_a {
    ($e:expr) => {
        {
            let a = 42;
            $e
        }
    };
}

let a = "outer";
let result = using_a!(a); // Uses outer 'a', not the one in macro
```

To intentionally break hygiene:

```rust
macro_rules! create_and_use {
    ($name:ident) => {
        let $name = 42;
        println!("{}", $name);
    };
}

create_and_use!(my_var); // Creates my_var in caller's scope
```

## Debugging Macros

### Using `trace_macros!`

```rust
#![feature(trace_macros)]

trace_macros!(true);
my_macro!(args);
trace_macros!(false);
```

### Using `log_syntax!`

```rust
#![feature(log_syntax)]

macro_rules! debug_macro {
    ($($arg:tt)*) => {
        log_syntax!($($arg)*);
    };
}
```

### Cargo Expand

```bash
cargo install cargo-expand
cargo expand
```

## Procedural Macros

Procedural macros are more powerful but require a separate crate:

### Types of Procedural Macros

1. **Custom Derive Macros**
2. **Attribute Macros**
3. **Function-like Macros**

### Setup

```toml
# Cargo.toml
[lib]
proc-macro = true

[dependencies]
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"
```

### Custom Derive Macro Example

```rust
// src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl HelloMacro for #name {
            fn hello() {
                println!("Hello from {}!", stringify!(#name));
            }
        }
    };

    gen.into()
}
```

Usage:

```rust
trait HelloMacro {
    fn hello();
}

#[derive(HelloMacro)]
struct MyStruct;

MyStruct::hello(); // Prints: Hello from MyStruct!
```

### Attribute Macro Example

```rust
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as syn::ItemFn);
    let args = parse_macro_input!(args as syn::LitStr);

    // Modify function based on attribute arguments
    quote! {
        #[web::route(#args)]
        #item
    }.into()
}
```

Usage:

```rust
#[route("/api/users")]
async fn get_users() -> Response {
    // Handler implementation
}
```

### Function-like Procedural Macro

```rust
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);
    // Parse SQL and generate code
    quote! {
        // Generated code here
    }.into()
}
```

Usage:

```rust
let query = sql!("SELECT * FROM users WHERE id = ?");
```

## Real-World Examples

### Builder Pattern Macro

```rust
macro_rules! builder {
    ($name:ident { $($field:ident: $type:ty),* }) => {
        pub struct $name {
            $(pub $field: $type,)*
        }

        paste::paste! {
            pub struct [<$name Builder>] {
                $($field: Option<$type>,)*
            }

            impl [<$name Builder>] {
                pub fn new() -> Self {
                    Self {
                        $($field: None,)*
                    }
                }

                $(
                    pub fn $field(mut self, value: $type) -> Self {
                        self.$field = Some(value);
                        self
                    }
                )*

                pub fn build(self) -> Result<$name, &'static str> {
                    Ok($name {
                        $($field: self.$field.ok_or(concat!("Missing field: ", stringify!($field)))?,)*
                    })
                }
            }
        }
    };
}

builder!(Person {
    name: String,
    age: u32,
    email: String
});

let person = PersonBuilder::new()
    .name("Alice".to_string())
    .age(30)
    .email("alice@example.com".to_string())
    .build()?;
```

### Test Generator Macro

```rust
macro_rules! test_cases {
    ($($name:ident: $input:expr => $expected:expr),*) => {
        $(
            #[test]
            fn $name() {
                let result = process($input);
                assert_eq!(result, $expected);
            }
        )*
    };
}

test_cases! {
    test_zero: 0 => 0,
    test_one: 1 => 1,
    test_negative: -5 => 5,
    test_large: 1000 => 1000
}
```

### DSL for State Machines

```rust
macro_rules! state_machine {
    (
        $name:ident {
            states: [$($state:ident),*],
            events: [$($event:ident),*],
            transitions: [
                $($from:ident + $on:ident => $to:ident),*
            ]
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum $name {
            $($state,)*
        }

        #[derive(Debug)]
        enum Event {
            $($event,)*
        }

        impl $name {
            fn transition(self, event: Event) -> Option<Self> {
                match (self, event) {
                    $(
                        (Self::$from, Event::$on) => Some(Self::$to),
                    )*
                    _ => None
                }
            }
        }
    };
}

state_machine! {
    DoorState {
        states: [Open, Closed, Locked],
        events: [OpenDoor, CloseDoor, LockDoor, UnlockDoor],
        transitions: [
            Closed + OpenDoor => Open,
            Open + CloseDoor => Closed,
            Closed + LockDoor => Locked,
            Locked + UnlockDoor => Closed
        ]
    }
}
```

## Common Macro Patterns

### Callback Pattern

```rust
macro_rules! with_callback {
    ($setup:expr, $callback:expr) => {{
        let result = $setup;
        $callback(&result);
        result
    }};
}

let data = with_callback!(
    vec![1, 2, 3],
    |v| println!("Created vector with {} elements", v.len())
);
```

### Configuration Pattern

```rust
macro_rules! config {
    ($($key:ident : $value:expr),* $(,)?) => {{
        #[derive(Debug)]
        struct Config {
            $($key: std::option::Option<String>,)*
        }

        Config {
            $($key: Some($value.to_string()),)*
        }
    }};
}

let cfg = config! {
    host: "localhost",
    port: "8080",
    database: "mydb"
};
```

## Best Practices

1. **Prefer Functions Over Macros**: Use macros only when functions can't achieve your goal
2. **Keep Macros Simple**: Complex macros are hard to debug and maintain
3. **Document Macro Behavior**: Include examples and expansion examples
4. **Use Internal Rules**: Hide implementation details with `@` prefixed rules
5. **Test Macro Expansions**: Use `cargo expand` to verify generated code
6. **Consider Procedural Macros**: For complex transformations, proc macros are clearer
7. **Maintain Hygiene**: Avoid capturing external variables unless intentional

## Limitations and Gotchas

1. **Type Information**: Macros run before type checking
2. **Error Messages**: Macro errors can be cryptic
3. **IDE Support**: Limited autocomplete and navigation
4. **Compilation Time**: Heavy macro use increases compile times
5. **Debugging**: Harder to debug than regular code

## Summary

Macros are a powerful metaprogramming tool in Rust:

- **Declarative macros** (`macro_rules!`) for pattern-based code generation
- **Procedural macros** for more complex AST transformations
- **Hygiene** prevents accidental variable capture
- **Pattern matching** on various syntax elements
- **Repetition** and **recursion** enable complex patterns

Use macros judiciously to eliminate boilerplate while maintaining code clarity.

## Additional Resources

- [The Little Book of Rust Macros](https://veykril.github.io/tlborm/)
- [Procedural Macros Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [syn Documentation](https://docs.rs/syn/)
- [quote Documentation](https://docs.rs/quote/)