# Unsafe Rust Bridge Course
**Active Recall Problems for Safe Unsafe Programming**

*Time Limit: 4 hours | Total Points: 140 | Bridge to: Advanced Unsafe Rust & Memory Mastery*

---

## Mission: Master Unsafe Rust Through Verification

**Critical Gap Addressed**: You understand safe Rust, but can you write unsafe code safely? This bridge course teaches you to use unsafe superpowers responsibly through systematic verification and measurement.

**You Will Master:**
- Writing unsafe code with proper safety invariants and documentation
- Memory layout analysis and optimization techniques  
- Raw pointer manipulation with mathematical safety proofs
- Custom memory management with verification
- Performance measurement of unsafe optimizations
- FFI integration with safety guarantees

**Next Level**: After this, you'll be ready for `06_unsafe_rust_mastery.md` - Advanced Unsafe Rust & Memory Mastery

---

## The Unsafe Rust Philosophy

**Rule 1**: Unsafe code must be faster, more memory-efficient, or enable new capabilities  
**Rule 2**: Every unsafe block must be accompanied by a safety proof  
**Rule 3**: All unsafe optimizations must be benchmarked to verify benefits  
**Rule 4**: Unsafe code must be tested more rigorously than safe code  

**Verification Methodology**: For every unsafe operation, you must demonstrate:
1. **Safety Invariants**: What conditions make this operation safe?
2. **Proof of Invariants**: Why do these conditions hold in this context?
3. **Performance Benefit**: Measured improvement over safe alternatives
4. **Comprehensive Testing**: Tests that would catch safety violations

---

## Section I: Safe Unsafe Code Fundamentals (40 points)

### Problem 1.1: Verified Unsafe Vector Operations (20 points)

Implement unsafe optimizations for a custom vector with complete safety verification:

```rust
// Start with this safe but potentially slow implementation
pub struct FastVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> FastVec<T> {
    pub fn new() -> Self {
        Self {
            ptr: std::ptr::NonNull::dangling().as_ptr(),
            len: 0,
            capacity: 0,
        }
    }
    
    // TASK 1: Implement unsafe push with complete safety verification
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }
        
        // YOUR UNSAFE IMPLEMENTATION HERE
        // Requirements:
        // 1. Write a safety comment explaining ALL invariants
        // 2. Prove mathematically that these invariants hold
        // 3. Benchmark against Vec<T> to prove performance benefit
        // 4. Write tests that would catch safety violations
        
        unsafe {
            // Safety invariants that must hold:
            // 1. ptr + len must point to valid, uninitialized memory
            // 2. ptr + len must be within our allocated capacity  
            // 3. ptr must be properly aligned for T
            // 4. No other references to ptr + len can exist
            //
            // Proof these invariants hold:
            // - grow() ensures capacity > len, so ptr + len is valid
            // - grow() allocates aligned memory for T
            // - We have &mut self, so no aliases exist
            // - TODO: Complete your proof
            
            std::ptr::write(self.ptr.add(self.len), value);
        }
        
        self.len += 1;
    }
    
    // TASK 2: Implement unsafe get with bounds checking elimination
    pub fn get_unchecked(&self, index: usize) -> &T {
        // YOUR IMPLEMENTATION: Use unsafe to eliminate bounds check
        // But first VERIFY that bounds check elimination is safe
        // Provide mathematical proof that index < self.len
        
        debug_assert!(index < self.len, "Index {} out of bounds for len {}", index, self.len);
        
        unsafe {
            // Safety: TODO - provide complete safety proof
            &*self.ptr.add(index)
        }
    }
    
    fn grow(&mut self) {
        // TASK 3: Implement safe growth strategy
        // Compare allocation strategies: doubling vs linear growth
        let new_capacity = if self.capacity == 0 { 4 } else { self.capacity * 2 };
        
        let new_layout = std::alloc::Layout::array::<T>(new_capacity)
            .expect("Capacity overflow");
            
        let new_ptr = unsafe {
            // Safety: TODO - explain why this allocation is safe
            std::alloc::alloc(new_layout) as *mut T
        };
        
        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(new_layout);
        }
        
        if self.capacity > 0 {
            unsafe {
                // TASK: Copy existing elements safely
                // Safety: TODO - complete safety proof for copy
                std::ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);
                
                // Deallocate old memory
                let old_layout = std::alloc::Layout::array::<T>(self.capacity).unwrap();
                std::alloc::dealloc(self.ptr as *mut u8, old_layout);
            }
        }
        
        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

// TASK 4: Implement Drop with verification
impl<T> Drop for FastVec<T> {
    fn drop(&mut self) {
        // YOUR IMPLEMENTATION: Safely clean up all resources
        // Must: 1. Drop all elements 2. Deallocate memory 3. Prove no leaks
        
        if self.capacity > 0 {
            // Drop all initialized elements
            for i in 0..self.len {
                unsafe {
                    // Safety: TODO - prove this drop is safe
                    std::ptr::drop_in_place(self.ptr.add(i));
                }
            }
            
            // Deallocate the buffer
            unsafe {
                let layout = std::alloc::Layout::array::<T>(self.capacity).unwrap();
                std::alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

// TASK 5: Comprehensive safety testing
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safety_invariants() {
        // YOUR TESTS: Create tests that would catch safety violations
        // Test cases that must pass:
        // 1. No use-after-free
        // 2. No double-free  
        // 3. No memory leaks
        // 4. No buffer overflows
        // 5. Proper alignment
        
        // TODO: Implement comprehensive safety tests
    }
    
    #[test] 
    fn test_performance_benefit() {
        // YOUR BENCHMARK: Prove unsafe version is actually faster
        // Compare with std::vec::Vec<T> for:
        // 1. Push performance
        // 2. Access performance  
        // 3. Memory usage
        
        // Use criterion for statistical significance
    }
}
```

**Safety Verification Requirements:**
1. **Complete Safety Comments**: Every unsafe block must have detailed safety invariants
2. **Mathematical Proofs**: Prove that safety invariants hold in all cases  
3. **Performance Justification**: Benchmark unsafe vs safe to prove benefits
4. **Comprehensive Testing**: Tests that would catch memory safety violations
5. **Edge Case Analysis**: Handle capacity overflow, null pointers, alignment

