# Kubernetes Operator Data Pipeline - The Full Mechanical Breakdown

## The Factory Floor Analogy

Picture yourself as the foreman of a data factory. You've got conveyor belts (pipelines), assembly stations (compute nodes), workers (pods), and a control room (CLI/UI). Your job is to orchestrate everything so data flows smoothly from raw materials to finished products.

## Day 1: Your Control Room - The Operator's Brain

The operator is your automated factory supervisor. While you're checking the Tekton UI or running `kubectl get jobs`, the operator is constantly watching, deciding, and acting.

### The Watch Loop - Your Factory Monitors

```rust
// This is like having security cameras everywhere
use kube::runtime::watcher;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let pipelines: Api<Pipeline> = Api::namespaced(client, "default");
    
    // This is your main control panel - watching for any changes
    let mut stream = watcher(pipelines, Default::default()).boxed();
    
    println!("🎛️  Control room online - watching for pipeline events...");
    
    while let Some(event) = stream.next().await {
        match event {
            Ok(Applied(pipeline)) => {
                println!("📋 New work order: {}", pipeline.name_any());
                // You just got a new pipeline to run
            }
            Ok(Deleted(pipeline)) => {
                println!("🗑️  Work order cancelled: {}", pipeline.name_any());
            }
            Err(e) => {
                println!("⚠️  Control room error: {}", e);
                // This is like a monitor going dark - investigate immediately
            }
        }
    }
    Ok(())
}
```

**What you see in practice:**
- You create a Pipeline with `kubectl apply -f my-pipeline.yaml`
- Your operator immediately sees the "Applied" event
- It starts planning which jobs to create and in what order

### The Authentication Dance - Your Security Badge

```rust
// This is how your operator proves it belongs here
async fn setup_auth() -> Result<Client, Box<dyn std::error::Error>> {
    // In-cluster: reads from service account token
    // Local dev: reads ~/.kube/config
    let client = Client::try_default().await?;
    
    // Test the connection
    let version = client.apiserver_version().await?;
    println!("🔐 Connected to cluster version: {}", version.git_version);
    
    Ok(client)
}
```

**Your role as the CLI operator:**
- When you run `kubectl get pipelines`, you're using the same auth system
- If you see "Forbidden" errors, your operator needs better RBAC permissions
- The operator needs to read Pipelines and write Jobs - just like you do manually

## Day 2: The Blueprint System - Custom Resource Definitions

Think of CRDs as standardized work orders. Instead of everyone writing different formats, you create a template.

### The Work Order Template

```yaml
# This is your standardized work order form
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: pipelines.dataworks.io
spec:
  group: dataworks.io
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
              # The assembly line steps
              steps:
                type: array
                items:
                  type: object
                  properties:
                    name:
                      type: string
                    type:
                      type: string
                      enum: ["fetch", "transform", "load", "fan-out", "quality-check"]
                    # Dependencies - what must finish before this step
                    dependsOn:
                      type: array
                      items:
                        type: string
                    # Step-specific configuration
                    config:
                      type: object
                    # Resource requirements
                    resources:
                      type: object
                      properties:
                        cpu: {type: string}
                        memory: {type: string}
                        parallelism: {type: integer}
          status:
            type: object
            properties:
              phase: {type: string}  # "Pending", "Running", "Completed", "Failed"
              startTime: {type: string}
              completionTime: {type: string}
              stepStatuses:
                type: array
                items:
                  type: object
                  properties:
                    name: {type: string}
                    status: {type: string}  # "waiting", "running", "completed", "failed"
                    startTime: {type: string}
                    completionTime: {type: string}
                    jobName: {type: string}  # The actual K8s job name
  scope: Namespaced
  names:
    plural: pipelines
    singular: pipeline
    kind: Pipeline
```

**What this means for you:**
- When you write `kubectl get pipelines`, you're querying this custom resource
- The `status` section is what you see when checking if things are working
- Each `step` becomes a Kubernetes Job that you can inspect with `kubectl get jobs`

## Day 3: The Decision Engine - The Reconciliation Loop

This is the heart of your operator - the factory supervisor's decision-making process.

### The Control Logic

```rust
use std::collections::HashMap;
use chrono::Utc;

pub async fn reconcile_pipeline(
    pipeline: Arc<Pipeline>, 
    ctx: Arc<Context>
) -> Result<Action, Error> {
    let name = pipeline.name_any();
    let namespace = pipeline.namespace().unwrap_or("default".to_string());
    
    println!("🔄 Reconciling pipeline: {}", name);
    
    // Step 1: What's the current state?
    let current_jobs = get_pipeline_jobs(&ctx.client, &name, &namespace).await?;
    let step_status_map = build_step_status_map(&current_jobs);
    
    // Step 2: What steps are ready to run?
    let ready_steps = find_ready_steps(&pipeline.spec, &step_status_map);
    
    // Step 3: Create jobs for ready steps
    for step in ready_steps {
        println!("🚀 Starting step: {}", step.name);
        create_job_for_step(&ctx.client, &pipeline, &step, &namespace).await?;
    }
    
    // Step 4: Handle failures
    let failed_steps = find_failed_steps(&step_status_map);
    if !failed_steps.is_empty() {
        println!("❌ Failed steps detected: {:?}", failed_steps);
        update_pipeline_status(&ctx.client, &pipeline, "Failed").await?;
        return Ok(Action::await_change()); // Stop here, wait for manual intervention
    }
    
    // Step 5: Check if we're done
    if all_steps_complete(&pipeline.spec, &step_status_map) {
        println!("✅ Pipeline {} completed successfully!", name);
        update_pipeline_status(&ctx.client, &pipeline, "Completed").await?;
        return Ok(Action::await_change());
    }
    
    // Step 6: Requeue to check again
    Ok(Action::requeue(Duration::from_secs(30)))
}

// This is your factory floor status checker
async fn get_pipeline_jobs(
    client: &Client, 
    pipeline_name: &str, 
    namespace: &str
) -> Result<Vec<Job>, Error> {
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), namespace);
    let label_selector = format!("pipeline={}", pipeline_name);
    
    let jobs = jobs_api.list(&ListParams::default().labels(&label_selector)).await?;
    Ok(jobs.items)
}

// This translates K8s job status into something useful
fn build_step_status_map(jobs: &[Job]) -> HashMap<String, StepStatus> {
    let mut status_map = HashMap::new();
    
    for job in jobs {
        if let Some(step_name) = job.labels().get("step") {
            let status = match &job.status {
                Some(status) => {
                    if status.succeeded.unwrap_or(0) > 0 {
                        "completed"
                    } else if status.failed.unwrap_or(0) > 0 {
                        "failed"
                    } else if status.active.unwrap_or(0) > 0 {
                        "running"
                    } else {
                        "pending"
                    }
                }
                None => "pending"
            };
            
            status_map.insert(step_name.clone(), StepStatus {
                name: step_name.clone(),
                status: status.to_string(),
                job_name: job.name_any(),
                start_time: job.status.as_ref()
                    .and_then(|s| s.start_time.as_ref())
                    .map(|t| t.0.to_rfc3339()),
            });
        }
    }
    
    status_map
}

// The dependency resolver - like figuring out assembly line order
fn find_ready_steps(
    spec: &PipelineSpec, 
    status_map: &HashMap<String, StepStatus>
) -> Vec<Step> {
    spec.steps.iter()
        .filter(|step| {
            // Skip if already running or completed
            if let Some(status) = status_map.get(&step.name) {
                return status.status == "failed"; // Allow retry of failed steps
            }
            
            // Check if all dependencies are completed
            step.depends_on.iter().all(|dep| {
                status_map.get(dep)
                    .map(|s| s.status == "completed")
                    .unwrap_or(false)
            })
        })
        .cloned()
        .collect()
}
```

