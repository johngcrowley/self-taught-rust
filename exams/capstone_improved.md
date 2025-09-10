# Distributed Storage Systems Engineering

*Build production-ready storage infrastructure like Neon's PostgreSQL backend*

## Overview

You're implementing the storage layer for a distributed database system. Your code must handle TB-scale data with microsecond-level performance requirements while maintaining ACID guarantees across a distributed cluster.

**Success Metrics**: 
- **Throughput**: 100,000+ page reads/second 
- **Latency**: <1ms P95 for page reconstruction
- **Reliability**: 99.99% uptime with automatic recovery
- **Efficiency**: <512MB memory per TB of managed data

---

## Architecture Overview

Modern distributed databases separate compute from storage:
- **Compute Layer**: PostgreSQL instances that handle queries
- **Storage Layer**: Distributed system that stores and serves data pages
- **Network Protocol**: Custom protocol for page requests/responses

Your storage layer must:
1. Store data pages efficiently across multiple nodes
2. Reconstruct any page at any point in time using WAL replay
3. Handle concurrent access from hundreds of compute nodes
4. Automatically compact and optimize storage layout
5. Recover from node failures without data loss

---

## Setup

```bash
cargo new --bin storage-engine
cd storage-engine
```

**Cargo.toml:**
```toml
[package]
name = "storage-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core async runtime
tokio = { version = "1.35", features = ["full", "tracing"] }
futures = "0.3"
pin-project = "1.1"

# Network and serialization
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
bytes = "1.5"

# Database integration
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }

# Performance and observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
metrics = "0.21"
metrics-prometheus = "0.6"

# Storage and concurrency
dashmap = "5.5"
async-trait = "0.1"
thiserror = "1.0"
uuid = { version = "1.6", features = ["v4"] }

# Development
criterion = { version = "0.5", features = ["html_reports"] }
dhat = "0.3"

[[bench]]
name = "storage_benchmarks"
harness = false

[dev-dependencies]
tempfile = "3.8"
```

**src/common.rs:**
```rust
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

// Storage primitives
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lsn(pub u64);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PageKey {
    pub relation_id: u32,
    pub block_num: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct TenantId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy)]
pub struct TimelineId(pub uuid::Uuid);

// Error types with structured information
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Page not found: {key:?} at LSN {lsn:?}")]
    PageNotFound { key: PageKey, lsn: Lsn },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Network timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

// Request context for tracing and metrics
pub struct RequestContext {
    pub request_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub timeline_id: TimelineId,
}
```

---

## Day 1: Concurrent Page Storage with Race Condition Debugging

### The Challenge

Build a thread-safe page storage system that handles concurrent reads/writes without race conditions.

**Performance Target**: 50,000+ concurrent page reads with <100μs latency

**File: `src/day1_storage.rs`**

