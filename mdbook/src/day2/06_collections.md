# Chapter 6: Collections Beyond Vec
## HashMap and HashSet for Real-World Applications

### Learning Objectives
By the end of this chapter, you'll be able to:
- Use HashMap<K, V> efficiently for key-value storage
- Apply HashSet<T> for unique value collections
- Master the Entry API for efficient map operations
- Choose between HashMap, BTreeMap, and other collections
- Work with custom types as keys

---

## Quick Collection Reference

| Collection | Use When You Need | Performance |
|------------|-------------------|-------------|
| `Vec<T>` | Ordered sequence, index access | O(1) index, O(n) search |
| `HashMap<K,V>` | Fast key-value lookups | O(1) average all operations |
| `HashSet<T>` | Unique values, fast membership test | O(1) average all operations |
| `BTreeMap<K,V>` | Sorted keys, range queries | O(log n) all operations |

---

## HashMap<K, V>: The Swiss Army Knife

### Basic Operations

```rust
use std::collections::HashMap;

fn hashmap_basics() {
    // Creation
    let mut scores = HashMap::new();
    scores.insert("Alice", 100);
    scores.insert("Bob", 85);
    
    // From iterator
    let teams = vec!["Blue", "Red"];
    let points = vec![10, 50];
    let team_scores: HashMap<_, _> = teams.into_iter()
        .zip(points.into_iter())
        .collect();
    
    // Accessing values
    if let Some(score) = scores.get("Alice") {
        println!("Alice's score: {}", score);
    }
    
    // Check existence
    if scores.contains_key("Alice") {
        println!("Alice is in the map");
    }
}
```

### The Entry API: Powerful and Efficient

```rust
use std::collections::HashMap;

fn entry_api_examples() {
    let mut word_count = HashMap::new();
    let text = "the quick brown fox jumps over the lazy dog the";
    
    // Count words efficiently
    for word in text.split_whitespace() {
        *word_count.entry(word).or_insert(0) += 1;
    }
    
    // Insert if absent
    let mut cache = HashMap::new();
    cache.entry("key").or_insert_with(|| {
        // Expensive computation only runs if key doesn't exist
        expensive_calculation()
    });
    
    // Modify or insert
    let mut scores = HashMap::new();
    scores.entry("Alice")
        .and_modify(|score| *score += 10)
        .or_insert(100);
}

fn expensive_calculation() -> String {
    "computed_value".to_string()
}
```

### HashMap with Custom Keys

```rust
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash)]
struct UserId(u64);

#[derive(Debug, Eq, PartialEq, Hash)]
struct CompositeKey {
    category: String,
    id: u32,
}

fn custom_keys() {
    let mut user_data = HashMap::new();
    user_data.insert(UserId(1001), "Alice");
    user_data.insert(UserId(1002), "Bob");
    
    let mut composite_map = HashMap::new();
    composite_map.insert(
        CompositeKey { category: "user".to_string(), id: 1 },
        "User One"
    );
    
    // Access with custom key
    if let Some(name) = user_data.get(&UserId(1001)) {
        println!("Found user: {}", name);
    }
}
```

---

## HashSet<T>: Unique Value Collections

### Basic Operations and Set Theory

```rust
use std::collections::HashSet;

fn hashset_operations() {
    // Create and populate
    let mut set1: HashSet<i32> = vec![1, 2, 3, 2, 4].into_iter().collect();
    let set2: HashSet<i32> = vec![3, 4, 5, 6].into_iter().collect();
    
    // Set operations
    let union: HashSet<_> = set1.union(&set2).cloned().collect();
    let intersection: HashSet<_> = set1.intersection(&set2).cloned().collect();
    let difference: HashSet<_> = set1.difference(&set2).cloned().collect();
    
    println!("Union: {:?}", union);           // {1, 2, 3, 4, 5, 6}
    println!("Intersection: {:?}", intersection); // {3, 4}
    println!("Difference: {:?}", difference);     // {1, 2}
    
    // Check membership
    if set1.contains(&3) {
        println!("Set contains 3");
    }
    
    // Insert returns bool indicating if value was new
    if set1.insert(10) {
        println!("10 was added (wasn't present before)");
    }
}

fn practical_hashset_use() {
    // Track visited items
    let mut visited = HashSet::new();
    let items = vec!["home", "about", "home", "contact", "about"];
    
    for item in items {
        if visited.insert(item) {
            println!("First visit to: {}", item);
        } else {
            println!("Already visited: {}", item);
        }
    }
}
```

---

## When to Use BTreeMap/BTreeSet

Use **BTreeMap/BTreeSet** when you need:
- Keys/values in sorted order
- Range queries (`map.range("a".."c")`)
- Consistent iteration order
- No hash function available for keys

```rust
use std::collections::BTreeMap;

// Example: Leaderboard that needs sorted scores
let mut leaderboard = BTreeMap::new();
leaderboard.insert(95, "Alice");
leaderboard.insert(87, "Bob");
leaderboard.insert(92, "Charlie");

// Iterate in score order (ascending)
for (score, name) in &leaderboard {
    println!("{}: {}", name, score);
}

// Get top 3 scores
let top_scores: Vec<_> = leaderboard
    .iter()
    .rev()  // Reverse for descending order
    .take(3)
    .collect();
```

---

## Common Pitfalls

### HashMap Key Requirements

```rust
use std::collections::HashMap;

// ❌ f64 doesn't implement Eq (NaN issues)
// let mut map: HashMap<f64, String> = HashMap::new();

// ✅ Use ordered wrapper or integer representation
#[derive(Debug, PartialEq, Eq, Hash)]
struct OrderedFloat(i64); // Store as integer representation

impl From<f64> for OrderedFloat {
    fn from(f: f64) -> Self {
        OrderedFloat(f.to_bits() as i64)
    }
}
```

