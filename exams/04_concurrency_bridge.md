# Concurrency Bridge Course
**Active Recall Problems for Concurrent Programming Mastery**

*Time Limit: 3 hours | Total Points: 120 | Bridge to: Advanced Concurrent Systems*

---

## Mission: Master Concurrent Rust Programming

**Critical Gap Addressed**: You can write basic async code, but can you design robust concurrent systems? This bridge course teaches you to build reliable, high-performance concurrent applications through practical problems.

**Prerequisites**: Pass `00_intermediate_rust_readiness.md` to prove intermediate Rust skills

**You Will Master:**
- Advanced async programming patterns and coordination  
- Thread-safe data structures and synchronization primitives
- Backpressure handling and flow control
- Concurrent system design and debugging
- Performance measurement of concurrent code

**Next Level**: After this, you'll be ready for `08_lock_free_concurrency_mastery.md` - Lock-Free Concurrency Mastery

---

## Section I: Async Runtime Deep Dive (40 points)

### Problem 1.1: Custom Async Executor (20 points)

Implement a simplified async executor that demonstrates understanding of runtime internals:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::collections::VecDeque;

// Your custom executor
struct SimpleExecutor {
    // What state does an executor need?
    ready_queue: VecDeque<TaskHandle>,
    // How do you track running tasks?
    // How do you handle wakeups?
}

struct TaskHandle {
    // What does a task handle need to store?
}

impl SimpleExecutor {
    fn new() -> Self {
        todo!("Initialize executor")
    }
    
    fn spawn<F>(&mut self, future: F) -> JoinHandle<F::Output>
    where 
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        // How do you spawn a task in your executor?
        // What happens to the Future?
        // How do you create the JoinHandle?
        todo!("Implement task spawning")
    }
    
    fn run(&mut self) {
        // Your executor's main loop
        // How do you:
        // 1. Poll ready tasks
        // 2. Handle task completion
        // 3. Handle wakeups
        // 4. Decide when to exit?
        todo!("Implement executor main loop")
    }
}

// Your JoinHandle implementation
struct JoinHandle<T> {
    // How do you represent a handle to a spawned task?
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // How does JoinHandle coordinate with the executor?
        todo!("Implement JoinHandle polling")
    }
}
```

**Deep Understanding Questions:**
1. **Task Scheduling**: How does your executor decide which task to run next?
2. **Waker Management**: How do you handle wake-ups without losing them?
3. **Memory Management**: How do you prevent memory leaks with unfinished tasks?
4. **Work Stealing**: How would you extend this to a multi-threaded work-stealing executor?

### Problem 1.2: Async Lifetime Puzzle (20 points)

Solve this complex async lifetime scenario:

```rust
// This code has lifetime issues - fix them!
struct AsyncCache<'a> {
    data: HashMap<String, &'a str>,
    pending_requests: HashMap<String, Vec<Waker>>,
}

impl<'a> AsyncCache<'a> {
    async fn get_or_fetch<'b>(&mut self, key: &str) -> Result<&'a str, CacheError>
    where 'b: 'a  // Is this bound correct?
    {
        // Check cache first
        if let Some(cached) = self.data.get(key) {
            return Ok(cached);
        }
        
        // Check if already fetching
        if self.pending_requests.contains_key(key) {
            // Wait for the other fetch to complete
            let (tx, rx) = tokio::sync::oneshot::channel();
            // How do you await the result without a lifetime conflict?
            todo!("Await pending fetch")
        }
        
        // Start new fetch
        let fetched_data = self.fetch_from_source(key).await?;
        
        // Cache the result - LIFETIME ISSUE: How do you store the fetched data?
        self.data.insert(key.to_string(), fetched_data);
        
        Ok(fetched_data)
    }
    
    async fn fetch_from_source(&self, key: &str) -> Result<String, FetchError> {
        // Simulate async fetch
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(format!("fetched_value_for_{}", key))
    }
}
```

**Your Challenges:**
1. **Fix Lifetime Issues**: Make this code compile while preserving the intent
2. **Async Coordination**: Handle multiple concurrent requests for the same key
3. **Memory Management**: Avoid leaking memory in the pending_requests map
4. **Alternative Design**: Redesign with owned data - what are the tradeoffs?

**Advanced Analysis:**
- How does `async fn` affect lifetime inference?
- What's the difference between `&'a str` and `String` in async contexts?
- How do you handle cases where the borrowed data outlives the cache?

---