```rust
use crate::common::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use metrics::{histogram, counter};
use tracing::{instrument, info, warn};

pub struct PageLayer {
    pub start_lsn: Lsn,
    pub end_lsn: Lsn,
    data: Arc<DashMap<PageKey, Bytes>>,
    size_bytes: std::sync::atomic::AtomicU64,
}

impl PageLayer {
    #[instrument(skip(self, ctx), fields(key = ?key, lsn = ?lsn))]
    pub async fn get_page(
        &self,
        key: PageKey,
        lsn: Lsn,
        ctx: &RequestContext,
    ) -> Result<Bytes, StorageError> {
        let start = std::time::Instant::now();
        
        // Check LSN bounds
        if lsn < self.start_lsn || lsn >= self.end_lsn {
            counter!("storage.page_requests_total", 1, "result" => "out_of_bounds");
            return Err(StorageError::PageNotFound { key, lsn });
        }
        
        // Simulate I/O delay and potential yield point
        tokio::time::sleep(std::time::Duration::from_micros(10)).await;
        
        match self.data.get(&key) {
            Some(page_data) => {
                histogram!("storage.page_read_duration", start.elapsed());
                counter!("storage.page_requests_total", 1, "result" => "success");
                Ok(page_data.clone())
            }
            None => {
                counter!("storage.page_requests_total", 1, "result" => "not_found");
                Err(StorageError::PageNotFound { key, lsn })
            }
        }
    }
}

pub struct Timeline {
    layers: Arc<RwLock<Vec<Arc<PageLayer>>>>,
    compaction_in_progress: std::sync::atomic::AtomicBool,
}

impl Timeline {
    // PRODUCTION BUG: Fix the race condition in this method
    pub async fn get_page_at_lsn(
        &self,
        key: PageKey,
        request_lsn: Lsn,
        ctx: &RequestContext,
    ) -> Result<Bytes, StorageError> {
        // Problem: What happens if compaction runs between acquiring the read lock
        // and calling get_page()? The layer might be removed from the map!
        
        let layers = self.layers.read().await;
        
        // Find the appropriate layer
        for layer in layers.iter().rev() {
            if layer.start_lsn <= request_lsn && request_lsn < layer.end_lsn {
                // RACE CONDITION: Layer might be deallocated during compaction
                return layer.get_page(key, request_lsn, ctx).await;
            }
        }
        
        Err(StorageError::PageNotFound { key, lsn: request_lsn })
    }
    
    pub async fn compact_layers(&self) {
        // Compaction creates new layers and removes old ones
        // This is where the race condition is introduced
        
        if self.compaction_in_progress.compare_exchange(
            false, true, 
            std::sync::atomic::Ordering::Acquire,
            std::sync::atomic::Ordering::Relaxed
        ).is_err() {
            warn!("Compaction already in progress");
            return;
        }
        
        let mut layers = self.layers.write().await;
        // ... compaction logic that replaces layers
        
        self.compaction_in_progress.store(false, std::sync::atomic::Ordering::Release);
    }
}

// Your task: Implement the fixed version
pub struct TimelineFixed {
    // How would you fix the race condition?
    // Hint: Think about Arc reference counting and layer lifetime management
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_concurrent_access_during_compaction() {
        // Test: 1000 concurrent reads while compaction runs
        // Should never see stale data or crashes
    }
    
    #[tokio::test] 
    async fn test_performance_under_load() {
        // Benchmark: 50,000 page reads in <1 second
    }
}
```

### Debugging Exercise

**Memory debugging with dhat:**
```rust
#[cfg(feature = "dhat")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// Run with: cargo run --features dhat
// Analyze allocation patterns during high concurrency
```

**Performance Requirements:**
- Handle 50,000+ concurrent page reads
- <100μs P95 latency per page read
- Zero race conditions under stress testing
- Memory usage grows linearly with active layers

---

## Day 2: Zero-Copy WAL Stream Processing

### The Challenge

Process PostgreSQL WAL (Write-Ahead Log) records at high throughput without unnecessary memory allocations.

**Performance Target**: Process 1GB/second of WAL data using <512MB RAM

**File: `src/day2_wal_processing.rs`**

