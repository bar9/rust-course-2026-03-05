# Chapter 5: Smart Pointers
## Advanced Memory Management Beyond Basic Ownership

### Learning Objectives
By the end of this chapter, you'll be able to:
- Use Box<T> for heap allocation and recursive data structures
- Share ownership safely with Rc<T> and Arc<T>
- Implement interior mutability with RefCell<T> and Mutex<T>
- Prevent memory leaks with Weak<T> references
- Choose the right smart pointer for different scenarios
- Understand the performance implications of each smart pointer type

---

## What Are Smart Pointers?

Smart pointers are data structures that act like pointers but have additional metadata and capabilities. Unlike regular references, smart pointers **own** the data they point to.

### Smart Pointers vs Regular References

| Feature | Regular Reference | Smart Pointer |
|---------|------------------|---------------|
| Ownership | Borrows data | Owns data |
| Memory location | Stack or heap | Usually heap |
| Deallocation | Automatic (owner drops) | Automatic (smart pointer drops) |
| Runtime overhead | None | Some (depends on type) |

### Comparison with C++/.NET

| Rust | C++ Equivalent | C#/.NET Equivalent |
|------|-----------------|-------------------|
| `Box<T>` | `std::unique_ptr<T>` | No direct equivalent |
| `Rc<T>` | `std::shared_ptr<T>` | Reference counting GC |
| `Arc<T>` | `std::shared_ptr<T>` (thread-safe) | Thread-safe references |
| `RefCell<T>` | No equivalent | Lock-free interior mutability |
| `Weak<T>` | `std::weak_ptr<T>` | `WeakReference<T>` |

---

## Box<T>: Single Ownership on the Heap

`Box<T>` is the simplest smart pointer - it provides heap allocation with single ownership.

### When to Use Box<T>

1. **Large data**: Move large structs to heap to avoid stack overflow
2. **Recursive types**: Enable recursive data structures
3. **Trait objects**: Store different types behind a common trait
4. **Unsized types**: Store dynamically sized types

### Basic Usage

```rust
fn main() {
    // Heap allocation
    let b = Box::new(5);
    println!("b = {}", b);  // Box implements Deref, so this works
    
    // Large struct - better on heap
    struct LargeStruct {
        data: [u8; 1024 * 1024],  // 1MB
    }
    
    let large = Box::new(LargeStruct { data: [0; 1024 * 1024] });
    // Only pointer stored on stack, data on heap
}
```

### Recursive Data Structures

```rust
// ❌ This won't compile - infinite size
// enum List {
//     Cons(i32, List),
//     Nil,
// }

// ✅ This works - Box has known size
#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn new() -> List {
        List::Nil
    }
    
    fn prepend(self, elem: i32) -> List {
        List::Cons(elem, Box::new(self))
    }
    
    fn len(&self) -> usize {
        match self {
            List::Cons(_, tail) => 1 + tail.len(),
            List::Nil => 0,
        }
    }
}

fn main() {
    let list = List::new()
        .prepend(1)
        .prepend(2)
        .prepend(3);
    
    println!("List: {:?}", list);
    println!("Length: {}", list.len());
}
```

### Box with Trait Objects

```rust
trait Draw {
    fn draw(&self);
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Draw for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

impl Draw for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

fn main() {
    let shapes: Vec<Box<dyn Draw>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 10.0, height: 5.0 }),
    ];
    
    for shape in shapes {
        shape.draw();
    }
}
```

---

## Rc<T>: Reference Counted Single-Threaded Sharing

`Rc<T>` (Reference Counted) enables multiple ownership of the same data in single-threaded scenarios.

### When to Use Rc<T>

- Multiple owners need to read the same data
- Data lifetime is determined by multiple owners
- Single-threaded environment only
- Shared immutable data structures (graphs, trees)

### Basic Usage

```rust
use std::rc::Rc;

fn main() {
    let a = Rc::new(5);
    println!("Reference count: {}", Rc::strong_count(&a));  // 1
    
    let b = Rc::clone(&a);  // Shallow clone, increases ref count
    println!("Reference count: {}", Rc::strong_count(&a));  // 2
    
    {
        let c = Rc::clone(&a);
        println!("Reference count: {}", Rc::strong_count(&a));  // 3
    }  // c dropped here
    
    println!("Reference count: {}", Rc::strong_count(&a));  // 2
}  // a and b dropped here, memory freed when count reaches 0
```

