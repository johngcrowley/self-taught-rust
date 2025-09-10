# Low-Level Systems Programming Mastery

**Expert-Level Operating Systems & Hardware Interface Programming**

*"Systems programming is where the rubber meets the road - you're directly interfacing with hardware, kernel, and network protocols."*

---

## Prerequisites

You must pass **06_unsafe_rust_mastery.md** (Advanced Unsafe Rust) before attempting this exam. This focuses on the **lowest-level systems programming** concepts in Rust.

**Core Requirements:**
- **Kernel Interface Mastery**: System calls, memory mapping, signal handling
- **Network Protocol Implementation**: Raw sockets, zero-copy networking, custom protocols  
- **Hardware Programming**: Memory-mapped I/O, interrupt simulation, device interfaces
- **Performance Engineering**: Assembly integration, NUMA awareness, real-time constraints

---

## Section I: Kernel Interface Programming (100 points)

### Problem 1.1: Custom Memory Allocator with mmap (40 points)

**Context**: Implement a high-performance allocator that directly interfaces with the kernel's virtual memory system.

```rust
use std::ptr::{self, NonNull};
use std::alloc::Layout;
use libc::{self, c_void, c_int, size_t};

/// High-performance allocator using direct mmap system calls
/// Must handle all edge cases and error conditions correctly
pub struct MmapAllocator {
    // Your design:
    // 1. How do you track allocated regions?
    // 2. How do you handle different page sizes?
    // 3. What about memory fragmentation?
    
    page_size: usize,
    large_alloc_threshold: usize,
    allocated_regions: Vec<MemoryRegion>,
}

#[derive(Debug)]
struct MemoryRegion {
    ptr: NonNull<u8>,
    size: usize,
    layout: RegionLayout,
}

#[derive(Debug)]
enum RegionLayout {
    SmallBlocks { block_size: usize, free_bitmap: u64 },
    LargeAllocation,
}

impl MmapAllocator {
    pub fn new() -> Result<Self, std::io::Error> {
        // Your implementation:
        // 1. Get system page size using sysconf
        // 2. Initialize tracking structures
        // 3. Set up large allocation threshold
        
        let page_size = unsafe {
            libc::sysconf(libc::_SC_PAGESIZE) as usize
        };
        
        if page_size == 0 {
            return Err(std::io::Error::last_os_error());
        }
        
        Ok(Self {
            page_size,
            large_alloc_threshold: page_size * 64, // 256KB threshold
            allocated_regions: Vec::new(),
        })
    }
    
    /// Allocate memory using mmap for large allocations or custom region management
    pub unsafe fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, std::io::Error> {
        // Your implementation:
        // 1. Decide between small block allocation vs large mmap
        // 2. For large allocations: direct mmap with size alignment
        // 3. For small allocations: suballocate from mmapped regions
        // 4. Handle alignment requirements correctly
        
        if layout.size() >= self.large_alloc_threshold {
            self.allocate_large(layout)
        } else {
            self.allocate_small(layout)
        }
    }
    
    unsafe fn allocate_large(&mut self, layout: Layout) -> Result<NonNull<u8>, std::io::Error> {
        // Your implementation:
        // 1. Round size up to page boundary
        // 2. Call mmap with appropriate flags
        // 3. Handle alignment requirements
        // 4. Track the allocation for later deallocation
        
        let size = self.round_up_to_page_size(layout.size());
        
        let ptr = libc::mmap(
            ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        );
        
        if ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }
        
        // Handle alignment - what if the returned pointer isn't aligned?
        let aligned_ptr = self.align_pointer(ptr as *mut u8, layout.align())?;
        
        // Track the region for cleanup
        self.allocated_regions.push(MemoryRegion {
            ptr: NonNull::new(aligned_ptr).unwrap(),
            size,
            layout: RegionLayout::LargeAllocation,
        });
        
        Ok(NonNull::new(aligned_ptr).unwrap())
    }
    
    unsafe fn allocate_small(&mut self, layout: Layout) -> Result<NonNull<u8>, std::io::Error> {
        // Your implementation:
        // 1. Find existing region with appropriate block size
        // 2. If none exists, create new region with mmap
        // 3. Suballocate within the region using bitmap tracking
        // 4. Handle fragmentation efficiently
        
        todo!("Implement small block allocation with region management")
    }
    
    /// Deallocate memory - must handle both large and small allocations
    pub unsafe fn deallocate(&mut self, ptr: NonNull<u8>) -> Result<(), std::io::Error> {
        // Your implementation:
        // 1. Find which region this pointer belongs to
        // 2. For large allocations: munmap the entire region
        // 3. For small allocations: mark block as free in bitmap
        // 4. Consolidate or release regions when fully free
        
        todo!("Implement deallocation with region tracking")
    }
    
    /// Get detailed memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        // Your implementation:
        // 1. Calculate total allocated vs used memory
        // 2. Track fragmentation metrics
        // 3. Count number of system calls made
        
        todo!("Implement comprehensive memory statistics")
    }
    
    /// Advise kernel about memory usage patterns
    pub unsafe fn memory_advice(&self, ptr: NonNull<u8>, size: usize, advice: MemoryAdvice) -> Result<(), std::io::Error> {
        // Your implementation using madvise:
        // 1. MADV_SEQUENTIAL - for sequential access patterns
        // 2. MADV_RANDOM - for random access patterns  
        // 3. MADV_WILLNEED - prefetch pages
        // 4. MADV_DONTNEED - release pages to kernel
        
        let madvise_flag = match advice {
            MemoryAdvice::Sequential => libc::MADV_SEQUENTIAL,
            MemoryAdvice::Random => libc::MADV_RANDOM,
            MemoryAdvice::WillNeed => libc::MADV_WILLNEED,
            MemoryAdvice::DontNeed => libc::MADV_DONTNEED,
        };
        
        let result = libc::madvise(ptr.as_ptr() as *mut c_void, size, madvise_flag);
        if result != 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    
    fn round_up_to_page_size(&self, size: usize) -> usize {
        (size + self.page_size - 1) & !(self.page_size - 1)
    }
    
    unsafe fn align_pointer(&self, ptr: *mut u8, align: usize) -> Result<*mut u8, std::io::Error> {
        // Handle over-alignment requirements
        let addr = ptr as usize;
        let aligned_addr = (addr + align - 1) & !(align - 1);
        
        if aligned_addr - addr > self.page_size {
            // Alignment requirement too large - need different strategy
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Alignment requirement too large"
            ));
        }
        
        Ok(aligned_addr as *mut u8)
    }
}

pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_used: usize,
    pub fragmentation_ratio: f64,
    pub region_count: usize,
    pub system_call_count: usize,
}

pub enum MemoryAdvice {
    Sequential,
    Random, 
    WillNeed,
    DontNeed,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_large_allocation() {
        let mut allocator = MmapAllocator::new().unwrap();
        let layout = Layout::from_size_align(1024 * 1024, 8).unwrap(); // 1MB
        
        unsafe {
            let ptr = allocator.allocate(layout).unwrap();
            
            // Verify allocation is usable
            std::ptr::write(ptr.as_ptr(), 42u8);
            assert_eq!(std::ptr::read(ptr.as_ptr()), 42u8);
            
            allocator.deallocate(ptr).unwrap();
        }
    }
    
    #[test]
    fn test_small_allocation() {
        // Test small allocations with different sizes and alignments
    }
    
    #[test]
    fn test_fragmentation() {
        // Test allocation patterns that cause fragmentation
        // Verify the allocator handles it efficiently
    }
}
```

**Questions (40 points):**
1. **System Call Analysis (15 points)**: When do you use mmap vs brk? What are the performance implications?
2. **Memory Management Strategy (15 points)**: How do you minimize fragmentation while maintaining performance?
3. **Error Handling (10 points)**: Handle all possible mmap failure modes correctly

### Problem 1.2: Signal-Safe Async Runtime (35 points)