```rust
use bytes::{Bytes, BytesMut, Buf};
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::{instrument, debug};

pub struct WalRecord {
    pub lsn: Lsn,
    pub record_type: u8,
    pub payload: Bytes,  // Zero-copy slice of original buffer
}

pub struct WalProcessor {
    buffer_pool: BufferPool,
    processed_bytes: std::sync::atomic::AtomicU64,
    processed_records: std::sync::atomic::AtomicU64,
}

pub struct BufferPool {
    // Implement buffer pooling to avoid allocation churn
    // Reuse BytesMut buffers for high-throughput processing
}

impl WalProcessor {
    #[instrument(skip(self, reader))]
    pub async fn process_stream<R: AsyncRead + Unpin>(
        &mut self,
        mut reader: R,
    ) -> Result<ProcessingStats, StorageError> {
        let mut buffer = self.buffer_pool.get_buffer(64 * 1024); // 64KB buffer
        let mut records_processed = 0u64;
        let mut bytes_processed = 0u64;
        
        loop {
            // Read next chunk, handling partial reads
            let bytes_read = match reader.read_buf(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(e) => return Err(StorageError::Io(e)),
            };
            
            bytes_processed += bytes_read as u64;
            
            // Process complete records from buffer
            while buffer.len() >= 8 { // Minimum record header size
                match self.try_decode_record(&mut buffer) {
                    Ok(Some(record)) => {
                        self.handle_record(record).await?;
                        records_processed += 1;
                        
                        if records_processed % 10000 == 0 {
                            debug!("Processed {} records, {} bytes", records_processed, bytes_processed);
                        }
                    }
                    Ok(None) => break, // Need more data
                    Err(e) => {
                        // Handle corrupt records gracefully
                        warn!("Skipping corrupt WAL record: {}", e);
                        buffer.advance(1); // Skip one byte and try again
                    }
                }
            }
            
            // Compact buffer if it's getting large
            if buffer.len() > 32 * 1024 {
                buffer.reserve(64 * 1024);
            }
        }
        
        Ok(ProcessingStats {
            records_processed,
            bytes_processed,
            processing_time: /* track timing */,
        })
    }
    
    fn try_decode_record(&self, buffer: &mut BytesMut) -> Result<Option<WalRecord>, WalDecodeError> {
        if buffer.len() < 8 {
            return Ok(None); // Need more data
        }
        
        // Parse record header (zero-copy)
        let lsn = Lsn(buffer.get_u64());
        
        if buffer.len() < 4 {
            // Put LSN back and wait for more data
            let mut temp = BytesMut::with_capacity(8);
            temp.put_u64(lsn.0);
            temp.extend_from_slice(buffer);
            *buffer = temp;
            return Ok(None);
        }
        
        let record_length = buffer.get_u32() as usize;
        let record_type = buffer.get_u8();
        
        if buffer.len() < record_length {
            // Put everything back and wait for more data
            // ... implementation
            return Ok(None);
        }
        
        // Extract payload as zero-copy slice
        let payload = buffer.split_to(record_length).freeze();
        
        Ok(Some(WalRecord {
            lsn,
            record_type,
            payload,
        }))
    }
    
    async fn handle_record(&self, record: WalRecord) -> Result<(), StorageError> {
        // Apply record to storage layer
        // Update metrics
        self.processed_records.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.processed_bytes.fetch_add(record.payload.len() as u64, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }
}

pub struct ProcessingStats {
    pub records_processed: u64,
    pub bytes_processed: u64,
    pub processing_time: std::time::Duration,
}

// Challenge: Implement buffer pooling for zero-allocation processing
impl BufferPool {
    fn get_buffer(&self, min_capacity: usize) -> BytesMut {
        // Your implementation: reuse buffers to avoid allocation churn
        todo!("Implement buffer pooling")
    }
    
    fn return_buffer(&self, buffer: BytesMut) {
        // Return buffer to pool for reuse
        todo!("Implement buffer return")
    }
}
```

### Performance Requirements

- Process 1GB/second of WAL data
- Use <512MB RAM regardless of input size
- Handle malformed/corrupt records gracefully
- Maintain processing metrics in real-time
- Zero-copy record extraction where possible

**Benchmarking:**
```rust
// benches/storage_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn benchmark_wal_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("wal_processing");
    group.throughput(Throughput::Bytes(1024 * 1024 * 100)); // 100MB
    
    group.bench_function("process_wal_stream", |b| {
        b.iter(|| {
            // Benchmark WAL processing throughput
        });
    });
}

criterion_group!(benches, benchmark_wal_processing);
criterion_main!(benches);
```

---

## Day 3: Distributed Compaction with Observability

### The Challenge

Implement background compaction that merges storage layers without blocking foreground operations.

**Performance Target**: Compact 10GB of layer data in <5 minutes with <1% impact on read latency

**File: `src/day3_compaction.rs`**

