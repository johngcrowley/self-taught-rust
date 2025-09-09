# Rust Performance Observatory: Local K3s Capstone Project

*Build, deploy, measure, and optimize a complete Rust streaming data pipeline with real performance insights.*

---

## Mission Statement

Create a local Kubernetes-native performance laboratory where you can deploy Rust microservices, generate realistic data streaming workloads, and extract deep performance insights. This capstone synthesizes everything you've learned into a production-like environment that answers the question: **"How performant is my Rust code, really?"**

**Goal**: Deploy a complete streaming data pipeline to local k3s, orchestrate it with a custom operator, and build comprehensive tooling to measure, profile, and optimize every aspect of your Rust performance.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    K3s Local Cluster                    │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐               │
│  │   Data Stream   │  │  Performance    │               │
│  │   Generator     │  │  Collector      │               │
│  │                 │  │                 │               │
│  │  • Market Data  │  │  • CPU Profiling│               │
│  │  • IoT Sensors  │  │  • Memory Usage │               │
│  │  • Log Events   │  │  • Latency      │               │
│  └─────────────────┘  └─────────────────┘               │
│           │                      ▲                      │
│           ▼                      │                      │
│  ┌─────────────────┐  ┌─────────────────┐               │
│  │   Processing    │  │   PostgreSQL    │               │
│  │   Pipeline      │  │   Database      │               │
│  │                 │  │                 │               │
│  │  • Zero-copy    │  │  • Time-series  │               │
│  │  • Async/Await  │  │  • Metrics      │               │
│  │  • Error Chain  │  │  • Results      │               │
│  └─────────────────┘  └─────────────────┘               │
│           │                                             │
│           ▼                                             │
│  ┌─────────────────┐  ┌─────────────────┐               │
│  │    API Layer    │  │  Rust Operator  │               │
│  │                 │  │                 │               │
│  │  • REST/GraphQL │  │  • Pipeline CRD │               │
│  │  • Real-time WS │  │  • Auto-scaling │               │
│  │  • Metrics      │  │  • Health Mgmt  │               │
│  └─────────────────┘  └─────────────────┘               │
└─────────────────────────────────────────────────────────┘
           │                      ▲
           ▼                      │
┌─────────────────────────────────────────────────────────┐
│              Performance Observatory                     │
├─────────────────────────────────────────────────────────┤
│  • Flamegraphs        • Memory Allocation Maps          │
│  • Latency Histograms • Throughput Benchmarks           │
│  • CPU Usage          • Network I/O Patterns            │
│  • Comparative Tests  • Regression Detection            │
└─────────────────────────────────────────────────────────┘
```

---

## Phase 1: Local K3s Performance Lab Setup (Day 1)

### Environment Bootstrap

Create a production-like local environment optimized for performance measurement:

```bash
#!/bin/bash
# setup-performance-lab.sh

# Install k3s with performance monitoring
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="--disable traefik --disable metrics-server" sh -s -

# Install performance tooling
kubectl apply -f https://github.com/prometheus-operator/prometheus-operator/releases/latest/download/bundle.yaml

# Setup local registry for fast iteration
k3s ctr images pull registry:2
kubectl create deployment registry --image=registry:2 --port=5000
kubectl expose deployment registry --port=5000 --target-port=5000

# Install Grafana for visualization
helm install grafana grafana/grafana \
  --set adminPassword=performance \
  --set service.type=NodePort \
  --set service.nodePort=30001

# Install PostgreSQL with performance extensions
helm install postgresql bitnami/postgresql \
  --set auth.postgresPassword=performance \
  --set primary.extendedConfiguration="shared_preload_libraries = 'pg_stat_statements'" \
  --set metrics.enabled=true
