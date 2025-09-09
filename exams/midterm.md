# CLASSIFIED: Submarine ML Operations Battle System
## Project Codename: KRAKEN - Autonomous Threat Detection & Response Platform

*You are the senior systems engineer aboard USS Nevada (SSBN-733). The Navy has tasked you with building a real-time ML operations pipeline that processes sonar data, satellite intelligence, and tactical communications to provide autonomous threat assessment and response recommendations.*

**Mission Classification**: TOP SECRET // NOFORN
**Clearance Required**: TS/SCI with Polygraph
**Mission Timeline**: 96 hours to operational deployment

---

## INTELLIGENCE BRIEFING

### Strategic Context
Enemy submarines are using advanced acoustic masking and AI-driven evasion patterns. Traditional sonar analysis takes 14 minutes per contact - too slow for modern naval warfare. You need real-time ML inference with <200ms latency for autonomous defensive systems.

### Technical Requirements (MISSION CRITICAL)
- **Latency**: Sub-200ms end-to-end response time
- **Throughput**: Process 10,000 sonar pings/second + satellite data streams
- **Availability**: 99.99% uptime (submarine operations cannot fail)
- **Security**: Zero data persistence on disk (all in-memory processing)
- **Scalability**: Auto-scale from 1-50 processing nodes based on threat level
- **Observability**: Real-time performance monitoring for combat readiness

---

## SYSTEM ARCHITECTURE: PROJECT KRAKEN

```
┌─────────────────────────────────────────────────────────┐
│                    K3s Combat Cluster                  │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌─────────────────┐               │
│ │   Sensor Data   │  │  ML Inference   │               │  
│ │   Ingestion     │  │   Pipeline      │               │
│ │                 │  │                 │               │
│ │ • Sonar Arrays  │  │ • Threat Models │               │
│ │ • Satellite     │  │ • Classification│               │
│ │ • Comms Intel   │  │ • Prediction    │               │
│ └─────────────────┘  └─────────────────┘               │
│          │                     │                       │
│          ▼                     ▼                       │
│ ┌─────────────────┐  ┌─────────────────┐               │
│ │   Real-time     │  │   PostgreSQL    │               │
│ │   Processing    │  │   Intel DB      │               │
│ │                 │  │                 │               │
│ │ • Stream Fusion │  │ • Contact DB    │               │
│ │ • Pattern Match │  │ • Model Store   │               │
│ │ • Threat Score  │  │ • Metrics       │               │
│ └─────────────────┘  └─────────────────┘               │
│          │                     │                       │
│          ▼                     ▼                       │
│ ┌─────────────────┐  ┌─────────────────┐               │
│ │   Command &     │  │   Auto-scale    │               │
│ │   Control API   │  │   Controller    │               │
│ │                 │  │                 │               │
│ │ • Axum REST     │  │ • Load Monitor  │               │
│ │ • WebSocket     │  │ • Pod Scaling   │               │
│ │ • Alerts        │  │ • Health Check  │               │
│ └─────────────────┘  └─────────────────┘               │
└─────────────────────────────────────────────────────────┘
```

---

## PERFORMANCE MONITORING & TESTING ARSENAL
*"You can't optimize what you can't measure"*

### Rust Performance Monitoring Tools

**Add to your `Cargo.toml` dev-dependencies:**
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
pprof = { version = "0.13", features = ["criterion", "flamegraph"] }
dhat = "0.3"
valgrind-request = "1.0"

[[bench]]
name = "sensor_benchmarks"
harness = false
```

### Latency & Throughput Measurement

**Criterion Benchmarking** - `benches/sensor_benchmarks.rs`:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use kraken_mlops::*;

fn benchmark_sensor_ingestion(c: &mut Criterion) {
    let mut group = c.benchmark_group("sensor_ingestion");
    group.throughput(Throughput::Elements(10000));
    
    group.bench_function("sonar_processing", |b| {
        b.iter(|| {
            // Benchmark your sensor processing pipeline
            process_sonar_batch(black_box(generate_test_sonar_data(1000)))
        });
    });
    
    group.bench_function("ml_inference", |b| {
        b.iter(|| {
            // Benchmark ML pipeline latency
            run_threat_classification(black_box(sample_sensor_data()))
        });
    });
}

criterion_group!(benches, benchmark_sensor_ingestion);
criterion_main!(benches);
```

**Run benchmarks with flamegraphs:**
```bash
cargo bench --features criterion/html_reports
# Generates HTML reports in target/criterion/
# Install flamegraph: cargo install flamegraph
sudo cargo flamegraph --bench sensor_benchmarks
```

### Memory Profiling & Leak Detection

