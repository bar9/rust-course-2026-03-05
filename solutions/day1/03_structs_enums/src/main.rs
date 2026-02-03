// Day 1, Chapter 3: Structs and Enums - Exercise Solutions

use std::thread;
use std::time::Duration;

// Exercise 1: Building a Library System
#[derive(Debug)]
struct Book {
    title: String,
    author: String,
    isbn: String,
    status: BookStatus,
}

#[derive(Debug)]
enum BookStatus {
    Available,
    CheckedOut {
        borrower: String,
        due_date: String,
    },
    Reserved {
        reserver: String,
    },
}

impl Book {
    fn new(title: String, author: String, isbn: String) -> Self {
        Book {
            title,
            author,
            isbn,
            status: BookStatus::Available,
        }
    }
    
    fn checkout(&mut self, borrower: String, due_date: String) -> Result<(), String> {
        match &self.status {
            BookStatus::Available => {
                self.status = BookStatus::CheckedOut { borrower, due_date };
                Ok(())
            }
            BookStatus::CheckedOut { borrower, .. } => {
                Err(format!("Book is already checked out by {}", borrower))
            }
            BookStatus::Reserved { reserver } => {
                if reserver == &borrower {
                    self.status = BookStatus::CheckedOut { borrower, due_date };
                    Ok(())
                } else {
                    Err(format!("Book is reserved by {}", reserver))
                }
            }
        }
    }
    
    fn return_book(&mut self) -> Result<(), String> {
        match &self.status {
            BookStatus::CheckedOut { .. } => {
                self.status = BookStatus::Available;
                Ok(())
            }
            _ => Err("Book is not checked out".to_string()),
        }
    }
    
    fn is_available(&self) -> bool {
        matches!(self.status, BookStatus::Available)
    }
    
    fn reserve(&mut self, reserver: String) -> Result<(), String> {
        match &self.status {
            BookStatus::Available => {
                self.status = BookStatus::Reserved { reserver };
                Ok(())
            }
            _ => Err("Book is not available for reservation".to_string()),
        }
    }
}

// Exercise 2: Calculator with Different Number Types
#[derive(Debug, Clone)]
enum Number {
    Integer(i64),
    Float(f64),
    Fraction { numerator: i64, denominator: i64 },
}

impl Number {
    fn add(self, other: Number) -> Number {
        // Convert both to floats for simplicity
        Number::Float(self.to_float() + other.to_float())
    }
    
    fn subtract(self, other: Number) -> Number {
        Number::Float(self.to_float() - other.to_float())
    }
    
    fn multiply(self, other: Number) -> Number {
        Number::Float(self.to_float() * other.to_float())
    }
    
    fn divide(self, other: Number) -> Result<Number, String> {
        let other_float = other.to_float();
        if other_float == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(Number::Float(self.to_float() / other_float))
        }
    }
    
    fn to_float(&self) -> f64 {
        match self {
            Number::Integer(i) => *i as f64,
            Number::Float(f) => *f,
            Number::Fraction { numerator, denominator } => {
                if *denominator == 0 {
                    f64::INFINITY
                } else {
                    (*numerator as f64) / (*denominator as f64)
                }
            }
        }
    }
    
    fn display(&self) -> String {
        match self {
            Number::Integer(i) => i.to_string(),
            Number::Float(f) => format!("{:.2}", f),
            Number::Fraction { numerator, denominator } => {
                format!("{}/{}", numerator, denominator)
            }
        }
    }
}

// Exercise 3: State Machine for a Traffic Light
struct TrafficLight {
    current_state: LightState,
    timer: u32,
}

#[derive(Debug, Clone)]
enum LightState {
    Red { duration: u32 },
    Yellow { duration: u32 },
    Green { duration: u32 },
}

impl TrafficLight {
    fn new() -> Self {
        TrafficLight {
            current_state: LightState::Red { duration: 30 },
            timer: 30,
        }
    }
    
