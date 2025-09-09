# Day 15: Rust Performance Toolkit Crash Course
**Active Recall Power Quiz - Debug, Benchmark, Profile**

*"Performance is a feature. If you can't measure it, you can't improve it."*

---

## Mission Statement

Master the essential tools for debugging and profiling Rust applications through hands-on challenges. You'll use GDB for debugging, criterion for benchmarking, and various profiling tools to diagnose performance issues in real Rust code.

**Learning Method**: Active Recall Power Quiz - Each section presents broken/slow code that you must debug and optimize using professional tools.

---

## Setup: Performance Lab Environment

```bash
# Install required tools
sudo apt update
sudo apt install gdb valgrind perf

# Install Rust profiling tools
cargo install flamegraph
cargo install cargo-profdata
cargo install hyperfine

# Add to Cargo.toml dependencies
[dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
pprof = { version = "0.13", features = ["flamegraph", "protobuf-codec"] }

[profile.release]
debug = true  # Enable debug symbols in release mode for profiling
```

---

## Section I: GDB Debugging Mastery (Active Recall Challenges)

### Challenge 1.1: The Mystery Segfault ⚡ **Fix in 10 minutes**

You have a Rust program that's crashing. Use GDB to find and fix the issue:

```rust
// src/mystery_crash.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct DataProcessor {
    cache: HashMap<String, Vec<u8>>,
    buffer: Vec<u8>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            buffer: Vec::with_capacity(1024),
        }
    }
    
    fn process_data(&mut self, key: &str, data: &[u8]) -> &[u8] {
        // This function has a subtle bug that causes crashes under certain conditions
        self.buffer.clear();
        self.buffer.extend_from_slice(data);
        
        // Process the data (double each byte)
        for byte in &mut self.buffer {
            *byte = byte.wrapping_mul(2);
        }
        
        // Cache the result - BUG IS HERE
        self.cache.insert(key.to_string(), self.buffer.clone());
        
        // Return reference to processed data - BUG CONTINUES HERE
        &self.buffer
    }
}

fn main() {
    let mut processor = DataProcessor::new();
    
    let data1 = vec![1, 2, 3, 4, 5];
    let result1 = processor.process_data("test1", &data1);
    println!("Result 1: {:?}", result1);
    
    let data2 = vec![10, 20, 30];
    let result2 = processor.process_data("test2", &data2);
    println!("Result 2: {:?}", result2);
    
    // This line will cause problems
    println!("Previous result: {:?}", result1); // CRASH HAPPENS HERE
}
```

**Your GDB Investigation Mission:**
1. **Compile with debug info**: `cargo build`
2. **Run with GDB**: `gdb target/debug/mystery_crash`
3. **Set breakpoints**: Where would you set them to trace the bug?
4. **Examine memory**: What GDB commands reveal the issue?
5. **Fix the code**: What's the root cause and solution?

**Active Recall Question**: Before running GDB, predict what's wrong with this code and where it will crash. Then verify with GDB.

### Challenge 1.2: The Async Deadlock Detective ⚡ **Debug in 15 minutes**

This async program hangs. Use GDB to find the deadlock:

```rust
// src/async_deadlock.rs
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[derive(Clone)]
struct SharedCounter {
    value1: Arc<Mutex<i32>>,
    value2: Arc<Mutex<i32>>,
}

impl SharedCounter {
    fn new() -> Self {
        SharedCounter {
            value1: Arc::new(Mutex::new(0)),
            value2: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn increment_both(&self) {
        // Task A: locks value1 then value2
        let _guard1 = self.value1.lock().unwrap();
        sleep(Duration::from_millis(10)).await; // Simulate work
        let _guard2 = self.value2.lock().unwrap();
        println!("Incremented both from task A");
    }
    
    async fn decrement_both(&self) {
        // Task B: locks value2 then value1 - DEADLOCK POTENTIAL
        let _guard2 = self.value2.lock().unwrap(); 
        sleep(Duration::from_millis(10)).await; // Simulate work
        let _guard1 = self.value1.lock().unwrap();
        println!("Decremented both from task B");
    }
}

#[tokio::main]
async fn main() {
    let counter = SharedCounter::new();
    
    let task_a = {
        let counter = counter.clone();
        tokio::spawn(async move {
            for _ in 0..10 {
                counter.increment_both().await;
            }
        })
    };
    
    let task_b = {
        let counter = counter.clone();
        tokio::spawn(async move {
            for _ in 0..10 {
                counter.decrement_both().await;
            }
        })
    };
    
    // This will hang - your job is to find where and why
    let _ = tokio::join!(task_a, task_b);
    println!("All done!"); // This never prints
}
```

