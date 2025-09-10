# 04. Production Exam: Engineering Skills
**4-Day Production Systems Assessment**

## Mission Statement

This intensive 4-day bridge course transforms Rust developers from homework-level exercises into production-ready systems engineers. You will master the advanced patterns and production concerns needed for the midterm examination and Neon storage backend project.

**Prerequisites**: Completion of homework assignments (days 1-14) covering basic async, error handling, serde, and lifetimes.

**Target Outcome**: Fluency in advanced lifetime management, zero-copy optimization, production error handling, and distributed system patterns required for enterprise storage systems.

### Methodology: Active Recall Under Pressure

Each day combines **theoretical deep-dives** with **crisis simulations** - you'll debug production-grade race conditions, optimize memory usage under constraints, and design resilient distributed systems. The exercises are designed to bridge the gap between textbook knowledge and the complex, interconnected challenges of real systems.

### Core Technology Stack

- **Advanced Error Handling**: thiserror, anyhow, custom error chains with context propagation
- **Zero-Copy Processing**: serde lifetimes, Bytes ecosystem, memory layout optimization  
- **Production Async**: Complex lifetime interactions, cancellation, backpressure, resource management
- **Observability**: tracing with spans, structured logging, performance profiling
- **Distributed Patterns**: Reconciliation loops, eventual consistency, partial failure handling

---

## Day 1: Advanced Lifetime Management & Production Race Conditions

### Mental Model: The Lifetime Detective

In production systems, lifetime management becomes a question of correctness and performance. Think of lifetimes as contracts about data validity across async boundaries. A small mistake can cause use-after-free bugs, memory leaks, or cache invalidation races that only appear under load.

### Crisis Scenario: The Cache Invalidation Race

You're debugging a production issue where a storage cache occasionally serves stale data under concurrent compaction. The bug only appears with multiple readers and writers, making it a classic async race condition.

### Challenge 1.1: Fix the Arc<RwLock> Race (90 mins)

**Context**: A document cache with concurrent readers and a background compaction process that occasionally serves stale data.

```rust
// This code has a subtle race condition
pub struct DocumentCache<'a> {
    documents: Arc<RwLock<HashMap<String, &'a Document>>>,
    compaction_task: Option<JoinHandle<()>>,
}

impl<'a> DocumentCache<'a> {
    pub async fn get(&self, key: &str) -> Option<&'a Document> {
        let guard = self.documents.read().await;
        guard.get(key).copied() // BUG: What happens to the guard?
    }
    
    pub async fn compact(&self) {
        let mut guard = self.documents.write().await;
        // Compaction replaces the entire HashMap
        *guard = self.build_compacted_map().await;
        // What about ongoing readers?
    }
}
```