**DHAT Memory Profiler** - Add to your main functions:
```rust
#[cfg(feature = "dhat")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat")]
    let _profiler = dhat::Profiler::new_heap();
    
    // Your application code here
}
```

**Valgrind Integration** (Linux):
```bash
# Check for memory leaks
cargo build --release
valgrind --leak-check=full --show-leak-kinds=all ./target/release/sensor_ingestion

# Memory usage profiling
cargo install cargo-valgrind
cargo valgrind --tool=massif run --bin sensor_ingestion
```

### Real-time Performance Monitoring

**Metrics Collection** - Add to your services:
```rust
use metrics::{counter, histogram, gauge};
use metrics_prometheus::PrometheusBuilder;
use std::time::Instant;

// Setup metrics endpoint
let prometheus_handle = PrometheusBuilder::new()
    .with_http_listener(([0, 0, 0, 0], 9090))
    .install()?;

// Track latency
let start = Instant::now();
process_sensor_data(data).await;
histogram!("sensor_processing_duration_ms", start.elapsed().as_millis() as f64);

// Track throughput  
counter!("sensors_processed_total", 1);
gauge!("active_connections", active_count as f64);
```

**Tokio Console Integration** - Monitor async tasks:
```toml
[dependencies]
console-subscriber = "0.2"
tokio = { version = "1", features = ["full", "tracing"] }
```

```rust
fn main() {
    console_subscriber::init();
    // Your tokio runtime here
}
```

```bash
# Install tokio-console
cargo install --locked tokio-console
# Run your app, then connect
tokio-console http://127.0.0.1:6669
```

### Load Testing & Stress Testing

**Artillery.js Config** - `load-tests/artillery.yml`:
```yaml
config:
  target: 'http://localhost:8080'
  phases:
    - duration: 60
      arrivalRate: 100
    - duration: 120  
      arrivalRate: 500
    - duration: 60
      arrivalRate: 1000

scenarios:
  - name: "Sensor Data Ingestion"
    weight: 70
    requests:
      - post:
          url: "/api/v1/sensors/data"
          json:
            sensor_id: "{{ $uuid }}"
            data_payload: "{{ $randomString() }}"
            
  - name: "Threat Assessment Query"
    weight: 30
    requests:
      - get:
          url: "/api/v1/threats/active"
```

**Custom Rust Load Tester**:
```rust
use tokio::time::{Duration, Instant};
use futures::future::join_all;

async fn load_test_api() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let concurrent_users = 1000;
    let requests_per_user = 100;
    
    let start = Instant::now();
    let tasks: Vec<_> = (0..concurrent_users).map(|_| {
        let client = client.clone();
        tokio::spawn(async move {
            for _ in 0..requests_per_user {
                let _response = client
                    .post("http://localhost:8080/api/v1/sensors/data")
                    .json(&sample_sensor_data())
                    .send()
                    .await?;
            }
            Ok::<_, reqwest::Error>(())
        })
    }).collect();
    
    join_all(tasks).await;
    let duration = start.elapsed();
    
    println!("Processed {} requests in {:?}", 
             concurrent_users * requests_per_user, duration);
    println!("RPS: {}", 
             (concurrent_users * requests_per_user) as f64 / duration.as_secs_f64());
    
    Ok(())
}
```

### System Resource Monitoring

**Process Metrics** - Monitor your application:
```rust
use sysinfo::{System, SystemExt, ProcessExt, PidExt};

fn monitor_system_resources() {
    let mut sys = System::new_all();
    sys.refresh_all();
    
    if let Some(process) = sys.process(sysinfo::get_current_pid().unwrap()) {
        gauge!("memory_usage_bytes", process.memory() * 1024);
        gauge!("cpu_usage_percent", process.cpu_usage() as f64);
    }
    
    gauge!("total_memory_bytes", sys.total_memory() * 1024);
    gauge!("available_memory_bytes", sys.available_memory() * 1024);
}
```

### Performance Testing Strategy

**Combat Readiness Tests**:
```bash
#!/bin/bash
# performance-test-suite.sh

echo "🎯 Combat Readiness Performance Validation"

# 1. Latency under load
echo "Testing API latency under 1000 RPS..."
artillery run load-tests/artillery.yml

# 2. Memory leak detection  
echo "Memory leak testing (10 minute stress test)..."
cargo run --bin sensor_ingestion --features dhat &
PID=$!
sleep 600
kill $PID

# 3. Throughput benchmarks
echo "Throughput benchmarks..."
cargo bench

# 4. System resource limits
echo "Testing resource consumption..."
stress-ng --cpu 8 --vm 2 --vm-bytes 1G --timeout 60s &
cargo test --release test_performance_under_load

echo "✅ Performance validation complete"
```