### Problem 1.2: Zero-Copy String Parser with Safety Verification (20 points)

Implement a zero-copy HTTP header parser using unsafe for performance:

```rust
// Zero-copy HTTP header parsing with unsafe optimizations
pub struct HttpHeaders<'a> {
    raw_data: &'a [u8],
    // TODO: Design internal structure for O(1) header lookup
}

impl<'a> HttpHeaders<'a> {
    // TASK 1: Parse headers with zero allocation using unsafe
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        // YOUR IMPLEMENTATION: Use unsafe to:
        // 1. Skip UTF-8 validation where safe (ASCII headers)
        // 2. Eliminate bounds checks in hot parsing loops  
        // 3. Use raw pointers for efficient scanning
        
        // Safety requirements:
        // - Prove all header names/values remain valid for lifetime 'a
        // - Ensure no out-of-bounds access during parsing
        // - Handle malformed input safely
        
        let mut parser = HeaderParser::new(data);
        parser.parse_all()
    }
    
    // TASK 2: O(1) header lookup using unsafe
    pub fn get(&self, name: &str) -> Option<&'a str> {
        // YOUR IMPLEMENTATION: Use unsafe to eliminate redundant UTF-8 checks
        // Headers are ASCII, so you can safely convert &[u8] to &str
        
        // Safety proof required:
        // - Why is skipping UTF-8 validation safe for HTTP headers?
        // - How do you handle malformed headers during parsing?
        
        todo!("Implement O(1) header lookup")
    }
}

struct HeaderParser<'a> {
    data: &'a [u8],
    pos: usize,
    // TODO: Add fields for efficient parsing state
}

impl<'a> HeaderParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    
    // TASK 3: Unsafe parsing with bounds check elimination  
    fn parse_header_line(&mut self) -> Result<Option<(&'a str, &'a str)>, ParseError> {
        // YOUR IMPLEMENTATION: Use unsafe to eliminate bounds checks
        // in the hot parsing loop
        
        // Find colon separator
        let start = self.pos;
        
        unsafe {
            // Safety: TODO - prove bounds are checked before entry
            // and that we never exceed self.data.len()
            
            while self.pos < self.data.len() {
                match *self.data.get_unchecked(self.pos) {
                    b':' => {
                        // Found separator, parse name and value
                        let name = self.parse_header_name(start)?;
                        let value = self.parse_header_value()?;
                        return Ok(Some((name, value)));
                    },
                    b'\r' | b'\n' => {
                        // End of headers
                        return Ok(None);
                    },
                    _ => self.pos += 1,
                }
            }
        }
        
        Err(ParseError::MalformedHeader)
    }
    
    // TASK 4: Unsafe UTF-8 conversion with validation
    unsafe fn bytes_to_str_unchecked(&self, start: usize, end: usize) -> &'a str {
        // Safety requirements:
        // 1. Prove start <= end <= self.data.len()  
        // 2. Prove bytes in range are valid UTF-8 (or ASCII)
        // 3. Prove lifetime 'a is correct for returned &str
        
        debug_assert!(start <= end && end <= self.data.len());
        debug_assert!(self.data[start..end].is_ascii()); // HTTP headers are ASCII
        
        let slice = std::slice::from_raw_parts(
            self.data.as_ptr().add(start),
            end - start
        );
        
        // This is safe because:
        // TODO: Complete safety justification
        std::str::from_utf8_unchecked(slice)
    }
    
    fn parse_header_name(&mut self, start: usize) -> Result<&'a str, ParseError> {
        // TODO: Implement with unsafe optimizations
        todo!()
    }
    
    fn parse_header_value(&mut self) -> Result<&'a str, ParseError> {
        // TODO: Implement with unsafe optimizations  
        todo!()
    }
    
    fn parse_all(mut self) -> Result<HttpHeaders<'a>, ParseError> {
        // TODO: Parse all headers and build efficient lookup structure
        todo!()
    }
}

// TASK 5: Performance verification
#[cfg(test)]
mod perf_tests {
    use super::*;
    use criterion::{black_box, Criterion};
    
    fn benchmark_header_parsing(c: &mut Criterion) {
        let headers = b"Host: example.com\r\nUser-Agent: test\r\nContent-Length: 123\r\n\r\n";
        
        c.bench_function("unsafe_header_parsing", |b| {
            b.iter(|| {
                let parsed = HttpHeaders::parse(black_box(headers)).unwrap();
                black_box(parsed.get("Host"));
            })
        });
        
        // TODO: Compare with safe parsing implementation
        // Measure: parsing time, memory allocations, lookup time
    }
    
    #[test]
    fn safety_edge_cases() {
        // YOUR TESTS: Verify safety with malformed input
        // Test cases:
        // 1. Headers without colons
        // 2. Headers with non-ASCII characters  
        // 3. Headers at buffer boundaries
        // 4. Empty headers
        // 5. Very long headers
        
        // Each test should either parse correctly or fail safely
        // No crashes, no undefined behavior
    }
}
```

**Unsafe Optimization Requirements:**
1. **Performance Justification**: Measure parsing performance vs safe alternatives
2. **Safety Documentation**: Every unsafe block needs complete safety analysis
3. **Edge Case Handling**: Malformed input must be handled safely
4. **Lifetime Verification**: Prove borrowed data lives long enough
5. **Fuzzing Integration**: Design for compatibility with fuzzing tools

---

## Section II: Memory Layout Analysis and Optimization (35 points)

### Problem 2.1: Struct Layout Optimization with Verification (20 points)

Optimize data structure layout for cache performance and measure the impact:

```rust
// Inefficient struct layout (common SRE data structures)
#[derive(Debug, Clone)]
pub struct MetricPointBad {
    timestamp: u64,      // 8 bytes
    value: f64,          // 8 bytes  
    is_valid: bool,      // 1 byte (+ 7 padding)
    metric_name: String, // 24 bytes (Vec overhead)
    tags: Vec<String>,   // 24 bytes (Vec overhead)  
    host_id: u16,        // 2 bytes (+ 6 padding)
}

// TASK 1: Optimize struct layout and measure cache performance
#[repr(C)] // or #[repr(packed)] - choose and justify
#[derive(Debug, Clone)]  
pub struct MetricPointOptimized {
    // YOUR OPTIMIZED LAYOUT:
    // 1. Minimize padding through field reordering
    // 2. Consider memory access patterns  
    // 3. Optimize for cache line size (64 bytes)
    // 4. Measure actual performance improvement
    
    // TODO: Reorder fields for optimal layout
    // Consider: which fields are accessed together?
    // Consider: can any fields be made smaller?
    // Consider: string interning for repeated values?
}

impl MetricPointOptimized {
    // TASK 2: Implement memory-efficient construction
    pub fn new(
        timestamp: u64,
        value: f64, 
        metric_name: &str,
        host_id: u16,
        tags: &[&str]
    ) -> Self {
        // YOUR IMPLEMENTATION: Minimize allocations
        // Consider: string interning, small string optimization
        // Measure: allocation count and total bytes allocated
        
        todo!()
    }
    
    // TASK 3: Implement cache-friendly batch operations
    pub fn calculate_batch_stats(metrics: &[Self]) -> BatchStats {
        // YOUR IMPLEMENTATION: Optimize for cache access patterns
        // Process data to maximize cache locality
        // Use unsafe if beneficial (with verification)
        
        todo!()
    }
}

// TASK 4: Memory layout verification
#[cfg(test)]
mod layout_tests {
    use super::*;
    use std::mem;
    
    #[test]
    fn verify_layout_optimization() {
        // YOUR TESTS: Verify layout improvements
        println!("MetricPointBad size: {} bytes", mem::size_of::<MetricPointBad>());
        println!("MetricPointBad align: {} bytes", mem::align_of::<MetricPointBad>());
        
        println!("MetricPointOptimized size: {} bytes", mem::size_of::<MetricPointOptimized>());
        println!("MetricPointOptimized align: {} bytes", mem::align_of::<MetricPointOptimized>());
        
        // TODO: Add assertions verifying size reduction
        // TODO: Use std::mem::offset_of! to verify field positions
    }
    
    #[test]
    fn cache_performance_test() {
        // YOUR BENCHMARK: Measure cache performance improvement
        // Create large arrays of both struct types
        // Measure: cache misses, memory bandwidth, access latency
        // Use perf stat or similar tools for hardware counters
        
        let count = 1_000_000;
        let bad_metrics: Vec<MetricPointBad> = (0..count)
            .map(|i| MetricPointBad {
                timestamp: i,
                value: i as f64,
                is_valid: true,
                metric_name: format!("metric_{}", i % 100),
                tags: vec!["tag1".to_string(), "tag2".to_string()],
                host_id: (i % 1000) as u16,
            })
            .collect();
            
        // TODO: Create optimized version and benchmark both
        // Measure: iteration speed, sum calculation, filtering performance
    }
}
```

**Layout Optimization Requirements:**
1. **Size Analysis**: Document exact memory savings achieved
2. **Alignment Verification**: Prove optimal field alignment
3. **Cache Performance**: Measure cache hit rates with perf
4. **Access Pattern Optimization**: Optimize for common use cases
5. **Trade-off Analysis**: Document any functionality trade-offs made

### Problem 2.2: Custom Allocator with Performance Measurement (15 points)

Implement a pool allocator for high-frequency allocations with full verification:

```rust
use std::alloc::{GlobalAlloc, Layout};
use std::ptr::NonNull;

// TASK 1: Implement a pool allocator for fixed-size allocations
pub struct PoolAllocator {
    // YOUR DESIGN: Choose appropriate data structure for free list
    // Consider: thread safety, fragmentation, allocation speed
    // Options: linked list, bitmap, stack-based approach
}

unsafe impl GlobalAlloc for PoolAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // YOUR IMPLEMENTATION: Fast path for pool-sized allocations
        // Fall back to system allocator for other sizes
        
        // Performance target: sub-100ns allocation time
        // Safety requirements:
        // 1. Return properly aligned memory
        // 2. Never return the same pointer twice
        // 3. Handle concurrent access safely
        
        todo!()
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // YOUR IMPLEMENTATION: Return memory to pool or system
        // Safety requirements:
        // 1. Verify ptr came from this allocator
        // 2. Handle double-free safely  
        // 3. Update free list atomically
        
        todo!()
    }
}

impl PoolAllocator {
    pub fn new(pool_size: usize, block_size: usize) -> Result<Self, AllocatorError> {
        // YOUR IMPLEMENTATION: Pre-allocate memory pool
        // Design decisions:
        // 1. How do you track free blocks?
        // 2. How do you handle pool exhaustion?
        // 3. What alignment guarantees do you provide?
        
        todo!()
    }
    
    // TASK 2: Implement allocation statistics
    pub fn get_stats(&self) -> PoolStats {
        // YOUR IMPLEMENTATION: Collect allocation metrics
        // Metrics to track:
        // - Total allocations/deallocations
        // - Peak memory usage
        // - Fragmentation level
        // - Cache hit rate (if applicable)
        
        todo!()
    }
    
    // TASK 3: Implement memory defragmentation
    pub unsafe fn defragment(&mut self) -> DefragResult {
        // YOUR IMPLEMENTATION: Reduce fragmentation if needed
        // WARNING: This is extremely unsafe - moving allocated objects
        // Only implement if you can solve the reference update problem
        
        todo!()
    }
}

// TASK 4: Allocation performance benchmarking
#[cfg(test)]
mod allocator_tests {
    use super::*;
    use criterion::{black_box, Criterion};
    use std::time::Instant;
    
    #[test]
    fn benchmark_allocation_performance() {
        // YOUR BENCHMARK: Compare with system allocator
        let pool = PoolAllocator::new(1024 * 1024, 64).unwrap();
        
        // Measure allocation speed
        let iterations = 100_000;
        let start = Instant::now();
        
        unsafe {
            let layout = Layout::new::<[u8; 64]>();
            for _ in 0..iterations {
                let ptr = pool.alloc(layout);
                if !ptr.is_null() {
                    pool.dealloc(ptr, layout);
                }
            }
        }
        
        let duration = start.elapsed();
        println!("Pool allocator: {:?} per allocation", duration / iterations);
        
        // TODO: Compare with std::alloc::System
        // TODO: Measure allocation latency distribution
        // TODO: Test under concurrent access
    }
    
    #[test] 
    fn safety_stress_test() {
        // YOUR TESTS: Stress test for memory safety
        // Test scenarios:
        // 1. Rapid allocation/deallocation cycles
        // 2. Pool exhaustion handling
        // 3. Concurrent access from multiple threads
        // 4. Double-free detection
        // 5. Use-after-free detection
        
        // Run with AddressSanitizer and Valgrind for verification
    }
}

// TASK 5: Integration with existing code
#[global_allocator]
static GLOBAL: PoolAllocator = // TODO: Initialize safely

// Demonstrate usage in realistic SRE scenario
fn high_frequency_allocations() {
    // Simulate processing 1M log entries/second
    // Each entry requires temporary allocations for parsing
    // Measure: allocation overhead, memory fragmentation, throughput impact
    
    for _ in 0..1_000_000 {
        let log_entry = String::from("INFO Processing request 12345");
        let parsed = parse_log_entry(&log_entry);
        process_parsed_entry(parsed);
        // Entries go out of scope - frequent deallocation
    }
}
```

