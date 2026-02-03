// Chapter 12: Macros Exercise Solution

// =============================================================================
// Exercise: Create Useful Macros
// =============================================================================


// Part 1: Math Operations Macro
/// A macro that handles different math operations
#[macro_export]
macro_rules! math {
    ($a:expr, +, $b:expr) => {
        $a + $b
    };
    ($a:expr, -, $b:expr) => {
        $a - $b
    };
    ($a:expr, *, $b:expr) => {
        $a * $b
    };
    ($a:expr, /, $b:expr) => {
        $a / $b
    };
    ($a:expr, %, $b:expr) => {
        $a % $b
    };
}

// Part 2: HashMap Creation Macro
/// A macro for easy HashMap creation
#[macro_export]
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

// Part 3: Struct Generation Macro
/// A macro that generates simple structs with a constructor
#[macro_export]
macro_rules! make_struct {
    ($name:ident, $($field:ident: $type:ty),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $(pub $field: $type,)*
        }

        impl $name {
            pub fn new($($field: $type),*) -> Self {
                $name {
                    $($field,)*
                }
            }
        }
    };
}

// Part 4: Advanced Macro - Vec Creation with Repetition
/// A macro for creating vectors with repeated values
#[macro_export]
macro_rules! vec_repeat {
    ($value:expr; $count:expr) => {
        vec![$value; $count]
    };
    ($($element:expr),* $(,)?) => {
        vec![$($element,)*]
    };
}

// Part 5: Test Generation Macro
/// A macro that generates test functions
#[macro_export]
macro_rules! test_case {
    ($name:ident, $expression:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!($expression, $expected);
        }
    };
}

// Part 6: Conditional Compilation Macro
/// A macro for conditional debugging
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("[DEBUG] {}", format!($($arg)*));
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test math macro
    #[test]
    fn test_math_operations() {
        assert_eq!(math!(5, +, 3), 8);
        assert_eq!(math!(10, -, 2), 8);
        assert_eq!(math!(4, *, 6), 24);
        assert_eq!(math!(15, /, 3), 5);
        assert_eq!(math!(17, %, 5), 2);
    }

    #[test]
    fn test_math_with_variables() {
        let x = 10;
        let y = 5;
        assert_eq!(math!(x, +, y), 15);
        assert_eq!(math!(x, -, y), 5);
        assert_eq!(math!(x, *, y), 50);
        assert_eq!(math!(x, /, y), 2);
    }

    #[test]
    fn test_math_with_expressions() {
        assert_eq!(math!((2 + 3), *, (4 - 1)), 15);
        assert_eq!(math!(10, /, (3 - 1)), 5);
    }

    // Test hashmap macro
    #[test]
    fn test_hashmap_creation() {
        let ages = hashmap!(
            "Alice" => 30,
            "Bob" => 25,
            "Carol" => 35,
        );

        assert_eq!(ages.len(), 3);
        assert_eq!(ages["Alice"], 30);
        assert_eq!(ages["Bob"], 25);
        assert_eq!(ages["Carol"], 35);
    }

    #[test]
    fn test_empty_hashmap() {
        let empty: HashMap<String, i32> = hashmap!();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_hashmap_different_types() {
        let mixed = hashmap!(
            1 => "one",
            2 => "two",
            3 => "three"
        );

        assert_eq!(mixed[&1], "one");
        assert_eq!(mixed[&2], "two");
        assert_eq!(mixed[&3], "three");
    }

    // Test struct generation macro
    #[test]
    fn test_struct_generation() {
        make_struct!(Person, name: String, age: u32);

        let person = Person::new("Alice".to_string(), 25);
        assert_eq!(person.name, "Alice");
        assert_eq!(person.age, 25);
    }

    #[test]
    fn test_struct_with_multiple_fields() {
        make_struct!(Book, title: String, author: String, pages: u32, published: bool);

        let book = Book::new(
            "1984".to_string(),
            "George Orwell".to_string(),
            328,
            true
        );

        assert_eq!(book.title, "1984");
        assert_eq!(book.author, "George Orwell");
        assert_eq!(book.pages, 328);
        assert_eq!(book.published, true);
    }

    #[test]
    fn test_struct_clone_and_debug() {
        make_struct!(Point, x: i32, y: i32);

        let point1 = Point::new(1, 2);
        let point2 = point1.clone();

        assert_eq!(point1, point2);
        assert_eq!(format!("{:?}", point1), "Point { x: 1, y: 2 }");
    }

    // Test vec_repeat macro
    #[test]
    fn test_vec_repeat() {
        let repeated = vec_repeat!(5; 3);
        assert_eq!(repeated, vec![5, 5, 5]);

        let list = vec_repeat!(1, 2, 3, 4);
        assert_eq!(list, vec![1, 2, 3, 4]);
    }

    // Generated test cases using test_case macro
    test_case!(test_addition, math!(7, +, 8), 15);
    test_case!(test_multiplication, math!(6, *, 7), 42);
    test_case!(test_division, math!(20, /, 4), 5);

    #[test]
    fn test_debug_print_compiles() {
        // This test just ensures the macro compiles
        debug_print!("This is a debug message: {}", 42);
        debug_print!("Simple message");
    }

    // Test complex macro usage
    #[test]
    fn test_macro_composition() {
        // Use multiple macros together
        make_struct!(Student, name: String, grades: HashMap<String, u32>);

        let grades = hashmap!(
            "Math".to_string() => 95,
            "Science".to_string() => 87,
            "English".to_string() => 92
        );

        let student = Student::new("John".to_string(), grades);

        assert_eq!(student.name, "John");
        assert_eq!(student.grades["Math"], 95);
        assert_eq!(student.grades.len(), 3);
    }

    #[test]
    fn test_nested_macro_calls() {
        make_struct!(Calculator, operations: HashMap<String, i32>);

        let calc = Calculator::new(hashmap!(
            "add".to_string() => math!(5, +, 3),
            "subtract".to_string() => math!(10, -, 2),
            "multiply".to_string() => math!(4, *, 6)
        ));

        assert_eq!(calc.operations[&"add".to_string()], 8);
        assert_eq!(calc.operations[&"subtract".to_string()], 8);
        assert_eq!(calc.operations[&"multiply".to_string()], 24);
    }

    #[test]
    fn test_macro_with_different_data_types() {
        let string_to_int = hashmap!(
            "zero" => 0,
            "one" => 1,
            "two" => 2
        );

        let int_to_string = hashmap!(
            0 => "zero".to_string(),
            1 => "one".to_string(),
            2 => "two".to_string()
        );

        assert_eq!(string_to_int["one"], 1);
        assert_eq!(int_to_string[&1], "one");
    }

    #[test]
    fn test_macro_edge_cases() {
        // Test with floating point numbers
        assert_eq!(math!(3.5, +, 2.1), 5.6);
        assert_eq!(math!(10.0, /, 2.0), 5.0);

        // Test with negative numbers
        assert_eq!(math!(-5, +, 10), 5);
        assert_eq!(math!(5, +, (-3)), 2);
    }
}