### Borrowing During Iteration

```rust
// ❌ Can't modify while iterating
// for (key, value) in &map {
//     map.insert(new_key, new_value); // Error!
// }

// ✅ Collect changes first, apply after
let changes: Vec<_> = map.iter()
    .filter(|(_, &v)| v > threshold)
    .map(|(k, v)| (format!("new_{}", k), v * 2))
    .collect();

for (key, value) in changes {
    map.insert(key, value);
}
```

---

## Exercise: Student Grade Management System

Create a system that manages student grades using HashMap and HashSet to practice collections operations and the Entry API:

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct GradeBook {
    // Student name -> HashMap of (subject -> grade)
    grades: HashMap<String, HashMap<String, f64>>,
    // Set of all subjects offered
    subjects: HashSet<String>,
}

impl GradeBook {
    fn new() -> Self {
        GradeBook {
            grades: HashMap::new(),
            subjects: HashSet::new(),
        }
    }

    fn add_subject(&mut self, subject: String) {
        // TODO: Add subject to the subjects set
        todo!()
    }

    fn add_grade(&mut self, student: String, subject: String, grade: f64) {
        // TODO: Add a grade for a student in a subject
        // Hints:
        // 1. Add subject to subjects set
        // 2. Use entry() API to get or create the student's grade map
        // 3. Insert the grade for the subject
        todo!()
    }

    fn get_student_average(&self, student: &str) -> Option<f64> {
        // TODO: Calculate average grade for a student across all their subjects
        // Return None if student doesn't exist
        // Hint: Use .values() and iterator methods
        todo!()
    }

    fn get_subject_average(&self, subject: &str) -> Option<f64> {
        // TODO: Calculate average grade for a subject across all students
        // Return None if no students have grades in this subject
        todo!()
    }

    fn get_students_in_subject(&self, subject: &str) -> Vec<&String> {
        // TODO: Return list of students who have a grade in the given subject
        // Hint: Filter students who have this subject in their grade map
        todo!()
    }

    fn get_top_students(&self, n: usize) -> Vec<(String, f64)> {
        // TODO: Return top N students by average grade
        // Format: Vec<(student_name, average_grade)>
        // Hint: Calculate averages, collect into Vec, sort, and take top N
        todo!()
    }

    fn remove_student(&mut self, student: &str) -> bool {
        // TODO: Remove a student and all their grades
        // Return true if student existed, false otherwise
        todo!()
    }

    fn list_subjects(&self) -> Vec<&String> {
        // TODO: Return all subjects as a sorted vector
        todo!()
    }
}

fn main() {
    let mut gradebook = GradeBook::new();

    // Add subjects
    gradebook.add_subject("Math".to_string());
    gradebook.add_subject("English".to_string());
    gradebook.add_subject("Science".to_string());

    // Add grades for students
    gradebook.add_grade("Alice".to_string(), "Math".to_string(), 95.0);
    gradebook.add_grade("Alice".to_string(), "English".to_string(), 87.0);
    gradebook.add_grade("Bob".to_string(), "Math".to_string(), 82.0);
    gradebook.add_grade("Bob".to_string(), "Science".to_string(), 91.0);
    gradebook.add_grade("Charlie".to_string(), "English".to_string(), 78.0);
    gradebook.add_grade("Charlie".to_string(), "Science".to_string(), 85.0);

    // Test the methods
    if let Some(avg) = gradebook.get_student_average("Alice") {
        println!("Alice's average: {:.2}", avg);
    }

    if let Some(avg) = gradebook.get_subject_average("Math") {
        println!("Math class average: {:.2}", avg);
    }

    let math_students = gradebook.get_students_in_subject("Math");
    println!("Students in Math: {:?}", math_students);

    let top_students = gradebook.get_top_students(2);
    println!("Top 2 students: {:?}", top_students);

    println!("All subjects: {:?}", gradebook.list_subjects());
}
```

**Implementation Hints:**

1. **add_grade() method:**
   - Use `self.grades.entry(student).or_insert_with(HashMap::new)`
   - Then insert the grade: `.insert(subject, grade)`

2. **get_student_average():**
   - Use `self.grades.get(student)?` to get the student's grades
   - Use `.values().sum::<f64>() / values.len() as f64`

3. **get_subject_average():**
   - Iterate through all students: `self.grades.iter()`
   - Filter students who have this subject: `filter_map(|(_, grades)| grades.get(subject))`
   - Calculate average from the filtered grades

4. **get_top_students():**
   - Use `map()` to convert students to (name, average) pairs
   - Use `collect::<Vec<_>>()` and `sort_by()` with float comparison
   - Use `take(n)` to get top N

**What you'll learn:**
- HashMap's Entry API for efficient insertions
- HashSet for tracking unique values
- Nested HashMap structures
- Iterator methods for data processing
- Working with Option types from HashMap lookups

---

## Key Takeaways

1. **HashMap<K,V>** for fast key-value lookups with the Entry API for efficiency
2. **HashSet<T>** for unique values and set operations
3. **BTreeMap/BTreeSet** when you need sorted data or range queries
4. **Custom keys** must implement Hash + Eq (or Ord for BTree*)
5. **Can't modify while iterating** - collect changes first
6. **Entry API** prevents redundant lookups and improves performance

**Next Up:** In Chapter 7, we'll explore traits - Rust's powerful system for defining shared behavior and enabling polymorphism without inheritance.