# CS 4803/6803: Advanced Rust Systems Programming
## Final Examination - Fall 2025
**Time Limit: 3 hours | Total Points: 200**

**Instructions:** This is a closed-book examination testing advanced Rust concepts. Write compilable code where requested. Explain your reasoning for conceptual questions. Partial credit will be awarded for correct analysis even if implementation is incomplete.

---

## **Section I: Lifetime Analysis & Borrowing (40 points)**

### **Problem 1.1 (15 points)**
Analyze the following code and explain why it fails to compile. Provide a corrected version that achieves the same functionality without changing the function signatures.

```rust
use std::collections::HashMap;

struct DataProcessor<'a> {
    cache: HashMap<String, &'a str>,
    active_keys: Vec<&'a str>,
}

impl<'a> DataProcessor<'a> {
    fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            active_keys: Vec::new(),
        }
    }
    
    fn process_batch(&mut self, data: Vec<String>) -> Vec<&str> {
        let mut results = Vec::new();
        for item in data.iter() {
            let processed = self.transform(item);
            self.cache.insert(item.clone(), processed);
            self.active_keys.push(processed);
            results.push(processed);
        }
        results
    }
    
    fn transform(&self, input: &str) -> &str {
        // Assume this does some complex transformation
        if input.len() > 10 {
            &input[..10]
        } else {
            input
        }
    }
}
```

### **Problem 1.2 (25 points)**
Implement a `StreamingBuffer` that holds references to data with multiple lifetime parameters. The buffer should support:
- Adding data with different lifetimes
- Retrieving the most recently added item of each lifetime
- Maintaining separate counters for each lifetime category

```rust
// Your implementation must satisfy these constraints:
struct StreamingBuffer<'short, 'long> {
    // Design your fields
}

impl<'short, 'long> StreamingBuffer<'short, 'long> 
where 
    'long: 'short  // 'long outlives 'short
{
    fn new() -> Self {
        todo!()
    }
    
    fn add_short_lived(&mut self, data: &'short str) {
        todo!()
    }
    
    fn add_long_lived(&mut self, data: &'long str) {
        todo!()
    }
    
    fn get_latest_short(&self) -> Option<&'short str> {
        todo!()
    }
    
    fn get_latest_long(&self) -> Option<&'long str> {
        todo!()
    }
    
    // This function should return data that can outlive 'short
    fn get_persistent_data(&self) -> Vec<&'long str> {
        todo!()
    }
}
```

---

## **Section II: Advanced Error Handling (50 points)**

### **Problem 2.1 (20 points)**
Design a comprehensive error handling system for a distributed data processing pipeline. Your error types should distinguish between:
- Recoverable network errors (should retry with exponential backoff)
- Data format errors (should skip record and continue)
- System resource errors (should circuit break)
- Configuration errors (should fail fast)

```rust
// Define your error types and implement proper conversions
#[derive(Debug)]
enum ProcessingError {
    // Your variants here
}

// Implement required traits and conversion functions
// Must support: Display, Error, From<io::Error>, From<serde_json::Error>
```

### **Problem 2.2 (30 points)**
Implement an async retry mechanism with the following requirements:
- Exponential backoff with jitter
- Different retry policies based on error type
- Circuit breaker pattern (fail fast after threshold)
- Comprehensive logging of retry attempts
- Graceful cancellation support

```rust
use std::time::Duration;
use tokio::time;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    max_attempts: usize,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    jitter: bool,
    circuit_breaker_threshold: usize,
}

pub struct RetryableService<T> {
    service: T,
    config: RetryConfig,
    // Add fields for circuit breaker state
}

impl<T> RetryableService<T> {
    pub fn new(service: T, config: RetryConfig) -> Self {
        todo!()
    }
    
    // This should retry based on error type and circuit breaker state
    pub async fn execute_with_retry<F, Fut, R, E>(
        &mut self,
        operation: F,
    ) -> Result<R, ProcessingError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<R, E>>,
        E: Into<ProcessingError>,
    {
        todo!()
    }
}
```

---

## **Section III: Async Programming & Concurrency (60 points)**

### **Problem 3.1 (25 points)**
Analyze this async code for correctness and performance issues. Identify at least 3 problems and provide a corrected implementation:

```rust
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::collections::HashMap;

struct DataStore {
    data: Arc<Mutex<HashMap<String, String>>>,
    pending_writes: Arc<Mutex<Vec<String>>>,
}

impl DataStore {
    async fn batch_update(&self, updates: Vec<(String, String)>) -> Result<(), Box<dyn std::error::Error>> {
        for (key, value) in updates {
            let mut data = self.data.lock().await;
            data.insert(key.clone(), value);
            
            let mut pending = self.pending_writes.lock().await;
            pending.push(key);
            
            // Simulate network operation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
    
    async fn concurrent_reads(&self, keys: Vec<String>) -> Vec<Option<String>> {
        let mut results = Vec::new();
        for key in keys {
            let data = self.data.lock().await;
            results.push(data.get(&key).cloned());
        }
        results
    }
}
```

### **Problem 3.2 (35 points)**
Implement a work-stealing executor that processes tasks with different priorities. Requirements:
- Tasks are distributed across multiple worker threads
- High-priority tasks can "steal" execution time from low-priority queues
- Support for both CPU-bound and async tasks
- Graceful shutdown with task completion guarantee
- Comprehensive metrics collection