**Context**: Build an async runtime that can handle Unix signals safely without deadlocks or data corruption.

```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::os::unix::io::{AsRawFd, RawFd};
use libc::{self, c_int, sigset_t, siginfo_t};
use std::mem::MaybeUninit;

/// Async runtime that safely handles Unix signals
/// Must prevent signal-async interaction bugs
pub struct SignalSafeRuntime {
    // Your design:
    // 1. How do you handle signals without blocking async tasks?
    // 2. What data structures are signal-safe?
    // 3. How do you coordinate between signal handlers and async tasks?
    
    signal_fd: RawFd,
    blocked_signals: sigset_t,
    signal_handlers: Arc<Mutex<SignalHandlerRegistry>>,
    shutdown_requested: AtomicBool,
}

struct SignalHandlerRegistry {
    handlers: std::collections::HashMap<c_int, Box<dyn Fn(SignalInfo) + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct SignalInfo {
    pub signal: c_int,
    pub pid: libc::pid_t,
    pub uid: libc::uid_t,
    pub code: c_int,
}

impl SignalSafeRuntime {
    /// Create runtime with signal handling capability
    pub fn new() -> Result<Self, RuntimeError> {
        // Your implementation:
        // 1. Block all signals for the main thread
        // 2. Create signalfd for synchronous signal handling
        // 3. Set up signal mask correctly
        // 4. Initialize data structures
        
        unsafe {
            let mut blocked_signals: sigset_t = std::mem::zeroed();
            
            // Block all signals initially
            libc::sigfillset(&mut blocked_signals);
            
            if libc::pthread_sigmask(libc::SIG_BLOCK, &blocked_signals, std::ptr::null_mut()) != 0 {
                return Err(RuntimeError::SignalSetupFailed);
            }
            
            // Create signalfd for synchronous signal handling
            let signal_fd = libc::signalfd(-1, &blocked_signals, libc::SFD_CLOEXEC);
            if signal_fd == -1 {
                return Err(RuntimeError::SignalFdCreationFailed);
            }
            
            Ok(Self {
                signal_fd,
                blocked_signals,
                signal_handlers: Arc::new(Mutex::new(SignalHandlerRegistry {
                    handlers: std::collections::HashMap::new(),
                })),
                shutdown_requested: AtomicBool::new(false),
            })
        }
    }
    
    /// Register a signal handler (must be signal-safe)
    pub fn register_signal_handler<F>(&self, signal: c_int, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(SignalInfo) + Send + Sync + 'static,
    {
        // Your implementation:
        // 1. Add signal to signalfd mask
        // 2. Store handler in registry
        // 3. Update signal mask atomically
        
        let mut handlers = self.signal_handlers.lock()
            .map_err(|_| RuntimeError::LockPoisoned)?;
            
        handlers.handlers.insert(signal, Box::new(handler));
        
        // Add this signal to our signalfd
        unsafe {
            let mut new_mask = self.blocked_signals;
            libc::sigaddset(&mut new_mask, signal);
            
            let new_fd = libc::signalfd(self.signal_fd, &new_mask, 0);
            if new_fd == -1 {
                return Err(RuntimeError::SignalUpdateFailed);
            }
        }
        
        Ok(())
    }
    
    /// Main runtime loop - handles both async tasks and signals
    pub async fn run(&self) -> Result<(), RuntimeError> {
        // Your implementation:
        // 1. Create async task scheduler
        // 2. Set up signal monitoring task
        // 3. Coordinate between signal handling and task execution
        // 4. Handle graceful shutdown
        
        let signal_task = self.spawn_signal_monitor();
        let scheduler_task = self.spawn_task_scheduler();
        
        // Wait for either task to complete or shutdown signal
        tokio::select! {
            result = signal_task => {
                eprintln!("Signal monitor exited: {:?}", result);
            }
            result = scheduler_task => {
                eprintln!("Task scheduler exited: {:?}", result);
            }
            _ = self.wait_for_shutdown() => {
                eprintln!("Shutdown requested");
            }
        }
        
        Ok(())
    }
    
    async fn spawn_signal_monitor(&self) -> Result<(), RuntimeError> {
        // Your implementation:
        // 1. Monitor signalfd for incoming signals
        // 2. Parse signalfd_siginfo structure
        // 3. Dispatch to registered handlers
        // 4. Handle signal handler errors gracefully
        
        use tokio::io::unix::AsyncFd;
        
        let async_fd = AsyncFd::new(self.signal_fd)
            .map_err(|_| RuntimeError::AsyncFdSetupFailed)?;
            
        loop {
            let mut guard = async_fd.readable().await
                .map_err(|_| RuntimeError::SignalMonitorFailed)?;
                
            // Read signal info from signalfd
            let mut buffer: [u8; std::mem::size_of::<libc::signalfd_siginfo>()] = [0; 128];
            
            match guard.try_io(|inner| {
                let result = unsafe {
                    libc::read(
                        inner.as_raw_fd(),
                        buffer.as_mut_ptr() as *mut libc::c_void,
                        buffer.len(),
                    )
                };
                
                if result > 0 {
                    Ok(result as usize)
                } else {
                    Err(std::io::Error::last_os_error())
                }
            }) {
                Ok(Ok(bytes_read)) => {
                    if bytes_read >= std::mem::size_of::<libc::signalfd_siginfo>() {
                        self.handle_signal_info(&buffer).await?;
                    }
                }
                Ok(Err(e)) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        return Err(RuntimeError::SignalReadFailed);
                    }
                }
                Err(_) => {
                    // Would block - continue loop
                    continue;
                }
            }
            
            guard.clear_ready();
        }
    }
    
    async fn handle_signal_info(&self, buffer: &[u8]) -> Result<(), RuntimeError> {
        // Your implementation:
        // 1. Parse signalfd_siginfo from buffer
        // 2. Extract signal number, pid, uid, etc.
        // 3. Look up registered handler
        // 4. Execute handler in async-safe manner
        
        unsafe {
            let siginfo = &*(buffer.as_ptr() as *const libc::signalfd_siginfo);
            
            let signal_info = SignalInfo {
                signal: siginfo.ssi_signo as c_int,
                pid: siginfo.ssi_pid as libc::pid_t,
                uid: siginfo.ssi_uid as libc::uid_t,
                code: siginfo.ssi_code as c_int,
            };
            
            // Find and execute handler
            let handlers = self.signal_handlers.lock()
                .map_err(|_| RuntimeError::LockPoisoned)?;
                
            if let Some(handler) = handlers.handlers.get(&signal_info.signal) {
                // Execute handler - must be careful about async safety
                tokio::task::spawn_blocking({
                    let handler = handler.clone();
                    let signal_info = signal_info.clone();
                    move || {
                        handler(signal_info);
                    }
                }).await
                .map_err(|_| RuntimeError::HandlerExecutionFailed)?;
            }
        }
        
        Ok(())
    }
    
    async fn spawn_task_scheduler(&self) -> Result<(), RuntimeError> {
        // Your implementation:
        // 1. Create work-stealing task scheduler
        // 2. Handle task spawning and completion
        // 3. Coordinate with signal handling
        // 4. Implement graceful shutdown
        
        todo!("Implement async task scheduler")
    }
    
    async fn wait_for_shutdown(&self) {
        while !self.shutdown_requested.load(Ordering::Acquire) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    pub fn shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::Release);
    }
}

impl Drop for SignalSafeRuntime {
    fn drop(&mut self) {
        unsafe {
            if self.signal_fd >= 0 {
                libc::close(self.signal_fd);
            }
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    SignalSetupFailed,
    SignalFdCreationFailed,
    SignalUpdateFailed,
    AsyncFdSetupFailed,
    SignalMonitorFailed,
    SignalReadFailed,
    HandlerExecutionFailed,
    LockPoisoned,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_signal_handling() {
        let runtime = SignalSafeRuntime::new().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        
        let counter_clone = counter.clone();
        runtime.register_signal_handler(libc::SIGUSR1, move |_| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        }).unwrap();
        
        // Send signal to self
        unsafe {
            libc::kill(libc::getpid(), libc::SIGUSR1);
        }
        
        // Give time for signal processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
    
    #[tokio::test]
    async fn test_concurrent_signals() {
        // Test handling multiple signals concurrently
        // Verify no race conditions or data corruption
    }
}
```