### Sharing Lists

```rust
use std::rc::Rc;

#[derive(Debug)]
enum List {
    Cons(i32, Rc<List>),
    Nil,
}

fn main() {
    let a = Rc::new(List::Cons(5, 
        Rc::new(List::Cons(10, 
        Rc::new(List::Nil)))));
    
    let b = List::Cons(3, Rc::clone(&a));
    let c = List::Cons(4, Rc::clone(&a));
    
    println!("List a: {:?}", a);
    println!("List b: {:?}", b);
    println!("List c: {:?}", c);
    println!("Reference count for a: {}", Rc::strong_count(&a));  // 3
}
```

### Tree with Shared Subtrees

```rust
use std::rc::Rc;

#[derive(Debug)]
struct TreeNode {
    value: i32,
    left: Option<Rc<TreeNode>>,
    right: Option<Rc<TreeNode>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(TreeNode {
            value,
            left: None,
            right: None,
        })
    }
    
    fn with_children(value: i32, left: Option<Rc<TreeNode>>, right: Option<Rc<TreeNode>>) -> Rc<Self> {
        Rc::new(TreeNode { value, left, right })
    }
}

fn main() {
    // Shared subtree
    let shared_subtree = TreeNode::with_children(
        10,
        Some(TreeNode::new(5)),
        Some(TreeNode::new(15)),
    );
    
    // Two different trees sharing the same subtree
    let tree1 = TreeNode::with_children(1, Some(Rc::clone(&shared_subtree)), None);
    let tree2 = TreeNode::with_children(2, Some(Rc::clone(&shared_subtree)), None);
    
    println!("Tree 1: {:?}", tree1);
    println!("Tree 2: {:?}", tree2);
    println!("Shared subtree references: {}", Rc::strong_count(&shared_subtree));  // 3
}
```

---

## RefCell<T>: Interior Mutability

`RefCell<T>` provides "interior mutability" - the ability to mutate data even when there are immutable references to it. The borrowing rules are enforced at runtime instead of compile time.

### When to Use RefCell<T>

- You need to mutate data behind shared references
- You're certain the borrowing rules are followed, but the compiler can't verify it
- Implementing patterns that require mutation through shared references
- Building mock objects for testing

### Basic Usage

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(5);
    
    // Borrow immutably
    {
        let r1 = data.borrow();
        let r2 = data.borrow();
        println!("r1: {}, r2: {}", r1, r2);  // Multiple immutable borrows OK
    }  // Borrows dropped here
    
    // Borrow mutably
    {
        let mut r3 = data.borrow_mut();
        *r3 = 10;
    }  // Mutable borrow dropped here
    
    println!("Final value: {}", data.borrow());
}
```

### Runtime Borrow Checking

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(5);
    
    let r1 = data.borrow();
    // let r2 = data.borrow_mut();  // ❌ Panic! Already borrowed immutably
    
    drop(r1);  // Drop immutable borrow
    let r2 = data.borrow_mut();  // ✅ OK now
    println!("Mutably borrowed: {}", r2);
}
```

### Combining Rc<T> and RefCell<T>

This is a common pattern for shared mutable data:

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(value: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            value,
            children: Vec::new(),
        }))
    }
    
    fn add_child(parent: &Rc<RefCell<Node>>, child: Rc<RefCell<Node>>) {
        parent.borrow_mut().children.push(child);
    }
}

fn main() {
    let root = Node::new(1);
    let child1 = Node::new(2);
    let child2 = Node::new(3);
    
    Node::add_child(&root, child1);
    Node::add_child(&root, child2);
    
    println!("Root: {:?}", root);
    
    // Modify child through shared reference
    root.borrow().children[0].borrow_mut().value = 20;
    
    println!("Modified root: {:?}", root);
}
```

---

## Arc<T>: Atomic Reference Counting for Concurrency

`Arc<T>` (Atomically Reference Counted) is the thread-safe version of `Rc<T>`.

### When to Use Arc<T>

- Multiple threads need to share ownership of data
- Thread-safe reference counting is needed
- Sharing immutable data across thread boundaries

### Basic Usage

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];
    
    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("Thread {}: {:?}", i, data_clone);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Reference count: {}", Arc::strong_count(&data));  // Back to 1
}
```

