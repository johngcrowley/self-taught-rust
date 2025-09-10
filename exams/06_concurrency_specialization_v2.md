# 06. Concurrency Specialization: Advanced Lock-Free Programming

**Expert-Level Lock-Free Concurrency & Memory Ordering Mastery**

*"Lock-free programming is where concurrency theory meets the harsh reality of modern CPU architectures."*

---

## Prerequisites

You must pass **03_mastery_exam_v2.md** (Advanced Unsafe Rust) and **05_systems_specialization_v2.md** (Low-Level Systems) before attempting this exam.

**Core Requirements:**
- **Lock-Free Algorithms**: Compare-and-swap, ABA problem, memory reclamation
- **Memory Ordering**: Acquire/release semantics, memory barriers, sequential consistency  
- **Performance Engineering**: When lock-free helps vs hurts performance
- **Correctness Proofs**: Formal reasoning about concurrent algorithms

---

## Section I: Lock-Free Data Structures (150 points)

### Problem 1.1: Lock-Free Stack with ABA Protection (50 points)

**Context**: Implement a lock-free stack that handles the ABA problem correctly.

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr::{self, NonNull};
use std::mem;

/// Lock-free stack that prevents ABA problem using epoch-based reclamation
/// Must handle high contention without blocking or data corruption
pub struct LockFreeStack<T> {
    // Your design:
    // 1. How do you prevent ABA problem?
    // 2. How do you reclaim memory safely?
    // 3. What about memory ordering requirements?
    
    head: AtomicPtr<Node<T>>,
    epoch: AtomicUsize,
    retired_nodes: [AtomicPtr<RetiredList<T>>; 3], // Triple buffering
}

struct Node<T> {
    data: T,
    next: AtomicPtr<Node<T>>,
    epoch: usize,
}

/// Retired node list for epoch-based memory reclamation
struct RetiredList<T> {
    nodes: Vec<*mut Node<T>>,
    epoch: usize,
}

/// RAII guard that tracks thread epoch for safe memory reclamation
pub struct EpochGuard<'a, T> {
    stack: &'a LockFreeStack<T>,
    local_epoch: usize,
}

impl<T> LockFreeStack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            epoch: AtomicUsize::new(0),
            retired_nodes: [
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
                AtomicPtr::new(ptr::null_mut()),
            ],
        }
    }
    
    /// Enter epoch for safe memory access
    pub fn enter(&self) -> EpochGuard<T> {
        // Your implementation:
        // 1. Get current global epoch
        // 2. Register thread in epoch system
        // 3. Return RAII guard for automatic cleanup
        
        let current_epoch = self.epoch.load(Ordering::Acquire);
        EpochGuard {
            stack: self,
            local_epoch: current_epoch,
        }
    }
    
    /// Push value onto stack (lock-free)
    pub fn push(&self, value: T) -> Result<(), StackError> {
        // Your implementation:
        // 1. Allocate new node
        // 2. Use compare-and-swap loop to link node
        // 3. Handle concurrent modifications
        // 4. What memory ordering is required?
        
        let new_node = Box::into_raw(Box::new(Node {
            data: value,
            next: AtomicPtr::new(ptr::null_mut()),
            epoch: self.epoch.load(Ordering::Relaxed),
        }));
        
        loop {
            let head = self.head.load(Ordering::Acquire);
            
            unsafe {
                (*new_node).next.store(head, Ordering::Relaxed);
            }
            
            // Critical: What happens if head changes between load and compare_exchange?
            match self.head.compare_exchange_weak(
                head,
                new_node,
                Ordering::Release, // Success ordering
                Ordering::Relaxed, // Failure ordering
            ) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    // Head changed - retry with new head value
                    // Why is compare_exchange_weak better than compare_exchange here?
                    continue;
                }
            }
        }
    }
    
    /// Pop value from stack (lock-free with ABA protection)
    pub fn pop(&self, guard: &EpochGuard<T>) -> Option<T> {
        // Your implementation:
        // 1. Load head with proper memory ordering
        // 2. Use epoch guard to prevent ABA
        // 3. Handle concurrent modifications
        // 4. Safely reclaim memory
        
        loop {
            let head = self.head.load(Ordering::Acquire);
            
            if head.is_null() {
                return None; // Stack is empty
            }
            
            // Validate node is still from our epoch
            let node = unsafe { &*head };
            if node.epoch != guard.local_epoch {
                // Node might be from different epoch - retry
                continue;
            }
            
            let next = node.next.load(Ordering::Relaxed);
            
            // Critical section: The ABA problem can occur here!
            // Between loading head and this compare_exchange,
            // another thread could:
            // 1. Pop this node
            // 2. Push it back  
            // 3. Our compare_exchange succeeds but we're operating on stale data
            
            match self.head.compare_exchange_weak(
                head,
                next,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Successfully removed node from stack
                    let data = unsafe { ptr::read(&(*head).data) };
                    
                    // Schedule node for deferred reclamation
                    self.retire_node(head, guard.local_epoch);
                    
                    return Some(data);
                }
                Err(_) => {
                    // Head changed - retry
                    continue;
                }
            }
        }
    }
    
    /// Retire node for later reclamation when safe
    fn retire_node(&self, node: *mut Node<T>, epoch: usize) {
        // Your implementation:
        // 1. Add node to retired list for current epoch
        // 2. Check if old epochs can be reclaimed
        // 3. Handle memory ordering correctly
        
        let retired_index = epoch % 3;
        let retired_list_ptr = self.retired_nodes[retired_index].load(Ordering::Acquire);
        
        // Add to retired list - this is tricky because multiple threads
        // might be retiring nodes simultaneously
        todo!("Implement safe node retirement")
    }
    
    /// Advance global epoch and reclaim old nodes
    pub fn advance_epoch(&self) {
        // Your implementation:
        // 1. Increment global epoch atomically
        // 2. Reclaim nodes from 2 epochs ago (safe distance)
        // 3. Handle concurrent epoch advancement
        
        let old_epoch = self.epoch.fetch_add(1, Ordering::AcqRel);
        let reclaim_epoch = if old_epoch >= 2 { old_epoch - 2 } else { return };
        
        // Reclaim nodes from old epoch
        let retired_index = reclaim_epoch % 3;
        let old_list = self.retired_nodes[retired_index].swap(ptr::null_mut(), Ordering::AcqRel);
        
        if !old_list.is_null() {
            unsafe {
                let retired_list = Box::from_raw(old_list);
                for node_ptr in retired_list.nodes {
                    drop(Box::from_raw(node_ptr));
                }
            }
        }
    }
    
    /// Get stack statistics for debugging
    pub fn stats(&self) -> StackStats {
        // Your implementation:
        // 1. Count active nodes (careful about races!)
        // 2. Count retired nodes
        // 3. Current epoch information
        
        todo!("Implement lock-free statistics collection")
    }
}

