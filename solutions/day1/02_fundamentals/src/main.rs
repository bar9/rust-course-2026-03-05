// Day 1, Chapter 2: Rust Fundamentals - Exercise Solutions

// Exercise 1: Basic Types and Functions
fn calculate_bmi(height: f64, weight: f64) -> f64 {
    weight / (height * height)
}

fn bmi_category(bmi: f64) -> &'static str {
    match bmi {
        b if b < 18.5 => "Underweight",
        b if b < 25.0 => "Normal",
        b if b < 30.0 => "Overweight",
        _ => "Obese",
    }
}

// Exercise 2: String Manipulation
fn find_longest_word(sentence: &str) -> Option<&str> {
    sentence
        .split_whitespace()
        .max_by_key(|word| word.len())
}

// Exercise 3: Collections and Pattern Matching
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
        *self.items.entry(name).or_insert(0) += quantity;
    }
    
    fn remove_item(&mut self, name: &str, quantity: u32) -> Result<(), String> {
        match self.items.get_mut(name) {
            Some(stock) if *stock >= quantity => {
                *stock -= quantity;
                if *stock == 0 {
                    self.items.remove(name);
                }
                Ok(())
            }
            Some(stock) => Err(format!(
                "Not enough items. Only {} available, but {} requested",
                stock, quantity
            )),
            None => Err(format!("Item '{}' not found in inventory", name)),
        }
    }
    
    fn check_stock(&self, name: &str) -> Option<u32> {
        self.items.get(name).copied()
    }
}

fn main() {
    println!("=== Exercise 1: BMI Calculator ===");
    exercise1_demo();
    println!();
    
    println!("=== Exercise 2: String Manipulation ===");
    exercise2_demo();
    println!();
    
    println!("=== Exercise 3: Inventory System ===");
    exercise3_demo();
}

fn exercise1_demo() {
    let test_cases = vec![
        (1.75, 70.0),  // Normal
        (1.60, 45.0),  // Underweight  
        (1.80, 85.0),  // Overweight
        (1.70, 95.0),  // Obese
    ];
    
    for (height, weight) in test_cases {
        let bmi = calculate_bmi(height, weight);
        let category = bmi_category(bmi);
        println!(
            "Height: {:.2}m, Weight: {:.1}kg => BMI: {:.1}, Category: {}",
            height, weight, bmi, category
        );
    }
}

fn exercise2_demo() {
    let test_sentences = vec![
        "Hello world rust programming",
        "The quick brown fox jumps over the lazy dog",
        "",
        "a bb ccc dddd",
        "Single",
    ];
    
    for sentence in test_sentences {
        match find_longest_word(sentence) {
            Some(word) => println!("Longest word in '{}': '{}'", sentence, word),
            None => println!("No words found in '{}'", sentence),
        }
    }
}

fn exercise3_demo() {
    let mut inventory = Inventory::new();
    
    // Add items
    println!("Adding items to inventory:");
    inventory.add_item("Apples".to_string(), 10);
    inventory.add_item("Bananas".to_string(), 5);
    inventory.add_item("Oranges".to_string(), 8);
    
    // Check initial stock
    for item in ["Apples", "Bananas", "Oranges"] {
        match inventory.check_stock(item) {
            Some(quantity) => println!("  {} in stock: {}", item, quantity),
            None => println!("  {} not found", item),
        }
    }
    
    println!("\nOperations:");
    
    // Successful removal
    match inventory.remove_item("Apples", 3) {
        Ok(()) => println!("✓ Removed 3 apples successfully"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Check updated stock
    match inventory.check_stock("Apples") {
        Some(quantity) => println!("  Apples remaining: {}", quantity),
        None => println!("  Apples not found"),
    }
    
    // Try to remove too many
    match inventory.remove_item("Bananas", 10) {
        Ok(()) => println!("✓ Removed 10 bananas successfully"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Try to remove non-existent item
    match inventory.remove_item("Grapes", 1) {
        Ok(()) => println!("✓ Removed 1 grape successfully"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Add more to existing item
    inventory.add_item("Apples".to_string(), 5);
    match inventory.check_stock("Apples") {
        Some(quantity) => println!("\nAfter adding 5 more apples: {}", quantity),
        None => println!("Apples not found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bmi_calculation() {
        let bmi = calculate_bmi(1.75, 70.0);
        assert!((bmi - 22.857).abs() < 0.01);
    }
    
    #[test]
    fn test_bmi_categories() {
        assert_eq!(bmi_category(17.0), "Underweight");
        assert_eq!(bmi_category(22.0), "Normal");
        assert_eq!(bmi_category(27.0), "Overweight");
        assert_eq!(bmi_category(35.0), "Obese");
    }
    
    #[test]
    fn test_longest_word() {
        // "world" is 5 chars, "Hello" is 5 chars, "rust" is 4 chars
        // max_by_key returns the last one when lengths are equal
        assert_eq!(find_longest_word("Hello world rust"), Some("world"));
        assert_eq!(find_longest_word(""), None);
        assert_eq!(find_longest_word("a bb ccc"), Some("ccc"));
        assert_eq!(find_longest_word("equal same size"), Some("equal"));
    }
    
    #[test]
    fn test_inventory() {
        let mut inventory = Inventory::new();
        
        // Test adding items
        inventory.add_item("Test".to_string(), 10);
        assert_eq!(inventory.check_stock("Test"), Some(10));
        
        // Test adding more to existing
        inventory.add_item("Test".to_string(), 5);
        assert_eq!(inventory.check_stock("Test"), Some(15));
        
        // Test successful removal
        assert!(inventory.remove_item("Test", 5).is_ok());
        assert_eq!(inventory.check_stock("Test"), Some(10));
        
        // Test removing too many
        assert!(inventory.remove_item("Test", 20).is_err());
        assert_eq!(inventory.check_stock("Test"), Some(10));
        
        // Test removing all
        assert!(inventory.remove_item("Test", 10).is_ok());
        assert_eq!(inventory.check_stock("Test"), None);
        
        // Test removing non-existent
        assert!(inventory.remove_item("NonExistent", 1).is_err());
    }
}