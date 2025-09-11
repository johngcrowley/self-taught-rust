# Performance Measurement Bridge Course
**Active Recall Problems for Performance Measurement Mastery**

*Time Limit: 3 hours | Total Points: 120 | Bridge to: Advanced Systems Diagnostic*

---

## Mission: Learn to Measure What You Build

**Critical Gap Addressed**: You can write Rust code, but can you measure its performance? This bridge course teaches you to benchmark, profile, and optimize using active recall problems.

**Prerequisites**: Pass `00_intermediate_rust_readiness.md` to prove intermediate Rust skills

**You Will Master:**
- Benchmarking with criterion and measuring latency/throughput
- Memory profiling and allocation analysis  
- CPU profiling and hotspot identification
- Async performance measurement with tokio-console
- Comparative performance analysis methodology

**Next Level**: After this, you'll be ready for `05_advanced_systems_diagnostic.md` - Advanced Systems Diagnostic

---

## Section I: Benchmarking Fundamentals (30 points)

### Problem 1.1: String Processing Performance (15 points)

You have three ways to process strings. Measure which is fastest and why:

```rust
// Add to Cargo.toml:
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Implementation A: String allocation
fn process_string_alloc(input: &str) -> String {
    let mut result = String::new();
    for c in input.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

// Implementation B: In-place with capacity
fn process_string_capacity(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

// Implementation C: Iterator chain  
fn process_string_iterator(input: &str) -> String {
    input.chars()
         .filter(|c| c.is_alphanumeric())
         .map(|c| c.to_ascii_lowercase())
         .collect()
}

fn benchmark_string_processing(c: &mut Criterion) {
    let inputs = vec![
        "Short!",
        "Medium length string with CAPS and symbols!!!",  
        "Very long string with lots of different characters, UPPERCASE, lowercase, numbers 123, and symbols !!!@@@###$$$%%%^^^&&&***((())))",
    ];
    
    let mut group = c.benchmark_group("string_processing");
    
    for input in inputs.iter() {
        // YOUR TASK: Complete the benchmarks for all three implementations
        // Measure: mean time, throughput, and memory allocations
        
        group.bench_with_input(BenchmarkId::new("alloc", input.len()), input, |b, input| {
            b.iter(|| process_string_alloc(black_box(input)))
        });
        
        // TODO: Add benchmarks for capacity and iterator versions
        // TODO: Add throughput measurements in bytes/second
        // TODO: Add memory profiling to count allocations
    }
    group.finish();
}

criterion_group!(benches, benchmark_string_processing);
criterion_main!(benches);
```

**Active Recall Tasks:**
1. **Complete the benchmarks**: Add criterion benchmarks for all three implementations with throughput measurement
2. **Measure allocations**: Use a custom allocator or profiler to count memory allocations for each approach
3. **Analyze results**: Which is fastest for short/medium/long strings and why?
4. **Memory impact**: How many allocations does each approach make per string?
5. **Cache behavior**: Which approach has better CPU cache locality?

### Problem 1.2: Data Structure Performance Comparison (15 points)

Compare the performance of different data structures for a common SRE use case:

```rust
use std::collections::{HashMap, BTreeMap, VecDeque};
use criterion::{black_box, Criterion, BatchSize};

// SRE Scenario: Time-series metric storage with different access patterns

struct Metric {
    timestamp: u64,
    value: f64,
    tags: String,
}

// YOUR TASK: Implement and benchmark three different storage strategies

// Strategy A: HashMap with timestamp key
fn benchmark_hashmap_storage(c: &mut Criterion) {
    c.bench_function("hashmap_insert_lookup", |b| {
        b.iter_batched(
            || {
                // Setup: Create test data
                let mut map = HashMap::new();
                let metrics: Vec<Metric> = (0..1000).map(|i| Metric {
                    timestamp: i,
                    value: i as f64 * 1.5,
                    tags: format!("host=server{}", i % 10),
                }).collect();
                (map, metrics)
            },
            |(mut map, metrics)| {
                // Benchmark: Insert all metrics then lookup random ones
                // TODO: Implement insert phase
                // TODO: Implement lookup phase  
                // TODO: Measure both insert and lookup performance separately
                black_box(map)
            },
            BatchSize::SmallInput
        );
    });
}

// Strategy B: BTreeMap for time-ordered access
fn benchmark_btreemap_range_queries(c: &mut Criterion) {
    // YOUR TASK: Benchmark range queries (last N minutes of metrics)
    // Compare with HashMap approach for time-range lookups
    todo!("Implement BTreeMap benchmarking with range queries")
}

// Strategy C: VecDeque for recent metrics (sliding window)
fn benchmark_vecdeque_sliding_window(c: &mut Criterion) {
    // YOUR TASK: Benchmark sliding window operations
    // Measure: append performance, window maintenance, recent value access
    todo!("Implement VecDeque sliding window benchmarking")
}
```