**Allocator Implementation Requirements:**
1. **Performance Measurement**: Sub-100ns allocation time target
2. **Memory Safety**: No double-free, use-after-free, or leaks  
3. **Concurrent Safety**: Thread-safe operation
4. **Fragmentation Analysis**: Measure and minimize fragmentation
5. **Integration Testing**: Works as global allocator

---

## Section III: FFI Safety and Verification (30 points)

### Problem 3.1: Safe C Library Integration (20 points)

Create safe Rust bindings for a C library with complete safety verification:

```rust
// Example C library interface (simulated)
// Library: high-performance compression library
extern "C" {
    // C function signatures (you'll wrap these safely)
    fn compress_init() -> *mut libc::c_void;
    fn compress_data(
        ctx: *mut libc::c_void,
        input: *const u8,
        input_len: libc::size_t,
        output: *mut u8,
        output_capacity: libc::size_t,
        output_len: *mut libc::size_t,
    ) -> libc::c_int;
    fn compress_finish(ctx: *mut libc::c_void) -> libc::c_int;
    fn compress_destroy(ctx: *mut libc::c_void);
}

// TASK 1: Design safe wrapper with complete safety verification
pub struct CompressionContext {
    // YOUR DESIGN: How do you safely manage the C context?
    // Consider: RAII, Send/Sync safety, null pointer handling
    inner: NonNull<libc::c_void>,
    // TODO: Add fields for safety tracking
}

impl CompressionContext {
    pub fn new() -> Result<Self, CompressionError> {
        // YOUR IMPLEMENTATION: Safe initialization
        // Safety requirements:
        // 1. Check for null pointer return
        // 2. Ensure proper cleanup on failure
        // 3. Document all assumptions about C library behavior
        
        let ptr = unsafe { 
            // Safety: TODO - document why this call is safe
            // What happens if compress_init fails?
            // How do you handle initialization errors?
            compress_init()
        };
        
        // TODO: Handle null pointer and other error cases
        todo!()
    }
    
    // TASK 2: Safe buffer management for C interface
    pub fn compress(&mut self, input: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // YOUR IMPLEMENTATION: Safe buffer passing to C
        // Safety challenges:
        // 1. Buffer lifetime must outlive C call
        // 2. Output buffer must be large enough
        // 3. C code must not retain references to buffers
        // 4. Handle partial compression scenarios
        
        if input.is_empty() {
            return Ok(Vec::new());
        }
        
        // Estimate output buffer size (library-specific heuristic)
        let output_capacity = input.len() + (input.len() / 10) + 64;
        let mut output = Vec::with_capacity(output_capacity);
        let mut actual_output_len: libc::size_t = 0;
        
        let result = unsafe {
            // Safety verification required:
            // 1. self.inner is valid (how do you ensure this?)
            // 2. input buffer is valid for read access
            // 3. output buffer is valid for write access  
            // 4. Lengths are accurate and within bounds
            // 5. C function doesn't retain pointers after return
            
            compress_data(
                self.inner.as_ptr(),
                input.as_ptr(),
                input.len() as libc::size_t,
                output.as_mut_ptr(),
                output_capacity as libc::size_t,
                &mut actual_output_len,
            )
        };
        
        // TODO: Handle C function return codes safely
        // TODO: Set output vector length correctly
        // TODO: Verify actual_output_len is reasonable
        
        todo!()
    }
    
    // TASK 3: Safe streaming compression
    pub fn compress_stream<R, W>(&mut self, input: R, output: W) -> Result<u64, CompressionError>
    where
        R: std::io::Read,
        W: std::io::Write,
    {
        // YOUR IMPLEMENTATION: Stream compression with safety
        // Handle: partial reads, partial writes, C library state
        // Ensure: no buffer overflows, proper error propagation
        
        todo!()
    }
}

// TASK 4: Safe resource cleanup
impl Drop for CompressionContext {
    fn drop(&mut self) {
        // YOUR IMPLEMENTATION: Ensure C resources are freed
        // Safety requirements:
        // 1. Context must be valid when Drop is called
        // 2. Handle double-free scenarios
        // 3. No panics in Drop implementation
        // 4. Cleanup must succeed even if context is corrupted
        
        unsafe {
            // Safety: TODO - justify why this cleanup is safe
            compress_destroy(self.inner.as_ptr());
        }
    }
}

// TASK 5: Thread safety analysis
unsafe impl Send for CompressionContext {
    // YOUR ANALYSIS: Is it safe to send between threads?
    // Questions to answer:
    // 1. Does the C library have internal state?
    // 2. Are C function calls thread-safe?
    // 3. What about the context pointer?
}

unsafe impl Sync for CompressionContext {
    // YOUR ANALYSIS: Is it safe to share between threads?
    // This is likely NOT safe for most C libraries
    // Document why Sync is or isn't appropriate
}

// TASK 6: Comprehensive safety testing
#[cfg(test)]
mod ffi_safety_tests {
    use super::*;
    
    #[test]
    fn test_basic_compression() {
        // YOUR TESTS: Verify basic functionality
        let mut ctx = CompressionContext::new().unwrap();
        let input = b"Hello, world! This is test data for compression.";
        let compressed = ctx.compress(input).unwrap();
        
        // Verify compressed data is reasonable
        assert!(!compressed.is_empty());
        // TODO: Add roundtrip decompression test
    }
    
    #[test]
    fn test_edge_cases() {
        // YOUR TESTS: Test edge cases that could cause safety issues
        let mut ctx = CompressionContext::new().unwrap();
        
        // Empty input
        assert!(ctx.compress(&[]).is_ok());
        
        // Very large input (test buffer management)
        let large_input = vec![0u8; 1024 * 1024];
        assert!(ctx.compress(&large_input).is_ok());
        
        // TODO: Add more edge cases
        // - Invalid input patterns
        // - Memory pressure scenarios  
        // - Concurrent access tests
    }
    
    #[test]
    fn test_error_handling() {
        // YOUR TESTS: Verify error handling doesn't cause safety issues
        // Simulate: C library failures, memory exhaustion, invalid input
        
        // TODO: How do you test C library failure modes safely?
    }
}
```

