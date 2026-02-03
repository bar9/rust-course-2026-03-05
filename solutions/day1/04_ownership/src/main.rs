// Day 1, Chapter 4: Ownership - Exercise Solutions

// Exercise 1: Ownership Transfer Chain
fn create_message() -> String {
    String::from("World")
}

fn add_greeting(mut message: String) -> String {
    message.insert_str(0, "Hello, ");
    message
}

fn add_punctuation(mut message: String) -> String {
    message.push('!');
    message
}

fn print_and_consume(message: String) {
    println!("Final message: {}", message);
    // message is dropped here
}

// Exercise 2: Reference vs Ownership - Fixed Version
fn analyze_text(text: &str) -> (usize, String) {
    let word_count = text.split_whitespace().count();
    let uppercase = text.to_uppercase();
    (word_count, uppercase)
}

fn analyze_text_detailed(text: &str) -> TextAnalysis {
    TextAnalysis {
        original: text.to_string(),
        word_count: text.split_whitespace().count(),
        char_count: text.chars().count(),
        uppercase: text.to_uppercase(),
        lowercase: text.to_lowercase(),
        reversed: text.chars().rev().collect(),
    }
}

#[derive(Debug)]
struct TextAnalysis {
    original: String,
    word_count: usize,
    char_count: usize,
    uppercase: String,
    lowercase: String,
    reversed: String,
}

// Exercise 3: Lifetime Annotations
fn longest_common_prefix<'a>(s1: &'a str, s2: &str) -> &'a str {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    let mut i = 0;
    while i < s1_chars.len() && 
          i < s2_chars.len() && 
          s1_chars[i] == s2_chars[i] {
        i += 1;
    }
    
    // Find the byte boundary for the character at position i
    let mut byte_index = 0;
    for (char_index, ch) in s1.chars().enumerate() {
        if char_index == i {
            break;
        }
        byte_index += ch.len_utf8();
    }
    
    &s1[..byte_index]
}

// Additional examples showing ownership patterns

// Example: Borrowing vs Moving in Function Parameters
fn demonstrate_borrowing() {
    let data = vec![1, 2, 3, 4, 5];
    
    // Borrowing - data is still usable after
    let sum = calculate_sum(&data);
    println!("Sum: {}", sum);
    println!("Original data: {:?}", data); // Still works!
    
    // Moving - data is consumed
    let product = calculate_product(data);
    println!("Product: {}", product);
    // println!("{:?}", data); // Error: data was moved
}

fn calculate_sum(numbers: &Vec<i32>) -> i32 {
    numbers.iter().sum()
}

fn calculate_product(numbers: Vec<i32>) -> i32 {
    numbers.iter().product()
}

// Example: Mutable References
fn demonstrate_mutable_references() {
    let mut text = String::from("Hello");
    
    // Multiple immutable borrows are OK
    let r1 = &text;
    let r2 = &text;
    println!("r1: {}, r2: {}", r1, r2);
    // r1 and r2 are no longer used after this point
    
    // Now we can have a mutable borrow
    let r3 = &mut text;
    r3.push_str(", World!");
    println!("After mutation: {}", r3);
}

// Example: Lifetime in Structs
struct BookExcerpt<'a> {
    content: &'a str,
    page: u32,
}

impl<'a> BookExcerpt<'a> {
    fn new(content: &'a str, page: u32) -> Self {
        BookExcerpt { content, page }
    }
    
    fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }
    
    fn first_sentence(&self) -> &str {
        self.content
            .split('.')
            .next()
            .unwrap_or(self.content)
    }
}

// Example: Smart Return Types
enum StringOrRef<'a> {
    Owned(String),
    Borrowed(&'a str),
}

fn get_greeting<'a>(custom: Option<&'a str>) -> StringOrRef<'a> {
    match custom {
        Some(greeting) => StringOrRef::Borrowed(greeting),
        None => StringOrRef::Owned(String::from("Hello, World!")),
    }
}

fn main() {
    println!("=== Exercise 1: Ownership Transfer Chain ===");
    exercise1_demo();
    println!();
    
    println!("=== Exercise 2: Reference vs Ownership ===");
    exercise2_demo();
    println!();
    
    println!("=== Exercise 3: Lifetime Annotations ===");
    exercise3_demo();
    println!();
    
    println!("=== Additional Ownership Examples ===");
    additional_examples();
}

