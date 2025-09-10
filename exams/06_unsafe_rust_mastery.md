# Advanced Unsafe Rust & Memory Mastery

**Expert-Level Systems Programming Assessment**

*"Unsafe Rust is where memory safety becomes your responsibility - you must maintain the invariants that safe Rust enforces automatically."*

---

## Prerequisites

You must pass the **Advanced Systems Diagnostic (05_advanced_systems_diagnostic.md)** before attempting this exam. This tests the **hardest concepts** in Rust - manual memory management while maintaining safety guarantees.

**Core Understanding Required:**
- **Unsafe Rust**: Manual memory safety, raw pointers, transmute
- **Memory Layout**: repr, alignment, padding, cache optimization  
- **Safety Invariants**: Building safe abstractions over unsafe code
- **Zero-Copy Mastery**: Raw pointer manipulation, lifetime erasure

---

## Section I: Advanced Unsafe Rust (150 points)

### Problem 1.1: Custom Allocator Implementation (50 points)

**Context**: Custom allocators are essential for high-performance systems. You must maintain memory safety while managing raw memory.

```rust
use std::alloc::{GlobalAlloc, Layout};
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// A bump allocator that never deallocates individual allocations
/// Used for short-lived computation phases where you allocate then free everything
pub struct BumpAllocator {
    start: AtomicPtr<u8>,
    current: AtomicPtr<u8>, 
    end: AtomicPtr<u8>,
    allocated_bytes: AtomicUsize,
}

impl BumpAllocator {
    /// Create a new bump allocator with the given capacity
    /// Safety: Must handle all edge cases correctly
    pub fn new(capacity: usize) -> Self {
        // Your implementation:
        // 1. Allocate aligned memory block
        // 2. Initialize atomic pointers correctly  
        // 3. Handle alignment requirements
        // 4. What happens if capacity is 0?
        
        todo!("Implement safe constructor")
    }
    
    /// Allocate memory with given layout
    /// Safety: Must maintain memory safety and alignment invariants
    unsafe fn alloc_impl(&self, layout: Layout) -> *mut u8 {
        // Your implementation:
        // 1. Handle alignment correctly (critical!)
        // 2. Check for overflow
        // 3. Atomic operations for thread safety
        // 4. What ordering guarantees do you need?
        
        let size = layout.size();
        let align = layout.align();
        
        // Critical: How do you handle alignment with atomic operations?
        todo!("Implement thread-safe aligned allocation")
    }
    
    /// Reset the allocator - makes all previous allocations invalid
    /// Safety: Caller must ensure no outstanding references exist
    pub unsafe fn reset(&self) {
        // Your implementation:
        // 1. Reset current pointer to start
        // 2. What memory ordering is required?
        // 3. How do you handle concurrent reset during allocation?
        
        todo!("Implement safe reset")
    }
    
    /// Get current memory usage statistics  
    pub fn stats(&self) -> AllocatorStats {
        // Your implementation:
        // 1. Calculate used bytes atomically
        // 2. Handle race conditions in reading statistics
        
        todo!("Implement consistent statistics")
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_impl(layout)
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocators don't deallocate individual allocations
        // Is this a no-op? What are the safety implications?
    }
}

unsafe impl Send for BumpAllocator {}
unsafe impl Sync for BumpAllocator {}

// Your safety proof:
// 1. Why is it safe to implement Send + Sync?
// 2. What invariants must the allocator maintain?
// 3. How do you prevent use-after-reset bugs?
// 4. What happens with concurrent allocations?

pub struct AllocatorStats {
    pub used_bytes: usize,
    pub peak_bytes: usize,
    pub total_capacity: usize,
    pub allocation_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_concurrent_allocation() {
        // Test that concurrent allocation from multiple threads works correctly
        // Must verify no memory corruption or alignment violations
    }
    
    #[test]
    fn test_alignment_invariants() {
        // Test various alignment requirements (1, 2, 4, 8, 16, 32 bytes)
        // Verify all returned pointers are properly aligned
    }
    
    #[test] 
    fn test_reset_safety() {
        // Test that reset invalidates all previous allocations
        // How do you test for use-after-reset without UB?
    }
}
```

**Questions (50 points):**
1. **Memory Ordering Analysis (15 points)**: Which atomic ordering is required for each operation and why?
2. **Alignment Handling (15 points)**: How do you ensure returned pointers meet alignment requirements atomically?
3. **Safety Invariant Proof (10 points)**: Prove that your implementation maintains memory safety
4. **Performance Analysis (10 points)**: What's the time complexity? How does it compare to malloc?

### Problem 1.2: Zero-Copy Network Protocol Parser (50 points)

**Context**: Parsing network protocols without copying data requires careful unsafe code.

