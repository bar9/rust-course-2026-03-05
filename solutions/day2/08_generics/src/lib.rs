// Chapter 8: Generics Exercise Solution

use std::fmt::{Debug, Display};
use std::cmp::Ord;

// =============================================================================
// Exercise: Generic Priority Queue with Constraints
// =============================================================================

// Part 1: Basic generic queue with trait bounds
#[derive(Debug)]
pub struct PriorityQueue<T>
where
    T: Ord + Debug,
{
    items: Vec<T>,
}

impl<T> PriorityQueue<T>
where
    T: Ord + Debug,
{
    pub fn new() -> Self {
        PriorityQueue { items: Vec::new() }
    }

    pub fn enqueue(&mut self, item: T) {
        self.items.push(item);
        self.items.sort();
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// Part 2: Generic trait for items that can be prioritized
pub trait Prioritized {
    type Priority: Ord;

    fn priority(&self) -> Self::Priority;
}

// Part 3: Advanced queue that works with any Prioritized type
pub struct AdvancedQueue<T>
where
    T: Prioritized + Debug,
{
    items: Vec<T>,
}

impl<T> AdvancedQueue<T>
where
    T: Prioritized + Debug,
{
    pub fn new() -> Self {
        AdvancedQueue { items: Vec::new() }
    }

    pub fn enqueue(&mut self, item: T) {
        // Insert item in correct position based on priority using binary search
        let priority = item.priority();
        let insert_pos = self.items
            .binary_search_by_key(&priority, |item| item.priority())
            .unwrap_or_else(|pos| pos);

        self.items.insert(insert_pos, item);
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop()
    }
}

// Part 4: Example types implementing Prioritized
#[derive(Debug, Eq, PartialEq)]
pub struct Task {
    pub name: String,
    pub urgency: u32,
}

impl Prioritized for Task {
    type Priority = u32;

    fn priority(&self) -> Self::Priority {
        self.urgency
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.urgency.cmp(&other.urgency)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Part 5: Generic function with multiple trait bounds
pub fn process_queue<T, Q>(queue: &mut Q, max_items: usize) -> Vec<T>
where
    T: Debug + Clone,
    Q: QueueOperations<T>,
{
    let mut processed = Vec::new();
    for _ in 0..max_items {
        if let Some(item) = queue.dequeue() {
            processed.push(item);
        } else {
            break;
        }
    }
    processed
}

// Part 6: Trait for queue operations (demonstrates trait design)
pub trait QueueOperations<T> {
    fn enqueue(&mut self, item: T);
    fn dequeue(&mut self) -> Option<T>;
    fn len(&self) -> usize;
}

// Implement QueueOperations for PriorityQueue<T>
impl<T> QueueOperations<T> for PriorityQueue<T>
where
    T: Ord + Debug,
{
    fn enqueue(&mut self, item: T) {
        self.enqueue(item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.dequeue()
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_queue_with_numbers() {
        let mut queue = PriorityQueue::new();

        queue.enqueue(5);
        queue.enqueue(1);
        queue.enqueue(10);
        queue.enqueue(3);

        assert_eq!(queue.len(), 4);
        assert_eq!(queue.peek(), Some(&10)); // Highest priority
        assert_eq!(queue.dequeue(), Some(10));
        assert_eq!(queue.dequeue(), Some(5));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), Some(1));
        assert!(queue.is_empty());
    }

    #[test]
    fn test_priority_queue_with_strings() {
        let mut queue = PriorityQueue::new();

        queue.enqueue("zebra".to_string());
        queue.enqueue("apple".to_string());
        queue.enqueue("mango".to_string());

        assert_eq!(queue.dequeue(), Some("zebra".to_string()));
        assert_eq!(queue.dequeue(), Some("mango".to_string()));
        assert_eq!(queue.dequeue(), Some("apple".to_string()));
    }

    #[test]
    fn test_task_creation_and_ordering() {
        let task1 = Task { name: "Low".to_string(), urgency: 1 };
        let task2 = Task { name: "High".to_string(), urgency: 5 };
        let task3 = Task { name: "Medium".to_string(), urgency: 3 };

        assert_eq!(task1.priority(), 1);
        assert_eq!(task2.priority(), 5);
        assert_eq!(task3.priority(), 3);

        // Test ordering
        assert!(task2 > task3);
        assert!(task3 > task1);
    }

    #[test]
    fn test_priority_queue_with_tasks() {
        let mut queue = PriorityQueue::new();

        queue.enqueue(Task { name: "Low".to_string(), urgency: 1 });
        queue.enqueue(Task { name: "High".to_string(), urgency: 5 });
        queue.enqueue(Task { name: "Medium".to_string(), urgency: 3 });

        let first = queue.dequeue().unwrap();
        assert_eq!(first.urgency, 5);
        assert_eq!(first.name, "High");

        let second = queue.dequeue().unwrap();
        assert_eq!(second.urgency, 3);
        assert_eq!(second.name, "Medium");

        let third = queue.dequeue().unwrap();
        assert_eq!(third.urgency, 1);
        assert_eq!(third.name, "Low");
    }

    #[test]
    fn test_advanced_queue_with_prioritized() {
        let mut queue = AdvancedQueue::new();

        queue.enqueue(Task { name: "First".to_string(), urgency: 2 });
        queue.enqueue(Task { name: "Second".to_string(), urgency: 4 });
        queue.enqueue(Task { name: "Third".to_string(), urgency: 1 });

        // Should come out in priority order
        let first = queue.dequeue().unwrap();
        assert_eq!(first.urgency, 4);

        let second = queue.dequeue().unwrap();
        assert_eq!(second.urgency, 2);

        let third = queue.dequeue().unwrap();
        assert_eq!(third.urgency, 1);
    }

    #[test]
    fn test_queue_operations_trait() {
        let mut queue: Box<dyn QueueOperations<i32>> = Box::new(PriorityQueue::new());

        queue.enqueue(3);
        queue.enqueue(1);
        queue.enqueue(2);

        assert_eq!(queue.len(), 3);

        // Should dequeue in priority order
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_process_queue_function() {
        let mut queue = PriorityQueue::new();
        queue.enqueue(5);
        queue.enqueue(2);
        queue.enqueue(8);
        queue.enqueue(1);

        // Process only 2 items
        let processed = process_queue(&mut queue, 2);
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0], 8); // Highest priority first
        assert_eq!(processed[1], 5);

        // Queue should still have 2 items
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_generic_constraints() {
        // Test that we can use different types with the same generic structure
        let mut int_queue = PriorityQueue::new();
        let mut str_queue = PriorityQueue::new();

        int_queue.enqueue(42);
        str_queue.enqueue("hello".to_string());

        assert_eq!(int_queue.dequeue(), Some(42));
        assert_eq!(str_queue.dequeue(), Some("hello".to_string()));
    }

    #[test]
    fn test_empty_queue_operations() {
        let mut queue = PriorityQueue::<i32>::new();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.peek(), None);
        assert_eq!(queue.dequeue(), None);
    }

    #[test]
    fn test_peek_functionality() {
        let mut queue = PriorityQueue::new();

        assert_eq!(queue.peek(), None);

        queue.enqueue(1);
        assert_eq!(queue.peek(), Some(&1));

        queue.enqueue(5);
        assert_eq!(queue.peek(), Some(&5)); // Should be highest

        queue.enqueue(3);
        assert_eq!(queue.peek(), Some(&5)); // Still highest

        // Peek should not remove item
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.dequeue(), Some(5));
        assert_eq!(queue.peek(), Some(&3)); // Next highest
    }

    #[test]
    fn test_multiple_same_priority() {
        let mut queue = PriorityQueue::new();

        queue.enqueue(Task { name: "First".to_string(), urgency: 3 });
        queue.enqueue(Task { name: "Second".to_string(), urgency: 3 });
        queue.enqueue(Task { name: "Third".to_string(), urgency: 3 });

        // All have same priority, but queue should handle them correctly
        assert_eq!(queue.len(), 3);

        let first = queue.dequeue().unwrap();
        let second = queue.dequeue().unwrap();
        let third = queue.dequeue().unwrap();

        // All should have urgency 3
        assert_eq!(first.urgency, 3);
        assert_eq!(second.urgency, 3);
        assert_eq!(third.urgency, 3);
    }

    #[test]
    fn test_advanced_queue_binary_search() {
        let mut queue = AdvancedQueue::new();

        // Add items in random order
        queue.enqueue(Task { name: "Medium".to_string(), urgency: 5 });
        queue.enqueue(Task { name: "Low".to_string(), urgency: 1 });
        queue.enqueue(Task { name: "High".to_string(), urgency: 10 });
        queue.enqueue(Task { name: "Medium2".to_string(), urgency: 5 });

        // Should maintain sorted order internally for efficient operations
        let first = queue.dequeue().unwrap();
        assert_eq!(first.urgency, 10);

        let second = queue.dequeue().unwrap();
        assert_eq!(second.urgency, 5);

        let third = queue.dequeue().unwrap();
        assert_eq!(third.urgency, 5);

        let fourth = queue.dequeue().unwrap();
        assert_eq!(fourth.urgency, 1);
    }
}