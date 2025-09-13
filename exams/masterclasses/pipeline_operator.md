# Data Pipeline Operator - Active Recall Master Class

## Core Competency: Kubernetes Job Orchestration via Web API

Master building a production-ready Kubernetes operator that creates and manages Jobs based on Axum web API calls, with full lifecycle management and observability.

## Part 1: Kubernetes Client Mastery

### 1.1 Core Kubernetes Types

**Active Recall Challenge**: Master essential k8s client types and operations:

```rust
use kube::{
    Api, Client, Resource, ResourceExt,
    api::{Patch, PatchParams, PostParams, DeleteParams, WatchEvent, WatchParams},
    runtime::{controller::Action, watcher, Controller, finalizer},
};
use k8s_openapi::api::{
    batch::v1::{Job, JobSpec},
    core::v1::{Pod, Container, PodSpec, PodTemplateSpec},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// Essential client creation
async fn create_kubernetes_client() -> Result<Client, kube::Error> {
    // In-cluster config (for pods running in k8s)
    if let Ok(config) = kube::Config::infer().await {
        return Ok(Client::try_from(config)?);
    }
    
    // Fallback to kubeconfig
    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;
    Ok(Client::try_from(config)?)
}

// Core API operations
struct KubernetesOperator {
    client: Client,
    job_api: Api<Job>,
    pod_api: Api<Pod>,
    namespace: String,
}

impl KubernetesOperator {
    async fn new(namespace: String) -> Result<Self, kube::Error> {
        let client = create_kubernetes_client().await?;
        let job_api = Api::namespaced(client.clone(), &namespace);
        let pod_api = Api::namespaced(client.clone(), &namespace);
        
        Ok(Self {
            client,
            job_api,
            pod_api,
            namespace,
        })
    }
    
    // Create job from template
    async fn create_job(&self, job_request: JobRequest) -> Result<Job, OperatorError> {
        let job = self.build_job_spec(job_request)?;
        
        let created_job = self.job_api
            .create(&PostParams::default(), &job)
            .await?;
            
        tracing::info!(
            job_name = created_job.name_any(),
            "Created Kubernetes job"
        );
        
        Ok(created_job)
    }
    
    // Watch job status changes
    async fn watch_job_status(&self, job_name: &str) -> Result<JobStatus, OperatorError> {
        let job = self.job_api.get(job_name).await?;
        
        let status = job.status.as_ref().ok_or_else(|| {
            OperatorError::InvalidState("Job has no status".to_string())
        })?;
        
        Ok(JobStatus {
            active: status.active.unwrap_or(0),
            succeeded: status.succeeded.unwrap_or(0),
            failed: status.failed.unwrap_or(0),
            completion_time: status.completion_time.clone(),
            conditions: status.conditions.clone().unwrap_or_default(),
        })
    }
    
    // Delete completed jobs
    async fn cleanup_job(&self, job_name: &str) -> Result<(), OperatorError> {
        let delete_params = DeleteParams {
            propagation_policy: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::DeletionPropagation::Background),
            ..Default::default()
        };
        
        self.job_api.delete(job_name, &delete_params).await?;
        
        tracing::info!(job_name, "Deleted completed job");
        Ok(())
    }
}
```

### 1.2 Job Template System

**Active Recall Challenge**: Implement flexible job templating:

```rust
use k8s_openapi::api::{
    batch::v1::{Job, JobSpec}, 
    core::v1::{
        Container, PodSpec, PodTemplateSpec, 
        EnvVar, Volume, VolumeMount, ResourceRequirements,
        Secret, ConfigMap, PersistentVolumeClaim
    },
    meta::v1::ObjectMeta,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub resources: ResourceSpec,
    pub volumes: Vec<VolumeSpec>,
    pub backoff_limit: Option<i32>,
    pub ttl_seconds_after_finished: Option<i32>,
    pub parallelism: Option<i32>,
    pub completions: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu_request: String,
    pub memory_request: String,
    pub cpu_limit: String,
    pub memory_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub name: String,
    pub mount_path: String,
    pub volume_type: VolumeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeType {
    ConfigMap { name: String },
    Secret { name: String },
    PersistentVolume { claim_name: String },
    EmptyDir,
}

impl KubernetesOperator {
    fn build_job_spec(&self, request: JobRequest) -> Result<Job, OperatorError> {
        let job_name = format!("{}-{}", request.name, uuid::Uuid::new_v4().to_string()[..8].to_string());
        
        // Build container spec
        let container = Container {
            name: "job-container".to_string(),
            image: Some(request.image),
            command: if request.command.is_empty() { None } else { Some(request.command) },
            args: if request.args.is_empty() { None } else { Some(request.args) },
            env: Some(self.build_env_vars(request.env_vars)?),
            volume_mounts: Some(self.build_volume_mounts(&request.volumes)?),
            resources: Some(self.build_resource_requirements(request.resources)?),
            ..Default::default()
        };
        
        // Build pod spec
        let pod_spec = PodSpec {
            containers: vec![container],
            restart_policy: Some("Never".to_string()),
            volumes: Some(self.build_volumes(&request.volumes)?),
            ..Default::default()
        };
        
        // Build job spec
        let job_spec = JobSpec {
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(self.build_job_labels(&request)?),
                    ..Default::default()
                }),
                spec: Some(pod_spec),
            },
            backoff_limit: request.backoff_limit,
            ttl_seconds_after_finished: request.ttl_seconds_after_finished,
            parallelism: request.parallelism,
            completions: request.completions,
            ..Default::default()
        };
        
        // Build complete job
        Ok(Job {
            metadata: ObjectMeta {
                name: Some(job_name.clone()),
                namespace: Some(self.namespace.clone()),
                labels: Some(self.build_job_labels(&request)?),
                annotations: Some(self.build_job_annotations(&request)?),
                ..Default::default()
            },
            spec: Some(job_spec),
            ..Default::default()
        })
    }
    
    fn build_env_vars(&self, env_vars: HashMap<String, String>) -> Result<Vec<EnvVar>, OperatorError> {
        let mut envs: Vec<EnvVar> = env_vars
            .into_iter()
            .map(|(name, value)| EnvVar {
                name,
                value: Some(value),
                ..Default::default()
            })
            .collect();
        
        // Add standard environment variables
        envs.extend(vec![
            EnvVar {
                name: "POD_NAME".to_string(),
                value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                    field_ref: Some(k8s_openapi::api::core::v1::ObjectFieldSelector {
                        field_path: "metadata.name".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            EnvVar {
                name: "POD_NAMESPACE".to_string(),
                value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                    field_ref: Some(k8s_openapi::api::core::v1::ObjectFieldSelector {
                        field_path: "metadata.namespace".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ]);
        
        Ok(envs)
    }
    
    fn build_resource_requirements(&self, spec: ResourceSpec) -> Result<ResourceRequirements, OperatorError> {
        use std::collections::BTreeMap;
        
        let mut requests = BTreeMap::new();
        requests.insert("cpu".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(spec.cpu_request));
        requests.insert("memory".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(spec.memory_request));
        
        let mut limits = BTreeMap::new();
        limits.insert("cpu".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(spec.cpu_limit));
        limits.insert("memory".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(spec.memory_limit));
        
        Ok(ResourceRequirements {
            requests: Some(requests),
            limits: Some(limits),
        })
    }
}
```

## Part 2: Axum Integration Layer

### 2.1 Pipeline API Endpoints

**Active Recall Challenge**: Create comprehensive pipeline management API:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, delete},
    Json, Router,
};