**Questions (35 points):**
1. **Signal Safety Analysis (15 points)**: Why is signalfd safer than traditional signal handlers?
2. **Async Coordination (10 points)**: How do you prevent deadlocks between signal handling and async tasks?
3. **Performance Implications (10 points)**: What's the overhead of signal-safe async runtime?

### Problem 1.3: Zero-Copy File I/O with io_uring (25 points)

**Context**: Implement high-performance file I/O using Linux's io_uring interface.

```rust
use std::os::unix::io::{AsRawFd, RawFd};
use std::ptr;
use std::mem::{self, MaybeUninit};
use libc::{self, c_void, off_t};

/// High-performance file I/O using io_uring
/// Must achieve maximum throughput with minimum CPU usage
pub struct IoUringFile {
    fd: RawFd,
    ring: IoUring,
    buffer_pool: BufferPool,
}

struct IoUring {
    // Your design for io_uring interface:
    // 1. Submission queue (SQ)
    // 2. Completion queue (CQ)  
    // 3. Memory mapping details
    // 4. Ring buffer management
    
    ring_fd: RawFd,
    sq_ring_ptr: *mut u8,
    cq_ring_ptr: *mut u8,
    sqes: *mut IoUringSqe,
    sq_ring_mask: u32,
    cq_ring_mask: u32,
    sq_ring_entries: u32,
    cq_ring_entries: u32,
}

#[repr(C)]
struct IoUringSqe {
    opcode: u8,
    flags: u8,
    ioprio: u16,
    fd: i32,
    off: u64,
    addr: u64,
    len: u32,
    user_data: u64,
}

#[repr(C)]
struct IoUringCqe {
    user_data: u64,
    res: i32,
    flags: u32,
}

impl IoUringFile {
    /// Create new io_uring-based file with specified queue depth
    pub fn open(path: &str, queue_depth: u32) -> Result<Self, std::io::Error> {
        // Your implementation:
        // 1. Open file with O_DIRECT for zero-copy I/O
        // 2. Set up io_uring with specified queue depth
        // 3. Map ring buffers into process memory
        // 4. Initialize buffer pool for aligned I/O
        
        use std::ffi::CString;
        
        let path_cstr = CString::new(path).unwrap();
        let fd = unsafe {
            libc::open(
                path_cstr.as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_DIRECT,
                0o644
            )
        };
        
        if fd == -1 {
            return Err(std::io::Error::last_os_error());
        }
        
        let ring = IoUring::new(queue_depth)?;
        let buffer_pool = BufferPool::new(64, 4096); // 64 buffers, 4KB each
        
        Ok(Self {
            fd,
            ring,
            buffer_pool,
        })
    }
    
    /// Perform asynchronous read with zero-copy semantics
    pub async fn read_async(&mut self, offset: u64, length: usize) -> Result<Vec<u8>, std::io::Error> {
        // Your implementation:
        // 1. Get aligned buffer from pool
        // 2. Submit read operation to io_uring
        // 3. Wait for completion asynchronously
        // 4. Handle partial reads and errors
        
        let buffer = self.buffer_pool.get_buffer(length)?;
        let user_data = buffer.as_ptr() as u64; // Use buffer address as completion identifier
        
        // Submit read request
        self.ring.submit_read(
            self.fd,
            offset,
            buffer.as_ptr() as u64,
            length as u32,
            user_data,
        )?;
        
        // Wait for completion
        let completion = self.ring.wait_for_completion(user_data).await?;
        
        if completion.res < 0 {
            return Err(std::io::Error::from_raw_os_error(-completion.res));
        }
        
        // Copy data from aligned buffer to result
        let bytes_read = completion.res as usize;
        let mut result = vec![0u8; bytes_read];
        unsafe {
            ptr::copy_nonoverlapping(buffer.as_ptr(), result.as_mut_ptr(), bytes_read);
        }
        
        self.buffer_pool.return_buffer(buffer);
        Ok(result)
    }
    
    /// Perform asynchronous write with zero-copy semantics
    pub async fn write_async(&mut self, offset: u64, data: &[u8]) -> Result<usize, std::io::Error> {
        // Your implementation:
        // 1. Get aligned buffer and copy data
        // 2. Submit write operation to io_uring  
        // 3. Wait for completion
        // 4. Handle partial writes
        
        todo!("Implement zero-copy async write")
    }
    
    /// Batch multiple I/O operations for maximum throughput
    pub async fn batch_io(&mut self, operations: Vec<IoOperation>) -> Result<Vec<IoResult>, std::io::Error> {
        // Your implementation:
        // 1. Submit all operations to the ring
        // 2. Wait for all completions
        // 3. Match completions to original operations
        // 4. Handle mixed success/failure scenarios
        
        todo!("Implement batched I/O operations")
    }
}

impl IoUring {
    fn new(entries: u32) -> Result<Self, std::io::Error> {
        // Your implementation:
        // 1. Call io_uring_setup system call
        // 2. Map submission and completion queues
        // 3. Initialize ring buffer pointers
        // 4. Set up proper memory barriers
        
        todo!("Implement io_uring initialization")
    }
    
    fn submit_read(&mut self, fd: RawFd, offset: u64, addr: u64, len: u32, user_data: u64) -> Result<(), std::io::Error> {
        // Your implementation:
        // 1. Get next SQE from submission queue
        // 2. Fill in read operation details
        // 3. Update queue tail pointer
        // 4. Call io_uring_enter if needed
        
        todo!("Implement read submission")
    }
    
    async fn wait_for_completion(&mut self, user_data: u64) -> Result<IoUringCqe, std::io::Error> {
        // Your implementation:
        // 1. Poll completion queue
        // 2. Handle completion queue overflow
        // 3. Match user_data to find specific completion
        // 4. Use async/await for non-blocking operation
        
        todo!("Implement async completion waiting")
    }
}

/// Buffer pool for aligned I/O operations
struct BufferPool {
    buffers: Vec<AlignedBuffer>,
    available: std::collections::VecDeque<usize>,
}

struct AlignedBuffer {
    ptr: NonNull<u8>,
    size: usize,
    alignment: usize,
}

impl BufferPool {
    fn new(count: usize, buffer_size: usize) -> Self {
        // Your implementation:
        // 1. Allocate aligned buffers for O_DIRECT I/O
        // 2. Initialize availability tracking
        // 3. Handle different alignment requirements
        
        todo!("Implement aligned buffer pool")
    }
    
    fn get_buffer(&mut self, min_size: usize) -> Result<&mut AlignedBuffer, std::io::Error> {
        // Get buffer that's at least min_size bytes
        todo!("Implement buffer allocation")
    }
    
    fn return_buffer(&mut self, buffer: &AlignedBuffer) {
        // Return buffer to available pool
        todo!("Implement buffer return")
    }
}

pub enum IoOperation {
    Read { offset: u64, length: usize },
    Write { offset: u64, data: Vec<u8> },
    Fsync,
}

pub struct IoResult {
    pub operation_id: usize,
    pub result: Result<Vec<u8>, std::io::Error>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_zero_copy_read() {
        let mut file = IoUringFile::open("/tmp/test_file", 32).unwrap();
        let data = file.read_async(0, 4096).await.unwrap();
        // Verify read succeeded and data is correct
    }
    
    #[tokio::test]
    async fn test_batch_io_performance() {
        // Benchmark batch I/O vs individual operations
        // Verify significant performance improvement
    }
}
```

**Questions (25 points):**
1. **io_uring Architecture (10 points)**: Explain how io_uring achieves zero-copy I/O
2. **Performance Analysis (10 points)**: Compare io_uring vs epoll vs synchronous I/O
3. **Error Handling (5 points)**: Handle all io_uring failure modes correctly