```rust
use std::mem;
use std::ptr;
use std::slice;
use std::marker::PhantomData;

/// A zero-copy parser for binary network protocols
/// Must handle endianness, alignment, and bounds checking safely
pub struct ProtocolParser<'data> {
    data: &'data [u8],
    position: usize,
    _phantom: PhantomData<&'data ()>,
}

impl<'data> ProtocolParser<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            position: 0,
            _phantom: PhantomData,
        }
    }
    
    /// Parse a value of type T from the buffer
    /// Safety: T must be safely transmutable from bytes
    unsafe fn read_unaligned<T>(&mut self) -> Result<T, ParseError>
    where
        T: Copy + 'static,
    {
        // Your implementation:
        // 1. Check bounds
        // 2. Handle potential alignment issues  
        // 3. Read without creating invalid references
        // 4. Handle endianness if needed
        
        let size = mem::size_of::<T>();
        if self.position + size > self.data.len() {
            return Err(ParseError::UnexpectedEof);
        }
        
        // Critical: How do you safely read unaligned data?
        // ptr::read_unaligned? transmute? Something else?
        todo!("Implement safe unaligned read")
    }
    
    /// Parse a string of known length without copying
    /// Returns a reference into the original buffer
    pub fn read_str(&mut self, len: usize) -> Result<&'data str, ParseError> {
        // Your implementation:
        // 1. Check bounds
        // 2. Validate UTF-8
        // 3. Return reference with correct lifetime
        // 4. Handle the case where len=0
        
        todo!("Implement zero-copy string parsing")
    }
    
    /// Parse an array of values without copying
    /// Safety: T must be safely transmutable and properly aligned
    unsafe fn read_array<T>(&mut self, count: usize) -> Result<&'data [T], ParseError>
    where
        T: Copy + 'static,
    {
        // Your implementation:
        // 1. Check bounds and alignment
        // 2. Create slice from raw pointer
        // 3. What are the safety requirements for slice::from_raw_parts?
        // 4. Handle count=0 case
        
        todo!("Implement zero-copy array parsing")
    }
    
    pub fn remaining(&self) -> usize {
        self.data.len() - self.position
    }
}

// Define a complex network packet structure
#[repr(C, packed)]
struct PacketHeader {
    magic: u32,           // Magic number (little-endian)
    version: u8,
    flags: u8,
    payload_length: u16,  // Big-endian
    checksum: u32,        // CRC32 (little-endian)
    timestamp: u64,       // Unix timestamp (big-endian)
}

// Your tasks:
impl<'data> ProtocolParser<'data> {
    /// Parse packet header with proper endianness handling
    pub fn parse_header(&mut self) -> Result<PacketHeader, ParseError> {
        // Must handle:
        // 1. Packed struct alignment issues
        // 2. Endianness conversion
        // 3. Validation of magic number and version
        
        todo!("Implement header parsing with endianness")
    }
    
    /// Parse variable-length payload based on header
    pub fn parse_payload(&mut self, header: &PacketHeader) -> Result<&'data [u8], ParseError> {
        // Must handle:
        // 1. Length validation against remaining buffer
        // 2. Checksum verification
        // 3. Zero-copy return of payload slice
        
        todo!("Implement payload parsing with validation")
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEof,
    InvalidMagic,
    UnsupportedVersion,
    ChecksumMismatch,
    InvalidUtf8,
    AlignmentError,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unaligned_access() {
        // Test reading u32 from unaligned buffer positions
        let buffer = [0u8; 100];
        // How do you test alignment safety without triggering UB?
    }
    
    #[test]
    fn test_endianness_handling() {
        // Test that big-endian fields are correctly converted
        let packet_bytes = create_test_packet();
        // Verify values match expected after endianness conversion
    }
    
    #[test]
    fn test_bounds_checking() {
        // Test all edge cases: empty buffer, truncated packets, etc.
    }
}
```

**Questions (50 points):**
1. **Alignment Safety (15 points)**: How do you safely read packed structs with potential alignment issues?
2. **Lifetime Correctness (15 points)**: Prove that returned references have correct lifetimes
3. **Endianness Handling (10 points)**: Implement efficient big-endian to native conversion
4. **Bounds Checking Strategy (10 points)**: Design a strategy to prevent buffer overruns

### Problem 1.3: Safe Abstraction Over Unsafe Ring Buffer (50 points)

**Context**: Building a safe, high-performance single-producer single-consumer ring buffer.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ptr;
use std::mem::{self, MaybeUninit};
use std::alloc::{alloc, dealloc, Layout};

