// Chapter 6: Collections Exercise Solution

use std::collections::{HashMap, HashSet};

// =============================================================================
// Exercise: Student Grade Management System
// =============================================================================

#[derive(Debug)]
pub struct GradeBook {
    // Student name -> HashMap of (subject -> grade)
    grades: HashMap<String, HashMap<String, f64>>,
    // Set of all subjects offered
    subjects: HashSet<String>,
}

impl GradeBook {
    pub fn new() -> Self {
        GradeBook {
            grades: HashMap::new(),
            subjects: HashSet::new(),
        }
    }

    pub fn add_subject(&mut self, subject: String) {
        self.subjects.insert(subject);
    }

    pub fn add_grade(&mut self, student: String, subject: String, grade: f64) {
        // Add subject to subjects set
        self.subjects.insert(subject.clone());

        // Use entry() API to get or create the student's grade map
        self.grades
            .entry(student)
            .or_insert_with(HashMap::new)
            .insert(subject, grade);
    }

    pub fn get_student_average(&self, student: &str) -> Option<f64> {
        // Get the student's grades map
        let student_grades = self.grades.get(student)?;

        if student_grades.is_empty() {
            return None;
        }

        // Calculate average using iterator methods
        let total: f64 = student_grades.values().sum();
        let count = student_grades.len() as f64;

        Some(total / count)
    }

    pub fn get_subject_average(&self, subject: &str) -> Option<f64> {
        let grades: Vec<f64> = self
            .grades
            .iter()
            .filter_map(|(_, grades)| grades.get(subject))
            .copied()
            .collect();

        if grades.is_empty() {
            None
        } else {
            let total: f64 = grades.iter().sum();
            Some(total / grades.len() as f64)
        }
    }

    pub fn get_students_in_subject(&self, subject: &str) -> Vec<&String> {
        self.grades
            .iter()
            .filter_map(|(student, grades)| {
                if grades.contains_key(subject) {
                    Some(student)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_top_students(&self, n: usize) -> Vec<(String, f64)> {
        let mut student_averages: Vec<(String, f64)> = self
            .grades
            .keys()
            .filter_map(|student| {
                self.get_student_average(student)
                    .map(|avg| (student.clone(), avg))
            })
            .collect();

        // Sort by average grade in descending order
        student_averages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top N
        student_averages.into_iter().take(n).collect()
    }

    pub fn remove_student(&mut self, student: &str) -> bool {
        self.grades.remove(student).is_some()
    }

    pub fn list_subjects(&self) -> Vec<&String> {
        let mut subjects: Vec<&String> = self.subjects.iter().collect();
        subjects.sort();
        subjects
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gradebook() {
        let gradebook = GradeBook::new();
        assert!(gradebook.grades.is_empty());
        assert!(gradebook.subjects.is_empty());
    }

    #[test]
    fn test_add_subject() {
        let mut gradebook = GradeBook::new();
        gradebook.add_subject("Math".to_string());
        gradebook.add_subject("English".to_string());

        assert!(gradebook.subjects.contains("Math"));
        assert!(gradebook.subjects.contains("English"));
    }

    #[test]
    fn test_add_grade() {
        let mut gradebook = GradeBook::new();
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 95.0);
        gradebook.add_grade("Alice".to_string(), "English".to_string(), 87.0);

        // Should automatically add subjects
        assert!(gradebook.subjects.contains("Math"));
        assert!(gradebook.subjects.contains("English"));

        // Should have student in grades
        assert!(gradebook.grades.contains_key("Alice"));
    }

    #[test]
    fn test_get_student_average() {
        let mut gradebook = GradeBook::new();
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 90.0);
        gradebook.add_grade("Alice".to_string(), "English".to_string(), 80.0);

        let avg = gradebook.get_student_average("Alice");
        assert_eq!(avg, Some(85.0));

        // Non-existent student
        assert_eq!(gradebook.get_student_average("Bob"), None);
    }

    #[test]
    fn test_get_subject_average() {
        let mut gradebook = GradeBook::new();
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 90.0);
        gradebook.add_grade("Bob".to_string(), "Math".to_string(), 80.0);
        gradebook.add_grade("Charlie".to_string(), "Math".to_string(), 70.0);

        let avg = gradebook.get_subject_average("Math");
        assert_eq!(avg, Some(80.0));

        // Non-existent subject
        assert_eq!(gradebook.get_subject_average("Science"), None);
    }

    #[test]
    fn test_get_students_in_subject() {
        let mut gradebook = GradeBook::new();
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 95.0);
        gradebook.add_grade("Bob".to_string(), "Math".to_string(), 82.0);
        gradebook.add_grade("Charlie".to_string(), "English".to_string(), 78.0);

        let math_students = gradebook.get_students_in_subject("Math");
        assert_eq!(math_students.len(), 2);
        assert!(math_students.contains(&&"Alice".to_string()));
        assert!(math_students.contains(&&"Bob".to_string()));

        let english_students = gradebook.get_students_in_subject("English");
        assert_eq!(english_students.len(), 1);
        assert!(english_students.contains(&&"Charlie".to_string()));
    }

    #[test]
    fn test_get_top_students() {
        let mut gradebook = GradeBook::new();
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 95.0);
        gradebook.add_grade("Alice".to_string(), "English".to_string(), 85.0); // avg: 90.0
        gradebook.add_grade("Bob".to_string(), "Math".to_string(), 80.0);
        gradebook.add_grade("Bob".to_string(), "English".to_string(), 90.0); // avg: 85.0
        gradebook.add_grade("Charlie".to_string(), "Math".to_string(), 70.0); // avg: 70.0

        let top_2 = gradebook.get_top_students(2);
        assert_eq!(top_2.len(), 2);
        assert_eq!(top_2[0].0, "Alice");
        assert_eq!(top_2[0].1, 90.0);
        assert_eq!(top_2[1].0, "Bob");
        assert_eq!(top_2[1].1, 85.0);

        let top_5 = gradebook.get_top_students(5); // More than available
        assert_eq!(top_5.len(), 3);
    }