**FFI Safety Requirements:**
1. **Null Pointer Handling**: All C pointers must be checked for null
2. **Buffer Safety**: No buffer overflows or underflows across FFI boundary
3. **Resource Management**: All C resources must be freed exactly once
4. **Error Handling**: C error codes must be handled without panics
5. **Thread Safety**: Send/Sync must be implemented correctly if applicable

### Problem 3.2: Performance-Critical FFI Optimization (10 points)

Optimize FFI calls for performance while maintaining safety:

```rust
// High-frequency FFI scenario: cryptographic hash function
extern "C" {
    fn hash_init(ctx: *mut [u8; 256]) -> libc::c_int;
    fn hash_update(ctx: *mut [u8; 256], data: *const u8, len: libc::size_t);
    fn hash_final(ctx: *mut [u8; 256], output: *mut [u8; 32]) -> libc::c_int;
}

// TASK 1: Zero-allocation FFI wrapper
pub struct FastHasher {
    // YOUR DESIGN: Minimize allocations for high-frequency use
    // Consider: stack allocation, context reuse, batch processing
    context: [u8; 256], // Inline context to avoid heap allocation
}

impl FastHasher {
    // TASK 2: Optimize for repeated use
    pub fn new() -> Result<Self, HashError> {
        let mut hasher = Self {
            context: [0u8; 256],
        };
        
        let result = unsafe {
            // Safety: context is properly aligned and sized
            hash_init(&mut hasher.context as *mut [u8; 256])
        };
        
        if result != 0 {
            return Err(HashError::InitializationFailed);
        }
        
        Ok(hasher)
    }
    
    pub fn update(&mut self, data: &[u8]) -> Result<(), HashError> {
        // YOUR OPTIMIZATION: Minimize FFI call overhead
        // Consider: batch small updates, eliminate bounds checks
        
        if data.is_empty() {
            return Ok(());
        }
        
        unsafe {
            // Safety: TODO - prove data pointer and length are valid
            hash_update(
                &mut self.context as *mut [u8; 256],
                data.as_ptr(),
                data.len() as libc::size_t,
            );
        }
        
        Ok(())
    }
    
    // TASK 3: Batch processing optimization  
    pub fn hash_batch(&mut self, inputs: &[&[u8]]) -> Result<Vec<[u8; 32]>, HashError> {
        // YOUR OPTIMIZATION: Process multiple inputs efficiently
        // Minimize: context reinitialization, memory allocation
        // Maximize: CPU cache utilization, FFI call amortization
        
        let mut results = Vec::with_capacity(inputs.len());
        
        for input in inputs {
            // Reset context for new hash
            let init_result = unsafe {
                hash_init(&mut self.context as *mut [u8; 256])
            };
            if init_result != 0 {
                return Err(HashError::ResetFailed);
            }
            
            // TODO: Optimize this loop for better performance
            self.update(input)?;
            
            let mut output = [0u8; 32];
            let final_result = unsafe {
                hash_final(
                    &mut self.context as *mut [u8; 256],
                    &mut output as *mut [u8; 32],
                )
            };
            
            if final_result != 0 {
                return Err(HashError::FinalizationFailed);
            }
            
            results.push(output);
        }
        
        Ok(results)
    }
}

// TASK 4: Performance benchmarking
#[cfg(test)]
mod ffi_perf_tests {
    use super::*;
    use criterion::{black_box, Criterion, BatchSize};
    
    #[test]
    fn benchmark_ffi_overhead() {
        // YOUR BENCHMARK: Measure FFI call overhead
        let mut hasher = FastHasher::new().unwrap();
        let data = b"benchmark data for hashing performance test";
        
        // Compare: single large update vs many small updates
        // Measure: FFI call overhead, memory allocation impact
        
        let iterations = 10_000;
        let start = std::time::Instant::now();
        
        for _ in 0..iterations {
            hasher.update(black_box(data)).unwrap();
        }
        
        let duration = start.elapsed();
        println!("FFI overhead: {:?} per call", duration / iterations);
    }
    
    #[test]
    fn benchmark_batch_processing() {
        // YOUR BENCHMARK: Compare batch vs individual processing
        let mut hasher = FastHasher::new().unwrap();
        
        let inputs: Vec<&[u8]> = (0..1000)
            .map(|i| format!("test data {}", i))
            .collect::<Vec<_>>()
            .iter()
            .map(|s| s.as_bytes())
            .collect();
        
        // Measure batch processing performance
        let start = std::time::Instant::now();
        let results = hasher.hash_batch(&inputs).unwrap();
        let batch_duration = start.elapsed();
        
        println!("Batch processing: {:?} total", batch_duration);
        println!("Per hash: {:?}", batch_duration / results.len() as u32);
        
        // TODO: Compare with individual hash_single calls
        // TODO: Measure memory allocation differences
    }
}
```