```rust
use futures::stream::{self, StreamExt};
use tokio_util::sync::CancellationToken;
use metrics::{histogram, counter, gauge};

pub struct CompactionJob {
    pub input_layers: Vec<Arc<PageLayer>>,
    pub target_layer_size: u64,
    pub compaction_type: CompactionType,
}

pub enum CompactionType {
    Level0ToLevel1,    // Merge recent writes
    LevelNToLevelN1,   // Merge historical data
    GarbageCollection, // Remove obsolete data
}

pub struct CompactionMetrics {
    pub input_size_bytes: u64,
    pub output_size_bytes: u64,
    pub records_merged: u64,
    pub garbage_collected_bytes: u64,
    pub compaction_duration: std::time::Duration,
}

pub struct Compactor {
    max_concurrent_jobs: usize,
    target_layer_size: u64,
    // Add comprehensive observability
    active_jobs: Arc<std::sync::atomic::AtomicUsize>,
    total_jobs_completed: Arc<std::sync::atomic::AtomicU64>,
}

impl Compactor {
    #[instrument(skip(self, job), fields(
        input_layers = job.input_layers.len(),
        compaction_type = ?job.compaction_type
    ))]
    pub async fn compact(
        &self,
        job: CompactionJob,
        cancel: CancellationToken,
    ) -> Result<Vec<Arc<PageLayer>>, StorageError> {
        let start_time = std::time::Instant::now();
        let job_id = uuid::Uuid::new_v4();
        
        info!("Starting compaction job {}", job_id);
        
        // Update metrics
        self.active_jobs.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        gauge!("compaction.active_jobs", self.active_jobs.load(std::sync::atomic::Ordering::Relaxed) as f64);
        
        // Calculate input size
        let input_size: u64 = job.input_layers.iter()
            .map(|layer| layer.size_bytes.load(std::sync::atomic::Ordering::Relaxed))
            .sum();
        
        histogram!("compaction.input_size_bytes", input_size as f64);
        
        // Perform k-way merge of sorted layer streams
        let output_layers = self.merge_layers(job.input_layers, job.target_layer_size, cancel).await?;
        
        // Calculate compaction efficiency
        let output_size: u64 = output_layers.iter()
            .map(|layer| layer.size_bytes.load(std::sync::atomic::Ordering::Relaxed))
            .sum();
        
        let compression_ratio = if input_size > 0 { 
            output_size as f64 / input_size as f64 
        } else { 
            1.0 
        };
        
        // Update completion metrics
        histogram!("compaction.duration_seconds", start_time.elapsed().as_secs_f64());
        histogram!("compaction.compression_ratio", compression_ratio);
        counter!("compaction.jobs_completed_total", 1);
        
        self.active_jobs.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        self.total_jobs_completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        info!("Completed compaction job {} in {:?}, compression: {:.2}x", 
              job_id, start_time.elapsed(), 1.0 / compression_ratio);
        
        Ok(output_layers)
    }
    
    async fn merge_layers(
        &self,
        input_layers: Vec<Arc<PageLayer>>,
        target_size: u64,
        cancel: CancellationToken,
    ) -> Result<Vec<Arc<PageLayer>>, StorageError> {
        // Implement k-way merge with proper backpressure
        let streams: Vec<_> = input_layers.into_iter()
            .map(|layer| self.layer_to_stream(layer))
            .collect();
        
        // Merge streams while respecting cancellation
        let mut merged_stream = self.k_way_merge(streams);
        let mut current_layer_builder = LayerBuilder::new(target_size);
        let mut output_layers = Vec::new();
        
        while let Some(record) = merged_stream.next().await {
            if cancel.is_cancelled() {
                return Err(StorageError::ResourceExhausted { 
                    resource: "Compaction cancelled".to_string() 
                });
            }
            
            if current_layer_builder.would_exceed_size(&record) {
                // Finalize current layer and start new one
                let layer = current_layer_builder.build().await?;
                output_layers.push(Arc::new(layer));
                current_layer_builder = LayerBuilder::new(target_size);
            }
            
            current_layer_builder.add_record(record);
        }
        
        // Finalize last layer
        if !current_layer_builder.is_empty() {
            let layer = current_layer_builder.build().await?;
            output_layers.push(Arc::new(layer));
        }
        
        Ok(output_layers)
    }
    
    pub fn health_status(&self) -> CompactionHealth {
        CompactionHealth {
            active_jobs: self.active_jobs.load(std::sync::atomic::Ordering::Relaxed),
            completed_jobs: self.total_jobs_completed.load(std::sync::atomic::Ordering::Relaxed),
            // Add more health indicators
        }
    }
}

pub struct CompactionHealth {
    pub active_jobs: usize,
    pub completed_jobs: u64,
    // Add: average_job_duration, jobs_failed, etc.
}

// Implement efficient k-way merge using binary heap
struct KWayMerge {
    // Your implementation: merge multiple sorted streams efficiently
}
```

