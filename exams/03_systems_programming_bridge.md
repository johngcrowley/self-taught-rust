# Systems Programming Bridge Course
**Active Recall Problems for Production Systems Engineering**

*Time Limit: 4 hours | Total Points: 160 | Bridge to: Advanced Low-Level Systems Programming*

---

## Mission: Build Observable, Reliable Systems

**Critical Gap Addressed**: You can write Rust applications, but can you build production-ready systems? This bridge course teaches you to create observable, reliable systems that work in the real world through hands-on problems.

**Prerequisites**: Pass `00_intermediate_rust_readiness.md` to prove intermediate Rust skills

**You Will Master:**
- Building observable systems with metrics, logging, and distributed tracing
- Network programming and protocol implementation with proper error handling
- File system and storage programming with reliability patterns  
- Process management and resource optimization
- System call usage and performance optimization
- Production deployment patterns and system reliability

**Next Level**: After this, you'll be ready for `07_low_level_systems_mastery.md` - Advanced Low-Level Systems Programming

---

## The Systems Engineering Philosophy

**Rule 1**: Systems must be observable - you can't debug what you can't see  
**Rule 2**: Every system will fail - design for graceful degradation  
**Rule 3**: Performance matters, but reliability matters more  
**Rule 4**: Measure everything that matters, alert on what's broken

**Production Readiness Checklist**: For every system component, you must demonstrate:
1. **Observability**: Metrics, logs, and traces for debugging
2. **Error Handling**: Graceful handling of all failure modes  
3. **Resource Management**: Bounded memory, CPU, and file descriptor usage
4. **Performance**: Meets latency and throughput requirements under load
5. **Reliability**: Continues operating during partial failures

---

## Section I: Distributed Systems Architecture (50 points)

### Problem 1.1: The Split-Brain Reconciliation Challenge (25 points)

Your distributed key-value store has experienced a network partition. Two nodes now have conflicting data:

**Node A State:**
```
key1: "value_a1" (timestamp: 1000, node_id: A)
key2: "value_a2" (timestamp: 1500, node_id: A)
key3: "deleted"   (timestamp: 2000, node_id: A)
```

**Node B State:**
```
key1: "value_b1" (timestamp: 1200, node_id: B)  
key2: "value_a2" (timestamp: 1500, node_id: A)  // Same as A
key3: "value_b3" (timestamp: 1800, node_id: B)  // Conflicts with A's delete
```

**Your Challenge:**
Design a reconciliation algorithm that resolves these conflicts deterministically.

```rust
#[derive(Debug, Clone)]
struct VersionedValue {
    value: Option<String>, // None represents deletion
    timestamp: u64,
    node_id: NodeId,
    vector_clock: VectorClock,
}

#[derive(Debug)]
struct ConflictResolution {
    winning_value: Option<String>,
    resolution_reason: ResolutionReason,
    requires_manual_intervention: bool,
}

// Your implementation:
impl VersionedValue {
    fn resolve_conflict(&self, other: &VersionedValue) -> ConflictResolution {
        // Design a conflict resolution strategy that handles:
        // 1. Timestamp conflicts (what if clocks are skewed?)
        // 2. Delete vs update conflicts
        // 3. Concurrent updates on different nodes
        // 4. Cases requiring manual intervention
        
        todo!("Implement sophisticated conflict resolution")
    }
    
    fn merge_vector_clocks(&self, other: &VersionedValue) -> VectorClock {
        // How do you merge vector clocks to detect causality?
        todo!("Implement vector clock merging")
    }
}

// Design the reconciliation engine
struct ReconciliationEngine {
    // What state does the reconciliation engine need?
}

impl ReconciliationEngine {
    async fn reconcile_partition(
        &mut self,
        local_state: HashMap<String, VersionedValue>,
        remote_state: HashMap<String, VersionedValue>,
    ) -> Result<HashMap<String, VersionedValue>, ReconciliationError> {
        // Your reconciliation algorithm here
        // Must handle: conflicts, tombstones, partial failures
        todo!("Implement full reconciliation")
    }
}
```

**Analysis Required:**
1. **Conflict Resolution Strategy**: Explain your algorithm's behavior for each conflict type
2. **Correctness Guarantees**: What invariants does your system maintain?
3. **CAP Theorem Tradeoffs**: How does your design choose between consistency and availability?
4. **Edge Cases**: Handle clock skew, network delays, and byzantine failures

### Problem 1.2: Backpressure Propagation (25 points)

Design a streaming system where backpressure propagates correctly through multiple async stages:

```rust
// Your pipeline: Producer → Transformer → Consumer
//                  1000/s  →    100/s   →    50/s
// The consumer is the bottleneck - how do you prevent memory explosion?

trait AsyncStage {
    type Input;
    type Output;
    
    async fn process(&mut self, input: Self::Input) -> Result<Self::Output, StageError>;
}

struct Pipeline<P, T, C> 
where 
    P: AsyncStage,
    T: AsyncStage<Input = P::Output>,
    C: AsyncStage<Input = T::Output>,
{
    producer: P,
    transformer: T,
    consumer: C,
    // What buffering/backpressure state do you need?
}

impl<P, T, C> Pipeline<P, T, C> {
    async fn run(&mut self) -> Result<(), PipelineError> {
        // Design the execution loop that:
        // 1. Prevents unbounded memory growth
        // 2. Maximizes throughput within memory constraints
        // 3. Handles stage failures gracefully
        // 4. Provides observability into backpressure state
        
        todo!("Implement backpressure-aware pipeline")
    }
}
```

**Design Requirements:**
1. **Bounded Memory**: System memory usage must not grow indefinitely
2. **Optimal Throughput**: Achieve maximum throughput given the slowest stage
3. **Fair Scheduling**: Don't starve fast stages when slow stages catch up
4. **Failure Recovery**: Handle stage failures without losing in-flight data

---

## Section II: Performance Engineering (50 points)

### Problem 2.1: The Zero-Copy Challenge (25 points)

Implement a high-performance HTTP request parser that minimizes allocations:

```rust
// Parse HTTP requests with zero-copy where possible
struct HttpRequest<'a> {
    method: &'a str,
    path: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    body: &'a [u8],
}

struct HttpParser {
    // What state does your parser need to maintain?
}

impl HttpParser {
    fn parse_request<'a>(&mut self, buffer: &'a [u8]) -> Result<HttpRequest<'a>, ParseError> {
        // Your zero-copy HTTP parser
        // Must handle: chunked encoding, headers spanning multiple lines,
        // malformed input, streaming input (partial requests)
        
        todo!("Implement zero-copy HTTP parser")
    }
    
    fn parse_streaming<'a>(&mut self, 
        chunk: &'a [u8]
    ) -> Result<Option<HttpRequest<'a>>, ParseError> {
        // Handle partial requests from streaming input
        // How do you maintain parser state across chunks?
        // What happens to borrowed references when new chunks arrive?
        
        todo!("Handle streaming input")
    }
}
```

**Performance Requirements:**
1. **Zero-Copy Parsing**: Minimize string allocations, use slices where possible
2. **Streaming Support**: Handle requests that arrive in multiple chunks  
3. **Error Recovery**: Malformed requests shouldn't crash the parser
4. **Benchmarking**: Provide criterion benchmarks showing allocation patterns

**Advanced Challenges:**
- How do you handle HTTP/2 multiplexing?
- What about WebSocket upgrade handling?
- How do you validate your parser against HTTP specification edge cases?

### Problem 2.2: Lock-Free Data Structure (25 points)

Implement a lock-free, concurrent queue with strong correctness guarantees:

```rust
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

// Lock-free queue that multiple producers and consumers can use concurrently
pub struct LockFreeQueue<T> {
    // Your lock-free data structure design
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

struct Node<T> {
    // What does a queue node need?
    data: Option<T>,
    next: AtomicPtr<Node<T>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        // Initialize the queue
        todo!()
    }
    
    pub fn enqueue(&self, item: T) {
        // Lock-free enqueue operation
        // Must handle: ABA problem, memory ordering, concurrent modifications
        todo!("Implement lock-free enqueue")
    }
    
    pub fn dequeue(&self) -> Option<T> {
        // Lock-free dequeue operation
        // Must handle: empty queue, concurrent dequeues, memory reclamation
        todo!("Implement lock-free dequeue")
    }
    
    pub fn is_empty(&self) -> bool {
        // How do you check if a concurrent queue is empty?
        todo!("Implement lock-free empty check")
    }
}

unsafe impl<T: Send> Send for LockFreeQueue<T> {}
unsafe impl<T: Send> Sync for LockFreeQueue<T> {}
```

**Correctness Requirements:**
1. **ABA Problem**: Handle the ABA problem with appropriate techniques
2. **Memory Ordering**: Use correct atomic orderings for all operations
3. **Memory Safety**: Prevent use-after-free and double-free bugs
4. **Progress Guarantees**: Ensure wait-free or lock-free progress

**Analysis Required:**
- Explain your memory reclamation strategy
- Prove that your implementation is linearizable
- Analyze the performance characteristics under contention
- Compare with mutex-based alternatives

---

## Section III: Memory Management & Safety (50 points)