**FFI Performance Requirements:**
1. **Allocation Minimization**: Avoid heap allocations in hot paths
2. **Batch Processing**: Amortize FFI call overhead across multiple operations
3. **Cache Optimization**: Design for good CPU cache utilization
4. **Measurement**: Quantify performance improvements over naive approach
5. **Safety Preservation**: Optimizations must not compromise memory safety

---

## Section IV: Unsafe Rust Integration and Real-World Application (35 points)

### Problem 4.1: High-Performance Network Parser (25 points)

Implement a zero-copy network protocol parser using unsafe optimizations:

```rust
// Network protocol: Simple key-value message format
// Format: [length:4][key_len:2][key][value_len:2][value]
// All multi-byte integers are little-endian

pub struct NetworkMessage<'a> {
    // YOUR DESIGN: Zero-copy representation
    // Requirements: no allocations, all data borrowed from input buffer
    raw_data: &'a [u8],
    // TODO: Add fields for efficient field access
}

impl<'a> NetworkMessage<'a> {
    // TASK 1: Zero-copy parsing with unsafe optimizations
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        // YOUR IMPLEMENTATION: Use unsafe to eliminate bounds checks
        // Safety requirements:
        // 1. Validate message length before any unsafe access
        // 2. Ensure all field boundaries are within buffer
        // 3. Handle malformed messages safely
        // 4. Prove lifetime 'a is sufficient for all borrowed data
        
        if data.len() < 4 {
            return Err(ParseError::MessageTooShort);
        }
        
        // Parse message length (4 bytes, little-endian)
        let message_len = unsafe {
            // Safety: We verified data.len() >= 4 above
            let len_bytes = std::slice::from_raw_parts(data.as_ptr(), 4);
            u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]])
        };
        
        if message_len as usize != data.len() - 4 {
            return Err(ParseError::LengthMismatch);
        }
        
        // TODO: Continue parsing key and value with unsafe optimizations
        // TODO: Validate all field boundaries before creating NetworkMessage
        
        todo!()
    }
    
    // TASK 2: Fast field access with unsafe
    pub fn key(&self) -> &'a str {
        // YOUR IMPLEMENTATION: Extract key with zero bounds checks
        // Safety proof required: key bounds are validated during parsing
        
        unsafe {
            // Safety: TODO - prove key bounds were validated during parsing
            todo!()
        }
    }
    
    pub fn value(&self) -> &'a [u8] {
        // YOUR IMPLEMENTATION: Extract value with zero bounds checks
        // Safety proof required: value bounds are validated during parsing
        
        unsafe {
            // Safety: TODO - prove value bounds were validated during parsing  
            todo!()
        }
    }
    
    // TASK 3: Unsafe iterator for batch processing
    pub fn parse_batch(data: &'a [u8]) -> BatchMessageIterator<'a> {
        // YOUR IMPLEMENTATION: Iterator that parses multiple messages
        // Use unsafe to eliminate repeated bounds checks across messages
        
        BatchMessageIterator::new(data)
    }
}

// TASK 4: High-performance batch iterator
pub struct BatchMessageIterator<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> BatchMessageIterator<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }
}

impl<'a> Iterator for BatchMessageIterator<'a> {
    type Item = Result<NetworkMessage<'a>, ParseError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // YOUR IMPLEMENTATION: Parse next message with unsafe optimizations
        // Eliminate bounds checks by maintaining position invariants
        
        if self.position >= self.data.len() {
            return None;
        }
        
        // Use unsafe to parse without redundant bounds checking
        // Safety requirements:
        // 1. Prove position is always within data bounds
        // 2. Ensure message boundaries don't exceed remaining data
        // 3. Handle partial messages at end of buffer
        
        unsafe {
            // Safety: TODO - prove position bounds and message parsing is safe
            todo!()
        }
    }
}

// TASK 5: Performance benchmarking and safety verification
#[cfg(test)]
mod network_parser_tests {
    use super::*;
    
    fn create_test_message(key: &str, value: &[u8]) -> Vec<u8> {
        let mut msg = Vec::new();
        let total_len = 2 + key.len() + 2 + value.len();
        msg.extend_from_slice(&(total_len as u32).to_le_bytes());
        msg.extend_from_slice(&(key.len() as u16).to_le_bytes());
        msg.extend_from_slice(key.as_bytes());
        msg.extend_from_slice(&(value.len() as u16).to_le_bytes());
        msg.extend_from_slice(value);
        msg
    }
    
    #[test]
    fn test_zero_copy_parsing() {
        let message = create_test_message("test_key", b"test_value");
        let parsed = NetworkMessage::parse(&message).unwrap();
        
        // Verify zero-copy: key and value should point into original buffer
        assert_eq!(parsed.key(), "test_key");
        assert_eq!(parsed.value(), b"test_value");
        
        // Verify addresses are within original buffer
        let msg_ptr = message.as_ptr() as usize;
        let msg_end = msg_ptr + message.len();
        let key_ptr = parsed.key().as_ptr() as usize;
        let value_ptr = parsed.value().as_ptr() as usize;
        
        assert!(key_ptr >= msg_ptr && key_ptr < msg_end);
        assert!(value_ptr >= msg_ptr && value_ptr < msg_end);
    }
    
    #[test]
    fn performance_benchmark() {
        // YOUR BENCHMARK: Compare unsafe vs safe parsing
        let messages = (0..10000)
            .map(|i| create_test_message(&format!("key_{}", i), &format!("value_{}", i).into_bytes()))
            .flatten()
            .collect::<Vec<u8>>();
        
        let start = std::time::Instant::now();
        let mut count = 0;
        for msg in NetworkMessage::parse_batch(&messages) {
            match msg {
                Ok(_) => count += 1,
                Err(_) => break,
            }
        }
        let unsafe_duration = start.elapsed();
        
        println!("Parsed {} messages in {:?}", count, unsafe_duration);
        println!("Per message: {:?}", unsafe_duration / count);
        
        // TODO: Compare with safe bounds-checking implementation
        // TODO: Measure memory allocation differences
        // TODO: Profile with perf to measure cache performance
    }
    
    #[test]
    fn safety_stress_test() {
        // YOUR TESTS: Verify safety with malformed input
        let test_cases = vec![
            vec![],                    // Empty buffer
            vec![1, 2, 3],            // Too short
            vec![10, 0, 0, 0, 1, 2],  // Length mismatch
            vec![4, 0, 0, 0, 255, 255], // Invalid key length
        ];
        
        for malformed in test_cases {
            // Should return error, not crash
            assert!(NetworkMessage::parse(&malformed).is_err());
        }
        
        // TODO: Add fuzzing integration for comprehensive safety testing
    }
}
```