## Section II: Advanced Synchronization (50 points)

### Problem 2.1: Lock-Free Counter with Memory Ordering (15 points)

Implement a lock-free counter that demonstrates understanding of memory ordering:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct MetricsCounter {
    value: AtomicU64,
    // What additional state do you need for advanced metrics?
}

impl MetricsCounter {
    fn new() -> Self {
        todo!()
    }
    
    fn increment(&self) -> u64 {
        // Simple increment - but what memory ordering?
        // Why did you choose that ordering?
        todo!("Implement with correct memory ordering")
    }
    
    fn add(&self, delta: u64) -> u64 {
        // Add arbitrary value
        todo!()
    }
    
    fn get(&self) -> u64 {
        // Read current value - what ordering guarantees do you need?
        todo!()
    }
    
    // Advanced: implement a compare-and-swap based increment with retry logic
    fn increment_with_backoff(&self) -> u64 {
        // Implement exponential backoff for high contention
        todo!("Implement CAS with backoff")
    }
}

// Advanced: Thread-safe metrics collection
struct AdvancedMetrics {
    counters: Vec<MetricsCounter>,
    // How do you handle per-thread metrics aggregation?
}

impl AdvancedMetrics {
    fn record_event(&self, thread_id: usize, event_type: EventType) {
        // Record event with minimal contention
        // How do you balance accuracy vs performance?
        todo!()
    }
    
    fn get_total(&self) -> u64 {
        // Sum across all threads - what memory ordering?
        todo!()
    }
}
```

**Memory Ordering Analysis:**
1. **Ordering Choice**: Explain your memory ordering choices for each operation
2. **Performance**: How does memory ordering affect performance under contention?
3. **Correctness**: Prove that your implementation maintains consistency
4. **Contention Handling**: How does your backoff strategy perform under high load?

### Problem 2.2: Async Semaphore Implementation (20 points)

Build an async semaphore from scratch using low-level primitives:

```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::task::Waker;

struct AsyncSemaphore {
    // Your semaphore implementation
    permits: Arc<Mutex<SemaphoreState>>,
}

struct SemaphoreState {
    available: usize,
    waiters: VecDeque<Waker>,
    // What other state do you need?
}

impl AsyncSemaphore {
    fn new(permits: usize) -> Self {
        todo!("Initialize semaphore")
    }
    
    async fn acquire(&self) -> SemaphorePermit<'_> {
        // Your async acquire implementation
        // Must handle:
        // 1. Fast path when permits available
        // 2. Slow path with proper wakeup
        // 3. Cancellation safety
        todo!("Implement async acquire")
    }
    
    fn try_acquire(&self) -> Option<SemaphorePermit<'_>> {
        // Non-blocking acquire
        todo!()
    }
    
    fn add_permits(&self, permits: usize) {
        // Add permits and wake waiters
        // How many waiters should you wake?
        // What's the wakeup order?
        todo!("Add permits with proper wakeup")
    }
}

struct SemaphorePermit<'a> {
    semaphore: &'a AsyncSemaphore,
    // What does a permit need to track?
}

impl Drop for SemaphorePermit<'_> {
    fn drop(&mut self) {
        // Release permit and wake next waiter
        todo!("Release permit on drop")
    }
}

// Advanced: Implement a fair semaphore
struct FairAsyncSemaphore {
    // How do you ensure FIFO ordering for waiters?
}
```

**Implementation Challenges:**
1. **Cancellation Safety**: What happens if acquire() is cancelled?
2. **Spurious Wakeups**: How do you handle spurious wakeups correctly?
3. **Fairness**: How do you ensure FIFO ordering of waiters?
4. **Performance**: Minimize lock contention in the fast path

### Problem 2.3: Producer-Consumer with Backpressure (15 points)

Design a channel system with sophisticated backpressure handling:

```rust
// Design requirements:
// - Bounded channel with configurable capacity
// - Backpressure that slows down producers
// - Consumer priority levels
// - Graceful degradation under overload

enum BackpressureStrategy {
    Block,           // Block producer when full
    DropOldest,      // Drop oldest items when full  
    DropNewest,      // Drop newest items when full
    SpillToDisk,     // Overflow to disk storage
}

struct BackpressureChannel<T> {
    // Your channel implementation
}

impl<T> BackpressureChannel<T> {
    fn new(capacity: usize, strategy: BackpressureStrategy) -> (Sender<T>, Receiver<T>) {
        todo!("Create channel with backpressure")
    }
}