/// Single-producer, single-consumer ring buffer with zero-copy semantics
/// Must be completely safe to use from the public API
pub struct RingBuffer<T> {
    buffer: *mut MaybeUninit<T>,
    capacity: usize,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    layout: Layout,
}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with given capacity (must be power of 2)
    pub fn new(capacity: usize) -> Self {
        // Your implementation:
        // 1. Allocate raw memory
        // 2. Initialize atomic positions
        // 3. Handle capacity validation
        // 4. What if T has special alignment requirements?
        
        assert!(capacity.is_power_of_two(), "Capacity must be power of 2");
        
        todo!("Implement safe constructor")
    }
    
    /// Try to push a value (non-blocking)
    /// Returns Ok(()) if pushed, Err(value) if buffer full
    pub fn try_push(&self, value: T) -> Result<(), T> {
        // Your implementation:
        // 1. Check if buffer is full (atomically)
        // 2. Write value to buffer slot
        // 3. Update write position atomically
        // 4. What memory ordering is required?
        
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        
        // How do you check if buffer is full with wraparound?
        if (write_pos + 1) & (self.capacity - 1) == read_pos {
            return Err(value);
        }
        
        // Critical: How do you safely write to uninitialized memory?
        unsafe {
            // Your unsafe implementation here
            todo!("Implement safe value writing")
        }
    }
    
    /// Try to pop a value (non-blocking)
    /// Returns Some(value) if available, None if buffer empty
    pub fn try_pop(&self) -> Option<T> {
        // Your implementation:
        // 1. Check if buffer is empty (atomically)
        // 2. Read value from buffer slot  
        // 3. Update read position atomically
        // 4. Handle MaybeUninit correctly
        
        todo!("Implement safe value reading")
    }
    
    /// Get current number of elements in buffer
    pub fn len(&self) -> usize {
        // How do you calculate length with potential wraparound?
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        
        todo!("Calculate length with wraparound")
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity - 1 // Why subtract 1?
    }
    
    pub fn is_empty(&self) -> bool {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        write_pos == read_pos
    }
}

// Critical: Implement safe cleanup
impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        // Your implementation:
        // 1. Drop all unconsumed values
        // 2. Deallocate buffer memory
        // 3. Handle the case where T has a custom Drop impl
        
        todo!("Implement safe cleanup")
    }
}

// Safety analysis required:
unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Send> Sync for RingBuffer<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    
    #[test]
    fn test_single_threaded_correctness() {
        // Test basic push/pop operations
    }
    
    #[test]
    fn test_concurrent_producer_consumer() {
        // Test with actual producer/consumer threads
        let buffer = Arc::new(RingBuffer::new(1024));
        
        // Spawn producer thread
        // Spawn consumer thread  
        // Verify no data corruption or loss
    }
    
    #[test]
    fn test_drop_safety() {
        // Ensure all values are properly dropped
        // Test with types that have custom Drop implementations
    }
    
    #[test]
    fn test_memory_ordering() {
        // How do you test memory ordering correctness?
        // Consider using loom for testing
    }
}
```

**Questions (50 points):**
1. **Memory Ordering Requirements (20 points)**: What ordering is required for each atomic operation and why?
2. **MaybeUninit Safety (15 points)**: How do you safely handle uninitialized memory?
3. **Drop Safety Analysis (15 points)**: Prove that your Drop implementation is correct and safe

---

## Section II: Memory Layout Mastery (100 points)

### Problem 2.1: SIMD-Optimized Data Structure (50 points)

**Context**: Design a data structure optimized for SIMD operations with precise memory layout control.

```rust
use std::arch::x86_64::*;
use std::mem::{align_of, size_of};

/// A collection of 3D vectors optimized for SIMD operations
/// Must guarantee proper alignment and memory layout for AVX2
#[repr(align(32))]  // AVX2 requires 32-byte alignment
pub struct Vector3Array {
    // Your design:
    // 1. How do you lay out x, y, z components for efficient SIMD?
    // 2. Structure of Arrays (SoA) vs Array of Structures (AoS)?
    // 3. How do you handle remainder elements when length isn't multiple of 8?
    
    data: Vec<f32>,
    len: usize,
    layout: MemoryLayout,
}

#[derive(Debug, Clone)]
pub enum MemoryLayout {
    ArrayOfStructures,    // [x1,y1,z1, x2,y2,z2, ...]
    StructureOfArrays,    // [x1,x2,x3,... y1,y2,y3,... z1,z2,z3,...]
}

impl Vector3Array {
    /// Create new array with specified layout and capacity
    pub fn new(capacity: usize, layout: MemoryLayout) -> Self {
        // Your implementation:
        // 1. Ensure proper alignment for SIMD
        // 2. Pre-allocate based on layout choice
        // 3. Handle the case where capacity is not multiple of 8
        
        todo!("Implement SIMD-aligned allocation")
    }
    