---

## Section II: Network Protocol Implementation (100 points)

### Problem 2.1: Custom Network Protocol with Raw Sockets (50 points)

**Context**: Implement a high-performance network protocol using raw sockets with custom packet format.

```rust
use std::net::{IpAddr, Ipv4Addr};
use std::os::unix::io::RawFd;
use libc::{self, sockaddr_in, c_int, c_void};
use std::mem;

/// Custom high-performance network protocol implementation
/// Must handle packet fragmentation, reordering, and loss recovery
pub struct CustomProtocol {
    // Your design:
    // 1. Raw socket management
    // 2. Packet sequence numbering
    // 3. Acknowledgment tracking
    // 4. Congestion control
    
    raw_socket: RawFd,
    local_addr: Ipv4Addr,
    remote_addr: Ipv4Addr,
    sequence_number: std::sync::atomic::AtomicU32,
    receive_window: ReceiveWindow,
    send_window: SendWindow,
}

/// Custom packet format optimized for low latency
#[repr(C, packed)]
struct ProtocolHeader {
    version: u8,
    packet_type: u8,
    flags: u8,
    header_length: u8,
    sequence_number: u32,
    acknowledgment: u32,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
    timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum PacketType {
    Data = 0,
    Ack = 1,
    Syn = 2,
    Fin = 3,
    Ping = 4,
    Pong = 5,
}

impl CustomProtocol {
    /// Create new protocol instance with raw socket
    pub fn new(local_addr: Ipv4Addr, remote_addr: Ipv4Addr) -> Result<Self, NetworkError> {
        // Your implementation:
        // 1. Create raw socket with IPPROTO_RAW
        // 2. Set socket options for performance
        // 3. Initialize sliding windows
        // 4. Set up packet processing
        
        let socket = unsafe {
            libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_RAW)
        };
        
        if socket == -1 {
            return Err(NetworkError::SocketCreationFailed);
        }
        
        // Enable IP header inclusion
        let flag = 1i32;
        unsafe {
            libc::setsockopt(
                socket,
                libc::IPPROTO_IP,
                libc::IP_HDRINCL,
                &flag as *const i32 as *const c_void,
                mem::size_of::<i32>() as libc::socklen_t,
            );
        }
        
        Ok(Self {
            raw_socket: socket,
            local_addr,
            remote_addr,
            sequence_number: std::sync::atomic::AtomicU32::new(1),
            receive_window: ReceiveWindow::new(65536),
            send_window: SendWindow::new(65536),
        })
    }
    
    /// Send data with reliability guarantees
    pub async fn send_reliable(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        // Your implementation:
        // 1. Fragment data if larger than MTU
        // 2. Add sequence numbers and checksums
        // 3. Implement stop-and-wait or sliding window
        // 4. Handle acknowledgments and retransmissions
        
        let mtu = 1500 - mem::size_of::<ProtocolHeader>() - 20; // IP header size
        let chunks: Vec<&[u8]> = data.chunks(mtu).collect();
        
        for chunk in chunks {
            let seq = self.sequence_number.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            let packet = self.create_data_packet(seq, chunk)?;
            self.send_packet(&packet).await?;
            
            // Wait for acknowledgment or timeout
            self.wait_for_ack(seq).await?;
        }
        
        Ok(())
    }
    
    /// Receive data with ordering guarantees
    pub async fn receive_reliable(&mut self) -> Result<Vec<u8>, NetworkError> {
        // Your implementation:
        // 1. Receive raw packets from socket
        // 2. Parse and validate headers
        // 3. Handle out-of-order delivery
        // 4. Send acknowledgments
        // 5. Reassemble fragmented messages
        
        todo!("Implement reliable receive with reordering")
    }
    
    /// Send data with minimal latency (no reliability)
    pub async fn send_unreliable(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        // Your implementation:
        // 1. Create packet with minimal header
        // 2. Send immediately without waiting for ACK
        // 3. Optimize for absolute minimum latency
        
        let seq = self.sequence_number.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let packet = self.create_data_packet(seq, data)?;
        self.send_packet(&packet).await
    }
    
    fn create_data_packet(&self, sequence: u32, data: &[u8]) -> Result<Vec<u8>, NetworkError> {
        // Your implementation:
        // 1. Create IP header
        // 2. Create custom protocol header
        // 3. Calculate checksums
        // 4. Combine headers and data
        
        let total_length = 20 + mem::size_of::<ProtocolHeader>() + data.len(); // IP + custom + data
        let mut packet = Vec::with_capacity(total_length);
        
        // IP Header (simplified)
        let ip_header = create_ip_header(
            self.local_addr,
            self.remote_addr,
            total_length as u16,
        )?;
        packet.extend_from_slice(&ip_header);
        
        // Custom Protocol Header
        let protocol_header = ProtocolHeader {
            version: 1,
            packet_type: PacketType::Data as u8,
            flags: 0,
            header_length: mem::size_of::<ProtocolHeader>() as u8,
            sequence_number: sequence.to_be(),
            acknowledgment: 0,
            window_size: (self.receive_window.available_space() as u16).to_be(),
            checksum: 0, // Calculate after
            urgent_pointer: 0,
            timestamp: get_timestamp_micros().to_be(),
        };
        
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &protocol_header as *const _ as *const u8,
                mem::size_of::<ProtocolHeader>(),
            )
        };
        packet.extend_from_slice(header_bytes);
        packet.extend_from_slice(data);
        
        // Calculate and update checksum
        let checksum = calculate_checksum(&packet[20..]); // Skip IP header
        let checksum_offset = 20 + offset_of!(ProtocolHeader, checksum);
        packet[checksum_offset..checksum_offset + 2].copy_from_slice(&checksum.to_be_bytes());
        
        Ok(packet)
    }
    
    async fn send_packet(&self, packet: &[u8]) -> Result<(), NetworkError> {
        // Your implementation:
        // 1. Send packet via raw socket
        // 2. Handle EAGAIN/EWOULDBLOCK
        // 3. Use sendto with proper addressing
        
        let addr = sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: u32::from(self.remote_addr).to_be(),
            },
            sin_zero: [0; 8],
        };
        
        let result = unsafe {
            libc::sendto(
                self.raw_socket,
                packet.as_ptr() as *const c_void,
                packet.len(),
                0,
                &addr as *const sockaddr_in as *const libc::sockaddr,
                mem::size_of::<sockaddr_in>() as libc::socklen_t,
            )
        };
        
        if result == -1 {
            Err(NetworkError::SendFailed)
        } else {
            Ok(())
        }
    }
    
    async fn wait_for_ack(&mut self, sequence: u32) -> Result<(), NetworkError> {
        // Your implementation:
        // 1. Set up timeout for retransmission
        // 2. Poll socket for incoming ACK packets
        // 3. Handle duplicate ACKs
        // 4. Implement exponential backoff for retransmissions
        
        todo!("Implement ACK waiting with timeout and retransmission")
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> ProtocolStats {
        ProtocolStats {
            packets_sent: self.send_window.packets_sent(),
            packets_received: self.receive_window.packets_received(),
            bytes_sent: self.send_window.bytes_sent(),
            bytes_received: self.receive_window.bytes_received(),
            retransmissions: self.send_window.retransmissions(),
            out_of_order_packets: self.receive_window.out_of_order_count(),
            average_rtt: self.send_window.average_rtt(),
        }
    }
}

// Support structures for windowing and reliability
struct ReceiveWindow {
    buffer: Vec<Option<Vec<u8>>>,
    base_sequence: u32,
    window_size: usize,
    bytes_received: std::sync::atomic::AtomicU64,
    packets_received: std::sync::atomic::AtomicU64,
}

struct SendWindow {
    unacked_packets: std::collections::BTreeMap<u32, PendingPacket>,
    base_sequence: u32,
    window_size: usize,
    bytes_sent: std::sync::atomic::AtomicU64,
    packets_sent: std::sync::atomic::AtomicU64,
    retransmissions: std::sync::atomic::AtomicU64,
    rtt_samples: std::collections::VecDeque<std::time::Duration>,
}

struct PendingPacket {
    data: Vec<u8>,
    sent_time: std::time::Instant,
    retransmit_count: u32,
}

pub struct ProtocolStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub retransmissions: u64,
    pub out_of_order_packets: u64,
    pub average_rtt: std::time::Duration,
}

#[derive(Debug)]
pub enum NetworkError {
    SocketCreationFailed,
    SendFailed,
    ReceiveFailed,
    ChecksumMismatch,
    SequenceError,
    TimeoutError,
}

// Helper functions
fn create_ip_header(src: Ipv4Addr, dst: Ipv4Addr, total_length: u16) -> Result<[u8; 20], NetworkError> {
    // Implement basic IP header creation
    todo!("Implement IP header creation")
}

fn calculate_checksum(data: &[u8]) -> u16 {
    // Implement Internet checksum algorithm
    let mut sum = 0u32;
    let mut i = 0;
    
    while i < data.len() - 1 {
        sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
        i += 2;
    }
    
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    
    !(sum as u16)
}

fn get_timestamp_micros() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reliable_transmission() {
        // Test that data is transmitted reliably with loss recovery
        // This requires root privileges for raw sockets
    }
    
    #[tokio::test]
    async fn test_performance_vs_tcp() {
        // Benchmark custom protocol vs TCP for various scenarios
        // Measure latency, throughput, and CPU usage
    }
}
```