### Arc<Mutex<T>>: Shared Mutable State

For mutable shared data across threads, combine `Arc<T>` with `Mutex<T>`:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final count: {}", *counter.lock().unwrap());  // Should be 10
}
```

---

## Weak<T>: Breaking Reference Cycles

`Weak<T>` provides a non-owning reference that doesn't affect reference counting. It's used to break reference cycles that would cause memory leaks.

### The Reference Cycle Problem

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,      // Weak reference to parent
    children: RefCell<Vec<Rc<Node>>>, // Strong references to children
}

impl Node {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(Node {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }
    
    fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
        // Set parent weak reference
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        // Add child strong reference
        parent.children.borrow_mut().push(child);
    }
}

fn main() {
    let parent = Node::new(1);
    let child = Node::new(2);
    
    Node::add_child(&parent, child);
    
    // Access parent from child
    let parent_from_child = parent.children.borrow()[0]
        .parent
        .borrow()
        .upgrade();  // Convert weak to strong reference
    
    if let Some(parent_ref) = parent_from_child {
        println!("Child's parent value: {}", parent_ref.value);
    }
    
    println!("Parent strong count: {}", Rc::strong_count(&parent));  // 1
    println!("Parent weak count: {}", Rc::weak_count(&parent));      // 1
}
```

### Observer Pattern with Weak References

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

trait Observer {
    fn notify(&self, message: &str);
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
    
    fn notify_all(&self, message: &str) {
        let mut observers = self.observers.borrow_mut();
        observers.retain(|weak_observer| {
            if let Some(observer) = weak_observer.upgrade() {
                observer.notify(message);
                true  // Keep this observer
            } else {
                false  // Remove dead observer
            }
        });
    }
}

struct ConcreteObserver {
    id: String,
}

impl Observer for ConcreteObserver {
    fn notify(&self, message: &str) {
        println!("Observer {} received: {}", self.id, message);
    }
}

fn main() {
    let subject = Subject::new();
    
    {
        let observer1 = Rc::new(ConcreteObserver { id: "1".to_string() });
        let observer2 = Rc::new(ConcreteObserver { id: "2".to_string() });
        
        subject.subscribe(Rc::downgrade(&observer1));
        subject.subscribe(Rc::downgrade(&observer2));
        
        subject.notify_all("Hello observers!");
    }  // Observers dropped here
    
    subject.notify_all("Anyone still listening?");  // Dead observers cleaned up
}
```

---

## Choosing the Right Smart Pointer

### Decision Tree

```
Do you need shared ownership?
├─ No → Use Box<T>
└─ Yes
   ├─ Single threaded?
   │  ├─ Yes
   │  │  ├─ Need interior mutability? → Rc<RefCell<T>>
   │  │  └─ Just sharing? → Rc<T>
   │  └─ No (multi-threaded)
   │     ├─ Need interior mutability? → Arc<Mutex<T>>
   │     └─ Just sharing? → Arc<T>
   └─ Breaking cycles? → Use Weak<T> in combination
```

### Performance Characteristics

| Smart Pointer | Allocation | Reference Counting | Thread Safety | Interior Mutability |
|---------------|------------|-------------------|---------------|-------------------|
| `Box<T>` | Heap | No | No | No |
| `Rc<T>` | Heap | Yes (non-atomic) | No | No |
| `Arc<T>` | Heap | Yes (atomic) | Yes | No |
| `RefCell<T>` | Stack/Heap | No | No | Yes (runtime) |
| `Weak<T>` | No allocation | Weak counting | Depends on target | No |

### Common Patterns

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

// Pattern 1: Immutable shared data (single-threaded)
fn pattern1() {
    let shared_data = Rc::new(vec![1, 2, 3, 4, 5]);
    let clone1 = Rc::clone(&shared_data);
    let clone2 = Rc::clone(&shared_data);
    // Multiple readers, no writers
}

// Pattern 2: Mutable shared data (single-threaded)
fn pattern2() {
    let shared_data = Rc::new(RefCell::new(vec![1, 2, 3]));
    shared_data.borrow_mut().push(4);
    let len = shared_data.borrow().len();
}

// Pattern 3: Immutable shared data (multi-threaded)
fn pattern3() {
    let shared_data = Arc::new(vec![1, 2, 3, 4, 5]);
    let clone = Arc::clone(&shared_data);
    std::thread::spawn(move || {
        println!("{:?}", clone);
    });
}

// Pattern 4: Mutable shared data (multi-threaded)
fn pattern4() {
    let shared_data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let clone = Arc::clone(&shared_data);
    std::thread::spawn(move || {
        clone.lock().unwrap().push(4);
    });
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Reference Cycles with Rc<T>

```rust
use std::rc::Rc;
use std::cell::RefCell;