```rust
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

enum Task {
    Sync(Box<dyn FnOnce() + Send + 'static>),
    Async(Pin<Box<dyn Future<Output = ()> + Send + 'static>>),
}

pub struct WorkStealingExecutor {
    // Design your internal structure
}

impl WorkStealingExecutor {
    pub fn new(num_workers: usize) -> Self {
        todo!()
    }
    
    pub async fn spawn_task(&self, task: Task, priority: Priority) -> Result<(), ExecutorError> {
        todo!()
    }
    
    pub async fn shutdown(self) -> ExecutorMetrics {
        todo!()
    }
    
    // Internal method for work stealing logic
    async fn steal_work(&self, worker_id: usize) -> Option<(Task, Priority)> {
        todo!()
    }
}

#[derive(Debug)]
pub struct ExecutorMetrics {
    tasks_completed: u64,
    average_wait_time_by_priority: HashMap<Priority, Duration>,
    steal_attempts: u64,
    successful_steals: u64,
}
```

---

## **Section IV: Streams & Iterator Chains (35 points)**

### **Problem 4.1 (20 points)**
Create a custom iterator that implements a sliding window with overlap over a stream of data. The iterator should:
- Yield windows of configurable size
- Support configurable step size (overlap control)
- Handle streams that don't divide evenly into windows
- Be memory efficient for large streams
- Support both owned and borrowed data

```rust
struct SlidingWindow<I> {
    // Your implementation
}

impl<I> SlidingWindow<I> 
where 
    I: Iterator,
    I::Item: Clone,
{
    fn new(iter: I, window_size: usize, step_size: usize) -> Self {
        todo!()
    }
}

impl<I> Iterator for SlidingWindow<I> 
where 
    I: Iterator,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;
    
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
    
    // Bonus: Implement size_hint for performance optimization
}

// Demonstrate usage with a complex iterator chain:
fn analyze_data_stream<T>(data: impl Iterator<Item = T>) -> impl Iterator<Item = f64>
where 
    T: Into<f64> + Copy,
{
    // Create a pipeline that:
    // 1. Converts to f64
    // 2. Creates sliding windows of size 5, step 2
    // 3. Calculates standard deviation for each window
    // 4. Filters out windows with std dev < 1.0
    // 5. Returns the coefficient of variation (std_dev / mean)
    todo!()
}
```

### **Problem 4.2 (15 points)**
Implement a parallel stream processor that maintains ordering guarantees:

```rust
use rayon::prelude::*;
use std::cmp::Ordering;

struct OrderedParallelProcessor<T> {
    // Maintain order despite parallel processing
}

impl<T> OrderedParallelProcessor<T> 
where 
    T: Send + Sync,
{
    fn process_parallel<F, R>(
        &self,
        data: Vec<T>,
        processor: F,
    ) -> Vec<R>
    where
        F: Fn(&T) -> R + Send + Sync,
        R: Send,
    {
        // Process items in parallel but return results in original order
        // Handle the case where processing times vary significantly
        todo!()
    }
}
```

---

## **Section V: Integration Challenge (15 points)**

### **Problem 5.1 (15 points)**
Design and implement a type that combines all the concepts above: a `StreamProcessor` that:
- Processes async streams with configurable error handling
- Maintains multiple lifetime parameters for caching
- Uses iterator chains for data transformation
- Implements work stealing for parallel processing

```rust
struct StreamProcessor<'cache, 'config, S> {
    // Combine all concepts in a coherent design
}

// Your implementation must handle:
// 1. Async stream processing with backpressure
// 2. Caching with lifetime management  
// 3. Parallel work distribution
// 4. Comprehensive error recovery
// 5. Iterator-based data pipelines

impl<'cache, 'config, S> StreamProcessor<'cache, 'config, S>
where
    'config: 'cache,
    S: futures::Stream<Item = Result<String, ProcessingError>> + Unpin,
{
    async fn process_stream(&mut self) -> Result<ProcessorSummary, ProcessingError> {
        todo!()
    }
}
```

---

## **Bonus Section: Conceptual Analysis (20 points extra credit)**

### **Question B.1 (10 points)**
Explain the performance implications of different error handling strategies in async Rust code. Compare:
- `Result<T, E>` propagation with `?`
- `panic!` for unrecoverable errors  
- Custom error types with `From` conversions
- Error handling in iterator chains vs. explicit loops

### **Question B.2 (10 points)**
Analyze the memory layout and performance characteristics of the following iterator chain. Identify potential optimizations and explain the zero-cost abstraction principle as it applies here:

```rust
fn complex_pipeline(data: Vec<i32>) -> impl Iterator<Item = String> {
    data.into_iter()
        .filter(|&x| x > 0)
        .map(|x| x * 2)
        .enumerate()
        .filter_map(|(i, x)| if i % 2 == 0 { Some(x) } else { None })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|x| format!("Value: {}", x))
        .take(100)
}
```

---

**End of Examination**

**Grading Rubric:**
- **A (90-100):** Complete implementations with optimal performance, comprehensive error handling, and clear understanding of lifetime semantics
- **B (80-89):** Mostly correct implementations with minor issues in edge cases or suboptimal performance
- **C (70-79):** Basic functionality working but missing error handling or performance considerations
- **D (60-69):** Partial implementations showing understanding of concepts but significant issues
- **F (<60):** Major conceptual misunderstandings or non-functional code

**Good luck! Remember to explain your reasoning and consider edge cases in your implementations.**