**Questions (50 points):**
1. **Protocol Design (20 points)**: Design efficient packet format and explain design choices
2. **Reliability Implementation (20 points)**: Implement sliding window with proper congestion control
3. **Performance Analysis (10 points)**: Compare your protocol's performance vs TCP/UDP

### Problem 2.2: Zero-Copy Network Server (50 points)

**Context**: Build a high-performance network server using zero-copy techniques and kernel bypass.

```rust
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use libc::{self, c_int, c_void, sockaddr_in, socklen_t};

/// High-performance network server with zero-copy I/O
/// Must achieve 1M+ connections and minimal per-connection overhead
pub struct ZeroCopyServer {
    // Your design:
    // 1. How do you handle massive connection counts?
    // 2. What zero-copy techniques can you use?
    // 3. How do you minimize memory allocations?
    // 4. What about CPU cache optimization?
    
    listen_fd: RawFd,
    epoll_fd: RawFd,
    connection_pool: ConnectionPool,
    buffer_pool: SharedBufferPool,
    active_connections: AtomicUsize,
    stats: Arc<ServerStats>,
}

/// Connection state with minimal memory footprint
struct Connection {
    fd: RawFd,
    state: ConnectionState,
    read_buffer_offset: usize,
    write_buffer_offset: usize,
    last_activity: std::time::Instant,
    // Minimize to fit in single cache line
}

#[derive(Debug, Clone, Copy)]
enum ConnectionState {
    Reading,
    Writing,
    Closing,
}

/// Memory pool for zero-copy buffer management
struct SharedBufferPool {
    // Your design:
    // 1. Large pre-allocated memory regions
    // 2. Lock-free allocation/deallocation
    // 3. NUMA-aware buffer placement
    // 4. Automatic memory advice to kernel
    
    regions: Vec<MemoryRegion>,
    free_buffers: crossbeam::queue::SegQueue<BufferHandle>,
    total_allocated: AtomicUsize,
}

struct MemoryRegion {
    ptr: std::ptr::NonNull<u8>,
    size: usize,
    numa_node: Option<usize>,
}

struct BufferHandle {
    ptr: std::ptr::NonNull<u8>,
    size: usize,
    region_id: usize,
}

impl ZeroCopyServer {
    /// Create server with specified configuration
    pub fn new(port: u16, config: ServerConfig) -> Result<Self, ServerError> {
        // Your implementation:
        // 1. Create listening socket with SO_REUSEPORT
        // 2. Set up epoll for edge-triggered I/O
        // 3. Initialize connection and buffer pools
        // 4. Configure socket options for performance
        
        let listen_fd = unsafe {
            libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_NONBLOCK, 0)
        };
        
        if listen_fd == -1 {
            return Err(ServerError::SocketCreationFailed);
        }
        
        // Set SO_REUSEPORT for multi-process scaling
        let flag = 1i32;
        unsafe {
            libc::setsockopt(
                listen_fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEPORT,
                &flag as *const i32 as *const c_void,
                std::mem::size_of::<i32>() as socklen_t,
            );
        }
        
        // Bind to port
        let addr = sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: port.to_be(),
            sin_addr: libc::in_addr { s_addr: 0 }, // INADDR_ANY
            sin_zero: [0; 8],
        };
        
        let bind_result = unsafe {
            libc::bind(
                listen_fd,
                &addr as *const sockaddr_in as *const libc::sockaddr,
                std::mem::size_of::<sockaddr_in>() as socklen_t,
            )
        };
        
        if bind_result == -1 {
            return Err(ServerError::BindFailed);
        }
        
        // Start listening
        unsafe {
            libc::listen(listen_fd, 128);
        }
        
        // Create epoll instance
        let epoll_fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
        if epoll_fd == -1 {
            return Err(ServerError::EpollCreationFailed);
        }
        
        // Add listening socket to epoll
        let mut event = libc::epoll_event {
            events: (libc::EPOLLIN | libc::EPOLLET) as u32,
            u64: listen_fd as u64,
        };
        
        unsafe {
            libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, listen_fd, &mut event);
        }
        
        Ok(Self {
            listen_fd,
            epoll_fd,
            connection_pool: ConnectionPool::new(config.max_connections),
            buffer_pool: SharedBufferPool::new(config.buffer_pool_size),
            active_connections: AtomicUsize::new(0),
            stats: Arc::new(ServerStats::new()),
        })
    }
    
    /// Run server event loop
    pub async fn run<H>(&mut self, handler: H) -> Result<(), ServerError>
    where
        H: ConnectionHandler + Send + Sync + 'static,
    {
        // Your implementation:
        // 1. Main epoll event loop
        // 2. Accept new connections
        // 3. Handle I/O events with zero-copy
        // 4. Clean up idle/closed connections
        
        let handler = Arc::new(handler);
        let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; 1024];
        
        loop {
            let event_count = unsafe {
                libc::epoll_wait(
                    self.epoll_fd,
                    events.as_mut_ptr(),
                    events.len() as c_int,
                    100, // 100ms timeout
                )
            };
            
            if event_count == -1 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(ServerError::EpollWaitFailed);
            }
            
            for i in 0..event_count as usize {
                let event = events[i];
                let fd = event.u64 as RawFd;
                
                if fd == self.listen_fd {
                    // Accept new connections
                    self.handle_accept().await?;
                } else {
                    // Handle existing connection I/O
                    self.handle_connection_io(fd, event.events, &handler).await?;
                }
            }
            
            // Periodic cleanup
            self.cleanup_idle_connections().await;
        }
    }
    
    async fn handle_accept(&mut self) -> Result<(), ServerError> {
        // Your implementation:
        // 1. Accept multiple connections in a loop (EPOLLET)
        // 2. Set connection to non-blocking
        // 3. Add to epoll with appropriate events
        // 4. Initialize connection state
        
        loop {
            let conn_fd = unsafe {
                libc::accept4(
                    self.listen_fd,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    libc::SOCK_NONBLOCK,
                )
            };
            
            if conn_fd == -1 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::WouldBlock {
                    break; // No more connections
                }
                continue; // Other errors, try again
            }
            
            // Set TCP_NODELAY for low latency
            let flag = 1i32;
            unsafe {
                libc::setsockopt(
                    conn_fd,
                    libc::IPPROTO_TCP,
                    libc::TCP_NODELAY,
                    &flag as *const i32 as *const c_void,
                    std::mem::size_of::<i32>() as socklen_t,
                );
            }
            
            // Add to epoll
            let mut event = libc::epoll_event {
                events: (libc::EPOLLIN | libc::EPOLLOUT | libc::EPOLLET) as u32,
                u64: conn_fd as u64,
            };
            
            unsafe {
                libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, conn_fd, &mut event);
            }
            
            // Initialize connection
            let connection = Connection {
                fd: conn_fd,
                state: ConnectionState::Reading,
                read_buffer_offset: 0,
                write_buffer_offset: 0,
                last_activity: std::time::Instant::now(),
            };
            
            self.connection_pool.add_connection(conn_fd, connection)?;
            self.active_connections.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(())
    }
    
    async fn handle_connection_io<H>(
        &mut self,
        fd: RawFd,
        events: u32,
        handler: &Arc<H>,
    ) -> Result<(), ServerError>
    where
        H: ConnectionHandler + Send + Sync + 'static,
    {
        // Your implementation:
        // 1. Handle read events with zero-copy buffers
        // 2. Handle write events with scatter-gather I/O
        // 3. Handle connection errors and cleanup
        // 4. Update connection state and statistics
        
        if events & (libc::EPOLLHUP | libc::EPOLLERR) as u32 != 0 {
            self.close_connection(fd).await;
            return Ok(());
        }
        
        if events & libc::EPOLLIN as u32 != 0 {
            self.handle_read_event(fd, handler).await?;
        }
        
        if events & libc::EPOLLOUT as u32 != 0 {
            self.handle_write_event(fd).await?;
        }
        
        Ok(())
    }
    
    async fn handle_read_event<H>(&mut self, fd: RawFd, handler: &Arc<H>) -> Result<(), ServerError>
    where
        H: ConnectionHandler + Send + Sync + 'static,
    {
        // Your implementation:
        // 1. Get zero-copy buffer from pool
        // 2. Read data using recvmsg/readv for efficiency
        // 3. Parse protocol/HTTP headers without copying
        // 4. Call handler with buffer references
        // 5. Handle partial reads correctly
        
        let buffer = self.buffer_pool.get_buffer(8192)?;
        
        loop {
            let bytes_read = unsafe {
                libc::read(
                    fd,
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.size(),
                )
            };
            
            if bytes_read == -1 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::WouldBlock {
                    break; // No more data
                }
                self.close_connection(fd).await;
                return Ok(());
            }
            
            if bytes_read == 0 {
                // Connection closed
                self.close_connection(fd).await;
                return Ok(());
            }
            
            // Process data with handler
            let data = unsafe {
                std::slice::from_raw_parts(buffer.as_ptr(), bytes_read as usize)
            };
            
            let response = handler.handle_request(fd, data).await;
            self.queue_response(fd, response).await?;
        }
        
        self.buffer_pool.return_buffer(buffer);
        Ok(())
    }
    
    async fn handle_write_event(&mut self, fd: RawFd) -> Result<(), ServerError> {
        // Your implementation:
        // 1. Get pending response data for connection
        // 2. Use sendfile or writev for zero-copy sending
        // 3. Handle partial writes
        // 4. Clean up completed responses
        
        todo!("Implement zero-copy write handling")
    }
    
    async fn close_connection(&mut self, fd: RawFd) {
        unsafe {
            libc::close(fd);
        }
        self.connection_pool.remove_connection(fd);
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
    
    async fn cleanup_idle_connections(&mut self) {
        // Clean up connections idle for too long
        // This prevents resource exhaustion
        todo!("Implement idle connection cleanup")
    }
    
    pub fn get_stats(&self) -> ServerStats {
        self.stats.snapshot()
    }
}

/// Handler trait for processing requests
pub trait ConnectionHandler {
    async fn handle_request(&self, connection_id: RawFd, data: &[u8]) -> Vec<u8>;
}

pub struct ServerConfig {
    pub max_connections: usize,
    pub buffer_pool_size: usize,
    pub idle_timeout: std::time::Duration,
}

pub struct ServerStats {
    pub active_connections: usize,
    pub total_connections: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub requests_processed: u64,
}

#[derive(Debug)]
pub enum ServerError {
    SocketCreationFailed,
    BindFailed,
    EpollCreationFailed,
    EpollWaitFailed,
    OutOfConnections,
    OutOfBuffers,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct EchoHandler;
    
    impl ConnectionHandler for EchoHandler {
        async fn handle_request(&self, _connection_id: RawFd, data: &[u8]) -> Vec<u8> {
            data.to_vec() // Simple echo
        }
    }
    
    #[tokio::test]
    async fn test_high_connection_count() {
        // Test server with 10,000+ concurrent connections
        // This requires adjusting ulimits
    }
    
    #[tokio::test]
    async fn test_zero_copy_performance() {
        // Benchmark against standard async server implementations
        // Measure memory allocations and CPU usage
    }
}
```