### Performance Requirements

- Compact 10GB in <5 minutes
- <1% impact on foreground read latency during compaction
- Handle cancellation gracefully (stop within 30 seconds)
- Comprehensive progress reporting and health metrics
- Memory usage bounded regardless of input size

---

## Day 4: High-Performance API with Circuit Breakers

### The Challenge

Build a production API that handles 100,000+ requests/second with comprehensive error handling and reliability patterns.

**Performance Target**: <1ms P95 latency at 100,000 QPS with 99.99% availability

**File: `src/day4_api.rs`**

```rust
use axum::{
    Router,
    extract::{Path, State, Query},
    response::Json,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::time::timeout;

pub struct ApiServer {
    storage: Arc<Timeline>,
    circuit_breaker: Arc<CircuitBreaker>,
    rate_limiter: Arc<RateLimiter>,
}

// Production-grade circuit breaker
pub struct CircuitBreaker {
    state: std::sync::RwLock<CircuitState>,
    failure_threshold: usize,
    recovery_timeout: std::time::Duration,
    failure_count: std::sync::atomic::AtomicUsize,
    last_failure: std::sync::RwLock<Option<std::time::Instant>>,
}

pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing fast
    HalfOpen,   // Testing recovery
}

impl ApiServer {
    #[instrument(skip(self, ctx), fields(
        page_key = ?page_key,
        lsn = ?lsn,
        tenant_id = ?ctx.tenant_id
    ))]
    pub async fn get_page_handler(
        &self,
        page_key: PageKey,
        lsn: Option<Lsn>,
        ctx: RequestContext,
    ) -> Result<Json<PageResponse>, ApiError> {
        let start = std::time::Instant::now();
        
        // Check rate limits
        self.rate_limiter.check_limit(&ctx.tenant_id).await?;
        
        // Check circuit breaker
        if self.circuit_breaker.is_open() {
            counter!("api.requests_total", 1, "result" => "circuit_open");
            return Err(ApiError::ServiceUnavailable);
        }
        
        // Apply timeout to prevent hanging requests
        let page_result = timeout(
            std::time::Duration::from_millis(100), // 100ms timeout
            self.storage.get_page_at_lsn(page_key, lsn.unwrap_or(Lsn::MAX), &ctx)
        ).await;
        
        match page_result {
            Ok(Ok(page_data)) => {
                // Success - reset circuit breaker
                self.circuit_breaker.record_success();
                
                histogram!("api.request_duration_ms", start.elapsed().as_millis() as f64);
                counter!("api.requests_total", 1, "result" => "success");
                
                Ok(Json(PageResponse {
                    page_key,
                    lsn: lsn.unwrap_or(Lsn::MAX),
                    data: page_data,
                    retrieved_at: chrono::Utc::now(),
                }))
            }
            Ok(Err(e)) => {
                // Storage error - update circuit breaker
                self.circuit_breaker.record_failure();
                
                counter!("api.requests_total", 1, "result" => "storage_error");
                error!("Storage error for page {:?}: {}", page_key, e);
                
                Err(ApiError::from(e))
            }
            Err(_) => {
                // Timeout
                self.circuit_breaker.record_failure();
                
                counter!("api.requests_total", 1, "result" => "timeout");
                warn!("Request timeout for page {:?}", page_key);
                
                Err(ApiError::Timeout)
            }
        }
    }
    
    pub async fn health_handler(&self) -> Result<Json<HealthStatus>, ApiError> {
        Ok(Json(HealthStatus {
            status: "healthy".to_string(),
            circuit_breaker_state: self.circuit_breaker.current_state(),
            active_connections: self.storage.active_connections(),
            uptime: /* track uptime */,
        }))
    }
    
    pub fn create_router(self) -> Router {
        Router::new()
            .route("/v1/pages/:relation/:block", axum::routing::get(
                |Path((relation, block)): Path<(u32, u32)>,
                 Query(params): Query<PageQuery>,
                 State(server): State<Arc<ApiServer>>| async move {
                    let page_key = PageKey { relation_id: relation, block_num: block };
                    let ctx = RequestContext {
                        request_id: uuid::Uuid::new_v4(),
                        tenant_id: params.tenant_id,
                        timeline_id: params.timeline_id,
                    };
                    server.get_page_handler(page_key, params.lsn, ctx).await
                }
            ))
            .route("/health", axum::routing::get(
                |State(server): State<Arc<ApiServer>>| async move {
                    server.health_handler().await
                }
            ))
            .with_state(Arc::new(self))
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .layer(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(30)))
    }
}

// Implement sophisticated rate limiting
pub struct RateLimiter {
    // Per-tenant rate limits with sliding windows
    // Implement token bucket or sliding window counter
}

impl RateLimiter {
    async fn check_limit(&self, tenant_id: &TenantId) -> Result<(), ApiError> {
        // Your implementation: per-tenant rate limiting
        todo!("Implement rate limiting")
    }
}

#[derive(serde::Deserialize)]
pub struct PageQuery {
    pub tenant_id: TenantId,
    pub timeline_id: TimelineId,
    pub lsn: Option<Lsn>,
}

#[derive(serde::Serialize)]
pub struct PageResponse {
    pub page_key: PageKey,
    pub lsn: Lsn,
    pub data: Bytes,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub circuit_breaker_state: String,
    pub active_connections: usize,
    pub uptime: std::time::Duration,
}
```