// ❌ This creates a reference cycle and memory leak
#[derive(Debug)]
struct BadNode {
    children: RefCell<Vec<Rc<BadNode>>>,
    parent: RefCell<Option<Rc<BadNode>>>,  // Strong reference = cycle!
}

// ✅ Use Weak for parent references
#[derive(Debug)]
struct GoodNode {
    children: RefCell<Vec<Rc<GoodNode>>>,
    parent: RefCell<Option<std::rc::Weak<GoodNode>>>,  // Weak reference
}
```

### Pitfall 2: RefCell Runtime Panics

```rust
use std::cell::RefCell;

fn dangerous_refcell() {
    let data = RefCell::new(5);
    
    let _r1 = data.borrow();
    let _r2 = data.borrow_mut();  // ❌ Panics at runtime!
}

// ✅ Safe RefCell usage
fn safe_refcell() {
    let data = RefCell::new(5);
    
    {
        let r1 = data.borrow();
        println!("Value: {}", r1);
    }  // r1 dropped
    
    {
        let mut r2 = data.borrow_mut();
        *r2 = 10;
    }  // r2 dropped
}
```

### Pitfall 3: Unnecessary Arc for Single-Threaded Code

```rust
// ❌ Unnecessary atomic operations
use std::sync::Arc;
fn single_threaded_sharing() {
    let data = Arc::new(vec![1, 2, 3]);  // Atomic ref counting overhead
    // ... single-threaded code only
}

// ✅ Use Rc for single-threaded sharing
use std::rc::Rc;
fn single_threaded_sharing_optimized() {
    let data = Rc::new(vec![1, 2, 3]);  // Faster non-atomic ref counting
    // ... single-threaded code only
}
```

---

## Key Takeaways

1. **Box<T>** for single ownership heap allocation and recursive types
2. **Rc<T>** for shared ownership in single-threaded contexts  
3. **RefCell<T>** for interior mutability with runtime borrow checking
4. **Arc<T>** for shared ownership across threads
5. **Weak<T>** to break reference cycles and avoid memory leaks
6. **Combine smart pointers** for complex sharing patterns (e.g., `Rc<RefCell<T>>`)
7. **Choose based on threading and mutability needs**

---

## Exercises

### Exercise 1: Binary Tree with Parent References

Implement a binary tree where nodes can access both children and parents without creating reference cycles:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct TreeNode {
    value: i32,
    left: Option<Rc<RefCell<TreeNode>>>,
    right: Option<Rc<RefCell<TreeNode>>>,
    parent: RefCell<Weak<RefCell<TreeNode>>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<RefCell<Self>> {
        // Implement
    }
    
    fn add_left_child(node: &Rc<RefCell<TreeNode>>, value: i32) {
        // Implement: Add left child and set its parent reference
    }
    
    fn add_right_child(node: &Rc<RefCell<TreeNode>>, value: i32) {
        // Implement: Add right child and set its parent reference
    }
    
    fn get_parent_value(&self) -> Option<i32> {
        // Implement: Get parent's value if it exists
    }
    
    fn find_root(&self) -> Option<Rc<RefCell<TreeNode>>> {
        // Implement: Traverse up to find root node
    }
}

fn main() {
    let root = TreeNode::new(1);
    TreeNode::add_left_child(&root, 2);
    TreeNode::add_right_child(&root, 3);
    
    let left_child = root.borrow().left.as_ref().unwrap().clone();
    TreeNode::add_left_child(&left_child, 4);
    
    // Test parent access
    let grandchild = left_child.borrow().left.as_ref().unwrap().clone();
    println!("Grandchild's parent: {:?}", grandchild.borrow().get_parent_value());
    
    // Test root finding
    if let Some(found_root) = grandchild.borrow().find_root() {
        println!("Root value: {}", found_root.borrow().value);
    }
}
```