### Problem 3.1: Custom Allocator Design (25 points)

Design a specialized allocator for a high-frequency trading system:

```rust
// Requirements:
// - Sub-microsecond allocation/deallocation
// - Predictable allocation patterns (no fragmentation)
// - Real-time guarantees (no GC pauses)
// - Memory usage monitoring

use std::alloc::{GlobalAlloc, Layout};

struct TradingAllocator {
    // Your allocator design
}

unsafe impl GlobalAlloc for TradingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Your high-performance allocation strategy
        todo!("Implement fast allocation")
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Your deallocation strategy
        todo!("Implement fast deallocation")  
    }
}

// Custom allocation strategies
impl TradingAllocator {
    fn new(pool_size: usize) -> Self {
        // How do you pre-allocate memory pools?
        todo!()
    }
    
    fn allocate_trade_message(&self) -> *mut TradeMessage {
        // Fast path for common allocation pattern
        todo!()
    }
    
    fn get_allocation_stats(&self) -> AllocationStats {
        // Real-time monitoring without affecting performance
        todo!()
    }
}
```

**Design Challenges:**
1. **Allocation Strategy**: Pool allocation, slab allocation, or custom strategy?
2. **Fragmentation Prevention**: How do you prevent memory fragmentation?
3. **Real-time Guarantees**: Worst-case allocation time bounds
4. **Memory Monitoring**: Track usage without performance penalty

### Problem 3.2: Unsafe Code Verification (25 points)

Review and fix this unsafe code that manages a custom data structure:

```rust
// Custom vector-like structure with unsafe optimizations
pub struct FastVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> FastVec<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        let ptr = if capacity == 0 {
            std::ptr::null_mut()
        } else {
            unsafe {
                let layout = Layout::array::<T>(capacity).unwrap();
                std::alloc::alloc(layout) as *mut T
            }
        };
        
        FastVec { ptr, len: 0, capacity }
    }
    
    pub fn push(&mut self, item: T) {
        if self.len >= self.capacity {
            self.grow();
        }
        
        unsafe {
            // BUG: What's wrong with this?
            *self.ptr.add(self.len) = item;
        }
        self.len += 1;
    }
    
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                // BUG: What's wrong here?
                Some(&*self.ptr.add(index))
            }
        } else {
            None
        }
    }
    
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 { 1 } else { self.capacity * 2 };
        
        let new_ptr = unsafe {
            let layout = Layout::array::<T>(new_capacity).unwrap();
            std::alloc::alloc(layout) as *mut T
        };
        
        unsafe {
            // BUG: Memory leak and use-after-free potential
            std::ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);
        }
        
        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl<T> Drop for FastVec<T> {
    fn drop(&mut self) {
        unsafe {
            // BUG: Incorrect cleanup
            std::alloc::dealloc(self.ptr as *mut u8, Layout::array::<T>(self.capacity).unwrap());
        }
    }
}
```

**Your Analysis:**
1. **Identify Bugs**: Find all memory safety issues in this code
2. **Fix Implementation**: Provide corrected versions of buggy functions
3. **Safety Proof**: Explain why your fixes maintain memory safety
4. **Test Strategy**: How would you test unsafe code for correctness?

---

## Section IV: Advanced Error Handling (50 points)

### Problem 4.1: Distributed Error Recovery (25 points)

Design an error handling system for a distributed transaction processor:

```rust
// Distributed transaction that can fail at multiple points
#[derive(Debug)]
enum DistributedError {
    NetworkPartition { affected_nodes: Vec<NodeId> },
    TransactionConflict { conflicting_tx: TransactionId },
    ResourceExhausted { resource: ResourceType },
    DataCorruption { checksum_mismatch: bool },
    TimeoutError { operation: Operation, elapsed: Duration },
}

// Your error recovery system
struct TransactionCoordinator {
    // What state do you need for recovery?
}

impl TransactionCoordinator {
    async fn execute_distributed_transaction(
        &mut self,
        transaction: Transaction
    ) -> Result<TransactionResult, DistributedError> {
        // Implement 2-phase commit with sophisticated error recovery
        // Must handle: partial failures, network partitions, node crashes
        
        todo!("Implement distributed transaction with recovery")
    }
    
    async fn recover_from_failure(
        &mut self,
        error: DistributedError,
        transaction_state: TransactionState
    ) -> RecoveryAction {
        // Design recovery strategy based on error type and system state
        match error {
            DistributedError::NetworkPartition { .. } => {
                // How do you recover from network partitions?
                todo!()
            },
            DistributedError::TransactionConflict { .. } => {
                // How do you resolve transaction conflicts?
                todo!()
            },
            // Handle other error types...
            _ => todo!()
        }
    }
}
```

