# 01. Advanced Systems Diagnostic: Prerequisites for Expert Rust

**Time Limit: 3 hours | Pass Threshold: 85%**

**Purpose**: This diagnostic tests whether you have the foundational knowledge required for **expert-level systems programming** in Rust. Only attempt the specialization exams if you pass this.

**Target Audience**: Engineers preparing for systems-level Rust development who need to master unsafe code, lock-free programming, and zero-copy optimization.

---

## Section I: Memory Model & Safety Invariants (25 points)

### Problem 1.1: Aliasing & Mutation Analysis (15 points)

**Context**: Understanding aliasing rules is prerequisite for unsafe Rust mastery.

```rust
use std::ptr;

struct Buffer {
    data: Vec<u8>,
    len: usize,
}

impl Buffer {
    // Analyze this implementation for safety issues
    fn get_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr(),
                self.len
            )
        }
    }
    
    fn extend_from_slice(&mut self, other: &[u8]) {
        let current_slice = self.get_mut_slice();
        // Problem: What happens here?
        self.data.extend_from_slice(other);
        self.len = self.data.len();
        
        // More operations with current_slice...
        current_slice[0] = 42; // Is this safe?
    }
}
```

**Questions:**
1. Identify all potential undefined behavior in this code
2. Explain the specific aliasing violations
3. Provide a safe rewrite that maintains equivalent functionality
4. What invariants must any safe wrapper maintain about the `len` field?

### Problem 1.2: Memory Layout Control (10 points)

```rust
// Design a zero-copy network packet parser
#[repr(C)]
struct PacketHeader {
    version: u8,
    flags: u8,
    length: u16,  // Big-endian
    checksum: u32, // Big-endian
}

// Your task: Implement safe parsing from raw bytes
impl PacketHeader {
    // Parse from raw bytes - what are the safety requirements?
    unsafe fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        // Your implementation - consider alignment, endianness, bounds checking
    }
    
    // Safe parsing interface
    fn parse(bytes: &[u8]) -> Result<&Self, ParseError> {
        // Your implementation
    }
}

// Define appropriate error types and safety documentation
```

---

## Section II: Concurrency Fundamentals (25 points)

### Problem 2.1: Memory Ordering Comprehension (15 points)

**Context**: Understanding memory ordering is prerequisite for lock-free programming.

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

struct Counter {
    ready: AtomicBool,
    value: AtomicUsize,
}

impl Counter {
    fn increment_pattern_1(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
        self.ready.store(true, Ordering::Release);
    }
    
    fn read_pattern_1(&self) -> Option<usize> {
        if self.ready.load(Ordering::Acquire) {
            Some(self.value.load(Ordering::Relaxed))
        } else {
            None
        }
    }
    
    fn increment_pattern_2(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
        self.ready.store(true, Ordering::SeqCst);
    }
    
    fn read_pattern_2(&self) -> Option<usize> {
        if self.ready.load(Ordering::SeqCst) {
            Some(self.value.load(Ordering::SeqCst))
        } else {
            None
        }
    }
}
```

**Questions:**
1. Explain the visibility guarantees of each pattern
2. Which pattern provides stronger consistency? Why?
3. What could go wrong if you mixed orderings incorrectly?
4. Design a test that could potentially detect ordering violations
5. When would you choose Relaxed vs Acquire/Release vs SeqCst?

### Problem 2.2: Arc vs Mutex Design Choice (10 points)

```rust
// Compare these two designs for a shared counter

// Design A: Fine-grained locking
struct CounterA {
    counts: Vec<Mutex<usize>>,
}

// Design B: Coarse-grained locking
struct CounterB {
    counts: Mutex<Vec<usize>>,
}

// Design C: Lock-free approach
struct CounterC {
    counts: Vec<AtomicUsize>,
}
```

**Analysis Required:**
1. Performance characteristics under different contention patterns
2. Memory overhead analysis  
3. Scalability implications for 1000+ concurrent threads
4. When each design is appropriate
5. Implement a benchmark that demonstrates the performance differences

---

## Section III: Advanced Lifetime Scenarios (25 points)

### Problem 3.1: Self-Referential Structure Challenge (15 points)

```rust
// This is a fundamental challenge for async/futures
struct SelfReferential {
    data: String,
    reference: &'??? str,  // Points into data
}

impl SelfReferential {
    fn new(s: String) -> Self {
        // How do you safely create this?
    }
    
    fn get_reference(&self) -> &str {
        self.reference
    }
}

// Your tasks:
// 1. Explain why this is impossible with normal references
// 2. Implement using Pin<Box<T>>
// 3. Explain the Pin contract and why it's necessary
// 4. Show how this relates to async/await futures
```

### Problem 3.2: Higher-Ranked Trait Bounds (10 points)

```rust
// Understanding HRTB is crucial for advanced async patterns
fn process_with_callback<F>(callback: F) -> impl Future<Output = String>
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    async move {
        let data = "hello world".to_string();
        let result = callback(&data);
        result.to_string()
    }
}

// Problems with the above code:
// 1. Why doesn't this compile?
// 2. Fix the lifetime issues
// 3. Explain when you need `for<'a>` syntax
// 4. Show how this applies to async closures
```

---

## Section IV: Zero-Copy & Performance (25 points)

### Problem 4.1: Cache-Aware Data Structure Design (15 points)

```rust
// Design a hash table optimized for cache performance

// Naive approach - why is this slow?
struct HashTableNaive<K, V> {
    buckets: Vec<Option<(K, V)>>,
}

// Your task: Design a cache-friendly hash table
struct HashTableOptimized<K, V> {
    // Your design here - consider:
    // - Memory layout for cache lines
    // - Minimizing pointer chasing
    // - SIMD-friendly operations
}