**Questions (50 points):**
1. **Zero-Copy Techniques (20 points)**: Implement sendfile, splice, or similar zero-copy mechanisms
2. **Memory Management (15 points)**: Design efficient buffer pooling to minimize allocations
3. **Scalability Analysis (15 points)**: How does your server scale with connection count and load?

---

## Section III: Hardware Interface Programming (50 points)

### Problem 3.1: Memory-Mapped Device Interface (25 points)

**Context**: Create a safe interface for memory-mapped hardware devices.

```rust
use std::ptr::{read_volatile, write_volatile};
use std::marker::PhantomData;
use libc::{self, c_void, off_t};

/// Safe interface for memory-mapped hardware devices
/// Must prevent undefined behavior while allowing hardware access
pub struct MemoryMappedDevice<T> {
    // Your design:
    // 1. How do you safely map device memory?
    // 2. How do you handle different register types?
    // 3. What about cache coherency issues?
    // 4. How do you prevent compiler optimizations that break hardware access?
    
    base_addr: *mut u8,
    size: usize,
    _phantom: PhantomData<T>,
}

/// Hardware register with type-safe access
pub struct Register<T> {
    addr: *mut T,
    _phantom: PhantomData<T>,
}

impl<T> Register<T>
where
    T: Copy,
{
    /// Read register value (volatile)
    pub fn read(&self) -> T {
        // Your implementation:
        // 1. Ensure volatile read to prevent optimization
        // 2. Handle memory barriers if needed
        // 3. What about atomic access requirements?
        
        unsafe { read_volatile(self.addr) }
    }
    
    /// Write register value (volatile)
    pub fn write(&mut self, value: T) {
        // Your implementation:
        // 1. Ensure volatile write
        // 2. Memory barriers for ordering
        // 3. Handle write-only vs read-write registers
        
        unsafe { write_volatile(self.addr, value) }
    }
    
    /// Atomic read-modify-write operation
    pub fn modify<F>(&mut self, f: F)
    where
        F: FnOnce(T) -> T,
    {
        // Your implementation:
        // 1. Read current value
        // 2. Apply modification function
        // 3. Write back atomically
        // 4. Handle race conditions with interrupts/other cores
        
        let current = self.read();
        let new_value = f(current);
        self.write(new_value);
    }
}

impl<T> MemoryMappedDevice<T> {
    /// Map device memory safely
    pub fn map_device(physical_addr: u64, size: usize) -> Result<Self, DeviceError> {
        // Your implementation:
        // 1. Open /dev/mem (requires root)
        // 2. Use mmap to map physical memory
        // 3. Set appropriate memory flags
        // 4. Validate physical address range
        
        use std::os::unix::io::{AsRawFd, FromRawFd};
        use std::fs::OpenOptions;
        
        let mem_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/mem")
            .map_err(|_| DeviceError::MemoryAccessDenied)?;
            
        let base_addr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                mem_file.as_raw_fd(),
                physical_addr as off_t,
            )
        };
        
        if base_addr == libc::MAP_FAILED {
            return Err(DeviceError::MappingFailed);
        }
        
        Ok(Self {
            base_addr: base_addr as *mut u8,
            size,
            _phantom: PhantomData,
        })
    }
    
    /// Get register at specified offset
    pub fn register<R>(&self, offset: usize) -> Result<Register<R>, DeviceError>
    where
        R: Copy,
    {
        // Your implementation:
        // 1. Check bounds
        // 2. Check alignment
        // 3. Return register handle
        
        if offset + std::mem::size_of::<R>() > self.size {
            return Err(DeviceError::InvalidOffset);
        }
        
        if offset % std::mem::align_of::<R>() != 0 {
            return Err(DeviceError::MisalignedAccess);
        }
        
        Ok(Register {
            addr: unsafe { self.base_addr.add(offset) as *mut R },
            _phantom: PhantomData,
        })
    }
    
    /// Flush CPU cache for device memory
    pub fn flush_cache(&self) {
        // Your implementation:
        // 1. Use appropriate cache flush instructions
        // 2. Handle different cache levels
        // 3. Ensure memory ordering
        
        unsafe {
            // x86_64 cache flush
            for addr in (0..self.size).step_by(64) {
                let cache_line = self.base_addr.add(addr);
                std::arch::asm!(
                    "clflush ({})",
                    in(reg) cache_line,
                    options(nostack, preserves_flags)
                );
            }
            
            // Memory fence to ensure ordering
            std::arch::asm!("mfence", options(nostack, preserves_flags));
        }
    }
}

impl<T> Drop for MemoryMappedDevice<T> {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.base_addr as *mut c_void, self.size);
        }
    }
}

unsafe impl<T> Send for MemoryMappedDevice<T> where T: Send {}
unsafe impl<T> Sync for MemoryMappedDevice<T> where T: Sync {}

#[derive(Debug)]
pub enum DeviceError {
    MemoryAccessDenied,
    MappingFailed,
    InvalidOffset,
    MisalignedAccess,
}

// Example usage: GPIO controller
pub struct GpioController {
    device: MemoryMappedDevice<()>,
}

impl GpioController {
    pub fn new(base_addr: u64) -> Result<Self, DeviceError> {
        let device = MemoryMappedDevice::map_device(base_addr, 0x1000)?; // 4KB
        Ok(Self { device })
    }
    
    pub fn set_pin_output(&mut self, pin: u8, value: bool) -> Result<(), DeviceError> {
        // Your implementation:
        // 1. Calculate register offsets for GPIO
        // 2. Set pin direction
        // 3. Set pin value
        // 4. Handle pin validity
        
        if pin >= 32 {
            return Err(DeviceError::InvalidOffset);
        }
        
        // Direction register (offset 0x10)
        let mut dir_reg: Register<u32> = self.device.register(0x10)?;
        dir_reg.modify(|val| val | (1 << pin));
        
        // Data register (offset 0x00)
        let mut data_reg: Register<u32> = self.device.register(0x00)?;
        data_reg.modify(|val| {
            if value {
                val | (1 << pin)
            } else {
                val & !(1 << pin)
            }
        });
        
        Ok(())
    }
    
    pub fn read_pin(&self, pin: u8) -> Result<bool, DeviceError> {
        // Your implementation for reading pin state
        todo!("Implement pin reading")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_access() {
        // Note: This test requires root and actual hardware
        // In real tests, you'd use a mock or simulator
    }
    
    #[test]
    fn test_bounds_checking() {
        // Test that invalid offsets are rejected
    }
}
```