**Your GDB Async Debugging Mission:**
1. **Attach to hanging process**: `gdb -p <pid>`
2. **Examine all threads**: `info threads`
3. **Switch between threads**: `thread <N>`
4. **Check stack traces**: `bt` for each thread
5. **Identify the deadlock**: Which mutexes are involved?

**GDB Commands for Async Debugging:**
```bash
# Compile with debug symbols
cargo build

# Run in background to get PID
./target/debug/async_deadlock &
PID=$!

# Attach GDB to running process
gdb -p $PID

# In GDB:
(gdb) info threads              # Show all threads
(gdb) thread 2                  # Switch to thread 2
(gdb) bt                        # Backtrace for current thread
(gdb) frame 3                   # Examine specific frame
(gdb) info locals               # Show local variables
(gdb) p *mutex_ptr              # Examine mutex state
```

---

## Section II: Criterion Benchmarking Mastery

### Challenge 2.1: The Allocation Battle ⚡ **Benchmark in 20 minutes**

Compare different string processing approaches and identify the fastest:

```rust
// src/string_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Approach 1: String concatenation
fn concat_strings_owned(inputs: &[&str]) -> String {
    let mut result = String::new();
    for input in inputs {
        result.push_str(input);
        result.push(' ');
    }
    result
}

// Approach 2: Collect with join
fn concat_strings_join(inputs: &[&str]) -> String {
    inputs.join(" ")
}

// Approach 3: Pre-allocated capacity
fn concat_strings_capacity(inputs: &[&str]) -> String {
    let total_len: usize = inputs.iter().map(|s| s.len() + 1).sum();
    let mut result = String::with_capacity(total_len);
    for input in inputs {
        result.push_str(input);
        result.push(' ');
    }
    result
}

// Approach 4: Using format! macro
fn concat_strings_format(inputs: &[&str]) -> String {
    inputs.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ")
}

fn benchmark_string_concat(c: &mut Criterion) {
    let inputs = vec!["hello", "world", "this", "is", "a", "test", "string"];
    
    let mut group = c.benchmark_group("string_concatenation");
    
    // Your task: Add proper benchmarks for each approach
    group.bench_function("owned_concat", |b| {
        b.iter(|| concat_strings_owned(black_box(&inputs)))
    });
    
    // TODO: Add benchmarks for the other approaches
    // TODO: Add parameter variations (different input sizes)
    // TODO: Add memory usage measurement
    
    group.finish();
}

// Challenge: Add benchmarks that vary input size
fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_concat_scaling");
    
    for size in [10, 100, 1000, 10000].iter() {
        let inputs: Vec<String> = (0..*size)
            .map(|i| format!("string_{}", i))
            .collect();
        let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
        
        group.bench_with_input(BenchmarkId::new("owned", size), &input_refs, |b, inputs| {
            b.iter(|| concat_strings_owned(black_box(inputs)))
        });
        
        // Your task: Add other approaches with scaling
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_string_concat, benchmark_scaling);
criterion_main!(benches);
```

**Your Benchmarking Mission:**
1. **Complete the benchmarks**: Add all missing benchmark functions
2. **Run and analyze**: `cargo bench` - which approach is fastest?
3. **Memory profiling**: Use `cargo bench -- --profile-time=5` 
4. **Generate reports**: Find the HTML reports in `target/criterion/`
5. **Hypothesis testing**: Before running, predict which will be fastest and why

**Active Recall Questions:**
- Which approach will be fastest for small inputs (< 10 strings)?
- Which approach will scale best for large inputs (> 1000 strings)?
- What's the memory allocation pattern for each approach?

### Challenge 2.2: The Zero-Copy Championship ⚡ **Optimize in 25 minutes**

Benchmark different approaches to JSON parsing and find the fastest:

```rust
// src/json_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use serde::{Deserialize, Serialize};

// Test data structure
#[derive(Serialize, Deserialize, Clone)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    metadata: Vec<String>,
}

// Approach 1: Standard owned deserialization
fn parse_owned(json: &str) -> Result<LogEntry, serde_json::Error> {
    serde_json::from_str(json)
}

// Approach 2: Zero-copy with borrowed fields
#[derive(Deserialize)]
struct LogEntryBorrowed<'a> {
    #[serde(borrow)]
    timestamp: &'a str,
    #[serde(borrow)]
    level: &'a str,
    #[serde(borrow)]
    message: &'a str,
    #[serde(borrow)]
    metadata: Vec<&'a str>,
}

fn parse_borrowed(json: &str) -> Result<LogEntryBorrowed<'_>, serde_json::Error> {
    serde_json::from_str(json)
}

// Approach 3: Manual parsing (unsafe but fast)
fn parse_manual(json: &str) -> Option<(String, String, String)> {
    // Your challenge: implement a fast manual parser
    // Hint: Use str methods like find(), split(), etc.
    None // Placeholder
}

fn benchmark_json_parsing(c: &mut Criterion) {
    let json_data = r#"{"timestamp": "2023-01-01T12:00:00Z", "level": "INFO", "message": "User login successful", "metadata": ["user_id:123", "session:abc", "ip:192.168.1.1"]}"#;
    
    let mut group = c.benchmark_group("json_parsing");
    
    // Benchmark owned parsing
    group.bench_function("owned", |b| {
        b.iter(|| parse_owned(black_box(json_data)))
    });
    
    // Your task: Add borrowed parsing benchmark
    
    // Your task: Add manual parsing benchmark
    
    // Advanced: Add memory usage measurement
    group.bench_function("owned_with_allocation_tracking", |b| {
        b.iter_batched(
            || json_data.to_string(), // Setup: create owned string
            |owned_json| parse_owned(black_box(&owned_json)), // Test: parse it
            BatchSize::SmallInput // Recreate setup data each iteration
        )
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_json_parsing);
criterion_main!(benches);
```

**Your Optimization Challenge:**
1. **Implement missing benchmarks**: Complete the borrowed and manual parsing benchmarks
2. **Measure memory allocations**: Use criterion's allocation tracking features
3. **Create performance profiles**: Run with `--profile-time=10`
4. **Analyze the results**: Which approach is fastest? Why?

---

## Section III: Advanced Profiling Techniques

### Challenge 3.1: The CPU Hotspot Hunter ⚡ **Profile in 20 minutes**

Use flamegraphs to identify performance bottlenecks:

```rust
// src/cpu_intensive.rs
use std::collections::HashMap;

fn expensive_computation(n: usize) -> u64 {
    let mut result = 0;
    for i in 0..n {
        result += fibonacci(i % 30);
        result += prime_check(i);
        result += hash_operations(i);
    }
    result
}

fn fibonacci(n: usize) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        fibonacci(n - 1) + fibonacci(n - 2) // Inefficient recursive implementation
    }
}

fn prime_check(n: usize) -> u64 {
    if n < 2 { return 0; }
    for i in 2..n {  // Inefficient: should only check up to sqrt(n)
        if n % i == 0 {
            return 0;
        }
    }
    1
}

fn hash_operations(n: usize) -> u64 {
    let mut map = HashMap::new();
    // Inefficient: creates new HashMap every call
    for i in 0..n % 100 {
        map.insert(i, i * 2);
    }
    map.values().sum()
}

fn main() {
    let result = expensive_computation(1000);
    println!("Result: {}", result);
}
```

**Your Profiling Mission:**
```bash
# Generate CPU flamegraph
cargo build --release
sudo flamegraph target/release/cpu_intensive

# Alternative: Use perf
perf record --call-graph=dwarf target/release/cpu_intensive
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg

# Use criterion with profiling
cargo bench --bench cpu_intensive -- --profile-time=10
```

**Active Recall Questions:**
1. **Before profiling**: Which function do you think will be the bottleneck?
2. **After profiling**: Were you right? What's the actual hotspot?
3. **Optimization plan**: How would you fix the top 3 performance issues?

### Challenge 3.2: The Memory Leak Detective ⚡ **Hunt in 15 minutes**

Use valgrind and other tools to find memory issues:

```rust
// src/memory_issues.rs
use std::collections::HashMap;
use std::rc::Rc;

struct LeakyProcessor {
    cache: HashMap<String, Rc<Vec<u8>>>,
    temp_buffers: Vec<Vec<u8>>,
}

impl LeakyProcessor {
    fn new() -> Self {
        LeakyProcessor {
            cache: HashMap::new(),
            temp_buffers: Vec::new(),
        }
    }
    
    fn process_data(&mut self, key: &str, data: &[u8]) -> Rc<Vec<u8>> {
        // Memory issue 1: Temp buffers keep growing
        let mut temp = Vec::with_capacity(data.len() * 2);
        temp.extend_from_slice(data);
        temp.extend_from_slice(data); // Double the data
        self.temp_buffers.push(temp.clone()); // BUG: Never cleared
        
        let result = Rc::new(temp);
        
        // Memory issue 2: Cache grows indefinitely
        self.cache.insert(key.to_string(), result.clone()); // BUG: No size limit
        
        result
    }
    
    fn process_many(&mut self, count: usize) {
        for i in 0..count {
            let key = format!("key_{}", i);
            let data: Vec<u8> = (0..1000).map(|x| (x % 256) as u8).collect();
            self.process_data(&key, &data);
        }
    }
}

fn main() {
    let mut processor = LeakyProcessor::new();
    processor.process_many(10000); // This will consume lots of memory
    println!("Processing complete");
}
```

**Your Memory Profiling Mission:**
```bash
# Compile with debug info
cargo build --release

# Use valgrind to check for leaks
valgrind --tool=memcheck --leak-check=full target/release/memory_issues

# Use heaptrack for detailed allocation tracking
heaptrack target/release/memory_issues
heaptrack --analyze heaptrack.memory_issues.*.gz

# Monitor memory usage during execution
/usr/bin/time -v target/release/memory_issues
```

**Memory Analysis Questions:**
1. **Peak memory usage**: How much memory does the program use?
2. **Memory growth pattern**: Is memory usage linear or exponential?
3. **Fix strategy**: How would you cap memory usage while maintaining functionality?

---

## Section IV: Integration Challenge - The Complete Performance Audit

### Final Boss Challenge: The Real-World Performance Mystery ⚡ **60 minutes**

You have a "fast" HTTP server that's mysteriously slow under load. Use all your tools to diagnose and fix it:

```rust
// src/slow_server.rs
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

#[derive(Clone)]
struct SlowServer {
    cache: Arc<Mutex<HashMap<String, String>>>, // Contention point
    stats: Arc<Mutex<ServerStats>>,             // Another contention point
}

#[derive(Default)]
struct ServerStats {
    requests_handled: u64,
    total_response_time: Duration,
}

impl SlowServer {
    fn new() -> Self {
        SlowServer {
            cache: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ServerStats::default())),
        }
    }
    
    async fn handle_request(&self, mut stream: TcpStream) {
        let start_time = Instant::now();
        
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await {
            Ok(size) => {
                let request = String::from_utf8_lossy(&buffer[..size]);
                let response = self.process_request(&request).await;
                
                let http_response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    response.len(),
                    response
                );
                
                let _ = stream.write_all(http_response.as_bytes()).await;
            }
            Err(_) => {
                let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
            }
        }
        
        // Update stats (potential bottleneck)
        let elapsed = start_time.elapsed();
        let mut stats = self.stats.lock().unwrap(); // Blocking async code!
        stats.requests_handled += 1;
        stats.total_response_time += elapsed;
    }
    
    async fn process_request(&self, request: &str) -> String {
        // Simulate slow processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Check cache (another bottleneck)
        let cache_key = request.trim().to_string();
        {
            let cache = self.cache.lock().unwrap(); // More blocking!
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }
        
        // Expensive computation
        let result = format!("Processed: {}", request.trim());
        
        // Update cache
        let mut cache = self.cache.lock().unwrap();
        cache.insert(cache_key, result.clone());
        
        result
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = SlowServer::new();
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    println!("Server listening on 127.0.0.1:8080");
    
    loop {
        let (stream, _) = listener.accept().await?;
        let server = server.clone();
        
        // Spawn a task for each connection
        tokio::spawn(async move {
            server.handle_request(stream).await;
        });
    }
}
```

**Your Complete Performance Investigation:**