impl<K, V> HashTableOptimized<K, V> 
where 
    K: Hash + Eq,
{
    // Implement with cache optimization in mind
    fn insert(&mut self, key: K, value: V) {
        // Your implementation
    }
    
    fn get(&self, key: &K) -> Option<&V> {
        // Your implementation
    }
}

// Benchmark design:
// Write a benchmark that demonstrates the cache performance difference
```

### Problem 4.2: SIMD Optimization Prerequisites (10 points)

```rust
// Sum an array of f32 values - compare scalar vs SIMD readiness

fn sum_scalar(data: &[f32]) -> f32 {
    data.iter().sum()
}

// Your task: Prepare data structure for SIMD optimization
fn sum_simd_ready(data: &[f32]) -> f32 {
    // What are the alignment requirements?
    // How do you handle remainder elements?
    // Show understanding of memory layout for SIMD
}

// Analysis questions:
// 1. What alignment is required for AVX operations?
// 2. How do you handle arrays that aren't multiples of vector width?
// 3. When is SIMD optimization worth the complexity?
```

---

## Section V: Async Runtime Understanding (25 points)

### Problem 5.1: Custom Future Implementation (15 points)

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

// Implement a future that completes after a delay
struct DelayFuture {
    // What fields do you need?
    // How do you handle the timer?
    // What about thread safety?
}

impl Future for DelayFuture {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Your implementation:
        // 1. How do you track elapsed time?
        // 2. When do you return Poll::Pending vs Poll::Ready?
        // 3. How do you ensure the waker gets called?
        // 4. What are the safety requirements for Pin?
    }
}

// Bonus: Implement a timeout wrapper future
struct TimeoutFuture<F> {
    future: Pin<Box<F>>,
    timeout: DelayFuture,
}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = Result<F::Output, TimeoutError>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // How do you poll both futures correctly?
        // What about pin projection?
    }
}
```

### Problem 5.2: Async Cancellation Safety (10 points)

```rust
// This async function has cancellation safety issues
async fn process_data(data: Vec<u8>) -> Result<ProcessedData, Error> {
    let mut processed = Vec::new();
    
    for chunk in data.chunks(1024) {
        let result = expensive_processing(chunk).await?;
        processed.push(result);
        
        // Problem: What happens if cancelled here?
        save_to_database(&result).await?;
    }
    
    Ok(ProcessedData { data: processed })
}

// Your tasks:
// 1. Identify the cancellation safety issues
// 2. Redesign for cancellation safety
// 3. Explain the difference between "cancellation safe" and "drop safe"
// 4. Show how to test for cancellation bugs
```

---

## Section VI: Advanced Type System (25 points)

### Problem 6.1: Generic Associated Types (GATs) (15 points)

```rust
// Design a generic container that can hold references with different lifetimes
trait Container {
    type Item<'a> where Self: 'a;
    
    fn get<'a>(&'a self, index: usize) -> Option<Self::Item<'a>>;
    fn len(&self) -> usize;
}

// Implement for a vector of owned data
struct VecContainer<T> {
    data: Vec<T>,
}

impl<T> Container for VecContainer<T> {
    type Item<'a> = &'a T where Self: 'a;
    
    fn get<'a>(&'a self, index: usize) -> Option<Self::Item<'a>> {
        self.data.get(index)
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
}

// Now implement for a container that holds string slices
struct StringSliceContainer {
    // How do you store string slices of different lifetimes?
}

// Your tasks:
// 1. Complete the StringSliceContainer implementation
// 2. Explain why GATs are necessary here
// 3. Show how this enables zero-copy iteration patterns
// 4. Implement a generic function that works with any Container
```

### Problem 6.2: Complex Trait Bounds & Coherence (10 points)

```rust
// Design a serialization system with complex bounds
trait Serialize {
    fn serialize<W: Write>(&self, writer: W) -> Result<(), SerializeError>;
}

trait Deserialize: Sized {
    fn deserialize<R: Read>(reader: R) -> Result<Self, DeserializeError>;
}

// Problem: Implement for generic types with constraints
impl<T> Serialize for Vec<T> 
where
    T: Serialize,
{
    fn serialize<W: Write>(&self, mut writer: W) -> Result<(), SerializeError> {
        // Your implementation
    }
}

// Challenges:
// 1. What about Vec<Box<dyn Serialize>>?
// 2. How do you handle recursive types?
// 3. Orphan rule implications for external types?
// 4. Performance implications of monomorphization?
```

---

## Passing Criteria & Next Steps

### **Pass Requirements (85% minimum):**
- **Memory Safety**: Must demonstrate understanding of aliasing, memory layout, and safety invariants
- **Concurrency**: Must show comprehension of memory ordering and basic lock-free concepts  
- **Lifetimes**: Must handle complex lifetime scenarios correctly
- **Performance**: Must understand cache implications and zero-copy principles
- **Async**: Must demonstrate Pin usage and cancellation safety awareness
- **Type System**: Must correctly use GATs and complex trait bounds

### **If You Pass This Diagnostic:**
- **03_mastery_exam.md** - Advanced unsafe Rust and memory mastery
- **05_systems_specialization_exam.md** - Low-level systems programming
- **06_concurrency_specialization_exam.md** - Expert lock-free programming

### **If You Don't Pass:**
Focus on these prerequisites before attempting advanced exams:
1. **Rust Book Chapters 19 (Advanced Features), 16 (Concurrency)**
2. **Rustonomicon** (unsafe Rust guide)  
3. **Async Book** (futures and Pin)
4. **Practice with real systems code** (databases, networks, operating systems)

This diagnostic tests the **minimum** knowledge required for expert-level Rust systems programming. The specialization exams assume mastery of these concepts.