```

### Performance Monitoring Stack

```yaml
# performance-monitoring.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
data:
  prometheus.yml: |
    global:
      scrape_interval: 1s  # High-frequency for detailed performance data
    scrape_configs:
    - job_name: 'rust-services'
      static_configs:
      - targets: ['rust-api:8080', 'rust-processor:8081']
      metrics_path: /metrics
      scrape_interval: 100ms  # Sub-second metrics for performance analysis

---
apiVersion: v1  
kind: ConfigMap
metadata:
  name: grafana-dashboards
data:
  rust-performance.json: |
    {
      "dashboard": {
        "title": "Rust Performance Observatory",
        "panels": [
          {
            "title": "Request Latency Distribution",
            "type": "histogram",
            "targets": [{"expr": "histogram_quantile(0.95, rust_http_request_duration_seconds_bucket)"}]
          },
          {
            "title": "Memory Allocation Rate", 
            "type": "graph",
            "targets": [{"expr": "rate(rust_memory_allocations_total[1m])"}]
          },
          {
            "title": "CPU Usage by Function",
            "type": "flamegraph", 
            "targets": [{"expr": "rust_cpu_profile_samples"}]
          }
        ]
      }
    }
```

---

## Phase 2: Rust Service Architecture (Day 2-3)

### Data Stream Generator

Build a realistic data generator that creates various workload patterns:

```toml
# Cargo.toml
[package]
name = "performance-observatory"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["postgres", "chrono", "uuid"] }
axum = "0.7"
prometheus = "0.13"
tracing = "0.1"
criterion = { version = "0.5", features = ["html_reports"] }
pprof = { version = "0.13", features = ["flamegraph", "protobuf-codec"] }
bytes = "1.5"
dashmap = "5.5"
uuid = { version = "1.6", features = ["v4"] }

# Performance profiling
[profile.release]
debug = true  # Enable symbols for profiling in release mode

[[bin]]
name = "data-generator"
path = "src/data_generator.rs"

[[bin]] 
name = "stream-processor"
path = "src/stream_processor.rs"

[[bin]]
name = "performance-api"  
path = "src/performance_api.rs"

[[bin]]
name = "pipeline-operator"
path = "src/pipeline_operator.rs"
```

```rust
// src/data_generator.rs - Realistic streaming data patterns
use tokio::time::{Duration, interval, sleep};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use prometheus::{Counter, Histogram, register_counter, register_histogram};

#[derive(Serialize, Deserialize, Clone)]
struct MarketDataEvent {
    symbol: String,
    price: f64,
    volume: u64,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<String>, // Variable size payload
}

#[derive(Serialize, Deserialize, Clone)]
struct IoTSensorEvent {
    device_id: String,
    temperature: f32,
    humidity: f32,
    readings: Vec<f64>, // Variable length array
    raw_data: Vec<u8>,  // Simulates binary sensor data
}

struct DataGeneratorConfig {
    events_per_second: u64,
    burst_probability: f64,    // Chance of traffic spike
    burst_multiplier: u64,     // How much traffic increases
    event_size_range: (usize, usize), // Min/max event size
    patterns: Vec<DataPattern>,
}

enum DataPattern {
    Steady { rate: u64 },
    Periodic { base_rate: u64, period_seconds: u64, amplitude: f64 },
    Bursty { base_rate: u64, burst_duration: Duration, quiet_duration: Duration },
    Chaos { min_rate: u64, max_rate: u64 }, // Random load testing
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize performance monitoring
    let events_generated = register_counter!("data_generator_events_total", "Total events generated")?;
    let generation_duration = register_histogram!("data_generator_duration_seconds", "Time to generate events")?;
    
    let config = DataGeneratorConfig {
        events_per_second: std::env::var("EVENTS_PER_SEC")?.parse().unwrap_or(1000),
        burst_probability: 0.1,
        burst_multiplier: 10,
        event_size_range: (100, 10_000),
        patterns: vec![
            DataPattern::Steady { rate: 500 },
            DataPattern::Periodic { base_rate: 200, period_seconds: 60, amplitude: 2.0 },
            DataPattern::Bursty { 
                base_rate: 100, 
                burst_duration: Duration::from_secs(10),
                quiet_duration: Duration::from_secs(50)
            },
        ],
    };
    