**Continuous Performance Monitoring**:
- Set up Grafana dashboards for real-time metrics
- Configure alerts for latency > 200ms
- Monitor memory usage trends over time  
- Track error rates and success ratios
- Set up automated performance regression tests in CI

---

## SRE OPERATIONS BRIEF: KUBERNETES BATTLE MANAGEMENT
*"Systems fail. Your job is to make sure the mission doesn't."*

### OPERATIONAL INTELLIGENCE: SRE Mindset for Combat Systems

**CLASSIFICATION: FOR OFFICIAL USE ONLY**

As the senior Site Reliability Engineer aboard USS Nevada, you must think like both a systems architect and a combat operator. Your Kubernetes clusters are weapons systems - they must be reliable, observable, and resilient under enemy fire.

### TACTICAL SITUATION AWARENESS: The Four Golden Signals

**Intelligence Briefing: System Health Indicators**
```bash
# LATENCY - How long does it take to service a request?
kubectl top pods --sort-by=cpu -n kraken-system
kubectl get hpa --watch  # Watch auto-scaling decisions

# TRAFFIC - How much demand is being placed on your system?  
kubectl get ingress -o wide  # Traffic routing status
kubectl logs -f deployment/sensor-ingestion --tail=100 | grep "requests/sec"

# ERRORS - What is the rate of requests that fail?
kubectl get events --sort-by=.metadata.creationTimestamp
kubectl logs deployment/ml-inference | grep -i "error\|exception\|panic"

# SATURATION - How "full" is your service?
kubectl top nodes  # Resource utilization across cluster
kubectl describe hpa kraken-ml-pipeline  # Scaling headroom
```

### COMMAND & CONTROL: Kubernetes Operations Toolkit

**Essential SRE Weapons Systems:**

**Helm Charts - Standardized Deployment Packages**
```yaml
# charts/kraken-system/Chart.yaml
apiVersion: v2
name: kraken-system
description: Submarine ML Operations Battle System
version: 1.0.0
appVersion: "1.0"

dependencies:
- name: postgresql
  version: 12.0.0
  repository: https://charts.bitnami.com/bitnami
- name: prometheus-operator
  version: 45.7.1
  repository: https://prometheus-community.github.io/helm-charts
```

```yaml
# charts/kraken-system/values.yaml - Combat Configuration
global:
  threatLevel: "DEFCON-2"
  classification: "SECRET//NOFORN" 

sensorIngestion:
  replicaCount: 5
  resources:
    limits:
      memory: "1Gi"
      cpu: "500m"
    requests:  
      memory: "512Mi"
      cpu: "250m"
  
  # SRE Golden Rules
  maxSurge: 1          # Never deploy more than 1 pod at a time  
  maxUnavailable: 0    # Never allow service degradation
  
mlInference:
  replicaCount: 10     # Higher replica count for ML workload
  nodeSelector:
    gpu: "true"        # Only deploy on GPU nodes
  tolerations:
  - key: "gpu"
    operator: "Equal"
    value: "true"
    effect: "NoSchedule"

# Observability Stack
monitoring:
  prometheus:
    enabled: true
    retention: "7d"
  grafana:
    adminPassword: "ClassifiedDashboard123!"
  alertmanager:
    slackChannel: "#submarine-alerts"
```

**Deploy with Operational Security:**
```bash
# Intelligence-driven deployment
helm upgrade --install kraken-system ./charts/kraken-system \
  --namespace kraken-ops \
  --create-namespace \
  --set global.threatLevel="DEFCON-1" \
  --wait --timeout=600s

# Verify operational readiness
kubectl get pods -n kraken-ops -o wide
kubectl get svc,ingress -n kraken-ops
```

### BATTLE DAMAGE ASSESSMENT: Troubleshooting Under Fire

**Combat Debugging Procedures:**

**Phase 1: Situational Awareness**
```bash
# MISSION CRITICAL: Get cluster status in 30 seconds
kubectl cluster-info
kubectl get nodes -o wide
kubectl get pods --all-namespaces | grep -v Running
kubectl get events --sort-by=.metadata.creationTimestamp --all-namespaces | tail -20

# Resource pressure assessment
kubectl top nodes
kubectl describe nodes | grep -A 10 "Allocated resources"
```

**Phase 2: Service-Level Investigation**
```bash
# Investigate specific service failures
kubectl describe pod <failing-pod> -n kraken-ops
kubectl logs <failing-pod> -n kraken-ops --previous  # Previous crash logs
kubectl get endpoints kraken-ml-inference  # Service discovery issues

# Network diagnostics
kubectl exec -it <pod-name> -n kraken-ops -- nslookup kraken-ml-inference
kubectl get networkpolicies -n kraken-ops
```