**Questions (25 points):**
1. **Memory Mapping Safety (15 points)**: How do you ensure safe access to hardware memory?
2. **Cache Coherency (10 points)**: When and how do you handle CPU cache flushing?

### Problem 3.2: Interrupt Simulation and Real-Time Constraints (25 points)

**Context**: Simulate interrupt handling with real-time deadlines.

```rust
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Real-time interrupt simulation system
/// Must meet hard deadlines and track timing violations
pub struct InterruptController {
    // Your design:
    // 1. How do you simulate hardware interrupts?
    // 2. How do you ensure real-time deadlines are met?
    // 3. What about interrupt priority handling?
    // 4. How do you track and report deadline violations?
    
    pending_interrupts: Arc<Mutex<VecDeque<Interrupt>>>,
    interrupt_handlers: Arc<Mutex<Vec<Option<InterruptHandler>>>>,
    statistics: Arc<InterruptStatistics>,
    running: AtomicBool,
    high_priority_count: AtomicU64,
    missed_deadlines: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct Interrupt {
    pub irq_number: u8,
    pub priority: InterruptPriority,
    pub deadline: Instant,
    pub arrival_time: Instant,
    pub data: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterruptPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

pub struct InterruptHandler {
    pub handler_fn: Box<dyn Fn(&Interrupt) -> Duration + Send + Sync>,
    pub max_execution_time: Duration,
    pub deadline_requirement: Duration,
}

pub struct InterruptStatistics {
    pub total_interrupts: AtomicU64,
    pub handled_interrupts: AtomicU64,
    pub missed_deadlines: AtomicU64,
    pub total_latency: AtomicU64, // microseconds
    pub max_latency: AtomicU64,   // microseconds
    pub priority_counts: [AtomicU64; 4],
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            pending_interrupts: Arc::new(Mutex::new(VecDeque::new())),
            interrupt_handlers: Arc::new(Mutex::new(vec![None; 256])),
            statistics: Arc::new(InterruptStatistics {
                total_interrupts: AtomicU64::new(0),
                handled_interrupts: AtomicU64::new(0),
                missed_deadlines: AtomicU64::new(0),
                total_latency: AtomicU64::new(0),
                max_latency: AtomicU64::new(0),
                priority_counts: [
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                    AtomicU64::new(0),
                ],
            }),
            running: AtomicBool::new(false),
            high_priority_count: AtomicU64::new(0),
            missed_deadlines: AtomicU64::new(0),
        }
    }
    
    /// Register interrupt handler with timing constraints
    pub fn register_handler(
        &self,
        irq: u8,
        handler: InterruptHandler,
    ) -> Result<(), InterruptError> {
        // Your implementation:
        // 1. Store handler with timing constraints
        // 2. Validate timing requirements are feasible
        // 3. Set up priority scheduling if needed
        
        let mut handlers = self.interrupt_handlers.lock().unwrap();
        handlers[irq as usize] = Some(handler);
        Ok(())
    }
    
    /// Start interrupt processing loop
    pub async fn start_processing(&self) -> Result<(), InterruptError> {
        // Your implementation:
        // 1. Set high-priority thread scheduling
        // 2. Main interrupt processing loop
        // 3. Priority-based interrupt handling
        // 4. Deadline monitoring and violation tracking
        
        self.running.store(true, Ordering::Release);
        
        // Set real-time scheduling priority
        self.set_realtime_priority()?;
        
        while self.running.load(Ordering::Acquire) {
            if let Some(interrupt) = self.get_highest_priority_interrupt() {
                let start_time = Instant::now();
                
                // Check if deadline already passed
                if start_time > interrupt.deadline {
                    self.statistics.missed_deadlines.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Deadline missed for IRQ {}", interrupt.irq_number);
                    continue;
                }
                
                self.handle_interrupt(&interrupt, start_time).await?;
            } else {
                // No interrupts pending - short sleep to avoid busy waiting
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        }
        
        Ok(())
    }
    
    /// Simulate hardware interrupt arrival
    pub fn trigger_interrupt(&self, irq: u8, priority: InterruptPriority, deadline_us: u64) {
        // Your implementation:
        // 1. Create interrupt with timing information
        // 2. Add to priority queue
        // 3. Wake up interrupt handler if needed
        // 4. Track statistics
        
        let interrupt = Interrupt {
            irq_number: irq,
            priority,
            deadline: Instant::now() + Duration::from_micros(deadline_us),
            arrival_time: Instant::now(),
            data: 0,
        };
        
        let mut queue = self.pending_interrupts.lock().unwrap();
        
        // Insert in priority order
        let mut inserted = false;
        for (i, existing) in queue.iter().enumerate() {
            if interrupt.priority > existing.priority ||
               (interrupt.priority == existing.priority && interrupt.deadline < existing.deadline) {
                queue.insert(i, interrupt.clone());
                inserted = true;
                break;
            }
        }
        
        if !inserted {
            queue.push_back(interrupt);
        }
        
        // Update statistics
        self.statistics.total_interrupts.fetch_add(1, Ordering::Relaxed);
        self.statistics.priority_counts[priority as usize].fetch_add(1, Ordering::Relaxed);
    }
    
    fn get_highest_priority_interrupt(&self) -> Option<Interrupt> {
        let mut queue = self.pending_interrupts.lock().unwrap();
        queue.pop_front()
    }
    
    async fn handle_interrupt(&self, interrupt: &Interrupt, start_time: Instant) -> Result<(), InterruptError> {
        // Your implementation:
        // 1. Look up registered handler
        // 2. Execute handler with timeout
        // 3. Track execution time and deadline compliance
        // 4. Update statistics
        
        let handlers = self.interrupt_handlers.lock().unwrap();
        
        if let Some(ref handler) = handlers[interrupt.irq_number as usize] {
            // Execute handler with timeout
            let execution_result = tokio::time::timeout(
                handler.max_execution_time,
                tokio::task::spawn_blocking({
                    let handler_fn = handler.handler_fn.clone();
                    let interrupt = interrupt.clone();
                    move || {
                        (handler_fn)(&interrupt)
                    }
                }),
            ).await;
            
            let end_time = Instant::now();
            let total_latency = end_time.duration_since(interrupt.arrival_time);
            let execution_time = end_time.duration_since(start_time);
            
            // Update statistics
            let latency_us = total_latency.as_micros() as u64;
            self.statistics.total_latency.fetch_add(latency_us, Ordering::Relaxed);
            self.statistics.handled_interrupts.fetch_add(1, Ordering::Relaxed);
            
            // Update max latency atomically
            let mut current_max = self.statistics.max_latency.load(Ordering::Relaxed);
            while latency_us > current_max {
                match self.statistics.max_latency.compare_exchange_weak(
                    current_max,
                    latency_us,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => current_max = x,
                }
            }
            
            // Check deadline compliance
            if end_time > interrupt.deadline {
                self.statistics.missed_deadlines.fetch_add(1, Ordering::Relaxed);
                eprintln!(
                    "IRQ {} deadline violation: took {:?}, deadline was {:?}",
                    interrupt.irq_number,
                    execution_time,
                    interrupt.deadline.duration_since(interrupt.arrival_time)
                );
            }
            
            match execution_result {
                Ok(Ok(_actual_time)) => {
                    // Handler completed successfully
                }
                Ok(Err(_)) => {
                    // Handler panicked
                    eprintln!("Handler for IRQ {} panicked", interrupt.irq_number);
                }
                Err(_) => {
                    // Handler timed out
                    eprintln!("Handler for IRQ {} timed out", interrupt.irq_number);
                }
            }
        }
        
        Ok(())
    }
    
    fn set_realtime_priority(&self) -> Result<(), InterruptError> {
        // Your implementation:
        // 1. Set thread to real-time scheduling policy
        // 2. Set high priority for interrupt handling
        // 3. Handle platform-specific requirements
        
        #[cfg(target_os = "linux")]
        unsafe {
            let param = libc::sched_param {
                sched_priority: 99, // Highest real-time priority
            };
            
            let result = libc::sched_setscheduler(
                0, // Current thread
                libc::SCHED_FIFO,
                &param,
            );
            
            if result != 0 {
                return Err(InterruptError::SchedulingError);
            }
        }
        
        Ok(())
    }
    
    /// Get detailed performance statistics
    pub fn get_statistics(&self) -> InterruptStatistics {
        // Create snapshot of current statistics
        InterruptStatistics {
            total_interrupts: AtomicU64::new(
                self.statistics.total_interrupts.load(Ordering::Relaxed)
            ),
            handled_interrupts: AtomicU64::new(
                self.statistics.handled_interrupts.load(Ordering::Relaxed)
            ),
            missed_deadlines: AtomicU64::new(
                self.statistics.missed_deadlines.load(Ordering::Relaxed)
            ),
            total_latency: AtomicU64::new(
                self.statistics.total_latency.load(Ordering::Relaxed)
            ),
            max_latency: AtomicU64::new(
                self.statistics.max_latency.load(Ordering::Relaxed)
            ),
            priority_counts: [
                AtomicU64::new(self.statistics.priority_counts[0].load(Ordering::Relaxed)),
                AtomicU64::new(self.statistics.priority_counts[1].load(Ordering::Relaxed)),
                AtomicU64::new(self.statistics.priority_counts[2].load(Ordering::Relaxed)),
                AtomicU64::new(self.statistics.priority_counts[3].load(Ordering::Relaxed)),
            ],
        }
    }
    
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }
}

#[derive(Debug)]
pub enum InterruptError {
    HandlerRegistrationFailed,
    SchedulingError,
    DeadlineViolation,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_deadline_compliance() {
        let controller = InterruptController::new();
        
        // Register handler with strict deadline
        let handler = InterruptHandler {
            handler_fn: Box::new(|_| {
                std::thread::sleep(Duration::from_micros(50));
                Duration::from_micros(50)
            }),
            max_execution_time: Duration::from_micros(100),
            deadline_requirement: Duration::from_micros(200),
        };
        
        controller.register_handler(1, handler).unwrap();
        
        // Start processing
        tokio::spawn(async move {
            controller.start_processing().await.unwrap();
        });
        
        // Trigger interrupt with tight deadline
        controller.trigger_interrupt(1, InterruptPriority::High, 100);
        
        tokio::time::sleep(Duration::from_millis(1)).await;
        
        let stats = controller.get_statistics();
        assert_eq!(stats.total_interrupts.load(Ordering::Relaxed), 1);
        // Should meet deadline if system is not overloaded
    }
    
    #[tokio::test]
    async fn test_priority_ordering() {
        // Test that high-priority interrupts are handled before low-priority ones
    }
}
```