    // Start HTTP server for metrics
    tokio::spawn(start_metrics_server());
    
    // Generate data streams
    generate_data_streams(config, events_generated, generation_duration).await?;
    
    Ok(())
}

async fn generate_data_streams(
    config: DataGeneratorConfig,
    events_counter: Counter,
    duration_histogram: Histogram,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval_timer = interval(Duration::from_millis(1000 / config.events_per_second));
    let mut rng = rand::thread_rng();
    
    loop {
        interval_timer.tick().await;
        
        let start = std::time::Instant::now();
        
        // Generate different types of events
        let event = match rng.gen_range(0..3) {
            0 => generate_market_data(&config, &mut rng),
            1 => generate_iot_data(&config, &mut rng),
            _ => generate_log_event(&config, &mut rng),
        };
        
        // Send to stream processor
        send_to_processor(event).await?;
        
        events_counter.inc();
        duration_histogram.observe(start.elapsed().as_secs_f64());
        
        // Implement burst patterns
        if rng.gen::<f64>() < config.burst_probability {
            for _ in 0..config.burst_multiplier {
                let burst_event = generate_market_data(&config, &mut rng);
                send_to_processor(burst_event).await?;
                events_counter.inc();
            }
        }
    }
}
```

### Zero-Copy Stream Processor  

```rust
// src/stream_processor.rs - High-performance event processing
use bytes::Bytes;
use serde::Deserialize;
use tokio_stream::{Stream, StreamExt};
use std::sync::Arc;
use prometheus::{Histogram, Counter, register_histogram, register_counter};
use std::time::Instant;

// Zero-copy event processing with lifetime management
#[derive(Deserialize)]
struct EventBatch<'a> {
    #[serde(borrow)]
    events: Vec<RawEvent<'a>>,
    batch_id: u64,
    timestamp: u64,
}

#[derive(Deserialize)]  
struct RawEvent<'a> {
    #[serde(borrow)]
    event_type: &'a str,
    #[serde(borrow)]
    payload: &'a [u8],
    #[serde(borrow)]
    metadata: Option<&'a str>,
}

struct ProcessingMetrics {
    events_processed: Counter,
    processing_duration: Histogram,
    memory_allocations: Counter,
    zero_copy_ratio: Histogram,
}