// API request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePipelineRequest {
    pub pipeline_type: PipelineType,
    pub config: serde_json::Value,
    pub priority: Priority,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PipelineType {
    DataProcessing,
    ModelTraining,
    ETL,
    Backup,
    Custom { template: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal, 
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineResponse {
    pub id: uuid::Uuid,
    pub job_name: String,
    pub status: PipelineStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PipelineStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Timeout,
    Cancelled,
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub k8s_operator: Arc<KubernetesOperator>,
    pub job_tracker: Arc<RwLock<HashMap<uuid::Uuid, PipelineJob>>>,
    pub config: Arc<PipelineConfig>,
}

// Main router setup
pub fn create_pipeline_api(state: AppState) -> Router {
    Router::new()
        .route("/pipelines", post(create_pipeline))
        .route("/pipelines", get(list_pipelines))
        .route("/pipelines/:id", get(get_pipeline))
        .route("/pipelines/:id/cancel", post(cancel_pipeline))
        .route("/pipelines/:id/logs", get(get_pipeline_logs))
        .route("/pipelines/:id/restart", post(restart_pipeline))
        .route("/templates", get(list_pipeline_templates))
        .route("/health", get(health_check))
        .with_state(state)
}

// API handlers
async fn create_pipeline(
    State(state): State<AppState>,
    Json(request): Json<CreatePipelineRequest>,
) -> Result<impl IntoResponse, PipelineError> {
    let pipeline_id = uuid::Uuid::new_v4();
    
    // Validate request
    validate_pipeline_request(&request)?;
    
    // Convert to job request
    let job_request = build_job_request_from_pipeline(&request, pipeline_id)?;
    
    // Create job in Kubernetes
    let job = state.k8s_operator.create_job(job_request).await?;
    let job_name = job.name_any();
    
    // Store pipeline metadata
    let pipeline = PipelineJob {
        id: pipeline_id,
        job_name: job_name.clone(),
        pipeline_type: request.pipeline_type.clone(),
        status: PipelineStatus::Queued,
        config: request.config.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        timeout_seconds: request.timeout_seconds,
    };
    
    // Persist to database
    store_pipeline(&state.db_pool, &pipeline).await?;
    
    // Track in memory
    state.job_tracker.write().await.insert(pipeline_id, pipeline.clone());
    
    // Start monitoring job
    tokio::spawn(monitor_pipeline_job(
        state.k8s_operator.clone(),
        state.db_pool.clone(),
        pipeline_id,
        job_name,
    ));
    
    let response = PipelineResponse {
        id: pipeline_id,
        job_name,
        status: PipelineStatus::Queued,
        created_at: pipeline.created_at,
        updated_at: pipeline.updated_at,
        config: pipeline.config,
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

async fn list_pipelines(
    Query(params): Query<ListPipelinesParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, PipelineError> {
    let pipelines = fetch_pipelines(&state.db_pool, &params).await?;
    Ok(Json(pipelines))
}

async fn get_pipeline(
    Path(pipeline_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, PipelineError> {
    let pipeline = fetch_pipeline_by_id(&state.db_pool, pipeline_id)
        .await?
        .ok_or(PipelineError::NotFound)?;
    
    Ok(Json(pipeline))
}

async fn cancel_pipeline(
    Path(pipeline_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, PipelineError> {
    // Get pipeline info
    let pipeline = state.job_tracker.read().await.get(&pipeline_id).cloned()
        .ok_or(PipelineError::NotFound)?;
    
    // Delete Kubernetes job
    state.k8s_operator.cleanup_job(&pipeline.job_name).await?;
    
    // Update status
    update_pipeline_status(&state.db_pool, pipeline_id, PipelineStatus::Cancelled).await?;
    
    Ok(StatusCode::OK)
}
```

### 2.2 Job Template Management

**Active Recall Challenge**: Implement dynamic job template system:

```rust
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTemplate {
    pub name: String,
    pub description: String,
    pub image: String,
    pub command_template: Vec<String>,
    pub args_template: Vec<String>,
    pub env_defaults: HashMap<String, String>,
    pub resource_defaults: ResourceSpec,
    pub volume_templates: Vec<VolumeSpec>,
    pub config_schema: Value,
}

// Template processing
impl PipelineTemplate {
    fn render_job_request(
        &self,
        pipeline_id: uuid::Uuid,
        config: &Value,
    ) -> Result<JobRequest, PipelineError> {
        let context = self.build_template_context(pipeline_id, config)?;
        
        Ok(JobRequest {
            name: format!("pipeline-{}", pipeline_id),
            image: self.image.clone(),
            command: self.render_command_template(&context)?,
            args: self.render_args_template(&context)?,
            env_vars: self.render_env_vars(&context)?,
            resources: self.resource_defaults.clone(),
            volumes: self.volume_templates.clone(),
            backoff_limit: Some(3),
            ttl_seconds_after_finished: Some(3600),
            parallelism: config.get("parallelism")
                .and_then(|v| v.as_u64())
                .map(|v| v as i32),
            completions: config.get("completions")
                .and_then(|v| v.as_u64())
                .map(|v| v as i32),
        })
    }
    
    fn build_template_context(
        &self,
        pipeline_id: uuid::Uuid,
        config: &Value,
    ) -> Result<TemplateContext, PipelineError> {
        let mut context = TemplateContext::new();
        context.insert("pipeline_id", pipeline_id.to_string());
        context.insert("timestamp", chrono::Utc::now().timestamp().to_string());
        
        // Add config values to context
        if let Value::Object(obj) = config {
            for (key, value) in obj {
                context.insert(key, value.as_str().unwrap_or("").to_string());
            }
        }
        
        Ok(context)
    }
    
    fn render_command_template(
        &self,
        context: &TemplateContext,
    ) -> Result<Vec<String>, PipelineError> {
        self.command_template
            .iter()
            .map(|cmd| self.render_template_string(cmd, context))
            .collect()
    }
    
    fn render_template_string(
        &self,
        template: &str,
        context: &TemplateContext,
    ) -> Result<String, PipelineError> {
        let mut result = template.to_string();
        
        for (key, value) in context.iter() {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        // Check for unresolved placeholders
        if result.contains("{{") && result.contains("}}") {
            return Err(PipelineError::TemplateError(
                "Unresolved template placeholders found".to_string()
            ));
        }
        
        Ok(result)
    }
}

// Built-in templates
fn create_builtin_templates() -> Vec<PipelineTemplate> {
    vec![
        PipelineTemplate {
            name: "data-processing".to_string(),
            description: "Generic data processing pipeline".to_string(),
            image: "python:3.11-slim".to_string(),
            command_template: vec!["python".to_string(), "-c".to_string()],
            args_template: vec![
                "import os; print(f'Processing data for pipeline {{pipeline_id}}')".to_string()
            ],
            env_defaults: HashMap::from([
                ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
            ]),
            resource_defaults: ResourceSpec {
                cpu_request: "100m".to_string(),
                memory_request: "256Mi".to_string(),
                cpu_limit: "500m".to_string(),
                memory_limit: "1Gi".to_string(),
            },
            volume_templates: vec![],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "input_path": {"type": "string"},
                    "output_path": {"type": "string"},
                    "batch_size": {"type": "integer", "default": 1000}
                },
                "required": ["input_path", "output_path"]
            }),
        },
        // Add more templates...
    ]
}
```

## Part 3: Job Monitoring and Lifecycle Management

### 3.1 Real-time Job Status Tracking

**Active Recall Challenge**: Implement comprehensive job monitoring:

```rust
use kube::runtime::{watcher, WatchStreamExt};
use futures::StreamExt;

async fn monitor_pipeline_job(
    k8s_operator: Arc<KubernetesOperator>,
    db_pool: sqlx::PgPool,
    pipeline_id: uuid::Uuid,
    job_name: String,
) {
    let mut retries = 0;
    const MAX_RETRIES: u32 = 3;
    
    while retries < MAX_RETRIES {
        match monitor_job_inner(&k8s_operator, &db_pool, pipeline_id, &job_name).await {
            Ok(_) => break,
            Err(e) => {
                retries += 1;
                tracing::error!(
                    error = %e,
                    pipeline_id = %pipeline_id,
                    job_name = %job_name,
                    attempt = retries,
                    "Job monitoring failed"
                );
                
                if retries < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_secs(10 * retries as u64)).await;
                }
            }
        }
    }
}

async fn monitor_job_inner(
    k8s_operator: &KubernetesOperator,
    db_pool: &sqlx::PgPool,
    pipeline_id: uuid::Uuid,
    job_name: &str,
) -> Result<(), OperatorError> {
    // Watch for job status changes
    let watch_params = WatchParams::default()
        .fields(&format!("metadata.name={}", job_name))
        .timeout(600); // 10 minutes
    
    let mut stream = watcher(k8s_operator.job_api.clone(), watch_params)
        .default_backoff()
        .applied_objects()
        .boxed();
    
    let mut last_status: Option<JobStatus> = None;
    let start_time = std::time::Instant::now();
    
    while let Some(job) = stream.try_next().await? {
        let current_status = extract_job_status(&job)?;
        
        // Only update if status changed
        if last_status.as_ref() != Some(&current_status) {
            update_pipeline_from_job_status(db_pool, pipeline_id, &current_status).await?;
            
            tracing::info!(
                pipeline_id = %pipeline_id,
                job_name = %job_name,
                status = ?current_status,
                duration_secs = start_time.elapsed().as_secs(),
                "Pipeline status updated"
            );
            
            // Check for terminal states
            match current_status {
                JobStatus::Succeeded | JobStatus::Failed => {
                    // Collect final logs
                    collect_job_logs(k8s_operator, pipeline_id, job_name).await?;
                    
                    // Schedule cleanup
                    schedule_job_cleanup(k8s_operator, job_name).await?;
                    
                    break;
                }
                _ => {}
            }
            
            last_status = Some(current_status);
        }
        
        // Check for timeout
        if let Some(timeout_secs) = get_pipeline_timeout(db_pool, pipeline_id).await? {
            if start_time.elapsed().as_secs() > timeout_secs as u64 {
                handle_pipeline_timeout(k8s_operator, db_pool, pipeline_id, job_name).await?;
                break;
            }
        }
    }
    
    Ok(())
}

async fn collect_job_logs(
    k8s_operator: &KubernetesOperator,
    pipeline_id: uuid::Uuid,
    job_name: &str,
) -> Result<(), OperatorError> {
    // Get pods for this job
    let pods = k8s_operator.pod_api.list(&kube::api::ListParams {
        label_selector: Some(format!("job-name={}", job_name)),
        ..Default::default()
    }).await?;
    
    for pod in pods.items {
        let pod_name = pod.name_any();
        
        // Get logs for each container in the pod
        let log_params = kube::api::LogParams {
            container: Some("job-container".to_string()),
            timestamps: true,
            ..Default::default()
        };
        
        match k8s_operator.pod_api.logs(&pod_name, &log_params).await {
            Ok(logs) => {
                store_pipeline_logs(pipeline_id, &pod_name, &logs).await?;
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    pod_name = %pod_name,
                    "Failed to collect pod logs"
                );
            }
        }
    }
    
    Ok(())
}
```

### 3.2 Error Handling and Recovery

**Active Recall Challenge**: Implement robust error handling:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Kubernetes API error: {0}")]
    KubernetesError(#[from] kube::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Pipeline not found")]
    NotFound,
    
    #[error("Invalid pipeline configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Template rendering error: {0}")]
    TemplateError(String),
    
    #[error("Pipeline timeout after {timeout_secs} seconds")]
    Timeout { timeout_secs: u32 },
    
    #[error("Resource quota exceeded: {resource_type}")]
    ResourceQuotaExceeded { resource_type: String },
    
    #[error("Pipeline in invalid state: {current_state}")]
    InvalidState { current_state: String },
}

impl IntoResponse for PipelineError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            PipelineError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            PipelineError::InvalidConfig(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            PipelineError::ResourceQuotaExceeded { .. } => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            PipelineError::Timeout { .. } => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            _ => {
                tracing::error!("Pipeline operation failed: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };
        
        let body = Json(serde_json::json!({
            "error": error_message,
            "type": error_type(&self),
        }));
        
        (status, body).into_response()
    }
}

// Recovery mechanisms
async fn handle_pipeline_timeout(
    k8s_operator: &KubernetesOperator,
    db_pool: &sqlx::PgPool,
    pipeline_id: uuid::Uuid,
    job_name: &str,
) -> Result<(), OperatorError> {
    tracing::warn!(
        pipeline_id = %pipeline_id,
        job_name = %job_name,
        "Pipeline timeout, initiating cleanup"
    );
    
    // Attempt graceful termination first
    if let Err(e) = k8s_operator.cleanup_job(job_name).await {
        tracing::error!(
            error = %e,
            job_name = %job_name,
            "Failed graceful job termination"
        );
        
        // Force delete if graceful fails
        force_delete_job(k8s_operator, job_name).await?;
    }
    
    // Update pipeline status
    update_pipeline_status(db_pool, pipeline_id, PipelineStatus::Timeout).await?;
    
    // Collect partial logs if available
    let _ = collect_job_logs(k8s_operator, pipeline_id, job_name).await;
    
    Ok(())
}

async fn handle_failed_pipeline(
    db_pool: &sqlx::PgPool,
    pipeline_id: uuid::Uuid,
    failure_reason: &str,
) -> Result<(), OperatorError> {
    // Check if pipeline should be retried
    let pipeline = fetch_pipeline_by_id(db_pool, pipeline_id).await?
        .ok_or_else(|| OperatorError::InvalidState("Pipeline not found".to_string()))?;
    
    let retry_count = get_retry_count(db_pool, pipeline_id).await?;
    
    if retry_count < 3 && should_retry_failure(failure_reason) {
        // Schedule retry
        schedule_pipeline_retry(db_pool, pipeline_id, retry_count + 1).await?;
    } else {
        // Mark as permanently failed
        update_pipeline_status(db_pool, pipeline_id, PipelineStatus::Failed).await?;
        
        // Send failure notification
        send_failure_notification(&pipeline, failure_reason).await?;
    }
    
    Ok(())
}
```

## Active Recall Questions

Test your operator mastery:

1. **Client Setup**: How do you handle both in-cluster and kubeconfig authentication?
2. **Job Creation**: What are the essential fields for a production-ready Job spec?
3. **Status Monitoring**: How do you efficiently watch for job status changes?
4. **Template System**: How do you safely render user-provided templates?
5. **Error Recovery**: When should you retry vs fail permanently?
6. **Resource Management**: How do you prevent resource exhaustion?
7. **Observability**: What metrics should you expose for your operator?

## Production Operator Checklist

- [ ] Kubernetes client with proper authentication
- [ ] Flexible job templating system
- [ ] Real-time status monitoring
- [ ] Comprehensive error handling
- [ ] Automatic retry mechanisms
- [ ] Resource quota enforcement
- [ ] Log collection and storage
- [ ] Cleanup and garbage collection
- [ ] Health checks and metrics
- [ ] Rate limiting and backpressure
- [ ] Security context enforcement
- [ ] Audit logging for compliance

Master these patterns and you'll build production-ready Kubernetes operators that can reliably orchestrate complex workloads through web APIs.