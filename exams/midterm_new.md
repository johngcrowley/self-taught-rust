# High-Performance Data Pipeline Engineering

*Build production-ready data infrastructure with Rust*

## Overview

You're building the data platform that powers a high-growth SaaS company. Your pipeline must ingest, process, and serve data reliably at scale while maintaining strict SLA requirements.

**Success Criteria**: Process 10,000+ records/second with <10ms P95 latency and 99.9% uptime.

---

## Day 1: Database Connection Pool & Circuit Breaker

### The Problem

Your API is hitting database connection limits under load, causing cascade failures. Build a resilient connection management system.

**Setup:**
```bash
cargo new --bin data-pipeline
cd data-pipeline
```

**Cargo.toml:**
```toml
[package]
name = "data-pipeline"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono"] }
axum = "0.7"
uuid = { version = "1.6", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio-metrics = "0.3"
```

### Your Task

Implement a production-ready database layer:

**File: `src/day1_database.rs`**

```rust
use sqlx::{PgPool, Row};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("Circuit breaker open - too many failures")]
    CircuitOpen,
    #[error("Operation timeout")]
    Timeout,
}

pub struct DatabaseConfig {
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: Duration,
}

pub struct Database {
    pool: PgPool,
    circuit_breaker: Arc<CircuitBreaker>,
}

struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: std::sync::Mutex<Option<Instant>>,
    threshold: u32,
    timeout: Duration,
}

impl Database {
    pub async fn new(database_url: &str, config: DatabaseConfig) -> Result<Self, DatabaseError> {
        // Your implementation:
        // 1. Configure sqlx PgPool with connection limits
        // 2. Set up connection timeouts and idle timeouts  
        // 3. Initialize circuit breaker
        // 4. Test initial connection
        
        todo!("Implement production database connection")
    }
    
    pub async fn get_user(&self, user_id: uuid::Uuid) -> Result<User, DatabaseError> {
        // Your implementation:
        // 1. Check circuit breaker state
        // 2. Execute query with timeout
        // 3. Handle connection errors
        // 4. Update circuit breaker on success/failure
        
        todo!("Implement resilient database query")
    }
    
    pub async fn create_user(&self, user: CreateUser) -> Result<User, DatabaseError> {
        // Similar pattern for writes
        todo!("Implement resilient database write")
    }
    
    pub fn health_check(&self) -> DatabaseHealth {
        // Return connection pool stats and circuit breaker state
        todo!("Implement health check")
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize)]
pub struct CreateUser {
    pub email: String,
}

pub struct DatabaseHealth {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub circuit_breaker_open: bool,
    pub recent_failures: u32,
}
```

### Performance Requirements

- Handle 1000 concurrent requests without connection exhaustion
- Circuit breaker opens after 5 consecutive failures in 30 seconds
- Connection pool should auto-scale from 5-50 connections based on load
- All queries timeout after 5 seconds maximum
- Health endpoint shows real-time connection metrics

### Testing Your Implementation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_pool_limits() {
        // Spawn 100 concurrent queries, ensure no connection exhaustion
    }
    
    #[tokio::test] 
    async fn test_circuit_breaker() {
        // Simulate database failures, verify circuit opens
    }
    
    #[tokio::test]
    async fn test_connection_recovery() {
        // Verify system recovers after database comes back online
    }
}
```

---

## Day 2: Stream Processing with Backpressure

### The Problem

Your Kafka consumers are falling behind during traffic spikes, causing data freshness SLA violations. Implement intelligent backpressure handling.

**Additional Dependencies:**
```toml
rdkafka = { version = "0.36", features = ["cmake-build"] }
tokio-stream = "0.1"
futures = "0.3"
```

### Your Task

**File: `src/day2_streaming.rs`**

```rust
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message,
};
use tokio_stream::StreamExt;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};

pub struct StreamProcessor {
    consumer: StreamConsumer,
    database: Arc<Database>,
    batch_size: usize,
    max_concurrent_batches: usize,
}

pub struct ProcessingMetrics {
    pub messages_processed: u64,
    pub processing_lag_ms: u64,
    pub current_batch_size: usize,
    pub active_processors: usize,
}

impl StreamProcessor {
    pub async fn new(kafka_brokers: &str, topic: &str, database: Arc<Database>) -> Self {
        // Your implementation:
        // 1. Configure Kafka consumer with proper settings
        // 2. Set up consumer group for horizontal scaling
        // 3. Configure batch processing parameters
        // 4. Initialize backpressure controls
        
        todo!("Initialize Kafka stream processor")
    }
    