**Phase 3: Deep Battle Analysis**
```bash
# Container-level debugging
kubectl exec -it <pod-name> -n kraken-ops -- /bin/bash
kubectl port-forward service/kraken-api 8080:80  # Direct service access

# Performance investigation  
kubectl exec -it <pod-name> -n kraken-ops -- top
kubectl exec -it <pod-name> -n kraken-ops -- netstat -tlnp
```

### FORCE MULTIPLIER: Advanced Kubernetes SRE Patterns

**Custom Resource Definitions - Mission-Specific Operators**
```yaml
# Define ThreatResponse Custom Resource
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: threatresponses.kraken.navy.mil
spec:
  group: kraken.navy.mil
  versions:
  - name: v1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              threatLevel:
                type: string
                enum: ["BENIGN", "SUSPICIOUS", "HOSTILE", "IMMEDIATE"]
              responseAction:
                type: string
                enum: ["MONITOR", "TRACK", "INTERCEPT", "ENGAGE"]
              autoScale:
                type: boolean
              resourceLimits:
                type: object
  scope: Namespaced
  names:
    plural: threatresponses
    singular: threatresponse
    kind: ThreatResponse
```

**Implement Mission-Critical Operator Logic:**
```rust
// src/kubernetes/threat_response_operator.rs
use kube::{Client, Api, ResourceExt, runtime::{controller, watcher, Controller}};
use k8s_openapi::api::apps::v1::Deployment;
use futures::StreamExt;

#[derive(Debug, Clone)]
pub struct ThreatResponseController {
    client: Client,
}

impl ThreatResponseController {
    pub async fn run(&self) -> anyhow::Result<()> {
        let api: Api<ThreatResponse> = Api::all(self.client.clone());
        
        Controller::new(api, watcher::Config::default())
            .run(reconcile_threat_response, error_policy, Arc::new(self.client.clone()))
            .for_each(|reconciler_result| async move {
                match reconciler_result {
                    Ok(_) => tracing::info!("Threat response reconciliation successful"),
                    Err(e) => tracing::error!("Threat response reconciliation failed: {}", e),
                }
            })
            .await;
        
        Ok(())
    }
}

async fn reconcile_threat_response(
    threat_response: Arc<ThreatResponse>, 
    context: Arc<Client>
) -> Result<controller::Action, Box<dyn std::error::Error + Send + Sync>> {
    let name = threat_response.name_any();
    let namespace = threat_response.namespace().unwrap_or_default();
    
    match threat_response.spec.threat_level.as_str() {
        "IMMEDIATE" => {
            tracing::warn!("IMMEDIATE THREAT DETECTED - Scaling to combat readiness");
            scale_deployment(context, &namespace, "kraken-ml-inference", 20).await?;
            scale_deployment(context, &namespace, "kraken-api", 10).await?;
        },
        "HOSTILE" => {
            scale_deployment(context, &namespace, "kraken-ml-inference", 15).await?;
            scale_deployment(context, &namespace, "kraken-api", 8).await?;
        },
        "SUSPICIOUS" => {
            scale_deployment(context, &namespace, "kraken-ml-inference", 10).await?;
        },
        _ => {
            // BENIGN - scale back to normal operations
            scale_deployment(context, &namespace, "kraken-ml-inference", 5).await?;
            scale_deployment(context, &namespace, "kraken-api", 3).await?;
        }
    }
    
    Ok(controller::Action::requeue(Duration::from_secs(300)))
}
```

### OPERATIONAL READINESS: SRE Checklist for Combat Deployment