    /// Add a 3D vector to the collection
    pub fn push(&mut self, x: f32, y: f32, z: f32) {
        // Your implementation depends on memory layout
        match self.layout {
            MemoryLayout::ArrayOfStructures => {
                // How do you maintain SIMD alignment when growing?
                todo!("Implement AoS push")
            }
            MemoryLayout::StructureOfArrays => {
                // How do you coordinate updates across separate arrays?
                todo!("Implement SoA push")  
            }
        }
    }
    
    /// Compute dot products using SIMD (AVX2)
    /// Returns dot product of each vector with the given vector
    pub fn dot_products_simd(&self, other: &[f32; 3]) -> Vec<f32> {
        // Your implementation:
        // 1. Load other vector into SIMD register (broadcast)
        // 2. Process 8 vectors at a time using AVX2
        // 3. Handle remainder vectors with scalar code
        // 4. Ensure memory accesses are aligned
        
        unsafe {
            match self.layout {
                MemoryLayout::ArrayOfStructures => {
                    self.dot_products_aos_simd(other)
                }
                MemoryLayout::StructureOfArrays => {
                    self.dot_products_soa_simd(other)
                }
            }
        }
    }
    
    /// SIMD implementation for Array of Structures layout
    unsafe fn dot_products_aos_simd(&self, other: &[f32; 3]) -> Vec<f32> {
        // Your implementation:
        // 1. How do you efficiently load interleaved x,y,z data?
        // 2. Use _mm256_loadu_ps for potentially unaligned loads
        // 3. Shuffle operations to separate x, y, z components
        // 4. Multiply and add using _mm256_fmadd_ps
        
        todo!("Implement AoS SIMD dot products")
    }
    
    /// SIMD implementation for Structure of Arrays layout  
    unsafe fn dot_products_soa_simd(&self, other: &[f32; 3]) -> Vec<f32> {
        // Your implementation:
        // 1. Load 8 x-components, 8 y-components, 8 z-components
        // 2. Broadcast other[0], other[1], other[2] into SIMD registers
        // 3. Use _mm256_fmadd_ps for fused multiply-add
        // 4. This should be more efficient than AoS - why?
        
        todo!("Implement SoA SIMD dot products")
    }
    
    /// Benchmark both layouts and SIMD vs scalar implementations
    pub fn benchmark(&self) -> PerformanceResults {
        // Your implementation:
        // 1. Compare SIMD vs scalar performance
        // 2. Compare AoS vs SoA layout performance
        // 3. Measure memory bandwidth utilization
        
        todo!("Implement comprehensive benchmarking")
    }
}

pub struct PerformanceResults {
    pub simd_ops_per_second: f64,
    pub scalar_ops_per_second: f64, 
    pub memory_bandwidth_gb_s: f64,
    pub simd_speedup: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_alignment() {
        let array = Vector3Array::new(100, MemoryLayout::StructureOfArrays);
        // How do you verify proper alignment?
        // Check that data pointer is 32-byte aligned
    }
    
    #[test]
    fn test_simd_correctness() {
        // Verify SIMD results match scalar computation
        let array = create_test_vectors(1000);
        let simd_results = array.dot_products_simd(&[1.0, 2.0, 3.0]);
        let scalar_results = array.dot_products_scalar(&[1.0, 2.0, 3.0]);
        
        // Compare with appropriate floating-point tolerance
    }
    
    #[test]
    fn test_performance() {
        // Benchmark different configurations and verify SIMD is faster
    }
}
```

**Questions (50 points):**
1. **Memory Layout Analysis (20 points)**: Compare AoS vs SoA for SIMD performance. Which is better and why?
2. **SIMD Implementation (20 points)**: Implement efficient AVX2 dot product computation
3. **Performance Measurement (10 points)**: Design benchmarks that accurately measure SIMD benefits

### Problem 2.2: Cache-Optimized Hash Table (50 points)

**Context**: Design a hash table optimized for CPU cache performance.

```rust
use std::mem;
use std::ptr;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache-optimized hash table using Robin Hood hashing
/// Optimized for cache line utilization and minimal pointer chasing
#[repr(align(64))]  // Align to cache line
pub struct CacheHashMap<K, V> {
    // Your design choices:
    // 1. How do you minimize cache misses during lookup?
    // 2. Should metadata be separate from key-value pairs?
    // 3. How do you handle cache line splits?
    
    buckets: Vec<Bucket<K, V>>,
    len: usize,
    capacity: usize,
}

/// Each bucket should fit in a cache line (64 bytes)
/// Design this for optimal cache utilization
#[repr(C)]
struct Bucket<K, V> {
    // Your design:
    // 1. What metadata do you need per bucket?
    // 2. How do you pack key-value pairs efficiently?
    // 3. Consider probe distance for Robin Hood hashing
    