struct Sender<T> {
    // What state does sender need?
}

impl<T> Sender<T> {
    async fn send(&self, item: T) -> Result<(), SendError<T>> {
        // Implement send with backpressure
        // How do you apply the backpressure strategy?
        todo!()
    }
    
    async fn send_with_priority(&self, item: T, priority: Priority) -> Result<(), SendError<T>> {
        // Priority-aware sending
        todo!()
    }
    
    fn try_send(&self, item: T) -> Result<(), TrySendError<T>> {
        // Non-blocking send
        todo!()
    }
}

struct Receiver<T> {
    // What state does receiver need?
}

impl<T> Receiver<T> {
    async fn recv(&mut self) -> Option<T> {
        // Receive with proper consumer signaling
        todo!()
    }
    
    async fn recv_batch(&mut self, max_items: usize) -> Vec<T> {
        // Batch receive for efficiency
        todo!()
    }
}
```

**Design Decisions:**
1. **Backpressure Strategy**: When would you choose each strategy?
2. **Priority Handling**: How do you implement fair prioritization?
3. **Spill-to-Disk**: How would you implement overflow to persistent storage?
4. **Performance**: How do you minimize allocation in the hot path?

---

## Section III: Complex Concurrent Systems (60 points)

### Problem 3.1: Async Connection Pool (30 points)

Implement a production-ready async connection pool:

```rust
use std::time::{Duration, Instant};

// Connection pool requirements:
// - Bounded number of connections
// - Connection health checking
// - Connection timeout and retry
// - Load balancing across connections
// - Graceful shutdown

trait AsyncConnection: Send + Sync {
    async fn is_healthy(&self) -> bool;
    async fn execute(&self, query: &str) -> Result<QueryResult, ConnectionError>;
}

struct ConnectionPool<C: AsyncConnection> {
    // Your connection pool implementation
}

struct PoolConfig {
    min_connections: usize,
    max_connections: usize,
    connection_timeout: Duration,
    health_check_interval: Duration,
    max_idle_time: Duration,
    // What other config do you need?
}

impl<C: AsyncConnection + 'static> ConnectionPool<C> {
    async fn new<F>(
        config: PoolConfig,
        connection_factory: F
    ) -> Result<Self, PoolError>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<C, ConnectionError>> + Send>> + Send + Sync + 'static
    {
        // Initialize pool with min_connections
        // Start background health check task
        todo!("Initialize connection pool")
    }
    
    async fn get_connection(&self) -> Result<PooledConnection<C>, PoolError> {
        // Get connection from pool with timeout
        // Handle connection creation if needed
        // Implement load balancing strategy
        todo!("Implement connection acquisition")
    }
    
    async fn execute_query(&self, query: &str) -> Result<QueryResult, PoolError> {
        // High-level interface that manages connections
        todo!()
    }
    
    async fn shutdown(&self) {
        // Gracefully shutdown pool
        // Close all connections
        // Cancel background tasks
        todo!()
    }
}

struct PooledConnection<C> {
    // Wrapper that returns connection to pool on drop
    connection: Option<C>,
    pool: Arc<ConnectionPool<C>>,
    // How do you track connection usage?
}

impl<C: AsyncConnection> Drop for PooledConnection<C> {
    fn drop(&mut self) {
        // Return connection to pool or close if unhealthy
        todo!()
    }
}
```

**Advanced Features:**
1. **Health Checking**: Background task that validates connections
2. **Connection Lifecycle**: Creation, validation, retirement
3. **Load Balancing**: Fair distribution of load across connections
4. **Backpressure**: Handling more requests than available connections
5. **Observability**: Metrics on pool utilization and performance

### Problem 3.2: Distributed Work Scheduler (30 points)

Design a work scheduler that distributes tasks across multiple async workers:

```rust
// Work scheduler that:
// - Distributes work fairly across workers
// - Handles worker failures gracefully  
// - Provides work stealing for load balancing
// - Supports task priorities and dependencies

trait AsyncWorker: Send + Sync {
    async fn process_task(&self, task: Task) -> Result<TaskResult, WorkerError>;
    fn worker_id(&self) -> WorkerId;
    async fn health_check(&self) -> WorkerHealth;
}

struct WorkScheduler<W: AsyncWorker> {
    // Your scheduler implementation
    workers: Vec<Arc<W>>,
    // How do you track work distribution?
    // How do you handle worker failures?
    // How do you implement work stealing?
}