    #[test]
    fn test_remove_student() {
        let mut gradebook = GradeBook::new();
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 95.0);

        assert!(gradebook.remove_student("Alice"));
        assert!(!gradebook.remove_student("Bob")); // Non-existent
        assert!(gradebook.grades.is_empty());
    }

    #[test]
    fn test_list_subjects() {
        let mut gradebook = GradeBook::new();
        gradebook.add_subject("Math".to_string());
        gradebook.add_subject("English".to_string());
        gradebook.add_subject("Science".to_string());

        let subjects = gradebook.list_subjects();
        assert_eq!(subjects.len(), 3);
        // Should be sorted
        assert_eq!(subjects[0], "English");
        assert_eq!(subjects[1], "Math");
        assert_eq!(subjects[2], "Science");
    }

    #[test]
    fn test_entry_api_usage() {
        let mut gradebook = GradeBook::new();

        // Adding multiple grades for same student should work correctly
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 90.0);
        gradebook.add_grade("Alice".to_string(), "English".to_string(), 85.0);
        gradebook.add_grade("Alice".to_string(), "Math".to_string(), 95.0); // Update existing

        let alice_grades = gradebook.grades.get("Alice").unwrap();
        assert_eq!(alice_grades.get("Math"), Some(&95.0)); // Updated value
        assert_eq!(alice_grades.get("English"), Some(&85.0));
    }

    #[test]
    fn test_edge_cases() {
        let mut gradebook = GradeBook::new();

        // Empty gradebook
        assert_eq!(gradebook.get_student_average("Nobody"), None);
        assert_eq!(gradebook.get_subject_average("Nothing"), None);
        assert!(gradebook.get_students_in_subject("Nothing").is_empty());
        assert!(gradebook.get_top_students(5).is_empty());

        // Student with no grades (shouldn't happen with current implementation)
        gradebook
            .grades
            .insert("EmptyStudent".to_string(), HashMap::new());
        assert_eq!(gradebook.get_student_average("EmptyStudent"), None);
    }
}