**Network Parser Requirements:**
1. **Zero-Copy Design**: All parsing must avoid allocations
2. **Bounds Safety**: Malformed input must be handled safely
3. **Performance Measurement**: Quantify unsafe optimization benefits
4. **Batch Processing**: Efficient parsing of multiple messages
5. **Safety Verification**: Comprehensive testing including fuzzing

### Problem 4.2: Production Integration and Monitoring (10 points)

Integrate unsafe optimizations into a production system with monitoring:

```rust
// Production system: High-frequency log processing
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct UnsafeOptimizedLogProcessor {
    // YOUR DESIGN: Production-ready system using unsafe optimizations
    // Requirements: observability, error handling, graceful degradation
    
    // Performance counters (atomic for thread safety)
    messages_processed: AtomicU64,
    unsafe_optimizations_used: AtomicU64,
    safety_fallbacks_triggered: AtomicU64,
    
    // TODO: Add your unsafe optimization state
}

impl UnsafeOptimizedLogProcessor {
    pub fn new() -> Self {
        Self {
            messages_processed: AtomicU64::new(0),
            unsafe_optimizations_used: AtomicU64::new(0),
            safety_fallbacks_triggered: AtomicU64::new(0),
        }
    }
    
    // TASK 1: Production-ready processing with unsafe optimizations
    pub fn process_log_batch(&self, logs: &[&[u8]]) -> Result<ProcessingStats, ProcessingError> {
        let mut stats = ProcessingStats::default();
        
        for log_data in logs {
            match self.process_single_log_unsafe(log_data) {
                Ok(unsafe_stats) => {
                    // Unsafe optimization succeeded
                    stats.merge(unsafe_stats);
                    self.unsafe_optimizations_used.fetch_add(1, Ordering::Relaxed);
                }
                Err(UnsafeError::InvalidInput) => {
                    // Fall back to safe processing
                    match self.process_single_log_safe(log_data) {
                        Ok(safe_stats) => {
                            stats.merge(safe_stats);
                            self.safety_fallbacks_triggered.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(UnsafeError::Critical(e)) => {
                    // Critical error - stop processing
                    return Err(ProcessingError::Critical(e));
                }
            }
            
            self.messages_processed.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(stats)
    }
    
    // TASK 2: Unsafe fast path with monitoring
    fn process_single_log_unsafe(&self, data: &[u8]) -> Result<LogStats, UnsafeError> {
        // YOUR IMPLEMENTATION: Fast path using unsafe optimizations
        // Requirements:
        // 1. Validate input before using unsafe
        // 2. Monitor unsafe operation success/failure
        // 3. Provide detailed error information for fallback decisions
        // 4. Ensure no undefined behavior even on invalid input
        
        // Quick validation before unsafe operations
        if data.len() < 10 {
            return Err(UnsafeError::InvalidInput);
        }
        
        // Use your unsafe parsing/processing optimizations here
        unsafe {
            // Safety: TODO - document all safety requirements
            // Monitor: increment counters for observability
            todo!()
        }
    }
    
    // TASK 3: Safe fallback implementation
    fn process_single_log_safe(&self, data: &[u8]) -> Result<LogStats, ProcessingError> {
        // YOUR IMPLEMENTATION: Safe fallback when unsafe path fails
        // Must produce same results as unsafe path but with safety
        
        todo!()
    }
    
    // TASK 4: Production monitoring interface
    pub fn get_metrics(&self) -> ProcessingMetrics {
        ProcessingMetrics {
            total_processed: self.messages_processed.load(Ordering::Relaxed),
            unsafe_optimizations: self.unsafe_optimizations_used.load(Ordering::Relaxed),
            safety_fallbacks: self.safety_fallbacks_triggered.load(Ordering::Relaxed),
            
            // Calculate success rates
            unsafe_success_rate: {
                let total = self.messages_processed.load(Ordering::Relaxed);
                let unsafe_used = self.unsafe_optimizations_used.load(Ordering::Relaxed);
                if total > 0 {
                    (unsafe_used as f64) / (total as f64)
                } else {
                    0.0
                }
            },
            
            // TODO: Add more performance metrics
        }
    }
    
    // TASK 5: Health check for production deployment
    pub fn health_check(&self) -> HealthStatus {
        let metrics = self.get_metrics();
        
        // Define health criteria
        let unsafe_success_rate_threshold = 0.95; // 95% of operations should use unsafe path
        let total_processed_threshold = 1000;     // Minimum activity level
        
        if metrics.total_processed < total_processed_threshold {
            return HealthStatus::Unknown; // Not enough data
        }
        
        if metrics.unsafe_success_rate < unsafe_success_rate_threshold {
            return HealthStatus::Degraded {
                reason: format!(
                    "Unsafe optimization success rate too low: {:.2}%", 
                    metrics.unsafe_success_rate * 100.0
                ),
                fallback_rate: 1.0 - metrics.unsafe_success_rate,
            };
        }
        
        HealthStatus::Healthy
    }
}

// Supporting types for production integration
#[derive(Debug)]
pub struct ProcessingMetrics {
    pub total_processed: u64,
    pub unsafe_optimizations: u64,
    pub safety_fallbacks: u64,  
    pub unsafe_success_rate: f64,
    // TODO: Add latency histograms, error rates, etc.
}

#[derive(Debug)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String, fallback_rate: f64 },
    Unknown,
}

// TASK 6: Production testing and validation
#[cfg(test)]
mod production_integration_tests {
    use super::*;
    
    #[test]
    fn test_production_resilience() {
        let processor = UnsafeOptimizedLogProcessor::new();
        
        // Mix of valid and invalid log data
        let test_data = vec![
            b"valid log entry 1".as_slice(),
            b"".as_slice(), // Empty - should trigger fallback
            b"valid log entry 2".as_slice(),
            b"malformed\x00\xFF\x00".as_slice(), // Binary junk - should trigger fallback
            b"valid log entry 3".as_slice(),
        ];
        
        let result = processor.process_log_batch(&test_data).unwrap();
        let metrics = processor.get_metrics();
        
        // Verify system handled mix gracefully
        assert_eq!(metrics.total_processed, 5);
        assert!(metrics.safety_fallbacks > 0); // Some fallbacks triggered
        assert!(metrics.unsafe_optimizations > 0); // Some unsafe optimizations used
        
        // Verify health status
        let health = processor.health_check();
        println!("Health status: {:?}", health);
    }
    
    #[test]
    fn production_load_test() {
        // YOUR TEST: Simulate production load
        let processor = Arc::new(UnsafeOptimizedLogProcessor::new());
        
        // Generate realistic log data
        let log_data: Vec<Vec<u8>> = (0..100_000)
            .map(|i| format!("LOG INFO Processing request {} timestamp={}", i, i * 1000).into_bytes())
            .collect();
        
        let refs: Vec<&[u8]> = log_data.iter().map(|v| v.as_slice()).collect();
        
        // Process under load
        let start = std::time::Instant::now();
        let stats = processor.process_log_batch(&refs).unwrap();
        let duration = start.elapsed();
        
        println!("Processed {} logs in {:?}", log_data.len(), duration);
        println!("Throughput: {:.0} logs/sec", log_data.len() as f64 / duration.as_secs_f64());
        
        let metrics = processor.get_metrics();
        println!("Metrics: {:?}", metrics);
        
        // Verify performance targets
        let logs_per_second = log_data.len() as f64 / duration.as_secs_f64();
        assert!(logs_per_second > 10_000.0, "Performance target not met"); // 10k logs/sec minimum
    }
}
```