**Phase 1: Baseline Measurements (15 minutes)**
```bash
# Build optimized version
cargo build --release

# Start the server
./target/release/slow_server &
SERVER_PID=$!

# Load test with hyperfine
hyperfine --warmup 3 --runs 10 'curl -s http://127.0.0.1:8080/test'

# Concurrent load test
for i in {1..100}; do
    curl -s http://127.0.0.1:8080/request_$i &
done
wait

# Kill server
kill $SERVER_PID
```

**Phase 2: CPU Profiling (15 minutes)**
```bash
# Profile CPU usage
flamegraph --pid $SERVER_PID -o server_profile.svg

# Alternative: Use perf
perf record -g --pid=$SERVER_PID sleep 10
perf script | stackcollapse-perf.pl | flamegraph.pl > perf_profile.svg
```

**Phase 3: Concurrency Analysis (15 minutes)**
```bash
# Use GDB to examine thread states
gdb -p $SERVER_PID
# In GDB:
# (gdb) info threads
# (gdb) thread apply all bt
# Look for threads blocked on mutex locks
```

**Phase 4: Memory Profiling (15 minutes)**
```bash
# Check memory usage patterns
valgrind --tool=massif target/release/slow_server &
# Run load test, then analyze with:
massif-visualizer massif.out.*
```

**Active Recall Master Questions:**
1. **Bottleneck Prediction**: Before profiling, what do you think are the top 3 bottlenecks?
2. **Root Cause Analysis**: After profiling, what's actually causing the slowdown?
3. **Fix Priority**: Which issue should be fixed first for maximum impact?
4. **Architecture Redesign**: How would you redesign this server for better performance?

---

## Answer Key & Optimization Guide

### Challenge Solutions

**Section I Solutions:**

**1.1 Mystery Segfault**: The bug is in returning a reference to `self.buffer` that gets invalidated when the method is called again. The fix is to return `Vec<u8>` instead of `&[u8]`, or use a different architecture that doesn't reuse the buffer.

**1.2 Async Deadlock**: Classic deadlock - task A locks `value1` then `value2`, task B locks `value2` then `value1`. Fix by always acquiring locks in the same order, or use `try_lock()` with timeout.

**Section II Solutions:**

**2.1 Allocation Battle**: `with_capacity` approach typically wins for large inputs, `join` wins for simplicity and small inputs.

**2.2 Zero-Copy Championship**: Borrowed deserialization wins for throughput, but has lifetime constraints that may make it impractical.

**Section III Solutions:**

**3.1 CPU Hotspot**: Fibonacci function with recursive implementation will dominate CPU time. Fix with memoization or iterative approach.

**3.2 Memory Leak**: Two leaks - temp buffers never cleared, cache grows indefinitely. Fix with LRU cache and buffer reuse.

**Section IV Solution:**

**Final Boss Performance Issues:**
1. **Blocking mutexes in async code** - Replace with `tokio::sync::Mutex`
2. **Contended shared state** - Use dashmap or per-connection caches
3. **Unnecessary allocations** - Reuse buffers, use string slices
4. **No connection pooling** - Add proper async task management

### Performance Tool Mastery Checklist

After completing these challenges, you should be able to:

✅ **GDB Debugging**
- Attach to running processes and analyze thread states
- Set strategic breakpoints for async code debugging
- Examine memory layout and identify use-after-free bugs
- Debug deadlocks by analyzing mutex states across threads

✅ **Criterion Benchmarking**
- Design comparative benchmarks with proper controls
- Use `BatchSize` and `black_box` correctly
- Generate scaling analysis across input sizes
- Interpret criterion's statistical analysis

✅ **Advanced Profiling** 
- Generate and interpret CPU flamegraphs
- Use valgrind for memory leak detection
- Analyze allocation patterns with heap profilers
- Correlate profiling data with performance bottlenecks

✅ **Integration Skills**
- Combine multiple tools for comprehensive analysis
- Prioritize optimizations based on measurement data
- Validate performance improvements with before/after benchmarks
- Design performance-aware architectures from the start

---

## Graduation Test

**Can you now confidently answer:**
- "This Rust program is slow - how do I find out why?"
- "How do I know if my optimization actually helped?"  
- "What's the memory allocation pattern of this code?"
- "How do I debug a hanging async program?"
- "Which of these 3 implementations is fastest and by how much?"

**If yes, you've mastered the Rust performance toolkit! 🎯**

---

*"Measure twice, optimize once. Profile everything, assume nothing."*
