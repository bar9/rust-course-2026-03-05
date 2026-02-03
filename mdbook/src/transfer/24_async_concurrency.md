# Chapter 24: Async and Concurrency

## Learning Objectives
- Master thread-based concurrency with Arc, Mutex, and channels
- Understand async/await syntax and the Future trait
- Compare threads vs async for different workloads
- Build concurrent applications with Tokio
- Apply synchronization patterns effectively

## Concurrency in Rust: Two Approaches

Rust provides two main models for concurrent programming, each with distinct advantages:

| Aspect | Threads | Async/Await |
|--------|---------|-------------|
| **Best for** | CPU-intensive work | I/O-bound operations |
| **Memory overhead** | ~2MB per thread | ~2KB per task |
| **Scheduling** | OS kernel | User-space runtime |
| **Blocking operations** | Normal | Must use async variants |
| **Ecosystem maturity** | Complete | Growing rapidly |
| **Learning curve** | Moderate | Steeper initially |

## Part 1: Thread-Based Concurrency

### The Problem with Shared Mutable State

Rust prevents data races at compile time through its ownership system:

```rust
use std::thread;

// This won't compile - Rust prevents the data race
fn broken_example() {
    let mut counter = 0;

    let handle = thread::spawn(|| {
        counter += 1;  // Error: cannot capture mutable reference
    });

    handle.join().unwrap();
}
```

### Arc: Shared Ownership Across Threads

`Arc<T>` (Atomic Reference Counting) enables multiple threads to share ownership of the same data:

```rust
use std::sync::Arc;
use std::thread;

fn share_immutable_data() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            println!("Thread {}: sum = {}", i, data_clone.iter().sum::<i32>());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

Key properties of Arc:
- Reference counting is atomic (thread-safe)
- Cloning is cheap (only increments counter)
- Data is immutable by default
- Memory freed when last reference drops

### Mutex: Safe Mutable Access

`Mutex<T>` provides mutual exclusion for mutable data:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn safe_shared_counter() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
                // Lock automatically released when guard drops
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}
```

### RwLock: Optimizing for Readers

When reads significantly outnumber writes, `RwLock<T>` provides better performance:

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn reader_writer_pattern() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Multiple readers can access simultaneously
    for i in 0..5 {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let guard = data.read().unwrap();
            println!("Reader {}: {:?}", i, *guard);
        }));
    }

    // Single writer waits for all readers
    let data_clone = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        let mut guard = data_clone.write().unwrap();
        guard.push(4);
        println!("Writer: added element");
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}
```

### Channels: Message Passing

Channels avoid shared state entirely through message passing:

```rust
use std::sync::mpsc;
use std::thread;

fn channel_example() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let values = vec!["hello", "from", "thread"];
        for val in values {
            tx.send(val).unwrap();
        }
    });

    for received in rx {
        println!("Got: {}", received);
    }
}

// Multiple producers
fn fan_in_pattern() {
    let (tx, rx) = mpsc::channel();

    for i in 0..3 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            tx_clone.send(format!("Message from thread {}", i)).unwrap();
        });
    }

    drop(tx); // Close original sender

    for msg in rx {
        println!("{}", msg);
    }
}
```

### Synchronization Patterns

#### Worker Pool

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let receiver = Arc::clone(&receiver);
            let worker = thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv();
                match job {
                    Ok(job) => {
                        println!("Worker {} executing job", id);
                        job();
                    }
                    Err(_) => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            });
            workers.push(worker);
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}
```

## Part 2: Async Programming

### Understanding Futures