    // Possible fields:
    // - hash: u32 (or u64?)
    // - probe_distance: u8
    // - occupied: bool (or use hash=0 as empty?)
    // - key: K
    // - value: V
    
    // Challenge: How do you handle different sizes of K and V?
    todo!("Design cache-optimal bucket layout")
}

impl<K, V> CacheHashMap<K, V> 
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self::with_capacity(16)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        // Your implementation:
        // 1. Round capacity up to next power of 2
        // 2. Allocate aligned memory for cache performance
        // 3. Initialize all buckets as empty
        
        todo!("Implement cache-aligned allocation")
    }
    
    /// Insert key-value pair using Robin Hood hashing
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Your implementation:
        // 1. Compute hash and initial bucket
        // 2. Robin Hood probing: steal from rich, give to poor
        // 3. Handle resize when load factor gets too high
        // 4. Minimize cache misses during probing
        
        todo!("Implement Robin Hood insertion")
    }
    
    /// Look up value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        // Your implementation:
        // 1. Compute hash and start probing
        // 2. Use probe distance to terminate search early
        // 3. Prefetch next cache line during probing?
        // 4. How do you minimize branch mispredictions?
        
        todo!("Implement cache-optimized lookup")
    }
    
    /// Remove key-value pair
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // Your implementation:
        // 1. Find the key using same probing as get()
        // 2. Shift subsequent entries backward (Robin Hood deletion)
        // 3. Handle tombstones vs compaction tradeoff
        
        todo!("Implement Robin Hood deletion")
    }
    
    /// Resize hash table when load factor is too high
    fn resize(&mut self) {
        // Your implementation:
        // 1. Allocate new bucket array (double size)
        // 2. Rehash all existing entries
        // 3. How do you minimize memory usage during resize?
        
        todo!("Implement resize with minimal memory overhead")
    }
    
    /// Get cache performance statistics
    pub fn cache_stats(&self) -> CacheStats {
        // Your implementation:
        // 1. Calculate average probe distance
        // 2. Estimate cache line utilization
        // 3. Identify clustering patterns
        
        todo!("Implement cache performance analysis")
    }
}

pub struct CacheStats {
    pub average_probe_distance: f64,
    pub max_probe_distance: usize,
    pub cache_line_utilization: f64,
    pub clustering_factor: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_alignment() {
        let map: CacheHashMap<u64, u64> = CacheHashMap::new();
        // Verify that buckets are cache-line aligned
        let bucket_ptr = map.buckets.as_ptr() as usize;
        assert_eq!(bucket_ptr % 64, 0, "Buckets not cache-aligned");
    }
    
    #[test]
    fn test_robin_hood_invariants() {
        // Test that Robin Hood invariants are maintained
        // - No entry is more than 1 probe distance further than optimal
        // - Probe distances form a non-decreasing sequence
    }
    
    #[test]
    fn benchmark_cache_performance() {
        // Compare against std::collections::HashMap
        // Measure cache misses using performance counters
        // Verify better performance for cache-friendly access patterns
    }
}
```

**Questions (50 points):**
1. **Cache Line Optimization (20 points)**: Design bucket layout to maximize cache line utilization
2. **Robin Hood Analysis (15 points)**: Explain why Robin Hood hashing improves cache performance
3. **Performance Measurement (15 points)**: How do you measure cache miss rates in benchmarks?

---

## Section III: Safety Invariant Design (100 points)

### Problem 3.1: Safe FFI Wrapper (50 points)

**Context**: Create a safe Rust wrapper around a C library with complex memory management.

```rust
use std::os::raw::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::ptr::{self, NonNull};
use std::marker::PhantomData;

// External C library (simulated)
extern "C" {
    /// Create a new database connection
    /// Returns NULL on failure
    fn db_connect(host: *const c_char, port: c_int) -> *mut c_void;
    
    /// Close database connection and free resources
    /// Connection pointer becomes invalid after this call
    fn db_close(conn: *mut c_void);
    
    /// Execute query and return result handle
    /// Returns NULL on failure
    /// Result must be freed with db_result_free
    fn db_query(conn: *mut c_void, sql: *const c_char) -> *mut c_void;
    
    /// Free query result
    /// Result pointer becomes invalid after this call
    fn db_result_free(result: *mut c_void);
    
    /// Get next row from result
    /// Returns 0 when no more rows, -1 on error, 1 on success
    /// Row pointer is valid until next call to db_next_row or db_result_free
    fn db_next_row(result: *mut c_void, row: *mut *mut c_char) -> c_int;
}

/// Safe wrapper around C database library
/// Must prevent all possible memory safety violations
pub struct Database {
    // Your design:
    // 1. How do you store the raw C pointer safely?
    // 2. How do you prevent use-after-close?
    // 3. What lifetime relationships need to be enforced?
    