**Active Recall Challenges:**
1. **Implement all benchmarks**: Complete the three storage strategy benchmarks
2. **Measure different access patterns**: Insert, lookup, range query, sliding window
3. **Memory usage comparison**: Which uses least memory for 10k metrics?
4. **Cache performance**: Use `perf stat` to measure cache hit rates
5. **Real-world simulation**: Which performs best for high-frequency metrics (1000/sec)?

---

## Section II: Memory Profiling and Allocation Analysis (30 points)

### Problem 2.1: Zero-Copy vs Allocation Analysis (20 points)

Measure the memory impact of different serde deserialization strategies:

```rust
use serde::{Deserialize, Serialize};

// Test data: Large JSON with nested structures (common in SRE systems)
const LARGE_JSON: &str = r#"{
  "metrics": [
    {"name": "cpu_usage", "value": 85.2, "host": "server001", "timestamp": 1640995200},
    {"name": "memory_usage", "value": 67.8, "host": "server001", "timestamp": 1640995200},
    // ... imagine 1000+ metrics
  ],
  "metadata": {
    "collection_time": "2021-12-31T16:00:00Z",
    "datacenter": "us-east-1", 
    "environment": "production"
  }
}"#;

// Strategy A: Owned deserialization  
#[derive(Deserialize, Debug)]
struct MetricsResponseOwned {
    metrics: Vec<MetricOwned>,
    metadata: MetadataOwned,
}

#[derive(Deserialize, Debug)]
struct MetricOwned {
    name: String,      // Allocated
    value: f64,
    host: String,      // Allocated  
    timestamp: u64,
}

#[derive(Deserialize, Debug)]
struct MetadataOwned {
    collection_time: String,  // Allocated
    datacenter: String,       // Allocated
    environment: String,      // Allocated
}

// Strategy B: Zero-copy deserialization
#[derive(Deserialize, Debug)]
struct MetricsResponseBorrowed<'a> {
    metrics: Vec<MetricBorrowed<'a>>,
    metadata: MetadataBorrowed<'a>,
}

#[derive(Deserialize, Debug)]
struct MetricBorrowed<'a> {
    name: &'a str,        // Borrowed
    value: f64,
    host: &'a str,        // Borrowed
    timestamp: u64,
}

#[derive(Deserialize, Debug)]
struct MetadataBorrowed<'a> {
    collection_time: &'a str,   // Borrowed
    datacenter: &'a str,        // Borrowed  
    environment: &'a str,       // Borrowed
}

// YOUR TASK: Measure memory allocation differences

fn measure_allocation_patterns() {
    // TODO: Use a tracking allocator to measure exact allocation counts
    // TODO: Compare heap memory usage between owned vs borrowed
    // TODO: Measure parsing time differences
    // TODO: Test with different JSON sizes (1KB, 10KB, 100KB, 1MB)
    
    println!("Measuring serde allocation patterns...");
    
    // Hint: Use std::alloc::System and track allocations
    // Or use external tools like valgrind --tool=massif
}
```

**Active Recall Tasks:**
1. **Implement allocation tracking**: Use a custom allocator to count exact allocations for each strategy
2. **Memory usage measurement**: Compare peak heap usage with different JSON sizes  
3. **Performance impact**: Does zero-copy actually improve parsing performance? Measure it.
4. **Real-world analysis**: For a streaming SRE system processing 1000 JSON/sec, which strategy is better?
5. **Tool usage**: Use `valgrind --tool=massif` to profile heap growth patterns

### Problem 2.2: Async Memory Leak Detection (10 points)

Find and fix memory leaks in async code through profiling:

```rust
use tokio::task::JoinSet;
use std::sync::Arc;
use tokio::sync::Mutex;

// Potentially leaky async code (common in SRE systems)
struct MetricCollector {
    active_tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    // More state...
}

impl MetricCollector {
    async fn start_collection(&self) {
        let mut join_set = JoinSet::new();
        
        // Spawn many concurrent metric collection tasks
        for i in 0..1000 {
            let task = tokio::spawn(async move {
                // Simulate metric collection  
                let _data = vec![0u8; 1024 * 1024]; // 1MB allocation
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                // BUG: What happens if this task is cancelled?
            });
            
            // BUG: Are we properly managing these handles?
            let mut handles = self.active_tasks.lock().await;
            handles.push(task);
        }
        
        // BUG: Do we clean up finished tasks?
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            // Process metrics...
        }
    }
}

// YOUR TASK: Profile and fix the memory leaks
async fn profile_async_leaks() {
    // TODO: Use tokio-console to profile task lifetimes
    // TODO: Measure memory growth over time
    // TODO: Identify and fix the memory leaks
    // TODO: Verify fixes with profiling
}
```

**Memory Leak Hunting Tasks:**
1. **tokio-console profiling**: Use tokio-console to visualize task lifetimes and resource usage
2. **Memory growth measurement**: Track heap growth over 10 minutes of execution
3. **Leak identification**: What specific patterns cause memory leaks in this code?
4. **Fix implementation**: Provide corrected version with proper resource cleanup
5. **Verification**: Prove your fixes work with before/after memory profiles

---

## Section III: CPU Profiling and Hotspot Analysis (30 points)

### Problem 3.1: Flamegraph Analysis for Performance Optimization (20 points)

Profile a CPU-intensive data processing pipeline and optimize hotspots:

```rust
// SRE Scenario: Log processing pipeline (CPU-heavy)
use regex::Regex;
use std::collections::HashMap;

struct LogProcessor {
    patterns: Vec<Regex>,
    metrics: HashMap<String, u64>,
}

impl LogProcessor {
    fn new() -> Self {
        let patterns = vec![
            Regex::new(r"ERROR.*(\d+).*").unwrap(),
            Regex::new(r"WARN.*duration=(\d+)ms.*").unwrap(),
            Regex::new(r"INFO.*user_id=(\d+).*").unwrap(),
            // Many more patterns...
        ];
        
        Self {
            patterns,
            metrics: HashMap::new(),
        }
    }
    
    // Potentially inefficient processing
    fn process_log_line(&mut self, line: &str) {
        // Check every pattern against every line (O(n*m))
        for (i, pattern) in self.patterns.iter().enumerate() {
            if let Some(captures) = pattern.captures(line) {
                // String operations that might allocate
                let metric_name = format!("pattern_{}", i);
                let counter = self.metrics.entry(metric_name).or_insert(0);
                *counter += 1;
                
                // Extract numeric values (more string operations)
                if let Some(value_match) = captures.get(1) {
                    let value_str = value_match.as_str();
                    if let Ok(value) = value_str.parse::<u64>() {
                        let value_metric = format!("pattern_{}_value", i);
                        let sum = self.metrics.entry(value_metric).or_insert(0);
                        *sum += value;
                    }
                }
            }
        }
    }
    
    fn process_logs(&mut self, logs: &[String]) {
        for line in logs {
            self.process_log_line(line);
        }
    }
}

// YOUR TASK: Profile and optimize this pipeline

fn generate_test_logs() -> Vec<String> {
    // Generate realistic log data for profiling
    let mut logs = Vec::new();
    for i in 0..100_000 {
        match i % 4 {
            0 => logs.push(format!("ERROR Failed to connect to database, error_code={}", i % 100)),
            1 => logs.push(format!("WARN Slow query detected, duration={}ms, query_id={}", i % 5000, i)),
            2 => logs.push(format!("INFO User login successful, user_id={}, session_id={}", i % 1000, i)),
            _ => logs.push(format!("DEBUG Processing request {}", i)),
        }
    }
    logs
}

fn profile_log_processing() {
    // TODO: Generate flamegraph using cargo flamegraph
    // TODO: Identify the hotspots in the processing pipeline
    // TODO: Optimize the identified bottlenecks
    // TODO: Compare before/after performance with benchmarks
    
    let mut processor = LogProcessor::new();
    let logs = generate_test_logs();
    
    let start = std::time::Instant::now();
    processor.process_logs(&logs);
    let duration = start.elapsed();
    
    println!("Processed {} logs in {:?}", logs.len(), duration);
}
```

**CPU Profiling Tasks:**
1. **Generate flamegraph**: Use `cargo flamegraph` to profile the log processing pipeline
2. **Identify hotspots**: What functions consume the most CPU time?
3. **Regex optimization**: Are regex compilations a bottleneck? How can you optimize pattern matching?
4. **String allocation analysis**: How much time is spent in string allocation/formatting?
5. **Implement optimizations**: Provide an optimized version and measure the improvement