### Exercise 2: Thread-Safe Cache

Implement a thread-safe cache using Arc and Mutex:

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

struct Cache<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Cache<K, V> 
where
    K: Clone + Eq + std::hash::Hash + Send + 'static,
    V: Clone + Send + 'static,
{
    fn new() -> Self {
        // Implement
    }
    
    fn get(&self, key: &K) -> Option<V> {
        // Implement: Get value from cache
    }
    
    fn set(&self, key: K, value: V) {
        // Implement: Set value in cache
    }
    
    fn size(&self) -> usize {
        // Implement: Get cache size
    }
}

impl<K, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        // Implement: Clone should share the same underlying data
        Cache {
            data: Arc::clone(&self.data),
        }
    }
}

fn main() {
    let cache = Cache::new();
    let mut handles = vec![];
    
    // Spawn multiple threads that use the cache
    for i in 0..5 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            // Set some values
            cache_clone.set(format!("key{}", i), i * 10);
            
            // Get some values
            if let Some(value) = cache_clone.get(&format!("key{}", i)) {
                println!("Thread {}: got value {}", i, value);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final cache size: {}", cache.size());
}
```

### Exercise 3: Observer Pattern with Automatic Cleanup

Extend the observer pattern to automatically clean up observers and provide subscription management:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

trait Observer {
    fn update(&self, data: &str);
    fn id(&self) -> &str;
}

struct Subject {
    observers: RefCell<Vec<Weak<dyn Observer>>>,
}

impl Subject {
    fn new() -> Self {
        // Implement
    }
    
    fn subscribe(&self, observer: Weak<dyn Observer>) {
        // Implement: Add observer
    }
    
    fn unsubscribe(&self, observer_id: &str) {
        // Implement: Remove observer by ID
    }
    
    fn notify(&self, data: &str) {
        // Implement: Notify all observers, cleaning up dead ones
    }
    
    fn observer_count(&self) -> usize {
        // Implement: Count living observers
    }
}

struct ConcreteObserver {
    id: String,
}

impl ConcreteObserver {
    fn new(id: String) -> Rc<Self> {
        Rc::new(ConcreteObserver { id })
    }
}

impl Observer for ConcreteObserver {
    fn update(&self, data: &str) {
        println!("Observer {} received: {}", self.id, data);
    }
    
    fn id(&self) -> &str {
        &self.id
    }
}

fn main() {
    let subject = Subject::new();
    
    let observer1 = ConcreteObserver::new("obs1".to_string());
    let observer2 = ConcreteObserver::new("obs2".to_string());
    
    subject.subscribe(Rc::downgrade(&observer1));
    subject.subscribe(Rc::downgrade(&observer2));
    
    subject.notify("First message");
    println!("Observer count: {}", subject.observer_count());
    
    // Drop one observer
    drop(observer1);
    
    subject.notify("Second message");
    println!("Observer count after cleanup: {}", subject.observer_count());
    
    subject.unsubscribe("obs2");
    subject.notify("Third message");
    println!("Final observer count: {}", subject.observer_count());
}
```

---

## Additional Resources

- **[Rust Container Cheat Sheet](https://cs140e.sergio.bz/notes/lec3/cheat-sheet.pdf)** by Ralph Levien - An excellent visual reference for Rust containers and smart pointers, including Vec, String, Box, Rc, Arc, RefCell, and more. Perfect for quick lookups and comparisons.
- [The Rust Book - Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [Rust by Example - Smart Pointers](https://doc.rust-lang.org/rust-by-example/std/box.html)
- [RefCell and Interior Mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)

**Next Up:** In Day 2, we'll explore collections, traits, and generics - the tools that make Rust code both safe and expressive.