**Requirements:**
1. **ACID Properties**: Maintain transaction guarantees despite failures
2. **Recovery Strategies**: Different recovery for different failure types
3. **Partial Failure Handling**: Some nodes succeed, others fail
4. **Observability**: Error tracking and alerting for operations teams

### Problem 4.2: Error Context Propagation (25 points)

Implement a context-preserving error system for a complex service:

```rust
// Design an error system that preserves rich context through call stack
use std::backtrace::Backtrace;

trait ContextualError: std::error::Error {
    fn add_context(self, context: ErrorContext) -> Self;
    fn get_context_chain(&self) -> Vec<&ErrorContext>;
    fn root_cause(&self) -> &dyn std::error::Error;
}

#[derive(Debug)]
struct ErrorContext {
    operation: &'static str,
    metadata: HashMap<String, String>,
    timestamp: SystemTime,
    span_id: SpanId, // For distributed tracing
}

// Your error implementation
#[derive(Debug)]
struct ServiceError {
    // How do you preserve context chain and root cause?
}

impl ContextualError for ServiceError {
    fn add_context(mut self, context: ErrorContext) -> Self {
        // Preserve error context through call stack
        todo!("Add context while preserving chain")
    }
    
    fn get_context_chain(&self) -> Vec<&ErrorContext> {
        // Return full context chain for debugging
        todo!()
    }
    
    fn root_cause(&self) -> &dyn std::error::Error {
        // Find the original error cause
        todo!()
    }
}

// Usage in complex service
async fn complex_service_operation(user_id: UserId) -> Result<ServiceResult, ServiceError> {
    let user = fetch_user(user_id)
        .await
        .map_err(|e| e.add_context(ErrorContext {
            operation: "fetch_user",
            metadata: [("user_id".to_string(), user_id.to_string())].iter().cloned().collect(),
            timestamp: SystemTime::now(),
            span_id: current_span_id(),
        }))?;
        
    let permissions = check_permissions(&user)
        .await
        .map_err(|e| e.add_context(ErrorContext {
            operation: "check_permissions", 
            metadata: [("user_role".to_string(), user.role.to_string())].iter().cloned().collect(),
            timestamp: SystemTime::now(),
            span_id: current_span_id(),
        }))?;
        
    // More operations...
    
    Ok(ServiceResult::new())
}
```

**Design Requirements:**
1. **Context Preservation**: Maintain full error context through async boundaries
2. **Performance**: Minimal overhead in success case, rich info in error case
3. **Observability**: Integration with distributed tracing and logging
4. **Ergonomics**: Easy to add context without cluttering business logic

---

## Grading Rubric

### Expert Level (180-200 points):
- **Systems Thinking**: Demonstrates deep understanding of distributed systems principles
- **Performance Engineering**: Optimizes for real-world constraints, not just theoretical performance
- **Memory Safety**: Correctly handles complex unsafe code and memory management
- **Production Readiness**: Code includes proper error handling, observability, and failure recovery

### Advanced Level (160-179 points):
- **Solid Implementation**: Most solutions work correctly with minor issues
- **Good Performance Awareness**: Understands performance implications of design choices
- **Safety Conscious**: Handles memory safety correctly in most cases
- **Error Handling**: Implements comprehensive error strategies

### Intermediate Level (140-159 points):
- **Basic Correctness**: Solutions work for common cases
- **Some Performance Understanding**: Aware of performance but may miss optimizations
- **Memory Safety Gaps**: Some unsafe code issues or memory management problems
- **Limited Error Handling**: Basic error propagation without strategic design

### Developing (120-139 points):
- **Partial Solutions**: Some parts work but significant gaps remain
- **Performance Naive**: Little consideration of performance implications
- **Safety Issues**: Memory safety bugs or incorrect unsafe code usage
- **Poor Error Handling**: Inadequate error strategies

### Insufficient (<120 points):
- **Non-functional**: Major correctness issues or incomplete solutions
- **No Performance Consideration**: Ignores performance entirely
- **Unsafe Code**: Dangerous memory safety violations
- **No Error Strategy**: Panic-driven programming

---

## Next Steps

**If you scored 180+**: You're ready for senior systems engineering roles. Consider contributing to major Rust projects.

**If you scored 160-179**: Strong systems programming skills. Focus on distributed systems patterns and performance engineering.

**If you scored 140-159**: Good foundation. Practice more complex systems design and unsafe code patterns.

**If you scored <140**: Review fundamental concepts and work through more advanced homework problems.

---

*"Systems programming is not about writing clever code - it's about writing code that works correctly under all conditions, including the ones you didn't anticipate."*