### Performance Requirements

- Handle 100,000+ requests/second
- <1ms P95 latency
- Circuit breaker opens after 10 failures in 30 seconds
- Per-tenant rate limiting (1000 QPS default)
- Comprehensive health checks and metrics

---

## Day 5: Production-Ready Connection Management

### The Challenge

Build a connection pool that handles database failures, network partitions, and load spikes gracefully.

**Performance Target**: Maintain <5ms connection acquisition time under 10,000 concurrent requests

**File: `src/day5_connections.rs`**

```rust
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ConnectionManager {
    primary_pool: PgPool,
    replica_pools: Vec<PgPool>,
    circuit_breaker: Arc<DatabaseCircuitBreaker>,
    health_checker: Arc<HealthChecker>,
    metrics: ConnectionMetrics,
}

impl ConnectionManager {
    pub async fn execute_read<T>(&self, query: QueryBuilder<T>) -> Result<T, StorageError> {
        // Implement read-replica routing with fallback
        // 1. Try replica pools first (round-robin)
        // 2. Fall back to primary if replicas unavailable
        // 3. Implement proper retry logic with backoff
        
        todo!("Implement read routing with replica fallback")
    }
    
    pub async fn execute_write<T>(&self, query: QueryBuilder<T>) -> Result<T, StorageError> {
        // All writes go to primary with retry logic
        
        todo!("Implement write execution with retry")
    }
    
    pub fn health_status(&self) -> ConnectionHealth {
        // Return detailed connection pool health
        todo!("Implement comprehensive health reporting")
    }
}

// Advanced health checking with predictive failure detection
pub struct HealthChecker {
    // Implement sophisticated health monitoring:
    // - Connection latency trends
    // - Error rate monitoring  
    // - Predictive failure detection
    // - Automatic failover logic
}

impl HealthChecker {
    pub async fn continuous_health_monitoring(&self) -> ! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Check all connection pools
            for (pool_id, pool) in self.get_all_pools().iter() {
                match self.check_pool_health(pool).await {
                    Ok(health) => {
                        gauge!("database.pool_health_score", health.score, "pool" => pool_id);
                        
                        if health.score < 0.8 {
                            warn!("Pool {} health degrading: {:.2}", pool_id, health.score);
                        }
                    }
                    Err(e) => {
                        error!("Health check failed for pool {}: {}", pool_id, e);
                        // Trigger circuit breaker or failover
                    }
                }
            }
        }
    }
}
```

**Performance Requirements:**
- <5ms connection acquisition under load
- Automatic failover to replicas when primary fails
- Connection pool auto-scaling based on demand
- Predictive failure detection and alerting