impl ProcessingMetrics {
    fn new() -> Result<Self, prometheus::Error> {
        Ok(Self {
            events_processed: register_counter!("stream_processor_events_total", "Events processed")?,
            processing_duration: register_histogram!(
                "stream_processor_duration_seconds", 
                "Event processing duration",
                vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]
            )?,
            memory_allocations: register_counter!("stream_processor_allocations_total", "Memory allocations")?,
            zero_copy_ratio: register_histogram!(
                "stream_processor_zero_copy_ratio",
                "Ratio of zero-copy vs copied data"
            )?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = Arc::new(ProcessingMetrics::new()?);
    
    // Setup processing pipeline with backpressure
    let processor = StreamProcessor::new(ProcessorConfig {
        max_concurrent_batches: 10,
        batch_size: 1000,
        processing_timeout: Duration::from_millis(100),
        enable_zero_copy: true,
    });
    
    tokio::spawn(start_metrics_server());
    tokio::spawn(start_performance_profiler()); // CPU/memory profiling
    
    // Process incoming data streams
    processor.run(metrics).await?;
    
    Ok(())
}

struct StreamProcessor {
    config: ProcessorConfig,
    memory_pool: Arc<MemoryPool>, // For zero-copy processing
}

impl StreamProcessor {
    async fn process_batch<'a>(&self, 
        batch: EventBatch<'a>,
        metrics: Arc<ProcessingMetrics>
    ) -> Result<ProcessingResults, ProcessingError> {
        let start = Instant::now();
        
        // Zero-copy processing pipeline
        let results = batch.events
            .iter()
            .map(|event| self.process_event_zero_copy(event))
            .collect::<Result<Vec<_>, _>>()?;
        
        // Calculate zero-copy efficiency
        let zero_copy_bytes = calculate_zero_copy_bytes(&batch);
        let total_bytes = calculate_total_bytes(&batch);
        let zero_copy_ratio = zero_copy_bytes as f64 / total_bytes as f64;
        
        // Record metrics
        metrics.events_processed.inc_by(batch.events.len() as u64);
        metrics.processing_duration.observe(start.elapsed().as_secs_f64());
        metrics.zero_copy_ratio.observe(zero_copy_ratio);
        
        Ok(ProcessingResults {
            processed_count: results.len(),
            zero_copy_ratio,
            processing_time: start.elapsed(),
            memory_used: self.memory_pool.current_usage(),
        })
    }
    
    fn process_event_zero_copy<'a>(&self, event: &RawEvent<'a>) -> Result<ProcessedEvent<'a>, ProcessingError> {
        // Process without copying data - use references to original buffer
        match event.event_type {
            "market_data" => self.process_market_data_zero_copy(event.payload),
            "iot_sensor" => self.process_iot_data_zero_copy(event.payload),
            "log_event" => self.process_log_zero_copy(event.payload),
            _ => Err(ProcessingError::UnknownEventType),
        }
    }
}

// Performance profiling integration
async fn start_performance_profiler() -> Result<(), Box<dyn std::error::Error>> {
    use pprof::ProfilerGuard;
    
    let guard = ProfilerGuard::new(100)?; // Sample at 100Hz
    
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // Generate CPU flamegraph
        let report = guard.report().build()?;
        let file = std::fs::File::create("flamegraph.svg")?;
        report.flamegraph(file)?;
        
        // Collect memory allocation data
        collect_memory_profile().await?;
        
        println!("Performance profile updated");
    }
}
```

### Performance-Focused API Layer

```rust  
// src/performance_api.rs - API with deep performance insights
use axum::{extract::State, response::Json, routing::get, Router};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::Instant;

#[derive(Serialize)]
struct PerformanceReport {
    throughput: ThroughputMetrics,
    latency: LatencyMetrics,
    memory: MemoryMetrics,
    cpu: CpuMetrics,
    comparison: Option<ComparisonResults>,
}

#[derive(Serialize)]
struct ThroughputMetrics {
    events_per_second: f64,
    bytes_per_second: u64,
    peak_throughput: f64,
    sustained_throughput: f64,
}

#[derive(Serialize)]
struct LatencyMetrics {
    p50_ms: f64,
    p95_ms: f64, 
    p99_ms: f64,
    p999_ms: f64,
    max_ms: f64,
}

#[derive(Serialize)]
struct MemoryMetrics {
    heap_allocated: u64,
    heap_peak: u64,
    zero_copy_ratio: f64,
    allocation_rate: f64, // allocations per second
    gc_pressure: f64,     // fragmentation metric
}

#[derive(Serialize)]  
struct CpuMetrics {
    cpu_utilization: f64,
    instructions_per_cycle: f64,
    cache_hit_ratio: f64,
    context_switches: u64,
    hot_functions: Vec<HotFunction>,
}

#[derive(Serialize)]
struct HotFunction {
    name: String,
    cpu_percentage: f64,
    call_count: u64,
    avg_duration_ns: u64,
}

// Performance comparison system
async fn compare_implementations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ComparisonResults>, ApiError> {
    let baseline = run_baseline_benchmark().await?;
    let optimized = run_optimized_benchmark().await?;
    
    let comparison = ComparisonResults {
        throughput_improvement: (optimized.throughput / baseline.throughput - 1.0) * 100.0,
        latency_improvement: (1.0 - optimized.latency / baseline.latency) * 100.0,
        memory_efficiency: (1.0 - optimized.memory_usage / baseline.memory_usage) * 100.0,
        cpu_efficiency: (1.0 - optimized.cpu_usage / baseline.cpu_usage) * 100.0,
        recommendation: generate_optimization_recommendation(&baseline, &optimized),
    };
    
    Ok(Json(comparison))
}