### Problem 3.2: perf Integration for System-Level Analysis (10 points)

Use Linux perf to analyze system-level performance characteristics:

```rust
// System-intensive operations for perf analysis
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn system_intensive_workload() {
    // File I/O intensive
    let mut file = File::create("/tmp/perf_test.txt").unwrap();
    for i in 0..1_000_000 {
        writeln!(file, "Line number: {}", i).unwrap();
    }
    
    // CPU intensive  
    let mut sum = 0u64;
    for i in 0..10_000_000 {
        sum = sum.wrapping_add(i * i);
    }
    
    // Memory intensive
    let mut data: Vec<Vec<u8>> = Vec::new();
    for _ in 0..1000 {
        data.push(vec![0u8; 1024 * 1024]); // 1MB each
    }
    
    println!("Workload complete: sum={}, vectors={}", sum, data.len());
}

// YOUR TASK: Analyze with perf
```

**perf Analysis Tasks:**
1. **Basic CPU profiling**: Use `perf record` and `perf report` to identify CPU hotspots
2. **Cache analysis**: Use `perf stat` to measure L1/L2/L3 cache hit rates and page faults
3. **System call analysis**: Profile system call overhead with `perf trace`
4. **Memory bandwidth**: Measure memory bandwidth utilization during the workload
5. **Optimization guidance**: Based on perf data, what optimizations would you recommend?

---

## Section IV: Async Performance and Monitoring (30 points)

### Problem 4.1: tokio-console Integration and Analysis (15 points)

Set up tokio-console monitoring and analyze async performance characteristics:

```rust
// Add to Cargo.toml:
// [dependencies]
// tokio = { version = "1", features = ["full", "tracing"] }
// console-subscriber = "0.1"
// tracing = "0.1"

use tokio::task::JoinSet;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Initialize tokio-console subscriber
    console_subscriber::init();
    
    // Complex async workload for analysis
    analyze_async_performance().await;
}

async fn analyze_async_performance() {
    let mut join_set = JoinSet::new();
    
    // Spawn various types of async tasks
    for i in 0..50 {
        match i % 4 {
            0 => {
                // CPU-bound task (bad for async)
                join_set.spawn(async move {
                    let mut sum = 0u64;
                    for j in 0..1_000_000 {
                        sum = sum.wrapping_add(j);
                    }
                    sum
                });
            },
            1 => {
                // I/O-bound task (good for async)  
                join_set.spawn(async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    42
                });
            },
            2 => {
                // Mixed workload
                join_set.spawn(async move {
                    for _ in 0..5 {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        let _sum: u64 = (0..10_000).sum();
                    }
                    99
                });
            },
            _ => {
                // Blocking task (problematic)
                join_set.spawn(async move {
                    std::thread::sleep(Duration::from_millis(50)); // BLOCKING!
                    100
                });
            }
        }
    }
    
    // Wait for completion
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(value) => println!("Task completed: {}", value),
            Err(e) => println!("Task error: {}", e),
        }
    }
}
```

**tokio-console Analysis Tasks:**
1. **Setup monitoring**: Configure tokio-console and connect to the running application
2. **Task analysis**: Identify which tasks are blocking the executor
3. **Resource utilization**: Measure actual CPU and memory usage per task type
4. **Blocking detection**: Find and fix the blocking operations that hurt async performance
5. **Optimization recommendations**: Based on console data, how would you optimize this workload?

### Problem 4.2: Async Latency Distribution Analysis (15 points)

Measure and analyze latency distributions in async systems:

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;
use rand::Rng;

// Simulate SRE async services with varying latency characteristics
struct ServiceSimulator {
    name: String,
    base_latency: Duration,
    jitter_percent: f64,
}

impl ServiceSimulator {
    async fn call(&self) -> Duration {
        let mut rng = rand::thread_rng();
        
        // Add jitter to base latency
        let jitter = self.base_latency.as_nanos() as f64 * self.jitter_percent * rng.gen::<f64>();
        let actual_latency = self.base_latency + Duration::from_nanos(jitter as u64);
        
        let start = Instant::now();
        sleep(actual_latency).await;
        start.elapsed()
    }
}

