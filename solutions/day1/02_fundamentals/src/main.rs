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
    sentence.split_whitespace().max_by_key(|word| word.len())
}

fn main() {
    println!("=== Exercise 1: BMI Calculator ===");
    exercise1_demo();
    println!();

    println!("=== Exercise 2: String Manipulation ===");
    exercise2_demo();
    println!();
}

fn exercise1_demo() {
    let test_cases = vec![
        (1.75, 70.0), // Normal
        (1.60, 45.0), // Underweight
        (1.80, 85.0), // Overweight
        (1.70, 95.0), // Obese
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
}