// Real-time performance streaming endpoint
async fn performance_stream(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            
            let current_metrics = collect_realtime_metrics().await;
            let message = serde_json::to_string(&current_metrics).unwrap();
            
            if socket.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
    })
}
```

---

## Phase 3: Custom Pipeline Operator (Day 4)

### Kubernetes Operator for Pipeline Orchestration

```rust
// src/pipeline_operator.rs - Custom operator for performance orchestration
use kube::{Api, Client, CustomResource, Resource, ResourceExt};
use kube::runtime::{watcher, WatchStreamExt, Controller};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "performance.lab", version = "v1", kind = "PipelineBenchmark")]
#[kube(namespaced)]
struct PipelineBenchmarkSpec {
    // Workload configuration
    data_sources: Vec<DataSourceConfig>,
    processing_stages: Vec<ProcessingStageConfig>,
    performance_targets: PerformanceTargets,
    
    // Testing parameters
    duration_minutes: u32,
    concurrent_streams: u32,
    comparison_baseline: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
struct PerformanceTargets {
    min_throughput_eps: u64, // events per second
    max_p99_latency_ms: f64,
    max_memory_mb: u64,
    max_cpu_cores: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]  
struct DataSourceConfig {
    name: String,
    data_type: String, // "market_data", "iot_sensor", "log_event"
    generation_rate: u64,
    size_distribution: SizeDistribution,
}

async fn reconcile_pipeline(
    benchmark: Arc<PipelineBenchmark>,
    ctx: Arc<Context>,
) -> Result<Action, Error> {
    let client = ctx.client.clone();
    let name = benchmark.name_any();
    
    println!("🔄 Reconciling pipeline benchmark: {}", name);
    
    // Step 1: Deploy data generators
    deploy_data_generators(&client, &benchmark).await?;
    
    // Step 2: Deploy stream processors  
    deploy_stream_processors(&client, &benchmark).await?;
    
    // Step 3: Deploy performance collectors
    deploy_performance_collectors(&client, &benchmark).await?;
    
    // Step 4: Start benchmark execution
    if should_start_benchmark(&benchmark).await? {
        execute_benchmark(&client, &benchmark).await?;
    }
    
    // Step 5: Collect and analyze results
    if is_benchmark_complete(&benchmark).await? {
        analyze_performance_results(&client, &benchmark).await?;
    }
    
    Ok(Action::requeue(Duration::from_secs(30)))
}

async fn deploy_data_generators(
    client: &Client,
    benchmark: &PipelineBenchmark,
) -> Result<(), Error> {
    let jobs: Api<Job> = Api::namespaced(client.clone(), &benchmark.namespace().unwrap());
    
    for source in &benchmark.spec.data_sources {
        let job = create_data_generator_job(benchmark, source)?;
        jobs.create(&PostParams::default(), &job).await?;
        println!("✅ Deployed data generator: {}", source.name);
    }
    
    Ok(())
}

// Performance analysis and reporting
async fn analyze_performance_results(
    client: &Client,
    benchmark: &PipelineBenchmark,
) -> Result<(), Error> {
    println!("📊 Analyzing performance results for: {}", benchmark.name_any());
    
    // Collect metrics from all components
    let metrics = collect_comprehensive_metrics(client, benchmark).await?;
    
    // Generate performance report
    let report = PerformanceReport {
        benchmark_name: benchmark.name_any(),
        execution_time: metrics.total_duration,
        throughput: calculate_throughput(&metrics),
        latency_distribution: calculate_latency_distribution(&metrics),
        resource_utilization: calculate_resource_usage(&metrics),
        bottleneck_analysis: identify_bottlenecks(&metrics),
        optimization_recommendations: generate_recommendations(&metrics),
    };
    
    // Store results in PostgreSQL for comparison
    store_benchmark_results(&report).await?;
    
    // Update benchmark status
    update_benchmark_status(client, benchmark, BenchmarkPhase::Completed, Some(report)).await?;
    
    println!("✅ Performance analysis complete");
    Ok(())
}
```

---

## Phase 4: Performance Observatory Dashboard (Day 5)

### Comprehensive Performance Tooling

```yaml
# k8s/performance-dashboard.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: performance-dashboard
data:
  dashboard.json: |
    {
      "title": "Rust Performance Observatory",
      "panels": [
        {
          "title": "Throughput Comparison",
          "type": "stat",
          "targets": [
            {"expr": "rate(events_processed_total[5m])", "legendFormat": "Current"},
            {"expr": "rust_benchmark_baseline_throughput", "legendFormat": "Baseline"}
          ],
          "fieldConfig": {
            "custom": {
              "displayMode": "basic",
              "colorMode": "background"
            },
            "overrides": [
              {
                "matcher": {"id": "byName", "options": "Current"},
                "properties": [{"id": "color", "value": {"mode": "continuous-GrYlRd"}}]
              }
            ]
          }
        },
        {
          "title": "Memory Allocation Flamegraph", 
          "type": "flamegraph",
          "targets": [{"expr": "rust_memory_profile_samples"}],
          "options": {
            "displayMode": "table",
            "showTooltip": true,
            "sortOrder": "desc"
          }
        },
        {
          "title": "Latency Heatmap",
          "type": "heatmap", 
          "targets": [{"expr": "rust_request_duration_seconds_bucket"}],
          "heatmap": {
            "yAxis": {"unit": "ms", "logBase": 2}
          }
        },
        {
          "title": "Zero-Copy Efficiency",
          "type": "graph",
          "targets": [
            {"expr": "rust_zero_copy_bytes_rate / rust_total_bytes_rate", "legendFormat": "Zero-Copy Ratio"},
            {"expr": "rate(rust_memory_allocations_total[1m])", "legendFormat": "Allocation Rate"}
          ]
        },
        {
          "title": "Performance Regression Detection",
          "type": "table",
          "targets": [
            {"expr": "rust_benchmark_performance_score", "format": "table"},
            {"expr": "rust_benchmark_regression_detected", "format": "table"}
          ]
        }
      ]
    }

---
apiVersion: batch/v1
kind: CronJob
metadata:
  name: performance-benchmark
spec:
  schedule: "0 */6 * * *"  # Run every 6 hours
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: benchmark-runner
            image: localhost:5000/rust-performance:latest
            command: ["./benchmark-suite"]
            args: ["--compare-baseline", "--generate-report", "--detect-regressions"]
            resources:
              requests:
                cpu: "2"
                memory: "4Gi"
              limits:
                cpu: "4" 
                memory: "8Gi"
```

### Automated Performance Testing

```bash
#!/bin/bash
# scripts/run-performance-suite.sh

echo "🚀 Starting Rust Performance Observatory Test Suite"

# Build optimized containers
echo "📦 Building performance-optimized containers..."
docker build -f docker/Dockerfile.data-generator -t localhost:5000/data-generator:latest .
docker build -f docker/Dockerfile.stream-processor -t localhost:5000/stream-processor:latest .
docker build -f docker/Dockerfile.performance-api -t localhost:5000/performance-api:latest .

# Deploy complete stack
echo "🌟 Deploying to k3s cluster..."
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/postgresql.yaml
kubectl apply -f k8s/data-generator.yaml  
kubectl apply -f k8s/stream-processor.yaml
kubectl apply -f k8s/performance-api.yaml
kubectl apply -f k8s/pipeline-operator.yaml

# Wait for deployments
echo "⏳ Waiting for services to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment --all

# Create benchmark workloads
echo "📊 Creating performance benchmarks..."
kubectl apply -f - <<EOF
apiVersion: performance.lab/v1
kind: PipelineBenchmark
metadata:
  name: baseline-performance-test
spec:
  duration_minutes: 10
  concurrent_streams: 5
  data_sources:
  - name: market-data-source
    data_type: market_data
    generation_rate: 1000
    size_distribution:
      min_bytes: 256
      max_bytes: 4096
      distribution: normal
  processing_stages:
  - name: zero-copy-processor  
    type: stream_processor
    config:
      enable_zero_copy: true
      batch_size: 1000
  performance_targets:
    min_throughput_eps: 5000
    max_p99_latency_ms: 50.0
    max_memory_mb: 512
    max_cpu_cores: 2.0
EOF

# Monitor benchmark execution
echo "👀 Monitoring benchmark progress..."
kubectl logs -f deployment/pipeline-operator &

# Wait for completion
kubectl wait --for=condition=Complete --timeout=900s pipelinebenchmark/baseline-performance-test

# Collect results
echo "📈 Collecting performance results..."
kubectl port-forward service/performance-api 8080:80 &
curl -s http://localhost:8080/api/v1/benchmark/baseline-performance-test/report | jq '.'

# Generate comparative analysis
echo "🔍 Running performance comparison..."
./scripts/generate-performance-comparison.sh

echo "✅ Performance Observatory Test Suite Complete!"
echo "📊 View results at: http://localhost:30001 (Grafana: admin/performance)"
echo "🔬 API available at: http://localhost:8080/api/v1/performance"
```

---

## Performance Measurement Toolkit

### Multi-Layered Performance Analysis

```rust
// Performance measurement integration across the entire stack
use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use pprof::{ProfilerGuard, Report};
use std::sync::Arc;

// Comprehensive benchmarking suite
fn benchmark_complete_pipeline(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("complete_pipeline");
    group.significance_level(0.1);
    group.confidence_level(0.95);
    
    // Benchmark 1: End-to-end latency
    group.bench_function("e2e_latency", |b| {
        b.to_async(&rt).iter_batched(
            || create_test_event(),
            |event| async move {
                black_box(process_event_end_to_end(event).await)
            },
            BatchSize::SmallInput
        );
    });
    
    // Benchmark 2: Throughput under load
    group.bench_function("throughput_max", |b| {
        b.to_async(&rt).iter_batched(
            || create_event_batch(1000),
            |batch| async move {
                black_box(process_batch_parallel(batch).await)
            },
            BatchSize::LargeInput
        );
    });
    
    // Benchmark 3: Memory efficiency
    group.bench_function("memory_efficiency", |b| {
        b.iter_batched(
            || create_zero_copy_test_data(),
            |data| {
                let initial_memory = get_memory_usage();
                let result = process_zero_copy(data);
                let final_memory = get_memory_usage();
                black_box((result, final_memory - initial_memory))
            },
            BatchSize::SmallInput
        );
    });
    
    // Benchmark 4: CPU utilization
    group.bench_function("cpu_efficiency", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                black_box(cpu_intensive_processing());
            }
            start.elapsed()
        });
    });
    
    group.finish();
}