**Questions (25 points):**
1. **Real-Time Scheduling (15 points)**: How do you ensure hard deadlines are met?
2. **Priority Handling (10 points)**: Implement correct priority-based interrupt scheduling

---

## Success Criteria & Next Steps

### **Pass Requirements (85% minimum):**

1. **Kernel Interface Programming (85 points required)**:
   - Custom allocator using mmap system calls
   - Signal-safe async runtime design  
   - Zero-copy file I/O with io_uring

2. **Network Protocol Implementation (85 points required)**:
   - Custom protocol with raw sockets and reliability
   - Zero-copy network server with massive connection support
   - Comprehensive performance analysis

3. **Hardware Interface Programming (40 points required)**:
   - Safe memory-mapped device interface
   - Real-time interrupt simulation with deadline tracking

### **Advanced Features (Bonus)**:
- NUMA-aware memory allocation (+10 points)
- Hardware-specific assembly optimization (+10 points)  
- Kernel module integration (+15 points)

### **If You Pass This Exam:**
You are now an **expert systems programmer** ready for:
- **08_lock_free_concurrency_mastery.md** - Lock-free programming mastery
- **Operating system development** - Kernels, device drivers, embedded systems
- **High-frequency trading systems** - Ultra-low latency networking
- **Database engine development** - Storage systems, memory management

### **If You Don't Pass:**
Focus on these areas:
- **Linux system programming** - Stevens & Rago, Advanced Programming in the UNIX Environment
- **Network programming** - TCP/IP Illustrated, Unix Network Programming
- **Hardware interfaces** - Intel SDM, ARM Architecture Reference Manual
- **Real systems practice** - Contribute to Linux kernel, database engines, or embedded projects

This exam tests the **absolute hardest low-level systems programming concepts**. Passing demonstrates you can build the foundational infrastructure that everything else runs on.