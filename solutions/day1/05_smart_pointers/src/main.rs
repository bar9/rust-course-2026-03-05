// Day 1, Chapter 5: Smart Pointers - Exercise Solutions

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

// Exercise 1: Binary Tree with Parent References
#[derive(Debug)]
struct TreeNode {
    value: i32,
    left: Option<Rc<RefCell<TreeNode>>>,
    right: Option<Rc<RefCell<TreeNode>>>,
    parent: RefCell<Weak<RefCell<TreeNode>>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TreeNode {
            value,
            left: None,
            right: None,
            parent: RefCell::new(Weak::new()),
        }))
    }
    
    fn add_left_child(node: &Rc<RefCell<TreeNode>>, value: i32) {
        let child = TreeNode::new(value);
        *child.borrow().parent.borrow_mut() = Rc::downgrade(node);
        node.borrow_mut().left = Some(child);
    }
    
    fn add_right_child(node: &Rc<RefCell<TreeNode>>, value: i32) {
        let child = TreeNode::new(value);
        *child.borrow().parent.borrow_mut() = Rc::downgrade(node);
        node.borrow_mut().right = Some(child);
    }
    
    fn get_parent_value(&self) -> Option<i32> {
        self.parent
            .borrow()
            .upgrade()
            .map(|parent| parent.borrow().value)
    }
    
    fn find_root(&self) -> Option<Rc<RefCell<TreeNode>>> {
        let parent_weak = self.parent.borrow().clone();
        
        if let Some(parent) = parent_weak.upgrade() {
            // Recursively find the root
            parent.borrow().find_root()
        } else {
            // No parent means this node needs to be wrapped in Rc<RefCell>
            // In practice, we'd need a reference to self as Rc<RefCell>
            // For this exercise, we return None when called on non-root
            None
        }
    }
    
    // Helper method to find root from an Rc<RefCell<TreeNode>>
    fn find_root_from_rc(node: &Rc<RefCell<TreeNode>>) -> Rc<RefCell<TreeNode>> {
        let parent_weak = node.borrow().parent.borrow().clone();
        
        if let Some(parent) = parent_weak.upgrade() {
            TreeNode::find_root_from_rc(&parent)
        } else {
            Rc::clone(node)
        }
    }
}

// Exercise 2: Thread-Safe Cache
struct Cache<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + 'static,
    V: Clone + Send + 'static,
{
    fn new() -> Self {
        Cache {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn get(&self, key: &K) -> Option<V> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }
    
    fn set(&self, key: K, value: V) {
        let mut data = self.data.lock().unwrap();
        data.insert(key, value);
    }
    
    fn size(&self) -> usize {
        let data = self.data.lock().unwrap();
        data.len()
    }
    
    fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        data.clear();
    }
    
    fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.lock().unwrap();
        data.remove(key)
    }
}

impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Cache {
            data: Arc::clone(&self.data),
        }
    }
}

// Exercise 3: Observer Pattern with Automatic Cleanup
trait Observer: std::fmt::Debug {
    fn update(&self, data: &str);
    fn id(&self) -> &str;
}

struct Subject {
    observers: RefCell<Vec<Weak<dyn Observer>>>,
}

impl Subject {
    fn new() -> Self {
        Subject {
            observers: RefCell::new(Vec::new()),
        }
    }
    
    fn subscribe(&self, observer: Weak<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }
    
    fn unsubscribe(&self, observer_id: &str) {
        self.observers.borrow_mut().retain(|weak_observer| {
            if let Some(observer) = weak_observer.upgrade() {
                observer.id() != observer_id
            } else {
                false
            }
        });
    }
    
    fn notify(&self, data: &str) {
        let mut observers = self.observers.borrow_mut();
        let mut i = 0;
        
        while i < observers.len() {
            if let Some(observer) = observers[i].upgrade() {
                observer.update(data);
                i += 1;
            } else {
                // Remove dead observer
                observers.remove(i);
            }
        }
    }
    
    fn observer_count(&self) -> usize {
        let mut count = 0;
        let observers = self.observers.borrow();
        
        for weak_observer in observers.iter() {
            if weak_observer.upgrade().is_some() {
                count += 1;
            }
        }
        
        count
    }
}

#[derive(Debug)]
struct ConcreteObserver {
    id: String,
    data: RefCell<Vec<String>>,
}