---

## Final Integration: Production Load Testing

### Complete System Benchmark

**File: `src/main.rs`**

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize comprehensive observability
    tracing_subscriber::fmt()
        .with_env_filter("info,storage_engine=debug")
        .json()
        .init();
    
    // Start Prometheus metrics server
    let metrics_handle = tokio::spawn(async {
        let app = Router::new()
            .route("/metrics", axum::routing::get(|| async {
                let encoder = prometheus::TextEncoder::new();
                // Export metrics
            }));
        axum::Server::bind(&"0.0.0.0:9090".parse().unwrap())
            .serve(app.into_make_service())
            .await
    });
    
    // Initialize storage system
    let config = StorageConfig::from_env();
    let storage_system = StorageSystem::new(config).await?;
    
    // Start background tasks
    let compaction_handle = tokio::spawn(async move {
        storage_system.run_compaction_loop().await
    });
    
    let health_monitor = tokio::spawn(async move {
        storage_system.run_health_monitoring().await
    });
    
    // Start API server
    let api_server = ApiServer::new(storage_system);
    let app = api_server.create_router();
    
    println!("🚀 Storage engine starting on :8080");
    println!("📊 Metrics available on :9090/metrics");
    
    // Graceful shutdown handling
    let shutdown = shutdown_signal();
    
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await?;
    
    // Clean shutdown
    metrics_handle.abort();
    compaction_handle.abort();
    health_monitor.abort();
    
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    println!("Shutting down gracefully...");
}
```

### Load Testing Script

**scripts/load_test.sh:**
```bash
#!/bin/bash

echo "🔥 Production Load Test"

# Start monitoring stack
docker-compose up -d prometheus grafana

# Warm up
echo "Warming up system..."
bombardier -c 10 -n 1000 http://localhost:8080/v1/pages/1/1?tenant_id=test

# Load test scenarios
echo "Running load tests..."

# Scenario 1: Sustained throughput
echo "Test 1: Sustained throughput (60 seconds)"
bombardier -c 100 -d 60s http://localhost:8080/v1/pages/1/1?tenant_id=test

# Scenario 2: Spike test  
echo "Test 2: Traffic spike"
bombardier -c 1000 -d 30s http://localhost:8080/v1/pages/1/1?tenant_id=test

# Scenario 3: Mixed workload
echo "Test 3: Mixed workload with different page keys"
# Use custom script to generate varied requests

# Check results
echo "📊 Final Results:"
curl -s http://localhost:9090/api/v1/query?query=api_requests_total | jq
curl -s http://localhost:9090/api/v1/query?query=histogram_quantile(0.95,api_request_duration_ms) | jq
```

## Success Criteria

### Performance Benchmarks
- [ ] **Page Reads**: 100,000+ pages/second with <1ms P95 latency
- [ ] **WAL Processing**: 1GB/second throughput using <512MB RAM
- [ ] **Compaction**: 10GB in <5 minutes with <1% read impact
- [ ] **API Throughput**: 100,000 requests/second with 99.99% availability
- [ ] **Connection Management**: <5ms acquisition time under load

### Production Readiness
- [ ] Comprehensive metrics and alerting for all components
- [ ] Graceful handling of database failures and network partitions
- [ ] Automatic recovery from temporary failures
- [ ] Memory usage bounded under all load conditions
- [ ] Zero data loss during system failures

### Operational Excellence
- [ ] Complete runbooks for incident response
- [ ] Automated health checks with predictive failure detection
- [ ] Performance regression detection in CI/CD
- [ ] Load testing integrated into deployment pipeline
- [ ] Security audit passed (no secrets in logs, proper authentication)

**You've built a production-ready distributed storage system that can handle real database workloads at scale.**

---

## Next Steps

1. **Study the actual Neon codebase**: https://github.com/neondatabase/neon
2. **Explore advanced patterns**: Study pageserver implementation patterns
3. **Join the community**: Contribute to discussions and improvements
4. **Apply to Neon**: You now have demonstrable storage systems expertise

The patterns you've mastered - from race condition debugging to zero-copy optimization to distributed compaction - are the foundation of modern storage systems engineering.