    conn: NonNull<c_void>,
    _phantom: PhantomData<*mut c_void>, // Not Send/Sync
}

impl Database {
    /// Connect to database with safe error handling
    pub fn connect(host: &str, port: u16) -> Result<Self, DatabaseError> {
        // Your implementation:
        // 1. Convert Rust string to C string safely
        // 2. Handle NULL return from C function
        // 3. Ensure connection is properly validated
        
        let host_cstr = CString::new(host)
            .map_err(|_| DatabaseError::InvalidHost)?;
            
        let conn_ptr = unsafe {
            db_connect(host_cstr.as_ptr(), port as c_int)
        };
        
        // How do you safely wrap the raw pointer?
        todo!("Implement safe connection wrapping")
    }
    
    /// Execute SQL query with safe result handling
    pub fn query(&self, sql: &str) -> Result<QueryResult, DatabaseError> {
        // Your implementation:
        // 1. Convert SQL to C string
        // 2. Handle NULL result from C function
        // 3. Ensure result has correct lifetime relationship to database
        
        todo!("Implement safe query execution")
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // Your implementation:
        // 1. Call db_close with the raw pointer
        // 2. Ensure this is safe even if called multiple times
        
        unsafe {
            db_close(self.conn.as_ptr());
        }
    }
}

/// Safe wrapper around query results
/// Must ensure rows are not accessed after result is dropped
pub struct QueryResult<'db> {
    // Your design:
    // 1. How do you tie the lifetime to the database?
    // 2. How do you prevent iterator invalidation?
    // 3. What invariants must be maintained?
    
    result: NonNull<c_void>,
    _database: PhantomData<&'db Database>,
}

impl<'db> QueryResult<'db> {
    /// Create safe result wrapper
    /// Safety: result_ptr must be valid and owned by this instance
    unsafe fn new(result_ptr: *mut c_void, _db: &'db Database) -> Result<Self, DatabaseError> {
        // Your implementation:
        // 1. Validate the pointer is not null
        // 2. Create NonNull wrapper
        // 3. Set up lifetime relationship
        
        todo!("Implement safe result wrapping")
    }
}

impl<'db> Drop for QueryResult<'db> {
    fn drop(&mut self) {
        // Your implementation:
        // 1. Call db_result_free safely
        // 2. Handle the case where this might be called multiple times
        
        todo!("Implement safe result cleanup")
    }
}

/// Iterator over database rows
/// Must ensure row data is not accessed after iterator is dropped
pub struct RowIterator<'result> {
    // Your design:
    // 1. How do you tie lifetime to QueryResult?
    // 2. How do you handle the C library's stateful iteration?
    // 3. What happens if the underlying result is dropped?
    
    result: &'result QueryResult<'result>,
    current_row: Option<NonNull<c_char>>,
}

impl<'result> Iterator for RowIterator<'result> {
    type Item = Result<DatabaseRow<'result>, DatabaseError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Your implementation:
        // 1. Call db_next_row safely
        // 2. Handle the three possible return values
        // 3. Ensure returned row has correct lifetime
        
        let mut row_ptr: *mut c_char = ptr::null_mut();
        let result = unsafe {
            db_next_row(self.result.result.as_ptr(), &mut row_ptr)
        };
        
        match result {
            0 => None, // No more rows
            1 => {
                // Success - wrap the row safely
                todo!("Create safe row wrapper")
            }
            -1 => Some(Err(DatabaseError::QueryError)),
            _ => Some(Err(DatabaseError::UnknownError)),
        }
    }
}

/// Safe wrapper around a database row
/// Must ensure data is not accessed after row becomes invalid
pub struct DatabaseRow<'iter> {
    // Your design:
    // 1. How do you safely store the C string pointer?
    // 2. What lifetime guarantees are required?
    // 3. How do you handle the fact that row data can become invalid?
    
    row_data: NonNull<c_char>,
    _phantom: PhantomData<&'iter ()>,
}