**Pre-Combat Systems Check:**
```bash
#!/bin/bash
# sre-combat-readiness.sh - Run before every mission

echo "🎯 SUBMARINE SRE COMBAT READINESS CHECK"
echo "========================================"

# 1. Cluster Health Verification
echo "1. Cluster Status..."
kubectl get nodes --no-headers | awk '$2 != "Ready" {print "❌ Node " $1 " not ready"; exit 1}'
echo "✅ All nodes operational"

# 2. Critical Services Health
echo "2. Critical Services..."
CRITICAL_SERVICES="kraken-sensor-ingestion kraken-ml-inference kraken-api"
for service in $CRITICAL_SERVICES; do
    READY_REPLICAS=$(kubectl get deployment $service -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
    DESIRED_REPLICAS=$(kubectl get deployment $service -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
    
    if [ "$READY_REPLICAS" -eq "$DESIRED_REPLICAS" ] && [ "$READY_REPLICAS" -gt "0" ]; then
        echo "✅ $service: $READY_REPLICAS/$DESIRED_REPLICAS ready"
    else
        echo "❌ $service: $READY_REPLICAS/$DESIRED_REPLICAS ready - MISSION CRITICAL FAILURE"
        exit 1
    fi
done

# 3. Resource Pressure Check
echo "3. Resource Availability..."
NODE_PRESSURE=$(kubectl describe nodes | grep -i "pressure" | grep -i "true" | wc -l)
if [ "$NODE_PRESSURE" -gt "0" ]; then
    echo "⚠️  Warning: $NODE_PRESSURE nodes under resource pressure"
else
    echo "✅ No resource pressure detected"
fi

# 4. Network Policies Enforcement
echo "4. Security Posture..."
NETWORK_POLICIES=$(kubectl get networkpolicy --all-namespaces --no-headers | wc -l)
if [ "$NETWORK_POLICIES" -lt "3" ]; then
    echo "⚠️  Warning: Only $NETWORK_POLICIES network policies active"
else
    echo "✅ Network security policies active"
fi

# 5. Backup Systems Online
echo "5. Backup Systems..."
kubectl get pvc | grep -q "postgres-backup" && echo "✅ Database backup storage ready" || echo "❌ Database backup not configured"

# 6. Monitoring Stack
echo "6. Observability Stack..."
kubectl get pods -n monitoring | grep -q "prometheus.*Running" && echo "✅ Prometheus operational" || echo "❌ Prometheus down"
kubectl get pods -n monitoring | grep -q "grafana.*Running" && echo "✅ Grafana operational" || echo "❌ Grafana down"

echo ""
echo "🔥 SYSTEM STATUS: READY FOR COMBAT OPERATIONS"
echo "Mission authorization: GRANTED"
```

**Incident Response Playbooks:**
```yaml
# alerts/submarine-alerts.yaml - Mission-critical alerting
groups:
- name: submarine_critical
  rules:
  - alert: ThreatProcessingDown
    expr: up{job="kraken-ml-inference"} == 0
    for: 30s
    labels:
      severity: "MISSION_CRITICAL"
      classification: "SECRET"
    annotations:
      summary: "THREAT PROCESSING OFFLINE - COMBAT CAPABILITY COMPROMISED"
      action: "Execute emergency ML pipeline restart procedures"
      
  - alert: LatencyBreachCombat
    expr: histogram_quantile(0.95, http_request_duration_seconds) > 0.2
    for: 1m
    labels:
      severity: "WARNING"  
    annotations:
      summary: "Combat response time exceeds 200ms threshold"
      action: "Scale ML inference pods immediately"
      
  - alert: MemoryPressureWarning
    expr: (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) < 0.1
    for: 2m
    labels:
      severity: "WARNING"
    annotations:
      summary: "Node memory pressure - potential performance degradation"
      action: "Initiate pod eviction procedures if combat operations not active"
```

### INTELLIGENCE SUMMARY: SRE Best Practices for Mission Success

**The SRE Combat Mindset:**

1. **Assume Everything Will Fail** - Design for resilience, not just performance
2. **Measure Everything** - You can't improve what you don't measure
3. **Automate Routine Tasks** - Human operators focus on tactical decisions
4. **Plan for Disaster** - Practice failure scenarios in peacetime
5. **Security Through Defense in Depth** - Multiple layers of protection
6. **Documentation is Intelligence** - Clear runbooks save lives in combat

**Your mission as submarine SRE:** Keep the systems running while under fire. The crew depends on your ML pipeline for survival. There are no second chances in submarine warfare - your Kubernetes clusters must perform flawlessly when it matters most.

**Remember**: Every alert is potential enemy contact. Every latency spike could mean missed threat detection. Every memory leak is a vulnerability waiting to be exploited. Stay vigilant, stay ready.

---

## DAY 1: SENSOR DATA INGESTION SYSTEM
*"First, we need to hear everything in the ocean"*

### Mission: Build High-Performance Data Ingestion
Create a Rust service that ingests multiple data streams with zero data loss.

**Setup Your Workspace:**
```bash
cargo new --bin kraken-mlops
cd kraken-mlops
```