    pub async fn start_processing(&self) -> Result<(), ProcessingError> {
        // Your implementation:
        // 1. Consume messages in batches
        // 2. Implement adaptive batch sizing based on processing lag
        // 3. Use semaphore to limit concurrent batch processing
        // 4. Handle poison messages without stopping pipeline
        // 5. Commit offsets only after successful processing
        
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_batches));
        let mut message_stream = self.consumer.stream();
        
        while let Some(message_result) = message_stream.next().await {
            // Your batching and backpressure logic here
            todo!("Implement message processing with backpressure")
        }
        
        Ok(())
    }
    
    pub fn metrics(&self) -> ProcessingMetrics {
        // Return real-time processing metrics
        todo!("Implement metrics collection")
    }
}

#[derive(serde::Deserialize)]
pub struct UserEvent {
    pub user_id: uuid::Uuid,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub properties: serde_json::Value,
}
```

### Performance Requirements

- Process 10,000 messages/second sustained throughput
- Maintain processing lag below 1 second during normal operation  
- Gracefully handle traffic spikes up to 50,000 messages/second
- Recover automatically from downstream database slowness
- Zero message loss even during application restarts

### Adaptive Batching Strategy

Your processor should dynamically adjust batch sizes:
- Small lag (< 100ms): Use small batches (10-50 messages) for low latency
- Medium lag (100ms-1s): Increase batch size (100-500 messages) for throughput  
- High lag (> 1s): Use large batches (1000+ messages) and shed non-critical processing

---

## Day 3: Memory-Efficient Data Processing 

### The Problem

Your ETL jobs are consuming unbounded memory when processing large datasets, causing OOM kills in production.

### Your Task

**File: `src/day3_processing.rs`**

```rust
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

pub struct DataProcessor {
    max_memory_usage: usize,
    current_memory: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

pub struct ProcessingJob {
    pub input_source: String,  // File path or S3 URL
    pub output_destination: String,
    pub transformation: TransformationType,
}

pub enum TransformationType {
    JsonToParquet,
    CsvAggregation { group_by: Vec<String> },
    DataValidation { schema: ValidationSchema },
}

impl DataProcessor {
    pub async fn process_stream<R: AsyncBufRead + Unpin>(
        &self,
        reader: R,
        job: ProcessingJob,
    ) -> Result<ProcessingStats, ProcessingError> {
        // Your implementation:
        // 1. Stream process without loading entire dataset in memory
        // 2. Use bounded channels for backpressure
        // 3. Implement memory monitoring and adaptive processing
        // 4. Handle schema evolution and data quality issues
        // 5. Provide streaming output (don't buffer results)
        
        todo!("Implement memory-bounded stream processing")
    }
    