async fn measure_latency_distributions() {
    let services = vec![
        ServiceSimulator {
            name: "database".to_string(),
            base_latency: Duration::from_millis(10),
            jitter_percent: 0.5, // 50% jitter
        },
        ServiceSimulator {
            name: "cache".to_string(), 
            base_latency: Duration::from_millis(1),
            jitter_percent: 0.2, // 20% jitter
        },
        ServiceSimulator {
            name: "external_api".to_string(),
            base_latency: Duration::from_millis(100),
            jitter_percent: 1.0, // 100% jitter (unreliable)
        },
    ];
    
    // YOUR TASK: Measure latency distributions for each service
    
    for service in &services {
        println!("Measuring latency for service: {}", service.name);
        
        let mut latencies = Vec::new();
        for _ in 0..1000 {
            let latency = service.call().await;
            latencies.push(latency);
        }
        
        // TODO: Calculate and display latency statistics:
        // - Mean, median, P95, P99, P99.9 percentiles
        // - Standard deviation
        // - Min/max values
        // - Latency histogram
        
        // TODO: Identify outliers and analyze their impact
        // TODO: Compare concurrent vs sequential call patterns
        
        analyze_service_latency(&service.name, &latencies);
    }
}

fn analyze_service_latency(service_name: &str, latencies: &[Duration]) {
    // YOUR IMPLEMENTATION: Statistical analysis of latency data
    todo!("Implement comprehensive latency analysis")
}
```

**Latency Analysis Tasks:**
1. **Statistical analysis**: Implement percentile calculations (P50, P95, P99, P99.9)
2. **Distribution visualization**: Create latency histograms showing the distribution shape
3. **Outlier detection**: Identify and analyze latency outliers
4. **Concurrency impact**: Compare latency distributions when calling services concurrently vs sequentially
5. **SRE insights**: What latency characteristics would trigger alerts in production?

---

## Performance Mastery Assessment

### Scoring Rubric

**Performance Expert (108-120 points)**
- Demonstrates mastery of benchmarking, profiling, and optimization tools
- Can identify performance bottlenecks through systematic measurement
- Understands trade-offs between different optimization approaches
- Provides actionable optimization recommendations based on data

**Performance Proficient (96-107 points)**
- Correctly uses benchmarking and profiling tools
- Can identify obvious performance issues through measurement
- Shows understanding of basic performance optimization principles
- Makes some optimization recommendations

**Performance Developing (84-95 points)**
- Can run basic benchmarks but struggles with interpretation
- Limited ability to use profiling tools effectively
- Identifies some performance issues but misses systematic approach
- Optimization attempts may not be data-driven

**Performance Novice (60-83 points)**
- Can follow benchmarking examples but doesn't understand the results
- Struggles with profiling tools and interpretation
- Cannot systematically identify performance bottlenecks
- Optimization attempts are mostly guesswork

**Insufficient (<60 points)**
- Cannot effectively use performance measurement tools
- No systematic approach to performance analysis
- Cannot identify or fix performance issues
- Lacks understanding of performance optimization principles

---

## Bridge to Advanced Concepts

**You're Ready for Advanced Systems Diagnostic (05_advanced_systems_diagnostic.md) When:**

✅ You can benchmark any Rust code and interpret the results  
✅ You can profile memory allocation patterns and identify leaks  
✅ You can use CPU profiling to find and optimize hotspots  
✅ You can monitor async performance and identify blocking operations  
✅ You can measure and analyze latency distributions statistically

**Performance Tool Mastery Checklist:**
- [ ] criterion benchmarking with statistical analysis
- [ ] Memory profiling with valgrind/heaptrack
- [ ] CPU profiling with perf and flamegraphs  
- [ ] Async monitoring with tokio-console
- [ ] Custom allocator for allocation tracking
- [ ] Statistical analysis of performance data

---

## Next Steps by Performance Level

**Expert Level**: Ready for `05_advanced_systems_diagnostic.md` - Advanced Systems Diagnostic. You have the measurement skills to tackle unsafe Rust optimization and memory layout analysis.

**Proficient Level**: Practice more complex profiling scenarios. Focus on distributed system performance measurement before advancing.

**Developing Level**: Master the basic tools (criterion, perf, valgrind) with simpler problems before attempting advanced optimization.

**Novice Level**: Work through performance measurement fundamentals. Consider additional Rust performance resources before advancing.

---

*"You can't optimize what you can't measure. Master measurement first, optimization follows naturally."*

**Required Tools for This Course:**
```bash
# Install required profiling tools
cargo install flamegraph
cargo install tokio-console  
sudo apt install valgrind linux-perf  # Linux
brew install valgrind       # macOS
```