**Critical Dependencies** (`Cargo.toml`):
```toml
[package]
name = "kraken-mlops"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full", "tracing"] }
axum = { version = "0.7", features = ["tracing"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["trace", "cors", "compression"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono", "json"] }
tokio-postgres = { version = "0.7", features = ["with-chrono-0_4", "with-uuid-1", "with-serde_json-1"] }
reqwest = { version = "0.11", features = ["json", "stream"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
bytes = "1.5"
futures = { version = "0.3", features = ["async-await"] }
tokio-stream = { version = "0.1", features = ["sync"] }
dashmap = "5.5"
uuid = { version = "1.6", features = ["serde", "v4"] }
thiserror = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.4", features = ["derive", "env"] }
# ML/Data Processing
candle = { version = "0.4", features = ["cuda"] }
ndarray = "0.15"
# Metrics & Observability  
prometheus = "0.13"
metrics = "0.21"
metrics-prometheus = "0.6"

[[bin]]
name = "sensor_ingestion"
path = "src/day1_sensor_ingestion.rs"

[[bin]]  
name = "ml_inference"
path = "src/day2_ml_inference.rs"

[[bin]]
name = "combat_api"
path = "src/day3_combat_api.rs"

[[bin]]
name = "auto_scaler"
path = "src/day4_auto_scaler.rs"
```

### Day 1 Implementation: Sensor Data Pipeline

**File: `src/day1_sensor_ingestion.rs`**

Your mission is to build a service that:
1. **Ingests multiple data streams simultaneously** (sonar, satellite, comms)
2. **Processes 10,000+ messages/second** with sub-millisecond latency
3. **Maintains zero data loss** during system restarts
4. **Provides real-time health monitoring**

**Critical Performance Requirements:**
- Memory usage must stay below 512MB under full load
- CPU usage must not exceed 80% during normal operations  
- All data must be processed within 1ms of receipt
- System must handle 3x normal load without degradation

**The Challenge:**
```rust
// Your sensor ingestion pipeline must handle this data volume
// Sonar: 5,000 pings/second, each 2KB of acoustic data
// Satellite: 500 intel reports/second, each 50KB of imagery/signals  
// Comms: 2,000 intercepts/second, each 1KB of encrypted traffic
// = ~150MB/second sustained throughput with <1ms processing latency

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SensorReading {
    pub sensor_id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>, 
    pub sensor_type: SensorType,
    pub data_payload: bytes::Bytes,
    pub classification: SecurityClassification,
    pub priority: ThreatPriority,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SensorType {
    SonarPassive { frequency_mhz: f64, bearing: f64 },
    SonarActive { ping_strength: f64, return_time_ms: f64 },
    SatelliteOptical { lat: f64, lon: f64, zoom_level: u8 },
    SatelliteSignals { frequency_range: (f64, f64), signal_strength: f64 },
    CommsIntercept { frequency_mhz: f64, encryption_type: String },
}

// YOUR IMPLEMENTATION HERE
// Build the ingestion system that can handle this load
```

**Success Metrics:**
- Process 10,000 msgs/second sustained for 10 minutes
- Memory usage remains stable (no memory leaks)
- <1ms latency from ingestion to storage in PostgreSQL
- Zero message loss during simulated system restarts
- Comprehensive metrics exported for monitoring

---

## DAY 2: ML INFERENCE PIPELINE  
*"Intelligence is worthless if it arrives after the torpedoes"*

### Mission: Real-time ML Model Serving
Build a Rust-native ML inference pipeline that processes sensor data through neural networks with combat-grade performance requirements.

**File: `src/day2_ml_inference.rs`**

**The Challenge:**
```rust
// You must build an inference pipeline that:
// 1. Loads ML models from PostgreSQL model store
// 2. Processes streaming sensor data through neural networks
// 3. Outputs threat classifications with confidence scores
// 4. Maintains <200ms end-to-end latency under full combat load

#[derive(Debug, Clone)]
pub struct ThreatAssessment {
    pub contact_id: uuid::Uuid,
    pub threat_level: ThreatLevel,
    pub confidence_score: f64,          // 0.0 - 1.0
    pub recommended_action: CombatAction,
    pub time_to_threat_ms: Option<u64>, // Estimated time until weapon impact
    pub supporting_evidence: Vec<EvidencePoint>,
}

#[derive(Debug, Clone)]
pub enum ThreatLevel {
    Benign,           // Civilian or friendly
    Suspicious,       // Unknown contact, monitor
    Hostile,          // Confirmed enemy, track
    ImmediateThreat,  // Weapons hot, engage immediately
}

#[derive(Debug, Clone)]  
pub enum CombatAction {
    Monitor,
    Track,
    Intercept, 
    Engage,
    EmergencySurface,
}

// YOUR ML PIPELINE IMPLEMENTATION
// Build the inference system using candle/ndarray for neural networks
// Must handle: sonar signature classification, satellite image analysis,
// comms pattern recognition, multi-modal threat fusion
```