    fn tick(&mut self) {
        self.timer = self.timer.saturating_sub(1);
        
        if self.timer == 0 {
            self.current_state = match self.current_state {
                LightState::Red { .. } => {
                    self.timer = 25;
                    LightState::Green { duration: 25 }
                }
                LightState::Green { .. } => {
                    self.timer = 5;
                    LightState::Yellow { duration: 5 }
                }
                LightState::Yellow { .. } => {
                    self.timer = 30;
                    LightState::Red { duration: 30 }
                }
            };
        }
    }
    
    fn current_color(&self) -> &str {
        match self.current_state {
            LightState::Red { .. } => "Red",
            LightState::Yellow { .. } => "Yellow",
            LightState::Green { .. } => "Green",
        }
    }
    
    fn time_remaining(&self) -> u32 {
        self.timer
    }
}

fn main() {
    println!("=== Exercise 1: Library Management System ===");
    exercise1_library_demo();
    println!();
    
    println!("=== Exercise 2: Calculator with Different Number Types ===");
    exercise2_calculator_demo();
    println!();
    
    println!("=== Exercise 3: Traffic Light State Machine ===");
    exercise3_traffic_light_demo();
}

fn exercise1_library_demo() {
    let mut book = Book::new(
        "The Rust Programming Language".to_string(),
        "Steve Klabnik and Carol Nichols".to_string(),
        "978-1718500440".to_string(),
    );
    
    println!("Book created: {} by {}", book.title, book.author);
    println!("Initial status - Available: {}", book.is_available());
    println!();
    
    // Checkout the book
    match book.checkout("Alice".to_string(), "2025-12-01".to_string()) {
        Ok(()) => println!("✓ Book checked out successfully to Alice"),
        Err(e) => println!("✗ Checkout failed: {}", e),
    }
    println!("Available after checkout: {}", book.is_available());
    println!();
    
    // Try to checkout again (should fail)
    match book.checkout("Bob".to_string(), "2025-12-15".to_string()) {
        Ok(()) => println!("✓ Book checked out successfully to Bob"),
        Err(e) => println!("✗ Checkout failed: {}", e),
    }
    println!();
    
    // Return the book
    match book.return_book() {
        Ok(()) => println!("✓ Book returned successfully"),
        Err(e) => println!("✗ Return failed: {}", e),
    }
    println!("Available after return: {}", book.is_available());
    println!();
    
    // Reserve the book
    match book.reserve("Charlie".to_string()) {
        Ok(()) => println!("✓ Book reserved successfully by Charlie"),
        Err(e) => println!("✗ Reservation failed: {}", e),
    }
    
    // Try to checkout with different person (should fail)
    match book.checkout("Dave".to_string(), "2025-12-20".to_string()) {
        Ok(()) => println!("✓ Book checked out successfully to Dave"),
        Err(e) => println!("✗ Checkout failed: {}", e),
    }
    
    // Checkout with the reserver (should succeed)
    match book.checkout("Charlie".to_string(), "2025-12-20".to_string()) {
        Ok(()) => println!("✓ Book checked out successfully to Charlie"),
        Err(e) => println!("✗ Checkout failed: {}", e),
    }
    
    println!("\nFinal book status: {:?}", book.status);
}