fn exercise1_demo() {
    // Chain the functions together
    let msg = create_message();
    println!("Created: {}", msg);
    
    let msg = add_greeting(msg);
    println!("After greeting: {}", msg);
    // Original msg is no longer available - it was moved
    
    let msg = add_punctuation(msg);
    println!("After punctuation: {}", msg);
    
    print_and_consume(msg);
    // msg is no longer available - it was consumed
    
    // Alternative: chain everything
    println!("\nChaining all operations:");
    let message = create_message();
    print_and_consume(add_punctuation(add_greeting(message)));
}

fn exercise2_demo() {
    let article = String::from("Rust is a systems programming language");
    
    // Using the fixed version that takes a reference
    let (count, upper) = analyze_text(&article);
    
    println!("Original: {}", article);  // Now this works!
    println!("Word count: {}", count);
    println!("Uppercase: {}", upper);
    
    // We can analyze again since we only borrowed
    let (count2, _) = analyze_text(&article);
    println!("Second word count: {}", count2);
    
    // Detailed analysis
    let analysis = analyze_text_detailed(&article);
    println!("\nDetailed analysis:");
    println!("  Original: {}", analysis.original);
    println!("  Words: {}", analysis.word_count);
    println!("  Characters: {}", analysis.char_count);
    println!("  Uppercase: {}", analysis.uppercase);
    println!("  Lowercase: {}", analysis.lowercase);
    println!("  Reversed: {}", analysis.reversed);
    
    // article is still available!
    println!("\nOriginal still available: {}", article);
}

fn exercise3_demo() {
    let word1 = String::from("programming");
    let word2 = "program";
    
    let prefix = longest_common_prefix(&word1, word2);
    println!("Common prefix of '{}' and '{}': '{}'", word1, word2, prefix);
    
    // Both word1 and word2 are still usable
    println!("Word1 still usable: {}", word1);
    println!("Word2 still usable: {}", word2);
    
    // More test cases
    let test_cases = vec![
        ("hello", "help", "hel"),
        ("rust", "ruby", "ru"),
        ("abc", "xyz", ""),
        ("test", "test", "test"),
        ("", "anything", ""),
        ("prefix", "pre", "pre"),
    ];
    
    println!("\nTest cases:");
    for (s1, s2, expected) in test_cases {
        let result = longest_common_prefix(s1, s2);
        println!("  '{}' & '{}' => '{}' (expected: '{}')", 
            s1, s2, result, expected);
        assert_eq!(result, expected);
    }
}

fn additional_examples() {
    println!("\n--- Borrowing vs Moving ---");
    demonstrate_borrowing();
    
    println!("\n--- Mutable References ---");
    demonstrate_mutable_references();
    
    println!("\n--- Lifetimes in Structs ---");
    let novel = String::from("Call me Ishmael. Some years ago, never mind how long precisely, \
        having little or no money in my purse, and nothing particular to interest me on shore, \
        I thought I would sail about a little and see the watery part of the world.");
    
    let excerpt = BookExcerpt::new(&novel, 1);
    println!("Excerpt from page {}: '{}'", excerpt.page, excerpt.first_sentence());
    println!("Word count in excerpt: {}", excerpt.word_count());
    
    println!("\n--- Smart Return Types ---");
    match get_greeting(None) {
        StringOrRef::Owned(s) => println!("Got owned string: {}", s),
        StringOrRef::Borrowed(s) => println!("Got borrowed string: {}", s),
    }
    
    match get_greeting(Some("Custom greeting")) {
        StringOrRef::Owned(s) => println!("Got owned string: {}", s),
        StringOrRef::Borrowed(s) => println!("Got borrowed string: {}", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ownership_chain() {
        let msg = create_message();
        let msg = add_greeting(msg);
        let msg = add_punctuation(msg);
        assert_eq!(msg, "Hello, World!");
    }
    
    #[test]
    fn test_analyze_text() {
        let text = "Hello world rust";
        let (count, upper) = analyze_text(text);
        assert_eq!(count, 3);
        assert_eq!(upper, "HELLO WORLD RUST");
        
        // Test that text is still usable
        assert_eq!(text.len(), 16);
    }
    
    #[test]
    fn test_common_prefix() {
        assert_eq!(longest_common_prefix("hello", "help"), "hel");
        assert_eq!(longest_common_prefix("rust", "ruby"), "ru");
        assert_eq!(longest_common_prefix("abc", "xyz"), "");
        assert_eq!(longest_common_prefix("test", "test"), "test");
    }
    
    #[test]
    fn test_book_excerpt() {
        let text = String::from("This is a test. Another sentence.");
        let excerpt = BookExcerpt::new(&text, 5);
        
        assert_eq!(excerpt.first_sentence(), "This is a test");
        assert_eq!(excerpt.word_count(), 6);
        assert_eq!(excerpt.page, 5);
    }
}