**What you experience:**
- You run `kubectl describe pipeline my-data-job`
- You see status updates every 30 seconds (that's the requeue interval)
- When you check `kubectl get jobs`, you see new jobs appearing as dependencies complete
- Failed jobs don't block the whole pipeline - only their dependents

## Day 4: The Job Factory - Creating Work Units

Each step type needs its own job template. This is where you encode your domain knowledge.

### SFTP Fetch Jobs - The Data Ingesters

```rust
fn build_sftp_job(pipeline: &Pipeline, step: &Step) -> Job {
    let server = step.config["server"].as_str().unwrap();
    let path = step.config["path"].as_str().unwrap();
    let output_bucket = step.config.get("output_bucket")
        .and_then(|v| v.as_str())
        .unwrap_or("default-data-bucket");
    
    serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": format!("{}-{}", pipeline.name_any(), step.name),
            "labels": {
                "pipeline": pipeline.name_any(),
                "step": step.name,
                "step-type": "fetch"
            }
        },
        "spec": {
            "backoffLimit": 3,  // Retry failed fetches 3 times
            "template": {
                "spec": {
                    "containers": [{
                        "name": "sftp-fetcher",
                        "image": "your-registry/sftp-duckdb:v1.2",
                        "env": [
                            {"name": "SFTP_SERVER", "value": server},
                            {"name": "SFTP_PATH", "value": path},
                            {"name": "OUTPUT_BUCKET", "value": output_bucket},
                            {"name": "PIPELINE_NAME", "value": pipeline.name_any()},
                            {"name": "STEP_NAME", "value": &step.name}
                        ],
                        "resources": {
                            "requests": {
                                "memory": "512Mi",
                                "cpu": "500m"
                            },
                            "limits": {
                                "memory": "1Gi",
                                "cpu": "1"
                            }
                        }
                    }],
                    "restartPolicy": "Never",
                    // Use specific nodes for network-heavy work
                    "nodeSelector": {
                        "workload-type": "network-intensive"
                    }
                }
            }
        }
    })).unwrap()
}
```

### Transform Jobs - The Data Processors

```rust
fn build_postgres_transform_job(pipeline: &Pipeline, step: &Step) -> Job {
    let transform_sql = step.config["sql"].as_str().unwrap();
    let input_tables = step.config["input_tables"].as_array().unwrap();
    
    serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": format!("{}-{}", pipeline.name_any(), step.name),
            "labels": {
                "pipeline": pipeline.name_any(),
                "step": step.name,
                "step-type": "transform"
            }
        },
        "spec": {
            "template": {
                "spec": {
                    "initContainers": [{
                        // Set up the database
                        "name": "db-setup",
                        "image": "postgres:15",
                        "command": ["sh", "-c", "createdb analytics || true"],
                        "env": [
                            {"name": "POSTGRES_PASSWORD", "value": "transform-job"},
                            {"name": "PGPASSWORD", "value": "transform-job"}
                        ]
                    }],
                    "containers": [
                        {
                            // The transformation engine
                            "name": "transformer",
                            "image": "your-registry/sql-transformer:v2.1",
                            "env": [
                                {"name": "PG_HOST", "value": "localhost"},
                                {"name": "PG_USER", "value": "postgres"},
                                {"name": "PG_PASSWORD", "value": "transform-job"},
                                {"name": "TRANSFORM_SQL", "value": transform_sql},
                                {"name": "INPUT_BUCKET", "value": "data-staging"},
                                {"name": "OUTPUT_BUCKET", "value": "data-processed"}
                            ],
                            "resources": {
                                "requests": {
                                    "memory": "2Gi",
                                    "cpu": "1"
                                },
                                "limits": {
                                    "memory": "8Gi",
                                    "cpu": "4"
                                }
                            }
                        },
                        {
                            // The database sidecar
                            "name": "postgres",
                            "image": "postgres:15",
                            "env": [
                                {"name": "POSTGRES_PASSWORD", "value": "transform-job"}
                            ],
                            "resources": {
                                "requests": {
                                    "memory": "1Gi",
                                    "cpu": "500m"
                                }
                            }
                        }
                    ],
                    "restartPolicy": "Never",
                    // Use memory-optimized nodes for transforms
                    "nodeSelector": {
                        "workload-type": "compute-intensive"
                    }
                }
            }
        }
    })).unwrap()
}
```

## Day 5: Fan-Out Pattern - Parallel Processing at Scale

This is where things get interesting. When you have 1TB of data to process, you don't want one job - you want 50 jobs working in parallel.

### The Fan-Out Controller

```rust
fn build_fanout_job(pipeline: &Pipeline, step: &Step) -> Job {
    let parallelism = step.config["parallelism"].as_u64().unwrap_or(10);
    let chunk_size = step.config.get("chunk_size")
        .and_then(|v| v.as_str())
        .unwrap_or("100MB");
    
    // This creates a Job that spawns multiple pods
    serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": format!("{}-{}", pipeline.name_any(), step.name),
            "labels": {
                "pipeline": pipeline.name_any(),
                "step": step.name,
                "step-type": "fan-out"
            }
        },
        "spec": {
            "parallelism": parallelism,  // Run this many pods at once
            "completions": parallelism,  // Need all pods to succeed
            "backoffLimit": 6,  // Allow some failures
            "template": {
                "spec": {
                    "containers": [{
                        "name": "chunk-processor",
                        "image": "your-registry/parallel-processor:v1.5",
                        "env": [
                            {"name": "CHUNK_SIZE", "value": chunk_size},
                            {"name": "TOTAL_WORKERS", "value": parallelism.to_string()},
                            // Each pod gets a unique index
                            {"name": "WORKER_INDEX", "valueFrom": {
                                "fieldRef": {"fieldPath": "metadata.annotations['batch.kubernetes.io/job-completion-index']"}
                            }},
                            {"name": "INPUT_BUCKET", "value": "data-processed"},
                            {"name": "OUTPUT_BUCKET", "value": "data-final"}
                        ],
                        "resources": {
                            "requests": {
                                "memory": "4Gi",
                                "cpu": "2"
                            },
                            "limits": {
                                "memory": "8Gi",
                                "cpu": "4"
                            }
                        }
                    }],
                    "restartPolicy": "Never",
                    // Spread across nodes for better performance
                    "affinity": {
                        "podAntiAffinity": {
                            "preferredDuringSchedulingIgnoredDuringExecution": [{
                                "weight": 100,
                                "podAffinityTerm": {
                                    "labelSelector": {
                                        "matchExpressions": [{
                                            "key": "step",
                                            "operator": "In",
                                            "values": [step.name.clone()]
                                        }]
                                    },
                                    "topologyKey": "kubernetes.io/hostname"
                                }
                            }]
                        }
                    }
                }
            }
        }
    })).unwrap()
}
```

### Inside the Fan-Out Worker

```bash
#!/bin/bash
# This runs inside each parallel worker pod

WORKER_INDEX=${WORKER_INDEX:-0}
TOTAL_WORKERS=${TOTAL_WORKERS:-1}

echo "🔧 Worker $WORKER_INDEX of $TOTAL_WORKERS starting..."

# Download and process only our chunk of the data
gsutil -m cp "gs://data-processed/chunk_${WORKER_INDEX}_*.parquet" /tmp/

# Process our chunk
duckdb -c "
  COPY (
    SELECT * FROM read_parquet('/tmp/*.parquet')
    WHERE hash(id) % $TOTAL_WORKERS = $WORKER_INDEX
  ) TO 'gs://data-final/processed_chunk_${WORKER_INDEX}.parquet';
"

echo "✅ Worker $WORKER_INDEX completed successfully"
```

**What you see when this runs:**
- You run `kubectl get jobs` and see one job with 10 pods
- `kubectl get pods` shows 10 pods with names like `my-pipeline-process-xyz12-1`, `my-pipeline-process-xyz12-2`, etc.
- Each pod processes a different chunk of data in parallel
- The job only succeeds when ALL 10 pods complete successfully

## Day 6: Monitoring and Status Updates

This is your control room dashboard - how you know what's happening.

### Status Update Engine

```rust
async fn update_pipeline_status(
    client: &Client,
    pipeline: &Pipeline,
    phase: &str,
) -> Result<(), Error> {
    let name = pipeline.name_any();
    let namespace = pipeline.namespace().unwrap_or("default".to_string());
    
    // Get all jobs for this pipeline
    let jobs = get_pipeline_jobs(client, &name, &namespace).await?;
    let step_statuses = build_detailed_step_status(&jobs);
    
    let status = PipelineStatus {
        phase: phase.to_string(),
        start_time: pipeline.metadata.creation_timestamp.as_ref()
            .map(|t| t.0.to_rfc3339()),
        completion_time: if phase == "Completed" || phase == "Failed" {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        },
        step_statuses,
        total_steps: pipeline.spec.steps.len(),
        completed_steps: jobs.iter().filter(|j| 
            j.status.as_ref()
                .map(|s| s.succeeded.unwrap_or(0) > 0)
                .unwrap_or(false)
        ).count(),
        failed_steps: jobs.iter().filter(|j|
            j.status.as_ref()
                .map(|s| s.failed.unwrap_or(0) > 0)
                .unwrap_or(false)
        ).count(),
    };
    
    let pipelines: Api<Pipeline> = Api::namespaced(client.clone(), &namespace);
    pipelines.patch_status(
        &name,
        &PatchParams::default(),
        &Patch::Merge(json!({ "status": status }))
    ).await?;
    
    println!("📊 Updated pipeline status: {} -> {}", name, phase);
    Ok(())
}

fn build_detailed_step_status(jobs: &[Job]) -> Vec<StepStatus> {
    jobs.iter().map(|job| {
        let step_name = job.labels().get("step").unwrap_or(&"unknown".to_string()).clone();
        let step_type = job.labels().get("step-type").unwrap_or(&"unknown".to_string()).clone();
        
        let (status, message) = match &job.status {
            Some(status) => {
                if status.succeeded.unwrap_or(0) > 0 {
                    ("completed", "All tasks completed successfully".to_string())
                } else if status.failed.unwrap_or(0) > 0 {
                    let failed_count = status.failed.unwrap_or(0);
                    ("failed", format!("{} tasks failed", failed_count))
                } else if status.active.unwrap_or(0) > 0 {
                    let active_count = status.active.unwrap_or(0);
                    ("running", format!("{} tasks running", active_count))
                } else {
                    ("pending", "Waiting to start".to_string())
                }
            }
            None => ("pending", "Job not yet scheduled".to_string())
        };
        
        StepStatus {
            name: step_name,
            step_type,
            status: status.to_string(),
            message,
            job_name: job.name_any(),
            start_time: job.status.as_ref()
                .and_then(|s| s.start_time.as_ref())
                .map(|t| t.0.to_rfc3339()),
            completion_time: job.status.as_ref()
                .and_then(|s| s.completion_time.as_ref())
                .map(|t| t.0.to_rfc3339()),
        }
    }).collect()
}
```

## Day 7: The CLI - Your Mission Control

This is your interface to the whole system.

### Advanced CLI Tool

```rust
use clap::{Parser, Subcommand};
use comfy_table::{Table, Cell, Color};

#[derive(Parser)]
#[clap(name = "pipeline-ctl")]
#[clap(about = "Data Pipeline Controller")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new pipeline
    Run {
        #[clap(short, long)]
        config: String,
        #[clap(long)]
        dry_run: bool,
    },
    /// Check pipeline status
    Status {
        #[clap(short, long)]
        pipeline: Option<String>,
        #[clap(long)]
        watch: bool,
    },
    /// Cancel a running pipeline
    Cancel {
        pipeline: String,
    },
    /// Retry failed steps
    Retry {
        pipeline: String,
        #[clap(short, long)]
        step: Option<String>,
    },
    /// Show pipeline logs
    Logs {
        pipeline: String,
        #[clap(short, long)]
        step: Option<String>,
        #[clap(short, long)]
        follow: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = Client::try_default().await?;
    
    match cli.command {
        Commands::Run { config, dry_run } => {
            run_pipeline(&client, &config, dry_run).await?;
        }
        Commands::Status { pipeline, watch } => {
            if watch {
                watch_pipelines(&client, pipeline).await?;
            } else {
                show_status(&client, pipeline).await?;
            }
        }
        Commands::Cancel { pipeline } => {
            cancel_pipeline(&client, &pipeline).await?;
        }
        Commands::Retry { pipeline, step } => {
            retry_pipeline(&client, &pipeline, step).await?;
        }
        Commands::Logs { pipeline, step, follow } => {
            show_logs(&client, &pipeline, step, follow).await?;
        }
    }
    Ok(())
}

async fn show_status(client: &Client, pipeline_name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let pipelines: Api<Pipeline> = Api::namespaced(client.clone(), "default");
    
    let pipelines_list = if let Some(name) = pipeline_name {
        vec![pipelines.get(&name).await?]
    } else {
        pipelines.list(&ListParams::default()).await?.items
    };
    
    if pipelines_list.is_empty() {
        println!("No pipelines found");
        return Ok(());
    }
    
    let mut table = Table::new();
    table.set_header(vec!["Pipeline", "Phase", "Steps", "Duration", "Started"]);
    
    for pipeline in pipelines_list {
        let name = pipeline.name_any();
        let status = pipeline.status.as_ref();
        
        let phase = status.map(|s| s.phase.as_str()).unwrap_or("Unknown");
        let phase_cell = match phase {
            "Completed" => Cell::new(phase).fg(Color::Green),
            "Failed" => Cell::new(phase).fg(Color::Red),
            "Running" => Cell::new(phase).fg(Color::Yellow),
            _ => Cell::new(phase),
        };
        
        let steps = status.map(|s| 
            format!("{}/{}", s.completed_steps, s.total_steps)
        ).unwrap_or("0/0".to_string());
        
        let duration = calculate_duration(&pipeline);
        let started = pipeline.metadata.creation_timestamp.as_ref()
            .map(|t| format_timestamp(&t.0))
            .unwrap_or("Unknown".to_string());
        
        table.add_row(vec![
            Cell::new(name),
            phase_cell,
            Cell::new(steps),
            Cell::new(duration),
            Cell::new(started),
        ]);
    }
    
    println!("{}", table);
    Ok(())
}

async fn watch_pipelines(client: &Client, pipeline_name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Watching pipelines (Ctrl+C to exit)...\n");
    
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        show_status(client, pipeline_name.clone()).await?;
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn show_logs(
    client: &Client, 
    pipeline_name: &str, 
    step_name: Option<String>,
    follow: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), "default");
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), "default");
    
    let label_selector = if let Some(step) = step_name {
        format!("pipeline={},step={}", pipeline_name, step)
    } else {
        format!("pipeline={}", pipeline_name)
    };
    
    let jobs = jobs_api.list(&ListParams::default().labels(&label_selector)).await?;
    
    for job in jobs.items {
        let job_pods = pods_api.list(&ListParams::default()
            .labels(&format!("job-name={}", job.name_any()))).await?;
        
        for pod in job_pods.items {
            println!("📋 Logs for {} (step: {}):", pod.name_any(), 
                job.labels().get("step").unwrap_or(&"unknown".to_string()));
            
            let logs = pods_api.logs(&pod.name_any(), &LogParams {
                follow,
                ..Default::default()
            }).await?;
            
            println!("{}", logs);
            println!("---");
        }
    }
    Ok(())
}
```

## Day 8: Error Handling and Resilience

This is where you handle the real world - networks fail, nodes die, data is corrupt.

### Comprehensive Error Handling

```rust
#[derive(Debug, thiserror::Error)]
enum PipelineError {
    #[error("Step dependency failed: {step} depends on {dependency}")]
    DependencyFailed { step: String, dependency: String },
    
    #[error("Resource exhaustion: {resource} limit exceeded")]
    ResourceExhausted { resource: String },
    
    #[error("Data validation failed: {message}")]
    DataValidationFailed { message: String },
    
    #[error("Network timeout: {operation} took too long")]
    NetworkTimeout { operation: String },
    
    #[error("Authentication failed: {service}")]
    AuthenticationFailed { service: String },
}

async fn handle_job_failure(
    client: &Client,
    pipeline: &Pipeline,
    failed_job: &Job,
) -> Result<Action, Error> {
    let step_name = failed_job.labels().get("step").unwrap();
    let failure_count = failed_job.status.as_ref()
        .map(|s| s.failed.unwrap_or(0))
        .unwrap_or(0);
    
    println!("💥 Step {} failed (attempt {}/3)", step_name, failure_count);
    
    // Get the failed pod logs for debugging
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), 
        &pipeline.namespace().unwrap_or("default".to_string()));
    
    let failed_pods = pods_api.list(&ListParams::default()
        .labels(&format!("job-name={}", failed_job.name_any()))).await?;
    
    for pod in failed_pods.items {
        if let Some(container_statuses) = pod.status.as_ref()
            .and_then(|s| s.container_statuses.as_ref()) {
            
            for container in container_statuses {
                if let Some(state) = &container.state {
                    if let Some(terminated) = &state.terminated {
                        println!("🔍 Container {} failed with exit code: {}", 
                            container.name, terminated.exit_code);
                        
                        // Try to get logs for diagnosis
                        if let Ok(logs) = pods_api.logs(&pod.name_any(), &LogParams {
                            container: Some(container.name.clone()),
                            tail_lines: Some(50),
                            ..Default::default()
                        }).await {
                            println!("📄 Last 50 lines of logs:\n{}", logs);
                        }
                    }
                }
            }
        }
    }
    
    // Decide what to do based on the error
    let retry_strategy = determine_retry_strategy(&failed_job, failure_count);
    
    match retry_strategy {
        RetryStrategy::Retry => {
            println!("🔄 Retrying step {} (backoff: exponential)", step_name);
            // Delete the failed job so it can be recreated
            let jobs_api: Api<Job> = Api::namespaced(client.clone(), 
                &pipeline.namespace().unwrap_or("default".to_string()));
            jobs_api.delete(&failed_job.name_any(), &DeleteParams::default()).await?;
            Ok(Action::requeue(Duration::from_secs(60 * failure_count as u64)))
        }
        RetryStrategy::Escalate => {
            println!("🚨 Step {} failed permanently - manual intervention required", step_name);
            update_pipeline_status(client, pipeline, "Failed").await?;
            // Send alert to your monitoring system here
            send_failure_alert(pipeline, step_name, &failed_job).await?;
            Ok(Action::await_change())
        }
        RetryStrategy::Skip => {
            println!("⏭️  Skipping failed step {} and continuing pipeline", step_name);
            mark_step_as_skipped(client, pipeline, step_name).await?;
            Ok(Action::requeue(Duration::from_secs(10)))
        }
    }
}

#[derive(Debug)]
enum RetryStrategy {
    Retry,    // Try again with backoff
    Escalate, // Give up and alert humans
    Skip,     // Continue without this step
}

fn determine_retry_strategy(job: &Job, failure_count: i32) -> RetryStrategy {
    // Check exit codes and error patterns
    if let Some(status) = &job.status {
        if let Some(conditions) = &status.conditions {
            for condition in conditions {
                if condition.type_ == "Failed" {
                    if let Some(reason) = &condition.reason {
                        match reason.as_str() {
                            "BackoffLimitExceeded" => return RetryStrategy::Escalate,
                            "DeadlineExceeded" => return RetryStrategy::Retry,
                            _ => {}
                        }
                    }
                    
                    // Check error message for specific patterns
                    if let Some(message) = &condition.message {
                        if message.contains("Authentication failed") {
                            return RetryStrategy::Escalate; // Don't retry auth failures
                        }
                        if message.contains("Out of memory") {
                            return RetryStrategy::Retry; // Might work on different node
                        }
                        if message.contains("Data not found") && failure_count > 1 {
                            return RetryStrategy::Skip; // Optional data
                        }
                    }
                }
            }
        }
    }
    
    // Default strategy based on failure count
    if failure_count >= 3 {
        RetryStrategy::Escalate
    } else {
        RetryStrategy::Retry
    }
}

async fn send_failure_alert(
    pipeline: &Pipeline, 
    step_name: &str, 
    job: &Job
) -> Result<(), Error> {
    // This is where you'd integrate with your alerting system
    println!("🔔 ALERT: Pipeline {} step {} failed permanently", 
        pipeline.name_any(), step_name);
    
    // Example: Send to Slack, PagerDuty, etc.
    let alert = json!({
        "pipeline": pipeline.name_any(),
        "step": step_name,
        "job": job.name_any(),
        "namespace": pipeline.namespace().unwrap_or("default".to_string()),
        "timestamp": Utc::now().to_rfc3339(),
        "severity": "high"
    });
    
    // Your alerting logic here
    // webhook_client.send_alert(alert).await?;
    
    Ok(())
}
```

## Day 9: The Fan-Out Deep Dive - Handling 10 Parallel Jobs

Now let's talk about what happens when your fan-out step needs to create 10 parallel jobs and how you monitor them.

### Fan-Out Job Management

```rust
// This is the sophisticated version that handles complex fan-out scenarios
fn build_advanced_fanout_job(pipeline: &Pipeline, step: &Step) -> Job {
    let parallelism = step.config["parallelism"].as_u64().unwrap_or(10);
    let chunk_strategy = step.config.get("chunk_strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("size-based"); // "size-based", "count-based", "hash-based"
    
    let resources = step.config.get("resources").cloned().unwrap_or(json!({
        "cpu": "2",
        "memory": "4Gi"
    }));
    
    serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": format!("{}-{}", pipeline.name_any(), step.name),
            "labels": {
                "pipeline": pipeline.name_any(),
                "step": step.name,
                "step-type": "fan-out"
            },
            "annotations": {
                "parallelism": parallelism.to_string(),
                "chunk-strategy": chunk_strategy
            }
        },
        "spec": {
            "parallelism": parallelism,
            "completions": parallelism,
            "backoffLimit": (parallelism / 2).max(3), // Allow some failures
            "completionMode": "Indexed", // Each pod gets a unique index
            "template": {
                "spec": {
                    "containers": [{
                        "name": "worker",
                        "image": "your-registry/parallel-processor:v2.0",
                        "env": [
                            {"name": "WORKER_COUNT", "value": parallelism.to_string()},
                            {"name": "CHUNK_STRATEGY", "value": chunk_strategy},
                            {"name": "PIPELINE_ID", "value": pipeline.name_any()},
                            {"name": "STEP_NAME", "value": &step.name},
                            // This gives each pod a unique index (0, 1, 2, ..., 9)
                            {"name": "JOB_COMPLETION_INDEX", "valueFrom": {
                                "fieldRef": {"fieldPath": "metadata.annotations['batch.kubernetes.io/job-completion-index']"}
                            }}
                        ],
                        "resources": {
                            "requests": resources["requests"].clone(),
                            "limits": resources["limits"].clone()
                        }
                    }],
                    "restartPolicy": "Never",
                    // Spread pods across different nodes for better performance
                    "affinity": {
                        "podAntiAffinity": {
                            "preferredDuringSchedulingIgnoredDuringExecution": [{
                                "weight": 100,
                                "podAffinityTerm": {
                                    "labelSelector": {
                                        "matchExpressions": [{
                                            "key": "job-name",
                                            "operator": "In",
                                            "values": [format!("{}-{}", pipeline.name_any(), step.name)]
                                        }]
                                    },
                                    "topologyKey": "kubernetes.io/hostname"
                                }
                            }]
                        }
                    }
                }
            }
        }
    })).unwrap()
}

// This monitors the fan-out job and gives you detailed progress
async fn monitor_fanout_progress(
    client: &Client,
    pipeline: &Pipeline,
    job: &Job,
) -> Result<FanOutStatus, Error> {
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), 
        &pipeline.namespace().unwrap_or("default".to_string()));
    
    let job_pods = pods_api.list(&ListParams::default()
        .labels(&format!("job-name={}", job.name_any()))).await?;
    
    let mut worker_statuses = Vec::new();
    
    for pod in job_pods.items {
        let index = pod.annotations().get("batch.kubernetes.io/job-completion-index")
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or(0);
        
        let status = match pod.status.as_ref() {
            Some(status) => {
                match &status.phase {
                    Some(phase) => {
                        match phase.as_str() {
                            "Succeeded" => WorkerStatus::Completed,
                            "Failed" => WorkerStatus::Failed,
                            "Running" => WorkerStatus::Running,
                            _ => WorkerStatus::Pending,
                        }
                    }
                    None => WorkerStatus::Pending,
                }
            }
            None => WorkerStatus::Pending,
        };
        
        // Try to get progress from pod logs if running
        let progress = if status == WorkerStatus::Running {
            get_worker_progress(client, &pod).await.unwrap_or(0.0)
        } else {
            match status {
                WorkerStatus::Completed => 100.0,
                WorkerStatus::Failed => 0.0,
                _ => 0.0,
            }
        };
        
        worker_statuses.push(WorkerInfo {
            index,
            status,
            progress,
            pod_name: pod.name_any(),
            node_name: pod.spec.as_ref()
                .and_then(|s| s.node_name.clone())
                .unwrap_or("unknown".to_string()),
        });
    }
    
    // Sort by index for consistent display
    worker_statuses.sort_by_key(|w| w.index);
    
    let total_progress = worker_statuses.iter()
        .map(|w| w.progress)
        .sum::<f64>() / worker_statuses.len() as f64;
    
    Ok(FanOutStatus {
        total_workers: worker_statuses.len(),
        completed_workers: worker_statuses.iter().filter(|w| w.status == WorkerStatus::Completed).count(),
        failed_workers: worker_statuses.iter().filter(|w| w.status == WorkerStatus::Failed).count(),
        running_workers: worker_statuses.iter().filter(|w| w.status == WorkerStatus::Running).count(),
        total_progress,
        worker_details: worker_statuses,
    })
}

#[derive(Debug, Clone)]
struct FanOutStatus {
    total_workers: usize,
    completed_workers: usize,
    failed_workers: usize,
    running_workers: usize,
    total_progress: f64,
    worker_details: Vec<WorkerInfo>,
}

#[derive(Debug, Clone)]
struct WorkerInfo {
    index: u32,
    status: WorkerStatus,
    progress: f64,
    pod_name: String,
    node_name: String,
}

#[derive(Debug, Clone, PartialEq)]
enum WorkerStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

// This attempts to extract progress from worker logs
async fn get_worker_progress(client: &Client, pod: &Pod) -> Result<f64, Error> {
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), 
        &pod.namespace().unwrap_or("default".to_string()));
    
    // Look for progress indicators in the logs
    let logs = pods_api.logs(&pod.name_any(), &LogParams {
        tail_lines: Some(10),
        ..Default::default()
    }).await?;
    
    // Parse progress from logs (assuming your worker outputs progress)
    // Example log line: "PROGRESS: 45.2% completed"
    if let Some(progress_line) = logs.lines()
        .find(|line| line.contains("PROGRESS:")) {
        
        if let Some(percentage) = progress_line
            .split_whitespace()
            .find(|word| word.ends_with('%'))
            .and_then(|p| p.trim_end_matches('%').parse::<f64>().ok()) {
            return Ok(percentage);
        }
    }
    
    Ok(0.0)
}
```

### What You See as the Operator

When you're monitoring a fan-out job, here's what your experience looks like:

```bash
# You start the pipeline
$ pipeline-ctl run --config my-big-job.yaml
🚀 Pipeline big-data-processing-1234 started

# You check the status
$ pipeline-ctl status --pipeline big-data-processing-1234 --watch

Pipeline: big-data-processing-1234
Phase: Running
Steps: 2/3 completed
Current Step: fan-out-process (10 workers)

Worker Status:
┌───────┬──────────┬──────────┬──────────────┬─────────────────┐
│ Index │ Status   │ Progress │ Node         │ Pod Name        │
├───────┼──────────┼──────────┼──────────────┼─────────────────┤
│ 0     │ Complete │ 100.0%   │ worker-01    │ process-abc-0   │
│ 1     │ Complete │ 100.0%   │ worker-02    │ process-abc-1   │
│ 2     │ Running  │ 67.3%    │ worker-03    │ process-abc-2   │
│ 3     │ Running  │ 82.1%    │ worker-01    │ process-abc-3   │
│ 4     │ Complete │ 100.0%   │ worker-04    │ process-abc-4   │
│ 5     │ Running  │ 45.9%    │ worker-02    │ process-abc-5   │
│ 6     │ Failed   │ 0.0%     │ worker-05    │ process-abc-6   │
│ 7     │ Complete │ 100.0%   │ worker-03    │ process-abc-7   │
│ 8     │ Running  │ 91.2%    │ worker-04    │ process-abc-8   │
│ 9     │ Pending  │ 0.0%     │ -            │ process-abc-9   │
└───────┴──────────┴──────────┴──────────────┴─────────────────┘

Overall Progress: 71.6%
ETA: ~8 minutes

# You notice worker 6 failed - investigate
$ pipeline-ctl logs --pipeline big-data-processing-1234 --step fan-out-process
📋 Logs for process-abc-6 (step: fan-out-process):
2024-01-15T10:30:15Z INFO Starting worker 6 of 10
2024-01-15T10:30:16Z INFO Processing chunk range: 600000-699999
2024-01-15T10:30:45Z ERROR Out of memory: cannot allocate 2GB
2024-01-15T10:30:45Z FATAL Worker exiting with code 137

# The operator automatically retries the failed worker on a different node
# Worker 9 picks up the failed work
```

### Fan-Out Failure Handling

```rust
async fn handle_fanout_failure(
    client: &Client,
    pipeline: &Pipeline,
    job: &Job,
    fanout_status: &FanOutStatus,
) -> Result<Action, Error> {
    let failure_threshold = 0.3; // Allow 30% of workers to fail
    let failure_rate = fanout_status.failed_workers as f64 / fanout_status.total_workers as f64;
    
    if failure_rate > failure_threshold {
        println!("🚨 Fan-out job has too many failures ({:.1}% > {:.1}%)", 
            failure_rate * 100.0, failure_threshold * 100.0);
        
        // Cancel the whole fan-out and restart with different configuration
        let jobs_api: Api<Job> = Api::namespaced(client.clone(), 
            &pipeline.namespace().unwrap_or("default".to_string()));
        
        jobs_api.delete(&job.name_any(), &DeleteParams {
            propagation_policy: Some(PropagationPolicy::Foreground),
            ..Default::default()
        }).await?;
        
        // Restart with reduced parallelism and higher memory
        println!("🔄 Restarting fan-out with reduced parallelism and more memory");
        
        return Ok(Action::requeue(Duration::from_secs(60)));
    }
    
    // Handle individual worker failures
    for worker in &fanout_status.worker_details {
        if worker.status == WorkerStatus::Failed {
            println!("🔧 Investigating failed worker {}", worker.index);
            
            // Check if it's a resource issue
            if let Ok(pod_info) = get_pod_failure_reason(client, &worker.pod_name).await {
                match pod_info.failure_reason.as_str() {
                    "OutOfMemory" => {
                        println!("💾 Worker {} failed due to memory - restarting with more memory", worker.index);
                        // You could create a replacement job with higher memory here
                    }
                    "NodeNotReady" => {
                        println!("🖥️  Worker {} failed due to node issues - will retry on different node", worker.index);
                        // The Job controller will automatically retry on a different node
                    }
                    _ => {
                        println!("❓ Worker {} failed for unknown reason: {}", worker.index, pod_info.failure_reason);
                    }
                }
            }
        }
    }
    
    Ok(Action::requeue(Duration::from_secs(30)))
}

#[derive(Debug)]
struct PodFailureInfo {
    failure_reason: String,
    exit_code: Option<i32>,
    last_logs: String,
}

async fn get_pod_failure_reason(client: &Client, pod_name: &str) -> Result<PodFailureInfo, Error> {
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), "default");
    let pod = pods_api.get(pod_name).await?;
    
    let mut failure_reason = "Unknown".to_string();
    let mut exit_code = None;
    
    if let Some(status) = pod.status {
        if let Some(container_statuses) = status.container_statuses {
            for container in container_statuses {
                if let Some(state) = container.state {
                    if let Some(terminated) = state.terminated {
                        exit_code = Some(terminated.exit_code);
                        failure_reason = terminated.reason.unwrap_or("ContainerTerminated".to_string());
                    }
                }
            }
        }
    }
    
    let last_logs = pods_api.logs(pod_name, &LogParams {
        tail_lines: Some(20),
        ..Default::default()
    }).await.unwrap_or_else(|_| "No logs available".to_string());
    
    Ok(PodFailureInfo {
        failure_reason,
        exit_code,
        last_logs,
    })
}
```

## Day 10: Your Complete Operating Experience

Here's what your daily workflow looks like as the pipeline operator:

### Morning Routine - Checking Overnight Jobs

```bash
# First thing you do - check what happened overnight
$ pipeline-ctl status
┌─────────────────────────────┬───────────┬───────┬──────────┬─────────────────────┐
│ Pipeline                    │ Phase     │ Steps │ Duration │ Started             │
├─────────────────────────────┼───────────┼───────┼──────────┼─────────────────────┤
│ nightly-etl-2024-01-15     │ Completed │ 4/4   │ 2h 15m   │ 2024-01-15 02:00:00 │
│ weekly-aggregation-w03     │ Running   │ 2/3   │ 45m      │ 2024-01-15 07:30:00 │
│ customer-export-urgent     │ Failed    │ 1/2   │ 12m      │ 2024-01-15 08:15:00 │
└─────────────────────────────┴───────────┴───────┴──────────┴─────────────────────┘

# Good - nightly ETL completed successfully
# Weekly aggregation is still running (normal)
# Urgent export failed - need to investigate

$ pipeline-ctl logs --pipeline customer-export-urgent
📋 Logs for customer-export-urgent-fetch (step: fetch):
2024-01-15T08:15:23Z ERROR SFTP connection failed: timeout after 30s
2024-01-15T08:15:23Z ERROR Host customer-sftp.example.com unreachable

# Network issue - check with ops team, then retry
$ pipeline-ctl retry --pipeline customer-export-urgent
🔄 Retrying pipeline customer-export-urgent...
✅ Pipeline restarted successfully
```

### Mid-Day - Starting New Jobs

```bash
# Marketing team needs a custom report
$ cat marketing-analysis.yaml
apiVersion: dataworks.io/v1
kind: Pipeline
metadata:
  name: marketing-q1-analysis
spec:
  steps:
  - name: fetch-campaign-data
    type: fetch
    config:
      source: "campaigns_db"
      query: "SELECT * FROM campaigns WHERE quarter = 'Q1' AND year = 2024"
  - name: join-user-data
    type: transform
    dependsOn: ["fetch-campaign-data"]
    config:
      sql: |
        SELECT c.*, u.segment, u.ltv 
        FROM campaigns c 
        JOIN users u ON c.user_id = u.id
  - name: generate-insights
    type: fan-out
    dependsOn: ["join-user-data"]
    config:
      parallelism: 5
      script: "python analyze_segments.py"

$ kubectl apply -f marketing-analysis.yaml
pipeline.dataworks.io/marketing-q1-analysis created

$ pipeline-ctl status --pipeline marketing-q1-analysis --watch
🔄 Watching pipeline marketing-q1-analysis...

Pipeline: marketing-q1-analysis
Phase: Running
Steps: 1/3 completed
Current Step: join-user-data

Step Details:
✅ fetch-campaign-data (completed in 3m 45s)
🔄 join-user-data (running for 2m 12s)
⏳ generate-insights (waiting for dependencies)
```

### Afternoon - Handling Scale Issues

```bash
# The marketing job hit a snag
$ pipeline-ctl status --pipeline marketing-q1-analysis
Pipeline: marketing-q1-analysis
Phase: Running
Steps: 2/3 completed
Current Step: generate-insights (5 workers)

Worker Status:
┌───────┬──────────┬──────────┬──────────────┬─────────────────┐
│ Index │ Status   │ Progress │ Node         │ Pod Name        │
├───────┼──────────┼──────────┼──────────────┼─────────────────┤
│ 0     │ Complete │ 100.0%   │ worker-01    │ insights-xyz-0  │
│ 1     │ Complete │ 100.0%   │ worker-02    │ insights-xyz-1  │
│ 2     │ Failed   │ 15.3%    │ worker-03    │ insights-xyz-2  │
│ 3     │ Running  │ 78.9%    │ worker-04    │ insights-xyz-3  │
│ 4     │ Running  │ 82.1%    │ worker-01    │ insights-xyz-4  │
└───────┴──────────┴──────────┴──────────────┴─────────────────┘

# Worker 2 failed - let's see why
$ kubectl describe pod insights-xyz-2
...
Events:
  Type     Reason     Message
  ----     ------     -------
  Warning  Failed     Pod had insufficient memory, was OOMKilled

# Memory issue - the operator will automatically retry, but let's give it more resources
$ kubectl patch pipeline marketing-q1-analysis --type='merge' -p='
{
  "spec": {
    "steps": [
      {
        "name": "generate-insights",
        "config": {
          "resources": {
            "requests": {"memory": "4Gi", "cpu": "1"},
            "limits": {"memory": "8Gi", "cpu": "2"}
          }
        }
      }
    ]
  }
}'

# The operator sees the change and will apply new resource limits to future workers
```

### End of Day - Production Pipeline

```bash
# Time to run the big daily processing job
$ cat daily-production.yaml
apiVersion: dataworks.io/v1
kind: Pipeline
metadata:
  name: production-etl-2024-01-15
spec:
  steps:
  - name: extract-orders
    type: fetch
    config:
      source: "production_db"
      tables: ["orders", "order_items", "customers"]
      partition_date: "2024-01-15"
  
  - name: extract-events
    type: fetch
    config:
      source: "events_s3"
      prefix: "year=2024/month=01/day=15/"
  
  - name: clean-and-validate
    type: transform
    dependsOn: ["extract-orders", "extract-events"]
    config:
      validation_rules: "strict"
      sql: |
        -- Data quality checks and cleaning
        DELETE FROM raw_orders WHERE total_amount < 0;
        DELETE FROM raw_events WHERE user_id IS NULL;
  
  - name: build-fact-tables
    type: transform
    dependsOn: ["clean-and-validate"]
    config:
      sql: "transform_to_star_schema.sql"
  
  - name: update-aggregates
    type: fan-out
    dependsOn: ["build-fact-tables"]
    config:
      parallelism: 20  # High parallelism for the big aggregation job
      chunk_strategy: "hash-based"
      resources:
        requests: {"memory": "8Gi", "cpu": "4"}
        limits: {"memory": "16Gi", "cpu": "8"}
  
  - name: publish-to-warehouse
    type: load
    dependsOn: ["update-aggregates"]
    config:
      destination: "bigquery"
      dataset: "analytics_prod"
      mode: "overwrite"

$ kubectl apply -f daily-production.yaml
pipeline.dataworks.io/production-etl-2024-01-15 created

# This is the big one - 20 parallel workers processing terabytes of data
$ pipeline-ctl status --pipeline production-etl-2024-01-15 --watch
```

### Emergency Handling

```bash
# Oh no! The production pipeline failed
$ pipeline-ctl status --pipeline production-etl-2024-01-15
Pipeline: production-etl-2024-01-15
Phase: Failed
Steps: 3/6 completed
Failed Step: update-aggregates

# Check what happened
$ pipeline-ctl logs --pipeline production-etl-2024-01-15 --step update-aggregates
📋 Worker failures detected:
- 8 out of 20 workers failed with "disk full" errors
- 3 workers failed with "connection timeout" to BigQuery
- Remaining 9 workers completed successfully

# The cluster is under stress - need to restart with better resource management
$ kubectl get nodes -o wide
NAME        STATUS   CPU%   MEMORY%   DISK%
worker-01   Ready    95%    85%       98%    ← This node is full
worker-02   Ready    78%    72%       45%
worker-03   Ready    82%    69%       95%    ← This one too
worker-04   Ready    45%    42%       30%
worker-05   Ready    38%    35%       25%

# Clean up some disk space and restart the failed pipeline
$ kubectl delete jobs -l pipeline=production-etl-2024-01-15,step=update-aggregates
$ pipeline-ctl retry --pipeline production-etl-2024-01-15 --step update-aggregates

# This time, the operator will avoid the full nodes and use the healthy ones
```

## The Complete Mental Model

Think of yourself as running a smart factory where:

1. **You're the Foreman**: You create work orders (Pipelines) and check status
2. **The Operator is Your Supervisor**: It watches everything and makes decisions automatically
3. **Jobs are Work Stations**: Each step becomes a work station with specific tools
4. **Pods are Workers**: The actual people doing the work
5. **Fan-Out is Team Work**: When you need 10 people working on the same big task
6. **Failures are Normal**: Equipment breaks, people get sick - you handle it and move on
7. **Monitoring is Essential**: You always know what's running, what's broken, and what's next

The beauty of this system is that it scales from small jobs (1-2 steps) to massive data processing pipelines (hundreds of parallel workers), and you have complete visibility and control at every level.

## Day 11: Advanced Patterns - Real-World Scenarios

### Conditional Steps and Dynamic Pipelines

Sometimes you need pipelines that adapt based on the data they process:

```rust
// This handles conditional execution - steps that only run under certain conditions
async fn evaluate_step_conditions(
    client: &Client,
    pipeline: &Pipeline,
    step: &Step,
) -> Result<bool, Error> {
    // Check if this step has conditions
    if let Some(conditions) = step.config.get("conditions") {
        for condition in conditions.as_array().unwrap() {
            let condition_type = condition["type"].as_str().unwrap();
            
            match condition_type {
                "data_size_threshold" => {
                    let threshold = condition["min_size_gb"].as_f64().unwrap();
                    let actual_size = get_data_size_gb(client, pipeline, step).await?;
                    
                    if actual_size < threshold {
                        println!("⏭️  Skipping {} - data size {:.2}GB below threshold {:.2}GB", 
                            step.name, actual_size, threshold);
                        return Ok(false);
                    }
                }
                "business_hours_only" => {
                    let now = chrono::Utc::now();
                    let hour = now.hour();
                    
                    if hour < 9 || hour > 17 {
                        println!("⏭️  Skipping {} - outside business hours ({}:00)", 
                            step.name, hour);
                        return Ok(false);
                    }
                }
                "previous_step_result" => {
                    let required_result = condition["required_result"].as_str().unwrap();
                    let previous_step = condition["step"].as_str().unwrap();
                    
                    let previous_result = get_step_result(client, pipeline, previous_step).await?;
                    if previous_result != required_result {
                        println!("⏭️  Skipping {} - previous step {} result was '{}', not '{}'", 
                            step.name, previous_step, previous_result, required_result);
                        return Ok(false);
                    }
                }
                _ => {
                    println!("⚠️  Unknown condition type: {}", condition_type);
                }
            }
        }
    }
    
    Ok(true)
}

async fn get_data_size_gb(
    client: &Client, 
    pipeline: &Pipeline, 
    step: &Step
) -> Result<f64, Error> {
    // This would check your data store (S3, GCS, etc.) to see how much data is available
    // For demo purposes, we'll simulate it
    let data_location = step.config["input_path"].as_str().unwrap_or("unknown");
    
    // In reality, you'd call your cloud storage API here
    // let size_bytes = storage_client.get_object_size(data_location).await?;
    // Ok(size_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    
    // Simulated response
    Ok(15.7) // GB
}
```

### Pipeline Templates and Inheritance

For common patterns, you want reusable templates:

```yaml
# base-etl-template.yaml
apiVersion: dataworks.io/v1
kind: PipelineTemplate
metadata:
  name: standard-etl
spec:
  parameters:
  - name: source_system
    type: string
    required: true
  - name: target_date
    type: string
    default: "today"
  - name: parallelism
    type: integer
    default: 5
  
  steps:
  - name: extract-{{.source_system}}
    type: fetch
    config:
      source: "{{.source_system}}"
      date: "{{.target_date}}"
      
  - name: validate-data
    type: transform
    dependsOn: ["extract-{{.source_system}}"]
    config:
      validation_rules: "standard"
      
  - name: load-warehouse
    type: fan-out
    dependsOn: ["validate-data"]
    config:
      parallelism: "{{.parallelism}}"
      destination: "data_warehouse"

---
# Your actual pipeline uses the template
apiVersion: dataworks.io/v1
kind: Pipeline
metadata:
  name: salesforce-daily-sync
spec:
  template: standard-etl
  parameters:
    source_system: "salesforce"
    target_date: "2024-01-15"
    parallelism: 10
```

### Cross-Pipeline Dependencies

Sometimes pipelines depend on other pipelines:

```rust
// This checks if prerequisite pipelines have completed
async fn check_pipeline_dependencies(
    client: &Client,
    pipeline: &Pipeline,
) -> Result<bool, Error> {
    if let Some(dependencies) = pipeline.spec.pipeline_dependencies.as_ref() {
        let pipelines_api: Api<Pipeline> = Api::namespaced(client.clone(), 
            &pipeline.namespace().unwrap_or("default".to_string()));
            
        for dep in dependencies {
            match pipelines_api.get(&dep.name).await {
                Ok(dep_pipeline) => {
                    let status = dep_pipeline.status.as_ref()
                        .map(|s| s.phase.as_str())
                        .unwrap_or("Unknown");
                        
                    if status != "Completed" {
                        println!("⏳ Waiting for prerequisite pipeline {} (status: {})", 
                            dep.name, status);
                        return Ok(false);
                    }
                    
                    // Check if it completed recently enough
                    if let Some(max_age_hours) = dep.max_age_hours {
                        if let Some(completion_time) = dep_pipeline.status.as_ref()
                            .and_then(|s| s.completion_time.as_ref()) {
                            
                            let completed_at = chrono::DateTime::parse_from_rfc3339(completion_time)?;
                            let age_hours = chrono::Utc::now().signed_duration_since(completed_at).num_hours();
                            
                            if age_hours > max_age_hours as i64 {
                                println!("⚠️  Prerequisite pipeline {} is too old ({} hours > {} hours)", 
                                    dep.name, age_hours, max_age_hours);
                                return Ok(false);
                            }
                        }
                    }
                }
                Err(_) => {
                    println!("❌ Prerequisite pipeline {} not found", dep.name);
                    return Ok(false);
                }
            }
        }
    }
    
    Ok(true)
}
```

## Day 12: Production Monitoring and Alerting

### Comprehensive Monitoring Dashboard

```rust
// This creates metrics that your monitoring system (Prometheus, etc.) can scrape
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};

lazy_static! {
    static ref PIPELINE_STARTS: Counter = register_counter!(
        "pipeline_starts_total", 
        "Total number of pipelines started"
    ).unwrap();
    
    static ref PIPELINE_DURATION: Histogram = register_histogram!(
        "pipeline_duration_seconds",
        "Pipeline execution duration in seconds"
    ).unwrap();
    
    static ref ACTIVE_PIPELINES: Gauge = register_gauge!(
        "active_pipelines",
        "Number of currently active pipelines"
    ).unwrap();
    
    static ref FAILED_STEPS: Counter = register_counter!(
        "pipeline_step_failures_total",
        "Total number of step failures"
    ).unwrap();
}

async fn update_metrics(client: &Client) -> Result<(), Error> {
    let pipelines: Api<Pipeline> = Api::all(client.clone());
    let all_pipelines = pipelines.list(&ListParams::default()).await?;
    
    let mut active_count = 0;
    let mut total_duration = 0.0;
    
    for pipeline in all_pipelines.items {
        if let Some(status) = &pipeline.status {
            match status.phase.as_str() {
                "Running" => active_count += 1,
                "Completed" => {
                    if let (Some(start), Some(end)) = (&status.start_time, &status.completion_time) {
                        let start_time = chrono::DateTime::parse_from_rfc3339(start)?;
                        let end_time = chrono::DateTime::parse_from_rfc3339(end)?;
                        let duration = end_time.signed_duration_since(start_time).num_seconds() as f64;
                        
                        PIPELINE_DURATION.observe(duration);
                        total_duration += duration;
                    }
                }
                "Failed" => {
                    FAILED_STEPS.inc_by(status.failed_steps as f64);
                }
                _ => {}
            }
        }
    }
    
    ACTIVE_PIPELINES.set(active_count as f64);
    Ok(())
}

// Health check endpoint for your operator
async fn health_check() -> Result<impl warp::Reply, warp::Rejection> {
    let health = json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "active_watchers": get_active_watcher_count(),
        "last_reconcile": get_last_reconcile_time(),
    });
    
    Ok(warp::reply::json(&health))
}
```

### Smart Alerting Rules

```rust
// This implements intelligent alerting - not just "something failed" but "something needs attention"
#[derive(Debug, Clone)]
struct AlertRule {
    name: String,
    condition: AlertCondition,
    severity: AlertSeverity,
    cooldown_minutes: u32,
    last_fired: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
enum AlertCondition {
    PipelineStuckFor { minutes: u32 },
    HighFailureRate { threshold: f64, window_minutes: u32 },
    ResourceExhaustion { resource: String, threshold: f64 },
    UnusualDuration { pipeline_pattern: String, multiplier: f64 },
}

#[derive(Debug, Clone)]
enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

async fn check_alert_rules(client: &Client, rules: &mut [AlertRule]) -> Result<(), Error> {
    for rule in rules {
        // Skip if still in cooldown
        if let Some(last_fired) = rule.last_fired {
            let cooldown = chrono::Duration::minutes(rule.cooldown_minutes as i64);
            if chrono::Utc::now() - last_fired < cooldown {
                continue;
            }
        }
        
        let should_alert = match &rule.condition {
            AlertCondition::PipelineStuckFor { minutes } => {
                check_stuck_pipelines(client, *minutes).await?
            }
            AlertCondition::HighFailureRate { threshold, window_minutes } => {
                check_failure_rate(client, *threshold, *window_minutes).await?
            }
            AlertCondition::ResourceExhaustion { resource, threshold } => {
                check_resource_usage(client, resource, *threshold).await?
            }
            AlertCondition::UnusualDuration { pipeline_pattern, multiplier } => {
                check_unusual_duration(client, pipeline_pattern, *multiplier).await?
            }
        };
        
        if should_alert {
            send_alert(&rule).await?;
            rule.last_fired = Some(chrono::Utc::now());
        }
    }
    
    Ok(())
}

async fn check_stuck_pipelines(client: &Client, max_minutes: u32) -> Result<bool, Error> {
    let pipelines: Api<Pipeline> = Api::all(client.clone());
    let running_pipelines = pipelines.list(&ListParams::default()
        .labels("phase=Running")).await?;
    
    let max_duration = chrono::Duration::minutes(max_minutes as i64);
    
    for pipeline in running_pipelines.items {
        if let Some(status) = &pipeline.status {
            if let Some(start_time) = &status.start_time {
                let started_at = chrono::DateTime::parse_from_rfc3339(start_time)?;
                let running_for = chrono::Utc::now().signed_duration_since(started_at);
                
                if running_for > max_duration {
                    println!("🐌 Pipeline {} has been running for {} minutes (threshold: {})", 
                        pipeline.name_any(), 
                        running_for.num_minutes(), 
                        max_minutes);
                    return Ok(true);
                }
            }
        }
    }
    
    Ok(false)
}

async fn send_alert(rule: &AlertRule) -> Result<(), Error> {
    let alert_payload = json!({
        "alert_name": rule.name,
        "severity": format!("{:?}", rule.severity),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "condition": format!("{:?}", rule.condition),
    });
    
    match rule.severity {
        AlertSeverity::Critical => {
            // Send to PagerDuty, call oncall engineer
            println!("🚨 CRITICAL ALERT: {}", alert_payload);
            // pagerduty_client.trigger_incident(alert_payload).await?;
        }
        AlertSeverity::Warning => {
            // Send to Slack, create ticket
            println!("⚠️  WARNING: {}", alert_payload);
            // slack_client.send_message("#data-engineering", alert_payload).await?;
        }
        AlertSeverity::Info => {
            // Log to monitoring system
            println!("ℹ️  INFO: {}", alert_payload);
        }
    }
    
    Ok(())
}
```

## Day 13: Performance Optimization and Scaling

### Resource Management and Node Affinity

```rust
// This is how you optimize resource usage across your cluster
fn build_optimized_job(pipeline: &Pipeline, step: &Step) -> Job {
    let step_type = &step.step_type;
    let workload_class = determine_workload_class(step);
    
    let (node_selector, tolerations, affinity) = match workload_class {
        WorkloadClass::NetworkIntensive => {
            // Use nodes with fast network for SFTP, API calls
            (
                json!({"workload-type": "network", "network-speed": "10gbps"}),
                vec![json!({"key": "network-intensive", "operator": "Equal", "value": "true"})],
                None
            )
        }
        WorkloadClass::ComputeIntensive => {
            // Use CPU-optimized nodes for transformations
            (
                json!({"workload-type": "compute", "cpu-type": "high-performance"}),
                vec![json!({"key": "compute-intensive", "operator": "Equal", "value": "true"})],
                Some(json!({
                    "podAntiAffinity": {
                        "preferredDuringSchedulingIgnoredDuringExecution": [{
                            "weight": 100,
                            "podAffinityTerm": {
                                "labelSelector": {
                                    "matchExpressions": [{
                                        "key": "cpu-intensive",
                                        "operator": "In",
                                        "values": ["true"]
                                    }]
                                },
                                "topologyKey": "kubernetes.io/hostname"
                            }
                        }]
                    }
                }))
            )
        }
        WorkloadClass::MemoryIntensive => {
            // Use memory-optimized nodes for large datasets
            (
                json!({"workload-type": "memory", "memory-class": "xlarge"}),
                vec![json!({"key": "memory-intensive", "operator": "Equal", "value": "true"})],
                None
            )
        }
        WorkloadClass::IOIntensive => {
            // Use nodes with fast SSDs for data loading
            (
                json!({"workload-type": "storage", "disk-type": "nvme"}),
                vec![json!({"key": "io-intensive", "operator": "Equal", "value": "true"})],
                None
            )
        }
    };
    
    let resources = calculate_optimal_resources(step, &workload_class);
    
    serde_json::from_value(json!({
        "apiVersion": "batch/v1",
        "kind": "Job",
        "metadata": {
            "name": format!("{}-{}", pipeline.name_any(), step.name),
            "labels": {
                "pipeline": pipeline.name_any(),
                "step": step.name,
                "workload-class": format!("{:?}", workload_class).to_lowercase()
            }
        },
        "spec": {
            "template": {
                "spec": {
                    "nodeSelector": node_selector,
                    "tolerations": tolerations,
                    "affinity": affinity,
                    "containers": [{
                        "name": "worker",
                        "image": get_optimized_image(step_type),
                        "resources": resources,
                        // Add resource monitoring sidecar for large jobs
                        "env": [
                            {"name": "ENABLE_METRICS", "value": "true"},
                            {"name": "WORKLOAD_CLASS", "value": format!("{:?}", workload_class)}
                        ]
                    }],
                    "restartPolicy": "Never"
                }
            }
        }
    })).unwrap()
}

#[derive(Debug)]
enum WorkloadClass {
    NetworkIntensive,  // SFTP transfers, API calls
    ComputeIntensive,  // SQL transforms, ML inference
    MemoryIntensive,   // Large dataset joins, aggregations
    IOIntensive,       // File processing, data loading
}

fn determine_workload_class(step: &Step) -> WorkloadClass {
    match step.step_type {
        StepType::SftpFetch => WorkloadClass::NetworkIntensive,
        StepType::PostgresTransform => {
            // Check if it's a heavy transform
            if let Some(sql) = step.config.get("sql").and_then(|s| s.as_str()) {
                if sql.contains("JOIN") && sql.len() > 1000 {
                    WorkloadClass::MemoryIntensive
                } else {
                    WorkloadClass::ComputeIntensive
                }
            } else {
                WorkloadClass::ComputeIntensive
            }
        }
        StepType::OpensearchLoad => WorkloadClass::IOIntensive,
        StepType::FanOut => {
            // Determine based on what the fan-out is doing
            if let Some(operation) = step.config.get("operation").and_then(|s| s.as_str()) {
                match operation {
                    "file_processing" => WorkloadClass::IOIntensive,
                    "ml_inference" => WorkloadClass::ComputeIntensive,
                    "api_calls" => WorkloadClass::NetworkIntensive,
                    _ => WorkloadClass::ComputeIntensive,
                }
            } else {
                WorkloadClass::ComputeIntensive
            }
        }
    }
}

fn calculate_optimal_resources(step: &Step, workload_class: &WorkloadClass) -> serde_json::Value {
    let base_resources = match workload_class {
        WorkloadClass::NetworkIntensive => json!({
            "requests": {"cpu": "500m", "memory": "1Gi"},
            "limits": {"cpu": "1", "memory": "2Gi"}
        }),
        WorkloadClass::ComputeIntensive => json!({
            "requests": {"cpu": "2", "memory": "4Gi"},
            "limits": {"cpu": "8", "memory": "8Gi"}
        }),
        WorkloadClass::MemoryIntensive => json!({
            "requests": {"cpu": "1", "memory": "8Gi"},
            "limits": {"cpu": "4", "memory": "32Gi"}
        }),
        WorkloadClass::IOIntensive => json!({
            "requests": {"cpu": "1", "memory": "2Gi", "ephemeral-storage": "10Gi"},
            "limits": {"cpu": "4", "memory": "8Gi", "ephemeral-storage": "50Gi"}
        }),
    };
    
    // Override with step-specific requirements if provided
    if let Some(custom_resources) = step.config.get("resources") {
        merge_json_objects(&base_resources, custom_resources)
    } else {
        base_resources
    }
}
```

### Auto-Scaling Based on Pipeline Load

```rust
// This implements intelligent scaling based on pipeline queue depth and resource usage
async fn auto_scale_cluster(client: &Client) -> Result<(), Error> {
    let pipeline_load = analyze_pipeline_load(client).await?;
    let cluster_capacity = get_cluster_capacity(client).await?;
    
    let scaling_decision = make_scaling_decision(&pipeline_load, &cluster_capacity);
    
    match scaling_decision {
        ScalingDecision::ScaleUp { node_type, count } => {
            println!("📈 Scaling up: adding {} {} nodes", count, node_type);
            request_node_scale_up(&node_type, count).await?;
        }
        ScalingDecision::ScaleDown { node_type, count } => {
            println!("📉 Scaling down: removing {} {} nodes", count, node_type);
            request_node_scale_down(&node_type, count).await?;
        }
        ScalingDecision::NoAction => {
            println!("✅ Cluster capacity is optimal");
        }
    }
    
    Ok(())
}

#[derive(Debug)]
struct PipelineLoad {
    queued_pipelines: usize,
    running_pipelines: usize,
    pending_jobs: usize,
    average_queue_time_minutes: f64,
    workload_distribution: HashMap<WorkloadClass, usize>,
}

async fn analyze_pipeline_load(client: &Client) -> Result<PipelineLoad, Error> {
    let pipelines: Api<Pipeline> = Api::all(client.clone());
    let all_pipelines = pipelines.list(&ListParams::default()).await?;
    
    let mut queued = 0;
    let mut running = 0;
    let mut total_queue_time = 0.0;
    let mut workload_dist = HashMap::new();
    
    for pipeline in all_pipelines.items {
        if let Some(status) = &pipeline.status {
            match status.phase.as_str() {
                "Pending" => {
                    queued += 1;
                    if let Some(created) = &pipeline.metadata.creation_timestamp {
                        let queue_time = chrono::Utc::now()
                            .signed_duration_since(created.0)
                            .num_minutes() as f64;
                        total_queue_time += queue_time;
                    }
                }
                "Running" => running += 1,
                _ => {}
            }
            
            // Analyze workload types in the pipeline
            for step in &pipeline.spec.steps {
                let workload_class = determine_workload_class(step);
                *workload_dist.entry(workload_class).or_insert(0) += 1;
            }
        }
    }
    
    let jobs: Api<Job> = Api::all(client.clone());
    let pending_jobs = jobs.list(&ListParams::default()
        .field_selector("status.active=0")).await?.items.len();
    
    Ok(PipelineLoad {
        queued_pipelines: queued,
        running_pipelines: running,
        pending_jobs,
        average_queue_time_minutes: if queued > 0 { total_queue_time / queued as f64 } else { 0.0 },
        workload_distribution: workload_dist,
    })
}

#[derive(Debug)]
enum ScalingDecision {
    ScaleUp { node_type: String, count: u32 },
    ScaleDown { node_type: String, count: u32 },
    NoAction,
}

fn make_scaling_decision(load: &PipelineLoad, capacity: &ClusterCapacity) -> ScalingDecision {
    // Scale up if queue time is too long
    if load.average_queue_time_minutes > 10.0 && load.queued_pipelines > 5 {
        // Determine what type of nodes we need most
        let most_needed_workload = load.workload_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(workload, _)| workload);
            
        if let Some(workload) = most_needed_workload {
            let node_type = match workload {
                WorkloadClass::ComputeIntensive => "compute-optimized",
                WorkloadClass::MemoryIntensive => "memory-optimized", 
                WorkloadClass::IOIntensive => "storage-optimized",
                WorkloadClass::NetworkIntensive => "network-optimized",
            };
            
            let scale_count = (load.queued_pipelines / 3).max(1) as u32;
            return ScalingDecision::ScaleUp { 
                node_type: node_type.to_string(), 
                count: scale_count 
            };
        }
    }
    
    // Scale down if we have excess capacity
    if load.queued_pipelines == 0 && load.running_pipelines < 3 && capacity.idle_nodes > 2 {
        return ScalingDecision::ScaleDown { 
            node_type: "general-purpose".to_string(), 
            count: (capacity.idle_nodes / 2).max(1) 
        };
    }
    
    ScalingDecision::NoAction
}
```

## Your Complete Operator Mastery

Now you understand the complete system from the ground up. You're not just running `kubectl apply` and hoping for the best - you understand:

### The Mental Models
- **Control Loop**: Your operator is always watching, deciding, acting
- **State Reconciliation**: It continuously moves the cluster toward your desired state
- **Event-Driven**: Changes trigger actions, not scheduled polling
- **Failure Isolation**: One step failing doesn't break everything

### The Operational Reality
- **You kick off pipelines** with simple YAML files
- **The operator handles complexity** of job creation, scheduling, dependencies
- **You monitor progress** through CLI tools and dashboards
- **Failures are handled gracefully** with retries, alerts, and manual intervention points

### The Scale-Out Power
- **Single jobs** for simple tasks
- **Fan-out jobs** for parallel processing
- **Cross-pipeline dependencies** for complex workflows
- **Dynamic resource allocation** based on workload characteristics

### Your Daily Workflow
```bash
# Morning: Check overnight jobs
pipeline-ctl status

# Start new work
kubectl apply -f my-pipeline.yaml

# Monitor progress  
pipeline-ctl status --watch

# Handle failures
pipeline-ctl logs --pipeline failed-job
pipeline-ctl retry --pipeline failed-job

# Scale for big jobs
kubectl patch pipeline big-job --type='merge' -p='{"spec":{"steps":[{"name":"process","config":{"parallelism":50}}]}}'

# End of day: Production runs
kubectl apply -f production-daily-etl.yaml
```

You're now the master of a distributed data processing system that can handle anything from small reports to terabyte-scale analytics, with full visibility, control, and resilience. The operator handles the complexity while you focus on the business logic and data flow.

**Remember**: You built this from scratch. You understand every piece. When something breaks at 3 AM, you know exactly where to look and how to fix it. That's the difference between being a real engineer and just another YAML jockey.