impl<'a, T> Drop for EpochGuard<'a, T> {
    fn drop(&mut self) {
        // Your implementation:
        // 1. Unregister thread from epoch system
        // 2. Check if epoch can be advanced
        // 3. Potentially trigger memory reclamation
        
        todo!("Implement epoch guard cleanup")
    }
}

#[derive(Debug)]
pub enum StackError {
    OutOfMemory,
    EpochExpired,
}

pub struct StackStats {
    pub active_nodes: usize,
    pub retired_nodes: usize,
    pub current_epoch: usize,
    pub successful_pops: u64,
    pub successful_pushes: u64,
    pub retry_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;
    
    #[test]
    fn test_basic_operations() {
        let stack = LockFreeStack::new();
        let guard = stack.enter();
        
        stack.push(42).unwrap();
        stack.push(24).unwrap();
        
        assert_eq!(stack.pop(&guard), Some(24));
        assert_eq!(stack.pop(&guard), Some(42));
        assert_eq!(stack.pop(&guard), None);
    }
    
    #[test]
    fn test_aba_resistance() {
        // This test specifically tries to trigger ABA problem
        let stack = Arc::new(LockFreeStack::new());
        let counter = Arc::new(AtomicU64::new(0));
        
        // Pre-populate stack
        for i in 0..1000 {
            stack.push(i).unwrap();
        }
        
        // Spawn threads that do ABA-prone operations
        let handles: Vec<_> = (0..8).map(|_| {
            let stack = stack.clone();
            let counter = counter.clone();
            
            thread::spawn(move || {
                for _ in 0..10000 {
                    let guard = stack.enter();
                    
                    // Pop-push pattern that could trigger ABA
                    if let Some(value) = stack.pop(&guard) {
                        stack.push(value).unwrap();
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify no corruption occurred
        let guard = stack.enter();
        let mut remaining = 0;
        while stack.pop(&guard).is_some() {
            remaining += 1;
        }
        
        // Should have original 1000 elements
        assert_eq!(remaining, 1000);
        println!("ABA test completed: {} operations", counter.load(Ordering::Relaxed));
    }
    
    #[test]
    fn test_memory_reclamation() {
        // Test that memory is properly reclaimed
        use std::sync::atomic::AtomicU64;
        
        static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
        static DROP_COUNT: AtomicU64 = AtomicU64::new(0);
        
        struct TrackedValue(i32);
        
        impl Drop for TrackedValue {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        let stack = LockFreeStack::new();
        
        // Push many values
        for i in 0..1000 {
            stack.push(TrackedValue(i)).unwrap();
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        
        // Pop all values
        let guard = stack.enter();
        for _ in 0..1000 {
            stack.pop(&guard);
        }
        
        // Advance epoch multiple times to trigger reclamation
        for _ in 0..5 {
            stack.advance_epoch();
            thread::sleep(std::time::Duration::from_millis(10));
        }
        
        // Most values should be dropped by now
        let allocated = ALLOC_COUNT.load(Ordering::Relaxed);
        let dropped = DROP_COUNT.load(Ordering::Relaxed);
        
        println!("Allocated: {}, Dropped: {}", allocated, dropped);
        assert!(dropped > 900, "Memory reclamation not working properly");
    }
}
```

**Questions (50 points):**
1. **ABA Problem Analysis (20 points)**: Explain exactly how the ABA problem occurs in lock-free stacks and how your epoch system prevents it
2. **Memory Ordering Justification (20 points)**: Justify every memory ordering choice in your implementation
3. **Performance Analysis (10 points)**: Under what conditions is your lock-free stack faster than a mutex-protected stack?

### Problem 1.2: Lock-Free Hash Table with Hazard Pointers (50 points)

**Context**: Build a lock-free hash table using hazard pointers for memory reclamation.

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr::{self, NonNull};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Lock-free hash table with hazard pointer-based memory reclamation
/// Must handle resizing, key collisions, and safe memory management
pub struct LockFreeHashMap<K, V> {
    // Your design:
    // 1. How do you handle lock-free resizing?
    // 2. How do you implement hazard pointers?
    // 3. What about handling hash collisions?
    
    buckets: AtomicPtr<Bucket<K, V>>,
    bucket_count: AtomicUsize,
    size: AtomicUsize,
    hazard_pointers: HazardPointerSystem<K, V>,
}

struct Bucket<K, V> {
    head: AtomicPtr<Node<K, V>>,
}

struct Node<K, V> {
    key: K,
    value: V,
    hash: u64,
    next: AtomicPtr<Node<K, V>>,
    marked_for_deletion: std::sync::atomic::AtomicBool,
}

/// Hazard pointer system for safe memory reclamation
struct HazardPointerSystem<K, V> {
    // Your design:
    // 1. How do you track hazard pointers per thread?
    // 2. How do you coordinate global hazard pointer state?
    // 3. When is it safe to reclaim memory?
    
    global_hazards: AtomicPtr<HazardRecord<K, V>>,
    retired_nodes: AtomicPtr<RetiredNode<K, V>>,
    max_hazards_per_thread: usize,
}

struct HazardRecord<K, V> {
    hazard_pointers: [AtomicPtr<Node<K, V>>; 4], // Max 4 hazards per thread
    active: std::sync::atomic::AtomicBool,
    next: AtomicPtr<HazardRecord<K, V>>,
}

struct RetiredNode<K, V> {
    node: *mut Node<K, V>,
    next: AtomicPtr<RetiredNode<K, V>>,
}

/// RAII guard for hazard pointer protection
pub struct HazardGuard<'a, K, V> {
    hazard_system: &'a HazardPointerSystem<K, V>,
    hazard_record: NonNull<HazardRecord<K, V>>,
    hazard_index: usize,
}

impl<K, V> LockFreeHashMap<K, V> 
where 
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self::with_capacity(16)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        let bucket_count = capacity.next_power_of_two();
        let buckets = unsafe {
            let layout = std::alloc::Layout::array::<Bucket<K, V>>(bucket_count).unwrap();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut Bucket<K, V>;
            
            // Initialize all buckets
            for i in 0..bucket_count {
                ptr.add(i).write(Bucket {
                    head: AtomicPtr::new(ptr::null_mut()),
                });
            }
            
            ptr
        };
        
        Self {
            buckets: AtomicPtr::new(buckets),
            bucket_count: AtomicUsize::new(bucket_count),
            size: AtomicUsize::new(0),
            hazard_pointers: HazardPointerSystem::new(),
        }
    }
    
    /// Insert key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        // Your implementation:
        // 1. Acquire hazard pointer guard
        // 2. Find appropriate bucket with lock-free traversal
        // 3. Handle collisions with chaining
        // 4. Check if resize is needed
        
        let guard = self.hazard_pointers.acquire_guard();
        let hash = self.calculate_hash(&key);
        
        loop {
            let bucket_count = self.bucket_count.load(Ordering::Acquire);
            let buckets = self.buckets.load(Ordering::Acquire);
            
            if buckets.is_null() {
                // Table is being resized - help and retry
                self.help_resize();
                continue;
            }
            
            let bucket_index = hash as usize & (bucket_count - 1);
            let bucket = unsafe { &(*buckets.add(bucket_index)) };
            
            // Protected pointer access using hazard guard
            let old_value = self.insert_in_bucket(bucket, key.clone(), value.clone(), hash, &guard)?;
            
            // Check if resize is needed
            let current_size = self.size.fetch_add(1, Ordering::Relaxed);
            if current_size > bucket_count * 2 {
                self.try_resize(bucket_count * 2);
            }
            
            return old_value;
        }
    }
    
    fn insert_in_bucket(
        &self,
        bucket: &Bucket<K, V>,
        key: K,
        value: V,
        hash: u64,
        guard: &HazardGuard<K, V>,
    ) -> Option<Option<V>> {
        // Your implementation:
        // 1. Traverse chain looking for existing key
        // 2. Use hazard pointers to protect from concurrent deletion
        // 3. Insert new node or update existing
        // 4. Handle marked-for-deletion nodes
        
        loop {
            let head = bucket.head.load(Ordering::Acquire);
            
            // Protect head with hazard pointer
            guard.protect(0, head);
            
            // Verify head hasn't changed (ABA protection)
            if bucket.head.load(Ordering::Acquire) != head {
                continue;
            }
            
            if head.is_null() {
                // Empty bucket - try to insert new node
                let new_node = Box::into_raw(Box::new(Node {
                    key,
                    value,
                    hash,
                    next: AtomicPtr::new(ptr::null_mut()),
                    marked_for_deletion: std::sync::atomic::AtomicBool::new(false),
                }));
                
                match bucket.head.compare_exchange_weak(
                    ptr::null_mut(),
                    new_node,
                    Ordering::Release,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return Some(None), // Successfully inserted
                    Err(_) => {
                        // Someone else inserted - clean up and retry
                        unsafe { drop(Box::from_raw(new_node)); }
                        continue;
                    }
                }
            }
            
            // Traverse chain looking for key or insertion point
            let mut current = head;
            loop {
                let node = unsafe { &*current };
                
                if node.marked_for_deletion.load(Ordering::Acquire) {
                    // Help remove deleted node
                    self.help_remove_node(bucket, current, guard);
                    break; // Restart traversal
                }
                
                if node.hash == hash && node.key == key {
                    // Found existing key - update value
                    // This is tricky in lock-free setting...
                    return Some(Some(node.value.clone()));
                }
                
                let next = node.next.load(Ordering::Acquire);
                if next.is_null() {
                    // End of chain - insert new node
                    let new_node = Box::into_raw(Box::new(Node {
                        key,
                        value,
                        hash,
                        next: AtomicPtr::new(ptr::null_mut()),
                        marked_for_deletion: std::sync::atomic::AtomicBool::new(false),
                    }));
                    
                    match node.next.compare_exchange_weak(
                        ptr::null_mut(),
                        new_node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => return Some(None),
                        Err(_) => {
                            unsafe { drop(Box::from_raw(new_node)); }
                            break; // Retry from head
                        }
                    }
                }
                
                // Move to next node with hazard protection
                guard.protect(1, next);
                if node.next.load(Ordering::Acquire) != next {
                    break; // Chain changed, retry
                }
                current = next;
            }
        }
    }
    
    /// Get value for key
    pub fn get(&self, key: &K) -> Option<V> {
        // Your implementation:
        // 1. Acquire hazard guard
        // 2. Find bucket and traverse chain
        // 3. Use hazard pointers for memory safety
        // 4. Handle concurrent modifications
        
        todo!("Implement lock-free get operation")
    }
    
    /// Remove key from map
    pub fn remove(&self, key: &K) -> Option<V> {
        // Your implementation:
        // 1. Find node to remove
        // 2. Mark node for deletion atomically
        // 3. Physically unlink node from chain
        // 4. Schedule node for reclamation via hazard pointers
        
        todo!("Implement lock-free remove operation")
    }
    
    fn help_remove_node(
        &self,
        bucket: &Bucket<K, V>,
        node_to_remove: *mut Node<K, V>,
        guard: &HazardGuard<K, V>,
    ) {
        // Your implementation:
        // 1. Find predecessor of node to remove
        // 2. Atomically unlink node
        // 3. Retire node via hazard pointer system
        
        todo!("Implement cooperative node removal")
    }
    
    fn try_resize(&self, new_capacity: usize) {
        // Your implementation:
        // 1. Allocate new bucket array
        // 2. Coordinate resize with other threads
        // 3. Migrate entries to new buckets
        // 4. Safely reclaim old bucket array
        
        todo!("Implement lock-free resize")
    }
    
    fn help_resize(&self) {
        // Help ongoing resize operation
        todo!("Implement resize cooperation")
    }
    
    fn calculate_hash(&self, key: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl<K, V> HazardPointerSystem<K, V> {
    fn new() -> Self {
        Self {
            global_hazards: AtomicPtr::new(ptr::null_mut()),
            retired_nodes: AtomicPtr::new(ptr::null_mut()),
            max_hazards_per_thread: 4,
        }
    }
    
    /// Acquire hazard guard for current thread
    fn acquire_guard(&self) -> HazardGuard<K, V> {
        // Your implementation:
        // 1. Find or create hazard record for current thread
        // 2. Return RAII guard
        
        todo!("Implement hazard guard acquisition")
    }
    
    /// Check if pointer is protected by any hazard pointer
    fn is_hazardous(&self, ptr: *mut Node<K, V>) -> bool {
        // Your implementation:
        // 1. Scan all active hazard records
        // 2. Check if ptr is protected
        
        todo!("Implement hazard checking")
    }
    
    /// Retire node for later reclamation
    fn retire_node(&self, node: *mut Node<K, V>) {
        // Your implementation:
        // 1. Add node to retired list
        // 2. Check if reclamation can proceed
        // 3. Reclaim nodes that are no longer hazardous
        
        todo!("Implement node retirement")
    }
}

impl<'a, K, V> HazardGuard<'a, K, V> {
    /// Protect pointer with hazard pointer
    fn protect(&self, index: usize, ptr: *mut Node<K, V>) {
        // Your implementation:
        // 1. Store ptr in hazard pointer slot
        // 2. Ensure proper memory ordering
        
        if index < self.hazard_record.as_ref().hazard_pointers.len() {
            unsafe {
                self.hazard_record.as_ref().hazard_pointers[index].store(ptr, Ordering::Release);
            }
        }
    }
}

impl<'a, K, V> Drop for HazardGuard<'a, K, V> {
    fn drop(&mut self) {
        // Clear all hazard pointers
        unsafe {
            for hazard in &self.hazard_record.as_ref().hazard_pointers {
                hazard.store(ptr::null_mut(), Ordering::Release);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    
    #[test]
    fn test_concurrent_insertions() {
        let map = Arc::new(LockFreeHashMap::new());
        let handles: Vec<_> = (0..8).map(|thread_id| {
            let map = map.clone();
            thread::spawn(move || {
                for i in 0..1000 {
                    let key = thread_id * 1000 + i;
                    map.insert(key, key * 2);
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify all insertions succeeded
        for thread_id in 0..8 {
            for i in 0..1000 {
                let key = thread_id * 1000 + i;
                assert_eq!(map.get(&key), Some(key * 2));
            }
        }
    }
    
    #[test]
    fn test_hazard_pointer_safety() {
        // Test that hazard pointers prevent premature reclamation
        // This is a complex test that requires careful coordination
    }
}
```

**Questions (50 points):**
1. **Hazard Pointer Design (25 points)**: Explain how hazard pointers prevent use-after-free in your hash table
2. **Lock-Free Resize (15 points)**: Design a lock-free hash table resize algorithm
3. **Performance vs Correctness (10 points)**: When would you choose hazard pointers vs epoch-based reclamation?

### Problem 1.3: Wait-Free Queue with Formal Verification (50 points)

**Context**: Implement a wait-free queue and prove its correctness properties.

```rust
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;

/// Wait-free MPMC queue with bounded capacity
/// Must guarantee wait-freedom: every operation completes in bounded steps
pub struct WaitFreeQueue<T> {
    // Your design:
    // 1. How do you ensure wait-freedom (not just lock-freedom)?
    // 2. How do you handle the bounded capacity requirement?
    // 3. What invariants must be maintained?
    
    buffer: Box<[AtomicPtr<T>]>,
    capacity: usize,
    enqueue_pos: AtomicUsize,
    dequeue_pos: AtomicUsize,
    // What additional state is needed for wait-freedom?
}

impl<T> WaitFreeQueue<T> {
    /// Create queue with specified capacity (must be power of 2)
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "Capacity must be power of 2");
        
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(AtomicPtr::new(ptr::null_mut()));
        }
        
        Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            enqueue_pos: AtomicUsize::new(0),
            dequeue_pos: AtomicUsize::new(0),
        }
    }
    
    /// Enqueue value (wait-free)
    pub fn enqueue(&self, value: T) -> Result<(), QueueError> {
        // Your implementation:
        // 1. Must complete in bounded number of steps
        // 2. Cannot spin indefinitely
        // 3. Must handle full queue gracefully
        // 4. Prove wait-freedom property
        
        let value_ptr = Box::into_raw(Box::new(value));
        
        loop {
            let pos = self.enqueue_pos.load(Ordering::Relaxed);
            let index = pos & (self.capacity - 1);
            let slot = &self.buffer[index];
            
            // Check if slot is available
            let current = slot.load(Ordering::Acquire);
            if !current.is_null() {
                // Slot occupied - queue might be full
                // Wait-free requirement: we can't spin here indefinitely
                
                // Check if queue is actually full
                let dequeue_pos = self.dequeue_pos.load(Ordering::Acquire);
                if pos - dequeue_pos >= self.capacity {
                    // Queue is full
                    unsafe { drop(Box::from_raw(value_ptr)); }
                    return Err(QueueError::Full);
                }
                
                // Not full, but this slot is occupied
                // Help advance enqueue position
                self.enqueue_pos.compare_exchange_weak(
                    pos,
                    pos + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                );
                continue;
            }
            
            // Try to claim this slot
            match slot.compare_exchange_weak(
                ptr::null_mut(),
                value_ptr,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // Successfully enqueued
                    self.enqueue_pos.compare_exchange_weak(
                        pos,
                        pos + 1,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    );
                    return Ok(());
                }
                Err(_) => {
                    // Slot was taken by another thread
                    continue;
                }
            }
        }
    }
    
    /// Dequeue value (wait-free)
    pub fn dequeue(&self) -> Option<T> {
        // Your implementation:
        // 1. Must complete in bounded number of steps
        // 2. Cannot spin indefinitely even if queue is empty
        // 3. Must handle concurrent enqueues/dequeues
        // 4. Prove wait-freedom property
        
        loop {
            let pos = self.dequeue_pos.load(Ordering::Relaxed);
            let index = pos & (self.capacity - 1);
            let slot = &self.buffer[index];
            
            // Check if slot has data
            let current = slot.load(Ordering::Acquire);
            if current.is_null() {
                // Slot empty - queue might be empty
                let enqueue_pos = self.enqueue_pos.load(Ordering::Acquire);
                if enqueue_pos <= pos {
                    // Queue is empty
                    return None;
                }
                
                // Not empty, but this slot is empty
                // Help advance dequeue position
                self.dequeue_pos.compare_exchange_weak(
                    pos,
                    pos + 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                );
                continue;
            }
            
            // Try to claim this slot
            match slot.compare_exchange_weak(
                current,
                ptr::null_mut(),
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(ptr) => {
                    // Successfully dequeued
                    let value = unsafe { Box::from_raw(ptr) };
                    self.dequeue_pos.compare_exchange_weak(
                        pos,
                        pos + 1,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    );
                    return Some(*value);
                }
                Err(_) => {
                    // Slot was taken by another thread
                    continue;
                }
            }
        }
    }
    
    /// Get current queue size (approximate due to concurrency)
    pub fn len(&self) -> usize {
        let enqueue_pos = self.enqueue_pos.load(Ordering::Relaxed);
        let dequeue_pos = self.dequeue_pos.load(Ordering::Relaxed);
        enqueue_pos.saturating_sub(dequeue_pos)
    }
    
    /// Check if queue is empty (may have false negatives due to concurrency)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Check if queue is full (may have false positives due to concurrency)
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }
}

impl<T> Drop for WaitFreeQueue<T> {
    fn drop(&mut self) {
        // Clean up any remaining values
        while let Some(_) = self.dequeue() {}
    }
}

#[derive(Debug)]
pub enum QueueError {
    Full,
}

/// Formal verification proof obligations
/// You must prove these properties hold for your implementation
pub mod verification {
    use super::*;
    
    /// Proof that enqueue operation is wait-free
    /// Property: enqueue completes in at most O(capacity) steps
    pub fn prove_enqueue_wait_freedom<T>(_queue: &WaitFreeQueue<T>) {
        // Your proof:
        // 1. Show that the loop terminates in bounded steps
        // 2. Prove that no thread can block another indefinitely
        // 3. Show worst-case step count is bounded by queue capacity
        
        // Proof structure:
        // - Each iteration either succeeds or advances position
        // - Position can advance at most 'capacity' times before wrapping
        // - No thread can prevent another from making progress
        // - Therefore, maximum steps = O(capacity)
        
        todo!("Provide formal proof of wait-freedom for enqueue")
    }
    
    /// Proof that dequeue operation is wait-free
    pub fn prove_dequeue_wait_freedom<T>(_queue: &WaitFreeQueue<T>) {
        // Your proof similar to enqueue
        todo!("Provide formal proof of wait-freedom for dequeue")
    }
    
    /// Proof of linearizability
    /// Property: Every operation appears to take effect at a single point in time
    pub fn prove_linearizability<T>(_queue: &WaitFreeQueue<T>) {
        // Your proof:
        // 1. Define linearization points for enqueue/dequeue
        // 2. Show that these points preserve sequential semantics
        // 3. Prove that concurrent operations can be reordered to match sequential execution
        
        todo!("Provide linearizability proof")
    }
    
    /// Proof of progress guarantee
    /// Property: If threads keep trying to enqueue/dequeue, some thread makes progress
    pub fn prove_system_progress<T>(_queue: &WaitFreeQueue<T>) {
        todo!("Provide system progress proof")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;
    use std::time::Instant;
    
    #[test]
    fn test_wait_freedom_property() {
        // Test that operations complete in bounded time even under contention
        let queue = Arc::new(WaitFreeQueue::new(1024));
        let operations_completed = Arc::new(AtomicU64::new(0));
        
        let handles: Vec<_> = (0..16).map(|thread_id| {
            let queue = queue.clone();
            let ops_completed = operations_completed.clone();
            
            thread::spawn(move || {
                let start = Instant::now();
                
                for i in 0..1000 {
                    let value = thread_id * 1000 + i;
                    
                    // Measure time for each operation
                    let op_start = Instant::now();
                    
                    match queue.enqueue(value) {
                        Ok(_) => {
                            ops_completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            
                            // Verify operation completed in bounded time
                            let op_duration = op_start.elapsed();
                            assert!(op_duration.as_millis() < 10, "Operation took too long: {:?}", op_duration);
                        }
                        Err(QueueError::Full) => {
                            // Queue full - this is expected behavior
                            // But operation should still complete quickly
                            let op_duration = op_start.elapsed();
                            assert!(op_duration.as_millis() < 10, "Full queue detection took too long: {:?}", op_duration);
                        }
                    }
                }
                
                println!("Thread {} completed in {:?}", thread_id, start.elapsed());
            })
        }).collect();
        
        // Consumer threads
        let consumer_handles: Vec<_> = (0..4).map(|_| {
            let queue = queue.clone();
            thread::spawn(move || {
                for _ in 0..2000 {
                    let op_start = Instant::now();
                    let _ = queue.dequeue();
                    
                    // Verify dequeue completes in bounded time
                    let op_duration = op_start.elapsed();
                    assert!(op_duration.as_millis() < 10, "Dequeue took too long: {:?}", op_duration);
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        for handle in consumer_handles {
            handle.join().unwrap();
        }
        
        println!("Total operations completed: {}", operations_completed.load(std::sync::atomic::Ordering::Relaxed));
    }
    
    #[test]
    fn test_linearizability() {
        // Test that concurrent operations maintain sequential consistency
        let queue = Arc::new(WaitFreeQueue::new(256));
        
        // This test is complex - you need to verify that the outcome
        // of concurrent operations matches some sequential execution
        todo!("Implement linearizability test")
    }
}
```

**Questions (50 points):**
1. **Wait-Freedom Proof (30 points)**: Prove that both enqueue and dequeue are wait-free (complete in bounded steps)
2. **Linearizability Analysis (15 points)**: Define linearization points and prove linearizability
3. **Performance Comparison (5 points)**: Compare wait-free vs lock-free performance characteristics

---

## Section II: Memory Ordering & Performance Engineering (100 points)

### Problem 2.1: Memory Barrier Analysis & Optimization (50 points)

**Context**: Analyze and optimize memory ordering in concurrent algorithms.

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicPtr, Ordering};
use std::ptr;

/// High-performance producer-consumer with minimal memory barriers
/// Must achieve maximum throughput while maintaining correctness
pub struct OptimizedChannel<T> {
    // Your design:
    // 1. What's the minimal set of memory barriers needed?
    // 2. How do you optimize for different CPU architectures?
    // 3. What about cache line optimization?
    
    buffer: Box<[AtomicPtr<T>]>,
    capacity: usize,
    
    // Producer state (separate cache line)
    producer_pos: AtomicU64,
    producer_cached_consumer: AtomicU64,
    _producer_padding: [u8; 64 - 16], // Cache line padding
    
    // Consumer state (separate cache line) 
    consumer_pos: AtomicU64,
    consumer_cached_producer: AtomicU64,
    _consumer_padding: [u8; 64 - 16],
    
    // Synchronization
    producer_waiting: AtomicBool,
    consumer_waiting: AtomicBool,
}

impl<T> OptimizedChannel<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two());
        
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(AtomicPtr::new(ptr::null_mut()));
        }
        
        Self {
            buffer: buffer.into_boxed_slice(),
            capacity,
            producer_pos: AtomicU64::new(0),
            producer_cached_consumer: AtomicU64::new(0),
            _producer_padding: [0; 48],
            consumer_pos: AtomicU64::new(0),
            consumer_cached_producer: AtomicU64::new(0),
            _consumer_padding: [0; 48],
            producer_waiting: AtomicBool::new(false),
            consumer_waiting: AtomicBool::new(false),
        }
    }
    
    /// Send value with optimized memory ordering
    pub fn send(&self, value: T) -> Result<(), ChannelError> {
        // Your implementation:
        // 1. Use minimal memory ordering for maximum performance
        // 2. Implement efficient cache coherency
        // 3. Minimize false sharing
        // 4. Handle backpressure efficiently
        
        let value_ptr = Box::into_raw(Box::new(value));
        let mut pos = self.producer_pos.load(Ordering::Relaxed);
        
        loop {
            let index = pos as usize & (self.capacity - 1);
            let slot = &self.buffer[index];
            
            // Check if slot is available (optimistic)
            if slot.load(Ordering::Relaxed).is_null() {
                // Try to claim slot
                match slot.compare_exchange_weak(
                    ptr::null_mut(),
                    value_ptr,
                    Ordering::Release, // Ensure value is visible before position update
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Successfully stored value
                        // Update producer position with release ordering
                        self.producer_pos.store(pos + 1, Ordering::Release);
                        
                        // Wake consumer if waiting
                        if self.consumer_waiting.load(Ordering::Acquire) {
                            self.consumer_waiting.store(false, Ordering::Release);
                            // Would notify consumer here in real implementation
                        }
                        
                        return Ok(());
                    }
                    Err(_) => {
                        // Slot taken, try next position
                        pos += 1;
                        continue;
                    }
                }
            }
            
            // Slot occupied - check if queue is full
            let cached_consumer = self.producer_cached_consumer.load(Ordering::Relaxed);
            if pos - cached_consumer >= self.capacity as u64 {
                // Cache might be stale - refresh consumer position
                let actual_consumer = self.consumer_pos.load(Ordering::Acquire);
                self.producer_cached_consumer.store(actual_consumer, Ordering::Relaxed);
                
                if pos - actual_consumer >= self.capacity as u64 {
                    // Queue is actually full
                    unsafe { drop(Box::from_raw(value_ptr)); }
                    return Err(ChannelError::Full);
                }
            }
            
            pos += 1;
        }
    }
    
    /// Receive value with optimized memory ordering  
    pub fn recv(&self) -> Option<T> {
        // Your implementation:
        // 1. Minimize memory barriers
        // 2. Handle empty channel efficiently
        // 3. Coordinate with producer for backpressure
        
        let mut pos = self.consumer_pos.load(Ordering::Relaxed);
        
        loop {
            let index = pos as usize & (self.capacity - 1);
            let slot = &self.buffer[index];
            
            // Check if slot has data (optimistic)
            let current = slot.load(Ordering::Acquire); // Acquire to see producer's data
            if !current.is_null() {
                // Try to claim value
                match slot.compare_exchange_weak(
                    current,
                    ptr::null_mut(),
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(ptr) => {
                        // Successfully got value
                        let value = unsafe { Box::from_raw(ptr) };
                        
                        // Update consumer position
                        self.consumer_pos.store(pos + 1, Ordering::Release);
                        
                        // Wake producer if waiting
                        if self.producer_waiting.load(Ordering::Acquire) {
                            self.producer_waiting.store(false, Ordering::Release);
                            // Would notify producer here
                        }
                        
                        return Some(*value);
                    }
                    Err(_) => {
                        // Value taken by another consumer
                        pos += 1;
                        continue;
                    }
                }
            }
            
            // Slot empty - check if queue is empty
            let cached_producer = self.consumer_cached_producer.load(Ordering::Relaxed);
            if cached_producer <= pos {
                // Cache might be stale - refresh producer position
                let actual_producer = self.producer_pos.load(Ordering::Acquire);
                self.consumer_cached_producer.store(actual_producer, Ordering::Relaxed);
                
                if actual_producer <= pos {
                    // Queue is actually empty
                    return None;
                }
            }
            
            pos += 1;
        }
    }
    
    /// Benchmark different memory ordering strategies
    pub fn benchmark_orderings(&self) -> BenchmarkResults {
        // Your implementation:
        // 1. Test with different memory orderings
        // 2. Measure performance on different CPU architectures  
        // 3. Compare cache miss rates
        // 4. Analyze scalability with core count
        
        todo!("Implement comprehensive memory ordering benchmarks")
    }
}

/// Analysis of memory ordering choices
pub mod memory_analysis {
    use super::*;
    
    /// Explain why specific memory orderings were chosen
    pub fn analyze_ordering_choices<T>(_channel: &OptimizedChannel<T>) {
        // Your analysis:
        // 1. Why Release for value storage and Acquire for value loading?
        // 2. Why Relaxed for position caching?
        // 3. Platform-specific considerations (x86 vs ARM vs PowerPC)
        // 4. Performance implications of each choice
        
        println!("Memory Ordering Analysis:");
        println!("1. Release/Acquire pair for value transfer ensures:");
        println!("   - Producer's value write happens-before consumer's value read");
        println!("   - No reordering of value write past position update");
        println!("   - Minimal barrier overhead on x86 (no-op for Release/Acquire)");
        
        println!("2. Relaxed ordering for position caching:");
        println!("   - Cached positions are hints, exactness not required");
        println!("   - Reduces barrier overhead for frequent position checks");
        println!("   - Fresh position loaded with Acquire when cache miss detected");
        
        println!("3. Platform considerations:");
        println!("   - x86: Release/Acquire are nearly free (TSO memory model)");
        println!("   - ARM: Requires actual memory barriers (weaker memory model)");
        println!("   - PowerPC: Even weaker model, more barriers needed");
    }
    
    /// Prove correctness of memory ordering
    pub fn prove_correctness<T>(_channel: &OptimizedChannel<T>) {
        // Your proof:
        // 1. Show that value writes are always visible before position updates
        // 2. Prove no lost updates or torn reads
        // 3. Demonstrate absence of use-after-free
        // 4. Show linearizability is maintained
        
        todo!("Provide formal correctness proof")
    }
}

pub struct BenchmarkResults {
    pub throughput_ops_per_second: u64,
    pub average_latency_ns: u64,
    pub cache_misses_per_op: f64,
    pub cpu_utilization: f64,
}

#[derive(Debug)]
pub enum ChannelError {
    Full,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;
    
    #[test]
    fn test_memory_ordering_correctness() {
        let channel = Arc::new(OptimizedChannel::new(1024));
        let producer_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let consumer_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
        
        // Producers
        let handles: Vec<_> = (0..4).map(|producer_id| {
            let channel = channel.clone();
            let count = producer_count.clone();
            thread::spawn(move || {
                for i in 0..10000 {
                    let value = producer_id * 10000 + i;
                    while channel.send(value).is_err() {
                        thread::yield_now();
                    }
                    count.fetch_add(1, Ordering::Relaxed);
                }
            })
        }).collect();
        
        // Consumers  
        let consumer_handles: Vec<_> = (0..4).map(|_| {
            let channel = channel.clone();
            let count = consumer_count.clone();
            thread::spawn(move || {
                while count.load(Ordering::Relaxed) < 40000 {
                    if let Some(_value) = channel.recv() {
                        count.fetch_add(1, Ordering::Relaxed);
                    } else {
                        thread::yield_now();
                    }
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        for handle in consumer_handles {
            handle.join().unwrap();
        }
        
        assert_eq!(producer_count.load(Ordering::Relaxed), 40000);
        assert_eq!(consumer_count.load(Ordering::Relaxed), 40000);
    }
    
    #[test]
    fn benchmark_memory_orderings() {
        // Compare performance with different memory ordering strategies
        let channel = OptimizedChannel::new(1024);
        let results = channel.benchmark_orderings();
        
        println!("Benchmark Results: {:?}", results);
        // Should show performance benefits of optimized ordering
    }
}
```

**Questions (50 points):**
1. **Memory Ordering Optimization (30 points)**: Justify each memory ordering choice and prove correctness
2. **Cache Line Analysis (15 points)**: Explain cache line optimization and false sharing prevention
3. **Cross-Platform Behavior (5 points)**: How do your choices work on different CPU architectures?

### Problem 2.2: Lock-Free Performance Engineering (50 points)

**Context**: Engineer a lock-free system for maximum performance and analyze when lock-free is beneficial.

```rust
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};

/// Performance engineering toolkit for lock-free systems
pub struct LockFreePerformanceAnalyzer {
    // Metrics collection
    operations_completed: AtomicU64,
    total_latency_ns: AtomicU64,
    contention_events: AtomicU64,
    cache_misses: AtomicU64,
    retry_count: AtomicU64,
    
    // Configuration
    thread_count: usize,
    measurement_duration: Duration,
}

impl LockFreePerformanceAnalyzer {
    /// Compare lock-free vs mutex-based implementations
    pub async fn compare_implementations<T, F1, F2>(
        &self,
        lock_free_impl: F1,
        mutex_impl: F2,
        thread_counts: Vec<usize>,
    ) -> ComparisonResults
    where
        F1: Fn() -> T + Send + Sync + 'static,
        F2: Fn() -> T + Send + Sync + 'static,
        T: Send + 'static,
    {
        // Your implementation:
        // 1. Benchmark both implementations across different thread counts
        // 2. Measure throughput, latency, and scalability
        // 3. Identify crossover points where lock-free becomes beneficial
        // 4. Analyze contention and cache behavior
        
        let mut results = ComparisonResults::new();
        
        for &thread_count in &thread_counts {
            println!("Testing with {} threads", thread_count);
            
            // Benchmark lock-free implementation
            let lock_free_metrics = self.benchmark_implementation(
                &lock_free_impl,
                thread_count,
                "lock_free"
            ).await;
            
            // Benchmark mutex implementation
            let mutex_metrics = self.benchmark_implementation(
                &mutex_impl,
                thread_count,
                "mutex"
            ).await;
            
            results.add_measurement(thread_count, lock_free_metrics, mutex_metrics);
        }
        
        results
    }
    
    async fn benchmark_implementation<F, T>(
        &self,
        implementation: F,
        thread_count: usize,
        name: &str,
    ) -> PerformanceMetrics
    where
        F: Fn() -> T + Send + Sync + 'static,
        T: Send + 'static,
    {
        // Your implementation:
        // 1. Spawn specified number of threads
        // 2. Run implementation for measurement duration
        // 3. Collect detailed performance metrics
        // 4. Handle warmup period to avoid cold start effects
        
        let implementation = Arc::new(implementation);
        let start_time = Instant::now();
        
        // Warmup period
        let warmup_handles: Vec<_> = (0..thread_count).map(|_| {
            let impl_ref = implementation.clone();
            tokio::spawn(async move {
                for _ in 0..1000 {
                    let _ = impl_ref();
                }
            })
        }).collect();
        
        for handle in warmup_handles {
            handle.await.unwrap();
        }
        
        // Reset metrics
        self.operations_completed.store(0, Ordering::Relaxed);
        self.total_latency_ns.store(0, Ordering::Relaxed);
        self.contention_events.store(0, Ordering::Relaxed);
        self.retry_count.store(0, Ordering::Relaxed);
        
        // Main benchmark
        let measurement_start = Instant::now();
        let handles: Vec<_> = (0..thread_count).map(|thread_id| {
            let impl_ref = implementation.clone();
            let analyzer = self;
            
            tokio::spawn(async move {
                let mut thread_ops = 0u64;
                let mut thread_latency = 0u64;
                
                while measurement_start.elapsed() < analyzer.measurement_duration {
                    let op_start = Instant::now();
                    let _ = impl_ref();
                    let op_latency = op_start.elapsed().as_nanos() as u64;
                    
                    thread_ops += 1;
                    thread_latency += op_latency;
                }
                
                // Update global metrics
                analyzer.operations_completed.fetch_add(thread_ops, Ordering::Relaxed);
                analyzer.total_latency_ns.fetch_add(thread_latency, Ordering::Relaxed);
                
                (thread_ops, thread_latency)
            })
        }).collect();
        
        let mut thread_results = Vec::new();
        for handle in handles {
            thread_results.push(handle.await.unwrap());
        }
        
        let total_duration = measurement_start.elapsed();
        let total_ops = self.operations_completed.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ns.load(Ordering::Relaxed);
        
        PerformanceMetrics {
            implementation_name: name.to_string(),
            thread_count,
            throughput_ops_per_second: (total_ops as f64 / total_duration.as_secs_f64()) as u64,
            average_latency_ns: if total_ops > 0 { total_latency / total_ops } else { 0 },
            total_operations: total_ops,
            measurement_duration: total_duration,
            contention_events: self.contention_events.load(Ordering::Relaxed),
            retry_count: self.retry_count.load(Ordering::Relaxed),
            thread_results,
        }
    }
    
    /// Analyze when lock-free is beneficial
    pub fn analyze_lock_free_benefits(&self, results: &ComparisonResults) -> BenefitAnalysis {
        // Your analysis:
        // 1. Identify thread count thresholds
        // 2. Analyze scalability characteristics  
        // 3. Consider latency vs throughput tradeoffs
        // 4. Account for real-world deployment factors
        
        let mut crossover_point = None;
        let mut max_improvement = 0.0f64;
        let mut recommendations = Vec::new();
        
        for measurement in &results.measurements {
            let lock_free = &measurement.lock_free_metrics;
            let mutex = &measurement.mutex_metrics;
            
            let throughput_improvement = 
                lock_free.throughput_ops_per_second as f64 / mutex.throughput_ops_per_second as f64;
                
            let latency_improvement = 
                mutex.average_latency_ns as f64 / lock_free.average_latency_ns as f64;
            
            if throughput_improvement > 1.1 && crossover_point.is_none() {
                crossover_point = Some(measurement.thread_count);
            }
            
            if throughput_improvement > max_improvement {
                max_improvement = throughput_improvement;
            }
            
            // Generate recommendations based on results
            if measurement.thread_count <= 2 && throughput_improvement < 1.0 {
                recommendations.push(format!(
                    "Mutex is faster than lock-free at {} threads ({}x)",
                    measurement.thread_count, 1.0 / throughput_improvement
                ));
            } else if throughput_improvement > 2.0 {
                recommendations.push(format!(
                    "Lock-free shows significant benefit at {} threads ({}x improvement)",
                    measurement.thread_count, throughput_improvement
                ));
            }
        }
        
        BenefitAnalysis {
            crossover_thread_count: crossover_point,
            maximum_improvement_factor: max_improvement,
            recommendations,
            // Add more analysis...
        }
    }
    
    /// Profile contention patterns
    pub fn profile_contention<F>(&self, implementation: F, duration: Duration) -> ContentionProfile
    where
        F: Fn() + Send + Sync + 'static,
    {
        // Your implementation:
        // 1. Measure CAS failure rates
        // 2. Track retry patterns  
        // 3. Analyze cache line bouncing
        // 4. Identify hotspots
        
        todo!("Implement contention profiling")
    }
}

pub struct ComparisonResults {
    measurements: Vec<ComparisonMeasurement>,
}

pub struct ComparisonMeasurement {
    thread_count: usize,
    lock_free_metrics: PerformanceMetrics,
    mutex_metrics: PerformanceMetrics,
}

pub struct PerformanceMetrics {
    implementation_name: String,
    thread_count: usize,
    throughput_ops_per_second: u64,
    average_latency_ns: u64,
    total_operations: u64,
    measurement_duration: Duration,
    contention_events: u64,
    retry_count: u64,
    thread_results: Vec<(u64, u64)>, // (ops, latency) per thread
}

pub struct BenefitAnalysis {
    crossover_thread_count: Option<usize>,
    maximum_improvement_factor: f64,
    recommendations: Vec<String>,
}

pub struct ContentionProfile {
    cas_success_rate: f64,
    average_retries_per_operation: f64,
    cache_line_bouncing_events: u64,
    hotspot_analysis: Vec<String>,
}

impl ComparisonResults {
    fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }
    
    fn add_measurement(
        &mut self,
        thread_count: usize,
        lock_free_metrics: PerformanceMetrics,
        mutex_metrics: PerformanceMetrics,
    ) {
        self.measurements.push(ComparisonMeasurement {
            thread_count,
            lock_free_metrics,
            mutex_metrics,
        });
    }
}

/// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::collections::VecDeque;
    
    // Example: Compare lock-free stack vs mutex-protected stack
    struct LockFreeStack<T> {
        head: AtomicPtr<Node<T>>,
    }
    
    struct Node<T> {
        data: T,
        next: *mut Node<T>,
    }
    
    impl<T> LockFreeStack<T> {
        fn new() -> Self {
            Self {
                head: AtomicPtr::new(std::ptr::null_mut()),
            }
        }
        
        fn push(&self, value: T) {
            let new_node = Box::into_raw(Box::new(Node {
                data: value,
                next: std::ptr::null_mut(),
            }));
            
            loop {
                let head = self.head.load(Ordering::Acquire);
                unsafe { (*new_node).next = head; }
                
                if self.head.compare_exchange_weak(
                    head,
                    new_node,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    break;
                }
            }
        }
    }
    
    struct MutexStack<T> {
        data: Mutex<VecDeque<T>>,
    }
    
    impl<T> MutexStack<T> {
        fn new() -> Self {
            Self {
                data: Mutex::new(VecDeque::new()),
            }
        }
        
        fn push(&self, value: T) {
            let mut stack = self.data.lock().unwrap();
            stack.push_back(value);
        }
    }
    
    #[tokio::test]
    async fn test_performance_comparison() {
        let analyzer = LockFreePerformanceAnalyzer {
            operations_completed: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            contention_events: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            retry_count: AtomicU64::new(0),
            thread_count: 0,
            measurement_duration: Duration::from_secs(5),
        };
        
        let lock_free_stack = Arc::new(LockFreeStack::new());
        let mutex_stack = Arc::new(MutexStack::new());
        
        let results = analyzer.compare_implementations(
            {
                let stack = lock_free_stack.clone();
                move || stack.push(42)
            },
            {
                let stack = mutex_stack.clone();
                move || stack.push(42)
            },
            vec![1, 2, 4, 8, 16],
        ).await;
        
        let analysis = analyzer.analyze_lock_free_benefits(&results);
        
        println!("Lock-free benefits analysis:");
        if let Some(crossover) = analysis.crossover_thread_count {
            println!("Lock-free becomes beneficial at {} threads", crossover);
        }
        println!("Maximum improvement: {:.2}x", analysis.maximum_improvement_factor);
        
        for recommendation in analysis.recommendations {
            println!("- {}", recommendation);
        }
    }
}
```

**Questions (50 points):**
1. **Performance Analysis (30 points)**: When is lock-free programming beneficial vs harmful to performance?
2. **Benchmarking Methodology (15 points)**: Design proper benchmarks that account for warmup, contention, and real-world conditions
3. **Deployment Recommendations (5 points)**: Provide concrete guidance on when to choose lock-free vs locks

---

## Success Criteria & Mastery Validation

### **Pass Requirements (90% minimum):**

1. **Lock-Free Data Structures (135 points required)**:
   - Correctly implement ABA-resistant stack with epoch reclamation
   - Build hazard pointer-protected hash table with lock-free resize
   - Design wait-free queue with formal correctness proofs

2. **Memory Ordering Mastery (90 points required)**:
   - Optimize memory barriers for maximum performance while maintaining correctness
   - Engineer high-performance lock-free systems with detailed analysis
   - Provide formal proofs of correctness and progress properties

### **Advanced Mastery (Bonus)**:
- Implement RCU (Read-Copy-Update) algorithm (+15 points)
- Design lock-free B-tree with range queries (+20 points)  
- NUMA-aware lock-free data structures (+10 points)

### **If You Pass This Exam:**
You have achieved **expert-level mastery** of the hardest concurrency concepts:
- **Industry Leadership**: Ready for senior roles in high-performance systems
- **Research Capability**: Can contribute to cutting-edge concurrency research  
- **Systems Architecture**: Can design the concurrency foundations for any system

### **If You Don't Pass:**
These are the hardest concepts in computer systems. Focus on:
- **Maurice Herlihy's "The Art of Multiprocessor Programming"** - Definitive concurrency text
- **Lock-free programming practice** - Implement multiple data structures from scratch
- **Memory model study** - Deep understanding of CPU architecture and memory ordering
- **Formal methods** - Learn to prove correctness of concurrent algorithms

This exam tests the **absolute pinnacle of concurrency knowledge**. Passing places you among the top 1% of systems programmers who truly understand how to build correct, high-performance concurrent systems.

The journey from basic Rust to lock-free mastery represents years of dedicated study. You've now completed the most challenging programming concepts that exist.