fn exercise2_calculator_demo() {
    let a = Number::Integer(5);
    let b = Number::Float(3.14);
    let c = Number::Fraction { numerator: 1, denominator: 2 };
    let d = Number::Fraction { numerator: 3, denominator: 4 };
    
    println!("Numbers:");
    println!("  a = {} (Integer)", a.display());
    println!("  b = {} (Float)", b.display());
    println!("  c = {} = {:.2} (Fraction)", c.display(), c.to_float());
    println!("  d = {} = {:.2} (Fraction)", d.display(), d.to_float());
    println!();
    
    // Addition
    let result = a.clone().add(b.clone());
    println!("a + b = {} + {} = {}", 
        a.display(), b.display(), result.display());
    
    // Subtraction
    let result = b.clone().subtract(c.clone());
    println!("b - c = {} - {} = {}", 
        b.display(), c.display(), result.display());
    
    // Multiplication
    let result = c.clone().multiply(d.clone());
    println!("c * d = {} * {} = {}", 
        c.display(), d.display(), result.display());
    
    // Division
    match a.clone().divide(c.clone()) {
        Ok(result) => println!("a / c = {} / {} = {}", 
            a.display(), c.display(), result.display()),
        Err(e) => println!("Division error: {}", e),
    }
    
    // Division by zero
    let zero = Number::Integer(0);
    match b.clone().divide(zero.clone()) {
        Ok(result) => println!("b / 0 = {}", result.display()),
        Err(e) => println!("b / 0 = Error: {}", e),
    }
    
    // Complex calculation
    let complex = Number::Integer(10)
        .add(Number::Float(5.5))
        .multiply(Number::Fraction { numerator: 2, denominator: 3 });
    println!("\n(10 + 5.5) * (2/3) = {}", complex.display());
}

fn exercise3_traffic_light_demo() {
    let mut light = TrafficLight::new();
    
    println!("Traffic Light Simulation (speeded up)");
    println!("Red -> Green -> Yellow -> Red");
    println!("30s    25s      5s      30s");
    println!("\nStarting simulation...\n");
    
    for tick_count in 0..65 {
        // Print status every 5 ticks or when changing
        let prev_color = light.current_color().to_string();
        light.tick();
        let new_color = light.current_color();
        
        if tick_count % 5 == 0 || prev_color != new_color {
            println!("Tick {:3}: Light: {:6}, Time remaining: {:2}s", 
                tick_count, 
                new_color, 
                light.time_remaining());
            
            if prev_color != new_color && tick_count > 0 {
                println!("    >>> Light changed from {} to {} <<<", 
                    prev_color, new_color);
            }
        }
        
        // Simulate time passing (100ms instead of 1s for demo)
        thread::sleep(Duration::from_millis(50));
    }
    
    println!("\nSimulation complete!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_book_lifecycle() {
        let mut book = Book::new(
            "Test Book".to_string(),
            "Test Author".to_string(),
            "123456".to_string(),
        );
        
        assert!(book.is_available());
        
        // Checkout
        assert!(book.checkout("Alice".to_string(), "2025-12-01".to_string()).is_ok());
        assert!(!book.is_available());
        
        // Can't checkout when already checked out
        assert!(book.checkout("Bob".to_string(), "2025-12-01".to_string()).is_err());
        
        // Return
        assert!(book.return_book().is_ok());
        assert!(book.is_available());
        
        // Can't return when not checked out
        assert!(book.return_book().is_err());
        
        // Reserve
        assert!(book.reserve("Charlie".to_string()).is_ok());
        assert!(!book.is_available());
    }
    
    #[test]
    fn test_number_operations() {
        let a = Number::Integer(10);
        let b = Number::Float(5.5);
        
        let result = a.add(b);
        assert!((result.to_float() - 15.5).abs() < 0.001);
        
        let c = Number::Fraction { numerator: 1, denominator: 2 };
        assert!((c.to_float() - 0.5).abs() < 0.001);
        
        let zero = Number::Integer(0);
        assert!(Number::Integer(10).divide(zero).is_err());
    }
    
    #[test]
    fn test_traffic_light() {
        let mut light = TrafficLight::new();
        
        assert_eq!(light.current_color(), "Red");
        assert_eq!(light.time_remaining(), 30);
        
        // Tick down to transition
        for _ in 0..30 {
            light.tick();
        }
        
        assert_eq!(light.current_color(), "Green");
        assert_eq!(light.time_remaining(), 25);
        
        // Next transition
        for _ in 0..25 {
            light.tick();
        }
        
        assert_eq!(light.current_color(), "Yellow");
        assert_eq!(light.time_remaining(), 5);
        
        // Back to red
        for _ in 0..5 {
            light.tick();
        }
        
        assert_eq!(light.current_color(), "Red");
        assert_eq!(light.time_remaining(), 30);
    }
}