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

mod tests;