impl ConcreteObserver {
    fn new(id: String) -> Rc<Self> {
        Rc::new(ConcreteObserver {
            id,
            data: RefCell::new(Vec::new()),
        })
    }
    
    fn get_received_data(&self) -> Vec<String> {
        self.data.borrow().clone()
    }
}

impl Observer for ConcreteObserver {
    fn update(&self, data: &str) {
        println!("Observer {} received: {}", self.id, data);
        self.data.borrow_mut().push(data.to_string());
    }
    
    fn id(&self) -> &str {
        &self.id
    }
}

// Additional Example: Shared Graph Structure
#[derive(Debug)]
struct GraphNode {
    id: String,
    neighbors: RefCell<Vec<Rc<RefCell<GraphNode>>>>,
}

impl GraphNode {
    fn new(id: String) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(GraphNode {
            id,
            neighbors: RefCell::new(Vec::new()),
        }))
    }
    
    fn add_edge(from: &Rc<RefCell<GraphNode>>, to: &Rc<RefCell<GraphNode>>) {
        from.borrow().neighbors.borrow_mut().push(Rc::clone(to));
        to.borrow().neighbors.borrow_mut().push(Rc::clone(from));
    }
    
    fn neighbor_count(&self) -> usize {
        self.neighbors.borrow().len()
    }
    
    fn get_neighbors(&self) -> Vec<String> {
        self.neighbors
            .borrow()
            .iter()
            .map(|n| n.borrow().id.clone())
            .collect()
    }
}

fn main() {
    println!("=== Exercise 1: Binary Tree with Parent References ===");
    exercise1_tree_demo();
    println!();
    
    println!("=== Exercise 2: Thread-Safe Cache ===");
    exercise2_cache_demo();
    println!();
    
    println!("=== Exercise 3: Observer Pattern ===");
    exercise3_observer_demo();
    println!();
    
    println!("=== Additional: Graph Structure ===");
    graph_demo();
}

fn exercise1_tree_demo() {
    // Build tree:
    //       1
    //      / \
    //     2   3
    //    / \
    //   4   5
    
    let root = TreeNode::new(1);
    TreeNode::add_left_child(&root, 2);
    TreeNode::add_right_child(&root, 3);
    
    let left_child = root.borrow().left.as_ref().unwrap().clone();
    TreeNode::add_left_child(&left_child, 4);
    TreeNode::add_right_child(&left_child, 5);
    
    // Test parent access
    let grandchild = left_child.borrow().left.as_ref().unwrap().clone();
    
    println!("Tree structure created:");
    println!("Root value: {}", root.borrow().value);
    
    if let Some(parent_value) = grandchild.borrow().get_parent_value() {
        println!("Grandchild's parent value: {}", parent_value);
    }
    
    // Test root finding
    let found_root = TreeNode::find_root_from_rc(&grandchild);
    println!("Root found from grandchild: {}", found_root.borrow().value);
    
    // Check reference counts
    println!("Root strong count: {}", Rc::strong_count(&root));
    println!("Root weak count: {}", Rc::weak_count(&root));
    
    // Traverse tree
    println!("\nTree traversal:");
    print_tree(&root, 0);
}

fn print_tree(node: &Rc<RefCell<TreeNode>>, level: usize) {
    let indent = "  ".repeat(level);
    println!("{}Node {}", indent, node.borrow().value);
    
    if let Some(ref left) = node.borrow().left {
        print_tree(left, level + 1);
    }
    
    if let Some(ref right) = node.borrow().right {
        print_tree(right, level + 1);
    }
}