**Critical ML Requirements:**
- **Model Loading**: Hot-swap models without service restart
- **Batch Processing**: Process sensor data in optimal batch sizes
- **Multi-Modal Fusion**: Combine sonar + satellite + comms intelligence
- **Uncertainty Quantification**: Provide confidence scores for all predictions
- **A/B Testing**: Compare model performance in real-time

**Performance Benchmarks:**
- Process 1000 sensor readings through full ML pipeline in <200ms
- Support 5 different ML models simultaneously 
- Memory usage for ML inference <2GB total
- Model swapping must complete in <5 seconds
- 99.9% uptime during model updates

---

## DAY 3: COMBAT COMMAND & CONTROL API
*"The bridge needs answers, not excuses"*

### Mission: Build Command-Grade REST API
Create an Axum-based API that provides real-time tactical intelligence to submarine command systems.

**File: `src/day3_combat_api.rs`**

**API Requirements:**
```rust
// Your API must provide these endpoints with military-grade reliability:

// GET /api/v1/contacts - Real-time contact list with threat assessments
// POST /api/v1/contacts/{id}/classify - Manual threat reclassification
// GET /api/v1/threats/active - All active threats requiring immediate attention
// POST /api/v1/combat/action - Log combat actions taken by crew
// GET /api/v1/sensors/health - Real-time sensor system status
// WebSocket /ws/tactical - Real-time tactical picture updates
// GET /api/v1/metrics - Prometheus metrics for system monitoring
// POST /api/v1/models/deploy - Deploy new ML models hot-swap

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ContactSummary {
    pub id: uuid::Uuid,
    pub first_detected: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub position: Option<Position>,
    pub bearing: Option<f64>,
    pub range_km: Option<f64>,
    pub speed_knots: Option<f64>,
    pub threat_assessment: ThreatAssessment,
    pub tracking_history: Vec<TrackingPoint>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TacticalPicture {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub own_ship_position: Position,
    pub contacts: Vec<ContactSummary>,
    pub active_threats: Vec<ThreatAssessment>,
    pub system_alerts: Vec<SystemAlert>,
    pub recommended_actions: Vec<TacticalRecommendation>,
}
```

**API Performance Requirements:**
- **Latency**: <50ms response time for all REST endpoints
- **Throughput**: Handle 1000 requests/second sustained
- **WebSocket**: Real-time updates to 100 concurrent command stations  
- **Availability**: 99.99% uptime (submarine operations cannot fail)
- **Security**: JWT authentication + rate limiting + audit logging

**Advanced Features:**
- **Real-time streaming**: WebSocket tactical picture updates every 100ms
- **GraphQL**: Flexible queries for complex tactical scenarios
- **Caching**: Redis-compatible caching for frequently accessed data
- **Circuit breakers**: Graceful degradation when ML pipeline overloads

---

## DAY 4: KUBERNETES AUTO-SCALING CONTROLLER
*"System must adapt to threat level - from stealth patrol to combat stations"*

### Mission: Build Intelligent Auto-Scaler
Create a Rust-based Kubernetes controller that automatically scales your ML pipeline based on threat conditions and system load.

**File: `src/day4_auto_scaler.rs`**

**Controller Requirements:**
```rust
// Your auto-scaler must:
// 1. Monitor ML pipeline performance metrics
// 2. Scale pods based on threat level + processing load
// 3. Maintain minimum replicas for mission-critical reliability  
// 4. Implement custom resource definitions for threat-based scaling

use kube::{Client, CustomResource, Api, ResourceExt};
use k8s_openapi::api::apps::v1::Deployment;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone)]
#[kube(group = "kraken.navy.mil", version = "v1", kind = "ThreatScalePolicy")]
#[kube(namespaced)]
pub struct ThreatScalePolicySpec {
    pub target_deployment: String,
    pub min_replicas: i32,
    pub max_replicas: i32,
    pub threat_scaling: ThreatScalingConfig,
    pub performance_scaling: PerformanceScalingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreatScalingConfig {
    pub benign_replicas: i32,      // Normal patrol operations
    pub suspicious_replicas: i32,   // Elevated alertness  
    pub hostile_replicas: i32,     // Combat readiness
    pub immediate_replicas: i32,   // All hands battle stations
}

// YOUR CONTROLLER IMPLEMENTATION
// Build the Kubernetes controller using kube-rs
// Must react to threat level changes within 30 seconds
// Implement predictive scaling based on historical patterns
```

**Scaling Intelligence Requirements:**
- **Threat-based scaling**: Automatically increase replicas when threats detected
- **Performance-based scaling**: Scale based on CPU/memory/latency metrics  
- **Predictive scaling**: Pre-scale based on historical threat patterns
- **Combat readiness**: Maintain minimum capacity for immediate response
- **Resource optimization**: Minimize resource waste during peaceful operations