impl<'iter> DatabaseRow<'iter> {
    /// Get row data as string slice
    pub fn as_str(&self) -> Result<&str, DatabaseError> {
        // Your implementation:
        // 1. Convert C string to Rust string safely
        // 2. Handle potential NULL or invalid UTF-8
        // 3. Ensure lifetime is correct
        
        unsafe {
            let cstr = CStr::from_ptr(self.row_data.as_ptr());
            cstr.to_str()
                .map_err(|_| DatabaseError::InvalidUtf8)
        }
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionFailed,
    InvalidHost,
    QueryError, 
    InvalidUtf8,
    UnknownError,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_lifecycle() {
        // Test that connection is properly cleaned up
        // How do you test Drop without accessing freed memory?
    }
    
    #[test]
    fn test_result_lifetime_safety() {
        // Test that query results can't outlive the database
        // This should fail to compile:
        // let result = {
        //     let db = Database::connect("localhost", 5432)?;
        //     db.query("SELECT 1")?
        // }; // db dropped here
        // result.rows(); // Should be compile error
    }
    
    #[test]
    fn test_row_lifetime_safety() {
        // Test that rows can't outlive the result
        // Similar lifetime safety tests for row data
    }
}
```

**Questions (50 points):**
1. **Lifetime Design (20 points)**: Design the lifetime relationships to prevent use-after-free
2. **Error Handling (15 points)**: Handle all possible C library failure modes safely  
3. **Resource Management (15 points)**: Prove that resources are always properly cleaned up

### Problem 3.2: Thread-Safe Reference Counting (50 points)

**Context**: Implement a custom Arc-like type with additional safety features.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ptr::NonNull;
use std::ops::Deref;
use std::marker::PhantomData;

/// Thread-safe reference counting with weak references and cycle detection
/// Must provide stronger safety guarantees than std::sync::Arc
pub struct SafeArc<T> {
    // Your design:
    // 1. How do you store the data and reference counts?
    // 2. What additional safety features can you provide?
    // 3. How do you detect reference cycles?
    
    inner: NonNull<SafeArcInner<T>>,
    _phantom: PhantomData<SafeArcInner<T>>,
}

struct SafeArcInner<T> {
    // Your design:
    // 1. Strong and weak reference counts
    // 2. The actual data
    // 3. Additional safety metadata?
    
    strong_count: AtomicUsize,
    weak_count: AtomicUsize,
    data: T,
    
    // Bonus features:
    // - Creation timestamp for debugging
    // - Creator thread ID
    // - Leak detection support
}

impl<T> SafeArc<T> {
    pub fn new(data: T) -> Self {
        // Your implementation:
        // 1. Allocate SafeArcInner with proper initialization
        // 2. Initialize reference counts
        // 3. Set up any additional safety features
        
        todo!("Implement safe Arc creation")
    }
    
    /// Create a weak reference that doesn't keep data alive
    pub fn downgrade(&self) -> SafeWeak<T> {
        // Your implementation:
        // 1. Increment weak count atomically
        // 2. Return weak reference with same inner pointer
        // 3. What memory ordering is required?
        
        todo!("Implement weak reference creation")
    }
    
    /// Get current strong reference count (for debugging)
    pub fn strong_count(&self) -> usize {
        self.inner().strong_count.load(Ordering::Acquire)
    }
    
    /// Get current weak reference count (for debugging)
    pub fn weak_count(&self) -> usize {
        self.inner().weak_count.load(Ordering::Acquire)  
    }
    
    /// Check if this is the only strong reference
    pub fn is_unique(&self) -> bool {
        self.strong_count() == 1
    }
    
    /// Try to get mutable access if this is the only reference
    pub fn get_mut(&mut self) -> Option<&mut T> {
        // Your implementation:
        // 1. Check if strong_count == 1
        // 2. Also check weak_count == 0? Why or why not?
        // 3. Return mutable reference if unique
        
        todo!("Implement unique mutable access")
    }
    
    /// Detect potential reference cycles (advanced feature)
    pub fn detect_cycles(&self) -> bool {
        // Your implementation:
        // This is very challenging - how do you detect cycles in arbitrary data structures?
        // Consider implementing a mark-and-sweep style algorithm
        
        todo!("Implement cycle detection (bonus)")
    }
    
    fn inner(&self) -> &SafeArcInner<T> {
        unsafe { self.inner.as_ref() }
    }
}

impl<T> Clone for SafeArc<T> {
    fn clone(&self) -> Self {
        // Your implementation:
        // 1. Increment strong count atomically
        // 2. Return new SafeArc with same inner pointer
        // 3. Handle overflow (what if count reaches usize::MAX?)
        
        let old_count = self.inner().strong_count.fetch_add(1, Ordering::Relaxed);
        
        // Handle overflow
        if old_count == usize::MAX {
            // What should happen here?
            todo!("Handle reference count overflow")
        }
        
        SafeArc {
            inner: self.inner,
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for SafeArc<T> {
    fn drop(&mut self) {
        // Your implementation:
        // 1. Decrement strong count
        // 2. If count reaches 0, drop the data
        // 3. If both strong and weak counts are 0, deallocate
        // 4. What memory ordering is required for correctness?
        
        let inner = self.inner();
        let old_strong = inner.strong_count.fetch_sub(1, Ordering::Release);
        
        if old_strong == 1 {
            // Last strong reference - drop the data
            // But what about memory ordering with weak references?
            todo!("Implement safe drop with proper ordering")
        }
    }
}

impl<T> Deref for SafeArc<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

unsafe impl<T: Send + Sync> Send for SafeArc<T> {}
unsafe impl<T: Send + Sync> Sync for SafeArc<T> {}

/// Weak reference that doesn't keep data alive
pub struct SafeWeak<T> {
    inner: NonNull<SafeArcInner<T>>,
    _phantom: PhantomData<SafeArcInner<T>>,
}

impl<T> SafeWeak<T> {
    /// Try to upgrade to strong reference
    pub fn upgrade(&self) -> Option<SafeArc<T>> {
        // Your implementation:
        // 1. Try to increment strong count (but only if > 0)
        // 2. This requires a compare-and-swap loop
        // 3. Handle race condition where data is being dropped
        
        let inner = unsafe { self.inner.as_ref() };
        let mut strong_count = inner.strong_count.load(Ordering::Relaxed);
        
        loop {
            if strong_count == 0 {
                return None; // Data already dropped
            }
            
            match inner.strong_count.compare_exchange(
                strong_count,
                strong_count + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(SafeArc {
                        inner: self.inner,
                        _phantom: PhantomData,
                    });
                }
                Err(current) => {
                    strong_count = current;
                    // Continue loop
                }
            }
        }
    }
}

impl<T> Drop for SafeWeak<T> {
    fn drop(&mut self) {
        // Your implementation:
        // 1. Decrement weak count
        // 2. If both strong and weak counts are 0, deallocate
        
        todo!("Implement SafeWeak drop")
    }
}

impl<T> Clone for SafeWeak<T> {
    fn clone(&self) -> Self {
        // Similar to SafeArc::clone but for weak count
        todo!("Implement SafeWeak clone")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    
    #[test]
    fn test_basic_functionality() {
        let arc = SafeArc::new(42);
        assert_eq!(*arc, 42);
        assert_eq!(arc.strong_count(), 1);
    }
    
    #[test]
    fn test_clone_and_drop() {
        let arc1 = SafeArc::new(String::from("hello"));
        let arc2 = arc1.clone();
        assert_eq!(arc1.strong_count(), 2);
        drop(arc1);
        assert_eq!(arc2.strong_count(), 1);
    }
    
    #[test]
    fn test_weak_references() {
        let strong = SafeArc::new(42);
        let weak = strong.downgrade();
        
        assert!(weak.upgrade().is_some());
        drop(strong);
        assert!(weak.upgrade().is_none());
    }
    
    #[test]
    fn test_concurrent_access() {
        let arc = SafeArc::new(AtomicUsize::new(0));
        let handles: Vec<_> = (0..10).map(|_| {
            let arc_clone = arc.clone();
            thread::spawn(move || {
                for _ in 0..1000 {
                    arc_clone.fetch_add(1, Ordering::Relaxed);
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(arc.load(Ordering::Relaxed), 10000);
    }
}
```

**Questions (50 points):**
1. **Memory Ordering Analysis (20 points)**: What memory ordering is required for each atomic operation?
2. **Race Condition Prevention (20 points)**: How do you prevent races between upgrade() and drop()?
3. **Safety Proof (10 points)**: Prove that your implementation prevents use-after-free

---

## Success Criteria & Evaluation

### **Pass Requirements (90% minimum):**

1. **Unsafe Rust Mastery (50 points required)**:
   - Custom allocator with proper safety invariants
   - Zero-copy parser handling alignment and endianness  
   - Safe abstractions over unsafe ring buffer

2. **Memory Layout Control (45 points required)**:
   - SIMD-optimized data structures with correct alignment
   - Cache-friendly hash table design
   - Performance analysis showing measurable improvements

3. **Safety Invariant Design (45 points required)**:
   - Safe FFI wrapper with proper lifetime management
   - Thread-safe reference counting with race condition prevention
   - Comprehensive safety proofs for all unsafe code

### **Advanced Features (Bonus)**:
- Cycle detection in reference counting (+10 points)
- NUMA-aware memory allocation (+5 points) 
- Vectorized hash table operations (+5 points)

### **If You Pass This Exam:**
You have mastered the **hardest concepts in Rust** and are ready for:
- **07_low_level_systems_mastery.md** - Advanced kernel/OS interaction
- **08_lock_free_concurrency_mastery.md** - Expert lock-free programming
- **Real-world systems programming** - Database engines, operating systems, embedded systems

### **If You Don't Pass:**
Focus on these areas before retrying:
- **The Rustonomicon** - Complete guide to unsafe Rust
- **Intel Intrinsics Guide** - For SIMD programming
- **Systems Programming practice** - Implement allocators, parsers, data structures
- **Performance analysis tools** - perf, valgrind, Intel VTune

This exam tests the **absolute hardest concepts** in Rust. Passing demonstrates expert-level systems programming capability.