Futures represent values that will be available at some point:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// Futures are state machines polled to completion
trait SimpleFuture {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

// async/await is syntactic sugar for futures
async fn simple_async() -> i32 {
    42  // Returns impl Future<Output = i32>
}
```

### The Tokio Runtime

Tokio provides a production-ready async runtime:

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Starting");
    sleep(Duration::from_millis(100)).await;
    println!("Done after 100ms");
}

// Alternative runtime configurations
fn runtime_options() {
    // Single-threaded runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Multi-threaded runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        // Your async code here
    });
}
```

### Concurrent Async Operations

Multiple futures can run concurrently without threads:

```rust
use tokio::time::{sleep, Duration};

async fn concurrent_operations() {
    // Sequential - takes 300ms total
    operation("A", 100).await;
    operation("B", 100).await;
    operation("C", 100).await;

    // Concurrent - takes 100ms total
    tokio::join!(
        operation("X", 100),
        operation("Y", 100),
        operation("Z", 100)
    );
}

async fn operation(name: &str, ms: u64) {
    println!("Starting {}", name);
    sleep(Duration::from_millis(ms)).await;
    println!("Completed {}", name);
}
```

### Spawning Async Tasks

Tasks are the async equivalent of threads:

```rust
use tokio::task;

async fn spawn_tasks() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            i * i  // Return value
        });
        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    println!("Results: {:?}", results);
}
```

### Select: Racing Futures

The `select!` macro enables complex control flow:

```rust
use tokio::time::{sleep, Duration, timeout};

async fn select_example() {
    loop {
        tokio::select! {
            _ = sleep(Duration::from_secs(1)) => {
                println!("Timer expired");
            }
            result = async_operation() => {
                println!("Operation completed: {}", result);
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Interrupted");
                break;
            }
        }
    }
}

async fn async_operation() -> String {
    sleep(Duration::from_millis(500)).await;
    "Success".to_string()
}
```

### Async I/O Operations

Async excels at I/O-bound work:

```rust
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn file_io() -> Result<(), Box<dyn std::error::Error>> {
    // Read file
    let mut file = File::open("input.txt").await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    // Write file
    let mut output = File::create("output.txt").await?;
    output.write_all(contents.as_bytes()).await?;

    Ok(())
}

async fn tcp_server() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return,  // Connection closed
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read: {}", e);
                        return;
                    }
                };

                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("Failed to write: {}", e);
                    return;
                }
            }
        });
    }
}
```

### Error Handling in Async Code

Error handling follows the same patterns with async-specific considerations:

```rust
use std::time::Duration;
use tokio::time::timeout;

async fn with_timeout() -> Result<String, Box<dyn std::error::Error>> {
    // Timeout wraps the future
    timeout(Duration::from_secs(5), long_operation()).await?
}

async fn long_operation() -> Result<String, std::io::Error> {
    // Simulated long operation
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok("Completed".to_string())
}

// Retry with exponential backoff
async fn retry_operation<F, Fut, T, E>(
    mut f: F,
    max_attempts: u32,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = Duration::from_millis(100);

    for attempt in 1..=max_attempts {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) if attempt == max_attempts => return Err(e),
            Err(e) => {
                eprintln!("Attempt {} failed: {:?}, retrying...", attempt, e);
                tokio::time::sleep(delay).await;
                delay *= 2;  // Exponential backoff
            }
        }
    }

    unreachable!()
}
```

## Choosing Between Threads and Async

### When to Use Threads

Threads are optimal for:
- **CPU-intensive work**: Computation, data processing, cryptography
- **Parallel algorithms**: Matrix operations, image processing
- **Blocking operations**: Legacy libraries, system calls
- **Simple concurrency**: Independent units of work

Example of CPU-bound work better suited for threads:
```rust
use std::thread;

fn parallel_computation(data: Vec<u64>) -> u64 {
    let chunk_size = data.len() / num_cpus::get();
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            chunk.iter().map(|&x| x * x).sum::<u64>()
        });
        handles.push(handle);
    }

    handles.into_iter()
        .map(|h| h.join().unwrap())
        .sum()
}
```

### When to Use Async

Async is optimal for:
- **I/O-bound work**: Network requests, file operations, databases
- **Many concurrent operations**: Thousands of connections
- **Resource efficiency**: Limited memory environments
- **Coordinated I/O**: Complex workflows with dependencies

Example of I/O-bound work better suited for async:
```rust
async fn fetch_many_urls(urls: Vec<String>) -> Vec<Result<String, reqwest::Error>> {
    let futures = urls.into_iter().map(|url| {
        async move {
            reqwest::get(&url).await?.text().await
        }
    });

    futures::future::join_all(futures).await
}
```

### Hybrid Approaches

Sometimes combining both models is optimal:

```rust
use tokio::task;

async fn hybrid_processing(data: Vec<Data>) -> Vec<Result<Processed, Error>> {
    let mut handles = vec![];

    for chunk in data.chunks(100) {
        let chunk = chunk.to_vec();

        // Spawn blocking task for CPU work
        let handle = task::spawn_blocking(move || {
            process_cpu_intensive(chunk)
        });

        handles.push(handle);
    }

    // Await all CPU tasks
    let mut results = vec![];
    for handle in handles {
        results.extend(handle.await?);
    }

    // Async I/O for results
    store_results_async(results).await
}
```

## Common Pitfalls and Solutions

### Blocking in Async Context

```rust
// BAD: Blocks the async runtime
async fn bad_example() {
    std::thread::sleep(Duration::from_secs(1));  // Blocks executor
}

// GOOD: Use async sleep
async fn good_example() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// GOOD: Move blocking work to dedicated thread
async fn blocking_work() {
    let result = tokio::task::spawn_blocking(|| {
        // CPU-intensive or blocking operation
        expensive_computation()
    }).await.unwrap();
}
```

### Async Mutex vs Sync Mutex

```rust
// Use tokio::sync::Mutex for async contexts
use tokio::sync::Mutex as AsyncMutex;
use std::sync::Mutex as SyncMutex;

async fn async_mutex_example() {
    let data = Arc::new(AsyncMutex::new(vec![]));

    let data_clone = Arc::clone(&data);
    tokio::spawn(async move {
        let mut guard = data_clone.lock().await;  // Async lock
        guard.push(1);
    });
}

// Use std::sync::Mutex only for brief critical sections
fn sync_mutex_in_async() {
    let data = Arc::new(SyncMutex::new(vec![]));

    // OK if lock is held briefly and doesn't cross await points
    {
        let mut guard = data.lock().unwrap();
        guard.push(1);
    }  // Lock released before any await
}
```

## Performance Considerations

### Memory Usage

- **Thread**: ~2MB stack per thread (configurable)
- **Async task**: ~2KB per task
- **Implication**: Can spawn thousands of async tasks vs hundreds of threads

### Context Switching

- **Threads**: Kernel-level context switch (~1-10μs)
- **Async tasks**: User-space task switch (~100ns)
- **Implication**: Much lower overhead for many concurrent operations

### Throughput vs Latency

- **Threads**: Better for consistent latency requirements
- **Async**: Better for maximizing throughput with many connections

## Best Practices

1. **Start simple**: Use threads for CPU work, async for I/O
2. **Avoid blocking**: Never block the async runtime
3. **Choose appropriate synchronization**: Arc+Mutex for threads, channels for both
4. **Profile and measure**: Don't assume, benchmark your specific use case
5. **Handle errors properly**: Both models require careful error handling
6. **Consider the ecosystem**: Check library support for your chosen model

## Summary

Rust provides two powerful concurrency models:

- **Threads**: Best for CPU-intensive work and simple parallelism
- **Async**: Best for I/O-bound work and massive concurrency

Both models provide:
- Memory safety without garbage collection
- Data race prevention at compile time
- Zero-cost abstractions
- Excellent performance

Choose based on your workload characteristics, and don't hesitate to combine both approaches when appropriate. The key is understanding the trade-offs and selecting the right tool for each part of your application.