fn exercise2_cache_demo() {
    let cache = Cache::new();
    let mut handles = vec![];
    
    // Spawn multiple threads that use the cache
    for i in 0..5 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            // Set some values
            for j in 0..3 {
                let key = format!("thread_{}_key_{}", i, j);
                let value = i * 10 + j;
                cache_clone.set(key.clone(), value);
                println!("Thread {} set {} = {}", i, key, value);
            }
            
            // Get some values
            let key = format!("thread_{}_key_1", i);
            if let Some(value) = cache_clone.get(&key) {
                println!("Thread {} got {} = {}", i, key, value);
            }
            
            // Try to get from another thread's cache
            let other_key = format!("thread_{}_key_0", (i + 1) % 5);
            thread::sleep(std::time::Duration::from_millis(50)); // Wait a bit
            if let Some(value) = cache_clone.get(&other_key) {
                println!("Thread {} got {} = {} (from another thread)", i, other_key, value);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("\nFinal cache size: {}", cache.size());
    
    // Show some cached values
    println!("Sample cached values:");
    for i in 0..3 {
        let key = format!("thread_0_key_{}", i);
        if let Some(value) = cache.get(&key) {
            println!("  {} = {}", key, value);
        }
    }
}

fn exercise3_observer_demo() {
    let subject = Subject::new();
    
    // Create observers
    let observer1 = ConcreteObserver::new("obs1".to_string());
    let observer2 = ConcreteObserver::new("obs2".to_string());
    let observer3 = ConcreteObserver::new("obs3".to_string());
    
    // Subscribe observers
    subject.subscribe(Rc::downgrade(&observer1) as Weak<dyn Observer>);
    subject.subscribe(Rc::downgrade(&observer2) as Weak<dyn Observer>);
    subject.subscribe(Rc::downgrade(&observer3) as Weak<dyn Observer>);
    
    println!("Initial observer count: {}", subject.observer_count());
    
    // Send first message
    subject.notify("First message");
    
    // Drop one observer
    println!("\nDropping observer2...");
    drop(observer2);
    
    // Send second message
    subject.notify("Second message");
    println!("Observer count after drop: {}", subject.observer_count());
    
    // Unsubscribe observer3
    println!("\nUnsubscribing observer3...");
    subject.unsubscribe("obs3");
    
    // Send third message
    subject.notify("Third message");
    println!("Final observer count: {}", subject.observer_count());
    
    // Check what observer1 received
    println!("\nObserver1 received messages:");
    for msg in observer1.get_received_data() {
        println!("  - {}", msg);
    }
}

fn graph_demo() {
    // Create a simple graph
    //     A --- B
    //     |     |
    //     C --- D
    
    let node_a = GraphNode::new("A".to_string());
    let node_b = GraphNode::new("B".to_string());
    let node_c = GraphNode::new("C".to_string());
    let node_d = GraphNode::new("D".to_string());
    
    GraphNode::add_edge(&node_a, &node_b);
    GraphNode::add_edge(&node_a, &node_c);
    GraphNode::add_edge(&node_b, &node_d);
    GraphNode::add_edge(&node_c, &node_d);
    
    println!("Graph structure created");
    println!("Node A neighbors: {:?}", node_a.borrow().get_neighbors());
    println!("Node B neighbors: {:?}", node_b.borrow().get_neighbors());
    println!("Node C neighbors: {:?}", node_c.borrow().get_neighbors());
    println!("Node D neighbors: {:?}", node_d.borrow().get_neighbors());
    
    println!("\nReference counts:");
    println!("Node A: strong={}, weak={}", 
        Rc::strong_count(&node_a), Rc::weak_count(&node_a));
    println!("Node D: strong={}, weak={}", 
        Rc::strong_count(&node_d), Rc::weak_count(&node_d));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tree_parent_references() {
        let root = TreeNode::new(10);
        TreeNode::add_left_child(&root, 5);
        TreeNode::add_right_child(&root, 15);
        
        let left = root.borrow().left.as_ref().unwrap().clone();
        assert_eq!(left.borrow().get_parent_value(), Some(10));
        
        let right = root.borrow().right.as_ref().unwrap().clone();
        assert_eq!(right.borrow().get_parent_value(), Some(10));
    }
    
    #[test]
    fn test_cache_operations() {
        let cache: Cache<String, i32> = Cache::new();
        
        cache.set("key1".to_string(), 100);
        assert_eq!(cache.get(&"key1".to_string()), Some(100));
        assert_eq!(cache.size(), 1);
        
        cache.set("key2".to_string(), 200);
        assert_eq!(cache.size(), 2);
        
        assert_eq!(cache.remove(&"key1".to_string()), Some(100));
        assert_eq!(cache.size(), 1);
        
        cache.clear();
        assert_eq!(cache.size(), 0);
    }
    
    #[test]
    fn test_observer_cleanup() {
        let subject = Subject::new();
        
        {
            let obs1 = ConcreteObserver::new("test1".to_string());
            subject.subscribe(Rc::downgrade(&obs1) as Weak<dyn Observer>);
            assert_eq!(subject.observer_count(), 1);
        }
        // obs1 dropped
        
        subject.notify("test");
        assert_eq!(subject.observer_count(), 0);
    }
}