**Controller Performance:**
- React to scaling triggers within 30 seconds
- Support 50+ deployments across multiple namespaces
- Maintain scaling decision audit trail
- Integrate with Prometheus for metrics-based decisions
- Zero downtime during scaling operations

---

## OPERATIONAL DEPLOYMENT: K3s Combat Cluster

### Deployment Architecture
```bash
#!/bin/bash
# deploy-kraken-system.sh

# Install k3s optimized for combat operations
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="server --disable traefik --disable servicelb" sh -

# Deploy PostgreSQL cluster for intelligence storage
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres-intel
spec:
  replicas: 3
  selector:
    matchLabels:
      app: postgres-intel
  template:
    metadata:
      labels:
        app: postgres-intel
    spec:
      containers:
      - name: postgres
        image: postgres:15
        env:
        - name: POSTGRES_DB
          value: "submarine_intel"
        - name: POSTGRES_USER
          value: "kraken_ops"
        - name: POSTGRES_PASSWORD
          value: "ClassifiedPassword123!"
        ports:
        - containerPort: 5432
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi" 
            cpu: "1000m"
EOF

# Deploy your Kraken system components
kubectl apply -f kraken-manifests/
```

**Production Kubernetes Manifests:**
- **ConfigMaps**: Database connections, ML model configs, threat thresholds
- **Secrets**: Classified credentials and encryption keys
- **Services**: Load balancers for API and ML inference endpoints
- **Ingress**: Secure routing for command & control interfaces
- **NetworkPolicies**: Isolate classified data flows
- **ResourceQuotas**: Prevent resource exhaustion during combat operations

---

## SUCCESS CRITERIA: OPERATIONAL READINESS

### Performance Benchmarks
Your system passes operational readiness when:

**Throughput Requirements:**
- [ ] Process 10,000 sensor readings/second sustained
- [ ] ML inference pipeline handles 1000 classifications/second  
- [ ] API serves 1000 requests/second with <50ms latency
- [ ] WebSocket updates 100 concurrent command stations

**Reliability Requirements:**
- [ ] 99.99% uptime over 72-hour continuous operation
- [ ] Zero data loss during simulated equipment failures
- [ ] Automatic recovery from database connection failures
- [ ] Graceful degradation when ML models are unavailable

**Scalability Requirements:**
- [ ] Auto-scale from 5 to 50 pods within 60 seconds of threat escalation
- [ ] Maintain performance during 3x normal operational load
- [ ] Support deployment of new ML models without service interruption

**Security & Observability:**
- [ ] Complete audit trail of all threat assessments and combat actions
- [ ] Real-time metrics dashboard showing system health
- [ ] Automated alerts for performance degradation
- [ ] Encrypted data storage and transmission

### Combat Simulation Tests
Run these scenarios to validate operational readiness:

**Test 1: Submarine Detection Scenario**
- Inject 50,000 sonar contacts over 10 minutes
- Verify ML pipeline correctly classifies submarine signatures
- Confirm API provides real-time tactical picture to command

**Test 2: Multi-Threat Engagement** 
- Simulate detection of 5 hostile submarines simultaneously
- Verify auto-scaler increases ML processing capacity
- Confirm system maintains <200ms threat assessment latency

**Test 3: System Resilience Under Fire**
- Randomly kill 30% of pods during active threat scenario
- Verify system maintains operational capability
- Confirm zero data loss and rapid automatic recovery

---

## PORTFOLIO SHOWCASE

**This project demonstrates production-ready Rust skills:**

✅ **Distributed Systems**: Multi-service architecture with PostgreSQL  
✅ **High-Performance Computing**: 10,000+ ops/sec with <200ms latency  
✅ **Machine Learning Operations**: Real-time ML inference pipeline  
✅ **Kubernetes Native**: Custom controllers and auto-scaling  
✅ **Production APIs**: Axum REST + WebSocket with auth/monitoring  
✅ **Database Engineering**: PostgreSQL with complex queries and pooling  
✅ **Observability**: Comprehensive metrics, tracing, and alerting  
✅ **Security**: Classified data handling with proper access controls

**Perfect for ML Ops / Platform Engineering roles** - Shows you can build production systems that combine machine learning, distributed computing, and Kubernetes orchestration at scale.

---

*MISSION CLASSIFICATION: This system contains technical capabilities critical to national defense. Unauthorized disclosure is prohibited by law.*

**GOOD HUNTING, ENGINEER. THE FLEET IS COUNTING ON YOU.**