#[derive(Debug)]
struct Task {
    id: TaskId,
    priority: Priority,
    dependencies: Vec<TaskId>,
    payload: TaskPayload,
    retry_policy: RetryPolicy,
    // What other metadata do you need?
}

impl<W: AsyncWorker + 'static> WorkScheduler<W> {
    async fn new(workers: Vec<W>) -> Self {
        // Initialize scheduler
        // Start health monitoring
        // Initialize work queues
        todo!()
    }
    
    async fn submit_task(&self, task: Task) -> Result<TaskHandle, SchedulerError> {
        // Submit task with dependency checking
        // How do you handle priority scheduling?
        // How do you ensure dependencies are satisfied?
        todo!("Implement task submission")
    }
    
    async fn submit_batch(&self, tasks: Vec<Task>) -> Result<Vec<TaskHandle>, SchedulerError> {
        // Batch submission with dependency resolution
        todo!()
    }
    
    async fn wait_for_completion(&self, handle: TaskHandle) -> Result<TaskResult, SchedulerError> {
        // Wait for specific task completion
        todo!()
    }
    
    fn get_scheduler_stats(&self) -> SchedulerStats {
        // Return current scheduler state and metrics
        todo!()
    }
}

// Work stealing implementation
struct WorkStealingQueue<T> {
    // Lock-free work stealing queue
}

impl<T> WorkStealingQueue<T> {
    fn push(&self, item: T) {
        // Add work to local queue
        todo!()
    }
    
    fn pop(&self) -> Option<T> {
        // Take work from local queue
        todo!()
    }
    
    fn steal(&self) -> Option<T> {
        // Steal work from this queue (called by other workers)
        todo!()
    }
}
```

**Complex Requirements:**
1. **Dependency Resolution**: Tasks with complex dependency graphs
2. **Work Stealing**: Load balancing when some workers are idle  
3. **Failure Recovery**: Handling worker crashes and task retry
4. **Priority Scheduling**: Fair scheduling with priority levels
5. **Backpressure**: Preventing unbounded task accumulation

**Advanced Analysis:**
- How do you detect and break circular dependencies?
- What's the memory consistency model for work stealing queues?
- How do you handle partial failures in batch submissions?
- What metrics indicate healthy vs problematic scheduler state?

---

## Grading Rubric

### Expert Level (135-150 points):
- **Deep Runtime Understanding**: Demonstrates thorough knowledge of async runtime internals
- **Advanced Synchronization**: Correctly implements complex lock-free and async primitives
- **Systems Design**: Designs scalable, fault-tolerant concurrent systems
- **Performance Engineering**: Optimizes for real-world concurrent workloads

### Advanced Level (120-134 points):
- **Good Runtime Knowledge**: Understands most async runtime concepts
- **Solid Synchronization**: Implements most synchronization primitives correctly
- **Reasonable Design**: Creates working concurrent systems with minor issues
- **Performance Aware**: Considers performance implications of concurrency choices

### Intermediate Level (105-119 points):
- **Basic Async Understanding**: Can use async/await but limited runtime knowledge
- **Simple Synchronization**: Handles basic synchronization but struggles with advanced patterns
- **Functional Systems**: Builds systems that work but may have race conditions
- **Limited Performance Consideration**: Some awareness of concurrency performance

### Developing (90-104 points):
- **Surface Async Knowledge**: Uses async features without deep understanding
- **Synchronization Issues**: Significant race conditions or deadlocks
- **Fragile Systems**: Systems work in simple cases but fail under load
- **No Performance Optimization**: Little consideration of concurrent performance

### Insufficient (<90 points):
- **Async Confusion**: Fundamental misunderstanding of async programming
- **Dangerous Concurrency**: Race conditions, deadlocks, or data races
- **Broken Systems**: Non-functional concurrent systems
- **Performance Naive**: No understanding of concurrency performance implications

---

## Next Steps

**Expert Level**: Ready for async runtime development and high-performance concurrent systems.

**Advanced Level**: Strong concurrent programming skills. Focus on runtime internals and lock-free programming.

**Intermediate Level**: Good foundation. Practice more complex synchronization patterns and system design.

**Developing**: Work through async fundamentals and basic concurrency patterns.

**Insufficient**: Review homework assignments focusing on async and concurrency basics.

---

*"Concurrency is not parallelism, but both require careful reasoning about time, ordering, and shared state. Master the tools, understand the tradeoffs, and respect the complexity."*