**Production Integration Requirements:**
1. **Graceful Degradation**: Safe fallback when unsafe optimizations fail
2. **Observability**: Comprehensive metrics for monitoring unsafe code behavior  
3. **Health Checks**: Automated health assessment for production deployment
4. **Error Handling**: Proper classification of unsafe vs general errors
5. **Performance Targets**: Quantified performance improvements under production load

---

## Unsafe Rust Mastery Assessment

### Scoring Rubric

**Unsafe Expert (126-140 points)**
- Writes unsafe code with complete safety proofs and comprehensive verification
- Demonstrates deep understanding of memory layout and performance implications
- Creates production-ready unsafe optimizations with proper monitoring and fallbacks
- Shows mastery of FFI safety and complex lifetime scenarios

**Unsafe Proficient (112-125 points)**
- Writes generally safe unsafe code with good safety documentation
- Understands basic memory layout optimization principles
- Can integrate unsafe code safely but may lack comprehensive monitoring
- Handles basic FFI scenarios correctly

**Unsafe Developing (98-111 points)**
- Writes unsafe code that compiles but may have subtle safety issues  
- Limited understanding of memory layout impact on performance
- Unsafe optimizations may not provide measurable benefits
- FFI integration has basic functionality but lacks comprehensive safety

**Unsafe Novice (84-97 points)**
- Struggles with unsafe code safety invariants and verification
- Does not understand memory layout optimization
- Cannot justify performance benefits of unsafe code
- FFI integration has safety issues

**Insufficient (<84 points)**
- Cannot write safe unsafe code
- No understanding of performance optimization principles
- Unsafe code likely has undefined behavior
- FFI integration is dangerous or non-functional

---

## Bridge to Advanced Concepts

**You're Ready for Advanced Unsafe Rust & Memory Mastery (06_unsafe_rust_mastery.md) When:**

✅ You can write unsafe code with complete safety proofs  
✅ You understand memory layout optimization and can measure improvements  
✅ You can create safe FFI bindings with performance optimization  
✅ You can integrate unsafe optimizations into production systems safely  
✅ You can verify safety through testing, documentation, and tooling

**Unsafe Rust Mastery Checklist:**
- [ ] Every unsafe block has complete safety invariant documentation
- [ ] All unsafe optimizations are benchmarked to prove performance benefits
- [ ] Memory layout optimizations are measured with cache performance tools
- [ ] FFI bindings handle all error conditions and resource management safely
- [ ] Production integration includes monitoring, fallbacks, and health checks
- [ ] Safety verification includes testing, fuzzing, and static analysis

---

## Next Steps by Unsafe Proficiency Level

**Expert Level**: Ready for `06_unsafe_rust_mastery.md` - Advanced Unsafe Rust & Memory Mastery. You can handle the most complex memory layout optimizations and lock-free programming.

**Proficient Level**: Practice more complex unsafe scenarios. Focus on production integration and comprehensive safety verification before advancing.

**Developing Level**: Master the fundamentals of memory layout and FFI safety. Ensure all unsafe code has proper safety proofs before advancing.

**Novice Level**: Work through basic unsafe Rust tutorials and memory management concepts. Build solid foundation before attempting advanced techniques.

---

*"Unsafe code is not about being reckless - it's about being more careful than ever. Every unsafe block is a contract with the compiler that you will uphold memory safety yourself."*

**Required Tools for This Course:**
```bash
# Install safety verification tools
cargo install cargo-fuzz
cargo install cargo-miri  
sudo apt install valgrind
# AddressSanitizer: export RUSTFLAGS=-Zsanitizer=address
# Build with: cargo +nightly build --target x86_64-unknown-linux-gnu
```