**Your Mission:**
1. Identify the race condition (hint: it's about guard lifetimes vs returned references)
2. Fix it using proper lifetime management
3. Handle the case where compaction invalidates in-flight reads
4. Ensure zero-copy access where possible
5. Add proper error handling for concurrent modification scenarios

**Advanced Requirements:**
- Use multiple lifetime parameters where needed
- Implement a streaming buffer that holds references with different lifetimes
- Handle cases where 'long: 'short lifetime bounds are needed
- Design for both owned and borrowed data access patterns

### Challenge 1.2: Multiple Lifetime Parameters (60 mins)

Implement a `StreamingBuffer` that demonstrates advanced lifetime management:

```rust
struct StreamingBuffer<'cache, 'config> 
where 
    'config: 'cache  // Config data outlives cache data
{
    // Design your fields to handle:
    // - Short-lived cache entries
    // - Long-lived configuration data
    // - References that must outlive the buffer
    // - Zero-copy slicing operations
}

// Must support operations like:
// - add_cached_data(&'cache str)
// - add_config_data(&'config str) 
// - get_persistent_view() -> Vec<&'config str>
// - slice_recent(count: usize) -> &'cache [&'cache str]
```

### Deep Dive: Memory Layout and Performance

**Lifetime Elision Rules**: When you can omit lifetimes and when you can't
**Variance**: How lifetimes interact with generic types (covariance, contravariance)
**Higher-Ranked Trait Bounds**: When you need `for<'a> Fn(&'a str) -> &'a str`
**Async Lifetime Interactions**: How `async fn` affects lifetime inference

---

## Day 2: Zero-Copy Processing & Advanced Serde Patterns

### Mental Model: The Memory Efficiency Engineer

Every byte copied is a missed opportunity for performance. In high-throughput systems, the difference between borrowing and owning data can be the difference between 100K and 1M requests per second. You'll learn to think in terms of memory allocation patterns and cache efficiency.

### Crisis Scenario: The Memory Allocation Explosion

Your JSON processing service is hitting OOM errors under load. Profiling shows excessive memory allocation in deserialization. You need to transform it into a zero-copy processing pipeline without changing the external API.

### Challenge 2.1: Zero-Copy JSON Processing Pipeline (2 hours)

**Context**: A high-frequency trading system processing market data where every microsecond and byte matters.

```rust
// Current (broken) approach - allocates everything
#[derive(Deserialize)]
struct MarketData {
    symbol: String,           // 🔴 Allocates
    price: f64,
    volume: u64,
    metadata: TradeMetadata,  // 🔴 Nested allocations
    raw_feed: Vec<u8>,       // 🔴 Large allocation
}

#[derive(Deserialize)]
struct TradeMetadata {
    exchange: String,         // 🔴 Repeated values
    data_source: String,      // 🔴 Repeated values
    extra_info: String,       // 🔴 Often empty
}
```

**Your Mission:**
1. Transform to zero-copy with borrowed data (`&str` instead of `String`)
2. Use `#[serde(borrow)]` strategically
3. Implement string interning for repeated values
4. Handle optional fields efficiently
5. Skip unnecessary nested data with `#[serde(skip_deserializing)]`
6. Create a memory pool for temporary processing

**Advanced Patterns:**
- Custom deserializers with `#[serde(deserialize_with)]`
- Flattened structures to reduce indirection
- Arena allocation for temporary data
- Memory mapping for large datasets

### Challenge 2.2: Streaming Deserialization (90 mins)

Process a continuous stream of JSON without loading everything into memory:

```rust
// Handle streaming input where individual JSON objects 
// can be processed and discarded immediately
async fn process_json_stream<R: AsyncBufRead + Unpin>(
    reader: R,
    processor: impl Fn(MarketData<'_>) -> Result<(), ProcessingError>
) -> Result<StreamStats, StreamError> {
    // Your implementation:
    // 1. Stream JSON objects one at a time
    // 2. Deserialize with zero-copy where possible  
    // 3. Process immediately (don't accumulate)
    // 4. Handle malformed JSON gracefully
    // 5. Provide backpressure control
    // 6. Track processing statistics
}
```

### Deep Dive: The Bytes Ecosystem

**BytesMut vs Bytes**: When to use each and how to convert efficiently
**Memory Mapping**: Using `mmap` for zero-copy file access
**SIMD Processing**: Leveraging CPU vector instructions for parsing
**Cache-Friendly Data Layouts**: Struct field ordering for performance

---

## Day 3: Production Error Handling & Observability

### Mental Model: The System Reliability Engineer

Errors in distributed systems aren't just about returning the right error message - they're about maintaining system stability, providing actionable diagnostics, and enabling automated recovery. You'll think like an SRE: every error is a signal about system health.

### Crisis Scenario: The Error Cascade Failure

A single upstream service failure is bringing down your entire distributed system. Errors are propagating without context, retry storms are overwhelming backends, and you have no visibility into the failure modes. You need to implement comprehensive error handling and observability.

### Challenge 3.1: Hierarchical Error Context System (2 hours)

**Context**: Build an error handling system for a multi-service storage backend where errors need rich context for debugging and automated remediation.

```rust
// Design a comprehensive error system that handles:
#[derive(Debug, thiserror::Error)]
enum StorageError {
    // Network errors (should retry with backoff)
    #[error("Network error: {message}")]
    Network { message: String, #[source] source: Box<dyn std::error::Error + Send + Sync> },
    
    // Data errors (should skip and continue)
    #[error("Data corruption in {location}")]
    DataCorruption { location: String },
    
    // Resource errors (should circuit break)
    #[error("Resource exhaustion: {resource}")]
    ResourceExhausted { resource: String },
    
    // Configuration errors (should fail fast)
    #[error("Configuration error: {field}")]
    Configuration { field: String },
}

// Implement advanced error handling patterns:
pub struct RetryableService<T> {
    service: T,
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    metrics: ErrorMetrics,
}

impl<T> RetryableService<T> {
    // Implement:
    // - Exponential backoff with jitter
    // - Different retry policies by error type
    // - Circuit breaker integration
    // - Error correlation and tracing
    // - Graceful degradation strategies
}
```

**Your Mission:**
1. Implement custom error types with proper `From` conversions
2. Build a retry mechanism with different strategies per error type
3. Add circuit breaker pattern to prevent cascade failures
4. Include comprehensive error correlation with request IDs
5. Design for error recovery and automated remediation
6. Add metrics and structured logging for observability

### Challenge 3.2: Distributed Tracing & Performance Profiling (90 mins)

Implement production-grade observability:

```rust
// Build a tracing system that provides:
// - Request correlation across service boundaries
// - Performance profiling with timing spans
// - Structured logging with searchable fields
// - Conditional compilation for zero-cost in release
// - Integration with external monitoring systems

pub struct TracingContext {
    request_id: Uuid,
    tenant_id: TenantId,
    operation: &'static str,
    start_time: Instant,
    parent_span: Option<SpanId>,
}

// Implement macro-based instrumentation:
#[tracing::instrument(skip(self, ctx), fields(tenant_id = %ctx.tenant_id))]
pub async fn process_request(&self, ctx: TracingContext) -> Result<Response, ServiceError> {
    // Your implementation should:
    // - Create nested spans for sub-operations
    // - Track performance metrics automatically
    // - Correlate errors with request context
    // - Export to multiple backends (console, JSON, OTLP)
    // - Handle high-frequency operations efficiently
}
```

### Deep Dive: Production Observability

**Structured Logging**: JSON format, log aggregation, searchable fields
**Distributed Tracing**: OpenTelemetry, span context propagation
**Metrics Collection**: RED method (Rate, Errors, Duration)
**Performance Profiling**: CPU profiling, memory allocation tracking

---

## Day 4: Distributed System Patterns & Integration Challenge

### Mental Model: The Distributed Systems Architect

Real systems are distributed, partially failed, and eventually consistent. You'll learn to think in terms of CAP theorem tradeoffs, partial failure handling, and state reconciliation. Every operation might fail halfway through, and your system must handle this gracefully.

### Crisis Scenario: The Split-Brain Disaster

Your distributed storage system has experienced a network partition, resulting in split-brain where different nodes have inconsistent views of the data. You need to implement a reconciliation system that can detect and resolve inconsistencies while the system remains online.

### Challenge 4.1: State Reconciliation Engine (2.5 hours)

**Context**: Design a reconciliation system that can handle millions of files across multiple storage nodes with concurrent modifications.

```rust
// Your reconciliation system must handle:
pub struct ReconciliationEngine<S: RemoteStorage> {
    local_state: Arc<DashMap<FileId, FileMetadata>>,
    remote_storage: Arc<S>,
    in_flight_operations: Arc<DashMap<FileId, OperationType>>,
    reconciliation_metrics: Arc<ReconciliationMetrics>,
}

pub enum ReconciliationAction {
    Download { file_id: FileId, expected_version: Version },
    Upload { file_id: FileId, local_version: Version },
    Delete { file_id: FileId, reason: DeletionReason },
    Conflict { file_id: FileId, local: Version, remote: Version },
}

impl<S: RemoteStorage> ReconciliationEngine<S> {
    // Implement a production-ready reconciliation loop:
    pub async fn reconcile_continuously(
        &self,
        cancel: CancellationToken
    ) -> Result<(), ReconciliationError> {
        // Requirements:
        // 1. Stream file listings (don't load all in memory)
        // 2. Handle concurrent modifications during reconciliation
        // 3. Implement conflict resolution strategies
        // 4. Provide resumability after crashes
        // 5. Resource-aware (limit concurrent operations)
        // 6. Comprehensive metrics and observability
        // 7. Graceful handling of partial failures
    }
    
    // Handle the three-way merge problem:
    pub async fn resolve_conflict(
        &self,
        file_id: FileId,
        local_version: Version,
        remote_version: Version,
        base_version: Option<Version>
    ) -> Result<ConflictResolution, ConflictError> {
        // Implement conflict resolution strategies:
        // - Last-writer-wins with timestamps
        // - Version vector comparisons
        // - Content-based merging where possible
        // - Manual intervention escalation
    }
}
```

**Your Mission:**
1. Design a streaming reconciliation loop that handles millions of files
2. Implement proper concurrency control with semaphores and rate limiting
3. Handle partial failures gracefully (network issues, timeouts)
4. Add comprehensive observability and progress tracking
5. Design for crash recovery and resumability
6. Implement conflict resolution strategies
7. Add circuit breakers and backpressure control

### Challenge 4.2: End-to-End Integration Test (90 mins)

Create a comprehensive integration test that validates your entire system:

```rust
// Build a test that simulates:
#[tokio::test]
async fn test_production_scenarios() {
    // Scenario 1: High load with concurrent operations
    // Scenario 2: Network partitions and recovery
    // Scenario 3: Node failures during operations
    // Scenario 4: Data corruption detection and recovery
    // Scenario 5: Memory pressure and resource constraints
    
    // Your test should validate:
    // - Correctness under concurrent access
    // - Performance characteristics (latency/throughput)
    // - Resource usage (memory, file descriptors)
    // - Error handling and recovery
    // - Observability output quality
}

// Implement chaos engineering patterns:
pub struct ChaosSimulator {
    network_failures: f64,    // Probability of network issues
    slow_operations: f64,     // Probability of slowness
    memory_pressure: bool,    // Simulate resource constraints
    crash_recovery: bool,     // Test crash/restart scenarios
}
```

### Deep Dive: Production System Patterns

**Eventual Consistency**: CRDTs, vector clocks, conflict resolution
**Circuit Breakers**: Failure detection, automatic recovery, bulkhead isolation
**Backpressure**: Flow control, rate limiting, load shedding
**Graceful Degradation**: Fallback strategies, partial functionality
**Chaos Engineering**: Failure injection, resilience testing

---

## Assessment & Graduation Criteria

### Daily Standup Protocol

Each morning, demonstrate mastery by explaining:
1. **Yesterday's Crisis**: "Walk me through the race condition you fixed and how you validated the solution"
2. **System Thinking**: "How does this component interact with the broader distributed system?"
3. **Production Readiness**: "What could go wrong with this code at 3 AM on a weekend?"

### Final Integration Challenge

Your final project combines all concepts into a production-ready system:

**The Distributed Storage Cache**
- Multi-tenant document storage with CRUD operations
- Zero-copy deserialization for high-throughput ingestion
- Advanced error handling with circuit breakers and retries
- Comprehensive observability with distributed tracing
- State reconciliation across multiple storage backends
- Chaos engineering validation of failure scenarios

### Graduation Requirements

You're ready for the midterm and Neon project when you can:

1. **Debug lifetime-related race conditions** in concurrent systems
2. **Optimize memory usage** with zero-copy techniques and arena allocation
3. **Design error handling strategies** that prevent cascade failures
4. **Implement distributed tracing** that enables production debugging
5. **Build reconciliation systems** that handle partial failures gracefully
6. **Validate system behavior** under chaos engineering scenarios

### Success Metrics

- **Performance**: Your system handles 10K+ operations/second
- **Reliability**: Survives chaos engineering with <0.1% error rate
- **Observability**: Provides actionable diagnostics for all failure modes
- **Resource Efficiency**: Stays under memory/CPU limits during load tests

---

## Next Steps: Advanced Challenges

After completing this bridge course, you'll be ready for:

1. **Midterm Examination**: Advanced Rust concepts including complex lifetime scenarios, async programming patterns, and system design questions

2. **Neon Storage Backend Project**: 6-day intensive working on production storage system patterns including WAL processing, layer management, and distributed reconciliation

3. **Kubernetes Operator Capstone**: Building cloud-native operators for managing distributed Rust applications

The patterns you've mastered - from careful lifetime management to zero-copy optimizations to distributed reconciliation - are the daily tools of senior systems engineers. You're now ready to work on production distributed systems where every microsecond and byte matters.

## Resources & References

### Essential Reading
- **Rust Nomicon**: Advanced unsafe code and memory layout
- **Designing Data-Intensive Applications**: Distributed systems patterns
- **Site Reliability Engineering**: Production operations practices

### Key Dependencies
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
bytes = "1.5"
tracing = "0.1"
thiserror = "1.0"
anyhow = "1.0"
dashmap = "5.5"
```

### Performance Tools
- **cargo flamegraph**: CPU profiling
- **heaptrack**: Memory allocation tracking  
- **criterion**: Micro-benchmarking
- **tokio-console**: Async runtime debugging