// Profiling integration 
fn profile_memory_allocations() -> Result<Report, Box<dyn std::error::Error>> {
    let guard = ProfilerGuard::new(1000)?;
    
    // Run representative workload
    for _ in 0..10000 {
        process_event_with_allocations();
    }
    
    guard.report().build()
}

fn generate_performance_insights(report: &Report) -> PerformanceInsights {
    PerformanceInsights {
        hot_functions: extract_hot_functions(report),
        memory_leaks: detect_memory_leaks(report), 
        allocation_patterns: analyze_allocation_patterns(report),
        optimization_opportunities: identify_optimizations(report),
    }
}

criterion_group!(benches, benchmark_complete_pipeline);
criterion_main!(benches);
```

### Performance Comparison and Regression Detection

```rust
// Automated performance regression detection
struct PerformanceRegression {
    metric_name: String,
    baseline_value: f64,
    current_value: f64,
    regression_percentage: f64,
    significance_level: f64,
    recommendation: String,
}

async fn detect_performance_regressions(
    current_results: &BenchmarkResults,
    baseline_results: &BenchmarkResults,
) -> Vec<PerformanceRegression> {
    let mut regressions = Vec::new();
    
    // Throughput regression
    let throughput_change = calculate_percentage_change(
        baseline_results.throughput_eps,
        current_results.throughput_eps
    );
    
    if throughput_change < -5.0 { // 5% degradation threshold
        regressions.push(PerformanceRegression {
            metric_name: "Throughput".to_string(),
            baseline_value: baseline_results.throughput_eps,
            current_value: current_results.throughput_eps,
            regression_percentage: throughput_change,
            significance_level: 0.95,
            recommendation: "Consider profiling CPU usage and memory allocation patterns".to_string(),
        });
    }
    
    // Latency regression
    let latency_change = calculate_percentage_change(
        baseline_results.p99_latency_ms,
        current_results.p99_latency_ms  
    );
    
    if latency_change > 10.0 { // 10% latency increase threshold
        regressions.push(PerformanceRegression {
            metric_name: "P99 Latency".to_string(),
            baseline_value: baseline_results.p99_latency_ms,
            current_value: current_results.p99_latency_ms,
            regression_percentage: latency_change,
            significance_level: 0.95,
            recommendation: "Check for blocking operations in async code and review zero-copy implementations".to_string(),
        });
    }
    
    regressions
}
```

---

## Expected Learning Outcomes

### Technical Mastery Demonstrated

1. **Local Kubernetes Proficiency**: Deploy and manage complex microservice architectures
2. **Performance Engineering**: Measure, analyze, and optimize Rust code with data-driven insights  
3. **Systems Integration**: Orchestrate databases, message queues, APIs, and custom operators
4. **Production Tooling**: Build comprehensive monitoring and alerting systems
5. **Comparative Analysis**: Benchmark different implementations and detect regressions

### Performance Insights You'll Gain

- **Memory Layout Impact**: See how zero-copy vs copying affects real throughput
- **Async Runtime Behavior**: Understand tokio task scheduling and resource contention  
- **Database Performance**: Measure PostgreSQL interaction patterns and optimization opportunities
- **Network I/O Patterns**: Analyze service-to-service communication efficiency
- **Resource Utilization**: Track CPU, memory, and I/O usage across your entire system

### Deliverables

1. **Complete K3s Environment** running your Rust microservices
2. **Performance Observatory Dashboard** with real-time metrics and historical analysis
3. **Automated Benchmark Suite** that runs regression tests
4. **Custom Kubernetes Operator** that orchestrates your pipeline
5. **Performance Report** comparing different implementation strategies
6. **Optimization Recommendations** based on profiling data

---

## Quick Start Commands

```bash
# Clone and setup
git clone <your-repo>
cd rust-performance-observatory

# Setup local k3s cluster
./scripts/setup-k3s-cluster.sh

# Build and deploy services  
./scripts/build-and-deploy.sh

# Run performance test suite
./scripts/run-performance-suite.sh

# View results
kubectl port-forward service/grafana 3000:3000
open http://localhost:3000  # admin/performance

# Monitor in real-time
kubectl logs -f deployment/pipeline-operator
kubectl top pods --sort-by=cpu
```

This capstone project gives you a production-like environment where you can experiment with real performance optimization, see the impact of your Rust choices, and build the kind of observability tooling that's essential in professional systems development.

The beauty is that everything runs locally on k3s, so you can iterate quickly while still experiencing the full complexity of distributed systems deployment and monitoring.