    pub async fn aggregate_data(
        &self,
        input_stream: impl tokio_stream::Stream<Item = Record>,
        group_by: Vec<String>,
    ) -> Result<HashMap<String, AggregateValue>, ProcessingError> {
        // Challenge: Aggregate potentially unlimited data with bounded memory
        // Hint: Consider using external sorting or streaming aggregation
        
        todo!("Implement memory-efficient aggregation")
    }
}

pub struct ProcessingStats {
    pub records_processed: u64,
    pub records_failed: u64,
    pub peak_memory_usage: usize,
    pub processing_time: std::time::Duration,
}
```

### Memory Management Requirements

- Process 1GB+ files using maximum 512MB RAM
- Implement streaming aggregation for group-by operations
- Handle backpressure when downstream systems are slow
- Provide real-time memory usage monitoring
- Gracefully handle OOM conditions without data loss

---

## Day 4: Production Observability

### The Problem

You have no visibility into your pipeline performance during incidents. Build comprehensive monitoring.

**Additional Dependencies:**
```toml
metrics = "0.21"
metrics-prometheus = "0.6" 
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
opentelemetry = "0.20"
```

### Your Task

**File: `src/day4_observability.rs`**

```rust
use metrics::{counter, histogram, gauge};
use tracing::{info, error, warn, instrument};
use std::sync::Arc;
use std::time::Instant;

pub struct ObservabilityStack {
    pub metrics_server: MetricsServer,
    pub trace_exporter: TraceExporter,
}

impl ObservabilityStack {
    pub fn init() -> Self {
        // Your implementation:
        // 1. Set up structured logging with JSON output
        // 2. Initialize Prometheus metrics endpoint
        // 3. Configure distributed tracing
        // 4. Set up health check endpoints
        
        todo!("Initialize observability stack")
    }
}

// Instrument your existing functions with comprehensive observability
impl Database {
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_user(&self, user_id: uuid::Uuid) -> Result<User, DatabaseError> {
        let start = Instant::now();
        
        // Update metrics
        counter!("database_queries_total", 1, "operation" => "get_user");
        
        let result = match self.execute_query(user_id).await {
            Ok(user) => {
                info!("User retrieved successfully");
                histogram!("database_query_duration", start.elapsed());
                Ok(user)
            }
            Err(e) => {
                error!("Failed to retrieve user: {}", e);
                counter!("database_errors_total", 1, "operation" => "get_user", "error_type" => "query_failed");
                Err(e)
            }
        };
        
        // Update connection pool metrics
        gauge!("database_active_connections", self.pool.num_active() as f64);
        gauge!("database_idle_connections", self.pool.num_idle() as f64);
        
        result
    }
}

pub struct SystemMetrics {
    pub database_connections: ConnectionPoolMetrics,
    pub kafka_consumer: ConsumerMetrics, 
    pub processing_pipeline: PipelineMetrics,
    pub memory_usage: MemoryMetrics,
}

pub struct AlertRule {
    pub name: String,
    pub condition: String,  // PromQL query
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub runbook_url: String,
}
```

### Observability Requirements

- Comprehensive metrics for all system components
- Structured JSON logging with correlation IDs
- Distributed tracing across service boundaries  
- Real-time dashboards showing SLA compliance
- Automated alerting for SLA violations
- Runbooks for common failure scenarios

### Key Metrics to Track

```rust
// Throughput metrics
counter!("messages_processed_total");
counter!("database_queries_total");
counter!("http_requests_total");

// Latency metrics  
histogram!("request_duration_seconds");
histogram!("database_query_duration_seconds");
histogram!("kafka_processing_latency_seconds");

// Error metrics
counter!("errors_total");
gauge!("circuit_breaker_state");

// Resource metrics
gauge!("memory_usage_bytes");
gauge!("active_database_connections");
gauge!("kafka_consumer_lag");
```

---

## Day 5: Production Debugging Scenarios

### The Problem

Your system is experiencing mysterious performance degradation in production. Practice systematic debugging.

### Scenario 1: Memory Leak Hunt

Your service memory usage grows unbounded over 24 hours. Debug tools:

```rust
use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOCATOR: DhatAlloc = DhatAlloc;

fn main() {
    let _dhat = Dhat::start_heap_profiling();
    // Your application code
}
```

### Scenario 2: Database Connection Exhaustion  

Database queries start timing out randomly during peak hours.

**Investigation checklist:**
1. Connection pool configuration
2. Long-running transactions  
3. Connection leak detection
4. Query performance analysis
5. Lock contention issues

### Scenario 3: Kafka Consumer Lag

Your stream processor falls behind during traffic spikes.

**Debug approach:**
1. Consumer group rebalancing issues
2. Processing time per message
3. Downstream bottlenecks
4. Memory pressure causing GC pauses
5. Network partition tolerance

### Your Task

Create comprehensive debugging runbooks and automated detection:

```rust
pub struct HealthChecker {
    database: Arc<Database>,
    kafka_processor: Arc<StreamProcessor>, 
}

impl HealthChecker {
    pub async fn comprehensive_health_check(&self) -> SystemHealth {
        // Your implementation:
        // 1. Check all external dependencies
        // 2. Validate SLA compliance
        // 3. Detect resource exhaustion early
        // 4. Provide actionable diagnostics
        
        todo!("Implement comprehensive health checking")
    }
    
    pub async fn detect_anomalies(&self) -> Vec<Anomaly> {
        // Proactive anomaly detection before SLA violations
        todo!("Implement anomaly detection")
    }
}
```

---

## Final Integration: Load Testing

### Your Complete System Test

```bash
# Start your pipeline
cargo run --release

# Run load test
./load_test.sh
```

**load_test.sh:**
```bash
#!/bin/bash

echo "🚀 Production Load Test"

# Start monitoring
docker run -d -p 3000:3000 grafana/grafana
docker run -d -p 9090:9090 prom/prometheus

# Generate realistic load
# - 10,000 HTTP requests/second
# - 50,000 Kafka messages/second  
# - Database queries with realistic patterns
# - Simulate traffic spikes and failures

artillery run --config load-test.yml

echo "📊 Results:"
echo "Throughput: $(get_throughput_metrics)"
echo "P95 Latency: $(get_latency_metrics)" 
echo "Error Rate: $(get_error_rate)"
echo "Memory Usage: $(get_memory_usage)"
```

## Success Criteria

### Performance Benchmarks
- [ ] **Throughput**: 10,000+ records/second sustained
- [ ] **Latency**: <10ms P95 for database queries  
- [ ] **Memory**: Process 1GB files using <512MB RAM
- [ ] **Reliability**: 99.9% success rate over 24 hours
- [ ] **Recovery**: <30 second recovery from failures

### Production Readiness
- [ ] Comprehensive monitoring and alerting
- [ ] Automated health checks and circuit breakers
- [ ] Memory leak detection and prevention
- [ ] Graceful shutdown and zero-downtime deployments
- [ ] Complete debugging runbooks

### Code Quality
- [ ] Production error handling patterns
- [ ] Comprehensive testing including chaos engineering
- [ ] Performance benchmarks and regression detection
- [ ] Security best practices (no secrets in logs)

**You've built a production-ready data platform that can handle real SRE challenges.**