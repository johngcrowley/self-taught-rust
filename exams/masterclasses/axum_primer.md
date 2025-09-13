# Axum Master Class - Active Recall Primer

## What is Axum?

**Axum** is a modern, ergonomic web framework for Rust built on top of Tokio and Tower. It's designed around async/await and provides a type-safe approach to building web APIs with minimal boilerplate.

**Key Benefits:**
- **Type Safety**: Request/response handling checked at compile time
- **Performance**: Built on Tokio's async runtime for high concurrency
- **Composability**: Modular design with reusable components
- **Tower Ecosystem**: Leverages Tower's middleware and service abstractions
- **Developer Experience**: Intuitive APIs with helpful compiler errors

**When to use Axum:**
- Building REST APIs and web services
- Microservices that need high performance
- Applications requiring type-safe request handling
- Projects that want modern async Rust patterns

**Core Concepts:**
- **Handlers**: Functions that process requests and return responses
- **Extractors**: Type-safe way to extract data from requests (path params, JSON body, etc.)
- **State**: Shared application data (database pools, config, etc.)
- **Middleware**: Cross-cutting concerns (auth, logging, CORS, etc.)
- **Routing**: Mapping URLs to handler functions

## Core Competency: Production-Ready Web Services with Axum

Master Axum from fundamentals to production patterns for high-performance web services.

## Part 1: Essential Types and Extractors Mastery

### 1.1 Handler Function Types

**What are Handlers?**: Handler functions are the core of Axum applications - they receive HTTP requests and return HTTP responses. Axum uses Rust's type system to automatically extract request data and convert response types.

**Handler Signature Rules:**
- Must be `async fn`
- Can accept 0-16 extractors as parameters
- Must return something that implements `IntoResponse`
- Extractors are processed in order (order matters!)

**Common Return Types:**
- `impl IntoResponse` - Most flexible, any response type
- `Result<impl IntoResponse, E>` - For error handling
- `Json<T>` - JSON responses
- `(StatusCode, Json<T>)` - Custom status with JSON
- `String` or `&'static str` - Plain text responses

**Active Recall Challenge**: Understand the handler signature patterns:

```rust
use axum::{
    extract::{Path, Query, State, Json},
    response::{Response, IntoResponse},
    http::{StatusCode, HeaderMap},
};

// Basic handler - no extractors
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "healthy")
}

// Path extraction
async fn get_user(Path(user_id): Path<uuid::Uuid>) -> impl IntoResponse {
    format!("User: {}", user_id)
}

// Multiple extractors (order matters!)
async fn update_user(
    Path(user_id): Path<uuid::Uuid>,
    State(pool): State<sqlx::PgPool>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Handler logic here
    let user = update_user_in_db(&pool, user_id, payload).await?;
    Ok(Json(user))
}

// Query parameters
#[derive(serde::Deserialize)]
struct UserFilters {
    email: Option<String>,
    created_after: Option<chrono::DateTime<chrono::Utc>>,
    limit: Option<u32>,
}

async fn list_users(
    Query(filters): Query<UserFilters>,
    State(pool): State<sqlx::PgPool>,
) -> Result<impl IntoResponse, AppError> {
    let users = fetch_users_with_filters(&pool, filters).await?;
    Ok(Json(users))
}
```

### 1.2 State Management Mastery

**What is State?**: State in Axum is shared application data that's available to all handlers. This typically includes database connections, configuration, caches, and other resources that need to be shared across requests.

**State Requirements:**
- Must implement `Clone` (Axum clones state for each handler)
- Usually wrapped in `Arc` for shared ownership
- Passed to handlers via the `State` extractor
- Set on the Router with `.with_state(state)`

**Common State Patterns:**
- Database connection pools
- Application configuration
- Shared caches (Redis clients)
- Metrics collectors
- External service clients

**Active Recall Challenge**: Implement shared application state:

```rust
use axum::extract::State;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::Client,
    pub config: Arc<AppConfig>,
    pub metrics: Arc<prometheus::Registry>,
}

// State initialization
async fn create_app_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let db_pool = sqlx::PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let redis_client = redis::Client::open(env::var("REDIS_URL")?)?;
    let config = Arc::new(AppConfig::from_env()?);
    let metrics = Arc::new(prometheus::Registry::new());
    
    Ok(AppState {
        db_pool,
        redis_client,
        config,
        metrics,
    })
}

// Using state in handlers
async fn get_cached_user(
    Path(user_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Try cache first
    let cache_key = format!("user:{}", user_id);
    let mut redis_conn = state.redis_client.get_async_connection().await?;
    
    if let Ok(cached) = redis_conn.get::<_, String>(&cache_key).await {
        return Ok(Json(serde_json::from_str::<User>(&cached)?));
    }
    
    // Fallback to database
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await?;
    
    // Cache for next time
    let user_json = serde_json::to_string(&user)?;
    let _: () = redis_conn.setex(&cache_key, 300, user_json).await?;
    
    Ok(Json(user))
}
```

### 1.3 Request/Response Types Mastery

**Active Recall Challenge**: Handle complex request and response patterns:

```rust
use axum::{
    response::{Response, IntoResponse},
    body::Body,
    http::{HeaderValue, header},
};

// Custom response types
pub struct ApiResponse<T> {
    pub data: T,
    pub status: StatusCode,
    pub headers: HeaderMap,
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        let mut response = (self.status, Json(self.data)).into_response();
        response.headers_mut().extend(self.headers);
        response
    }
}

// File uploads
async fn upload_file(
    mut multipart: axum::extract::Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut files = Vec::new();
    
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("unknown").to_string();
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await?;
        
        // Process file data
        let file_id = save_file(&filename, &data).await?;
        files.push(UploadedFile { name, filename, file_id });
    }
    
    Ok(Json(files))
}

// Streaming responses
async fn export_users(
    State(pool): State<sqlx::PgPool>,
) -> Result<Response<Body>, AppError> {
    use futures::stream::StreamExt;
    use axum::body::Body;
    
    let stream = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch(&pool)
        .map(|result| {
            result.map(|user| {
                let json = serde_json::to_string(&user).unwrap();
                format!("{}\n", json)
            })
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
    
    let body = Body::from_stream(stream);
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-ndjson")
        .body(body)?)
}
```

## Part 2: Routing Architecture

### 2.1 Router Composition Mastery

**What is Router Composition?**: Large applications need organized routing. Axum allows you to compose multiple routers together, creating a modular architecture where different features can have their own routing logic.

**Router Composition Benefits:**
- **Modularity**: Separate concerns into different router modules
- **Maintainability**: Easier to understand and modify specific features
- **Testing**: Test individual router modules in isolation
- **Team Development**: Different developers can work on different router modules

**Key Router Methods:**
- `.merge()` - Combine routers together
- `.nest()` - Mount a router at a specific path prefix
- `.layer()` - Apply middleware to all routes in the router
- `.route_layer()` - Apply middleware to routes added after this call

**Active Recall Challenge**: Build modular, scalable routing:

```rust
use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware,
};

// Modular route structure
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .merge(api_routes())
        .merge(auth_routes())
        .merge(admin_routes())
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(middleware::from_fn(request_logging_middleware))
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/users/:id/avatar", post(upload_avatar))
        .nest("/projects", project_routes())
        .layer(middleware::from_fn(rate_limiting_middleware))
}

fn project_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/:id", get(get_project).put(update_project))
        .route("/:id/deploy", post(deploy_project))
        .route("/:id/logs", get(stream_project_logs))
}

fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/metrics", get(get_metrics))
        .route("/health", get(health_check))
        .route("/config", get(get_config).post(update_config))
        .layer(middleware::from_fn(admin_auth_middleware))
        .route_layer(middleware::from_fn(audit_logging_middleware))
}
```

### 2.2 Path Parameter Extraction

**Active Recall Challenge**: Handle complex path patterns:

```rust
use axum::extract::Path;

// Single parameter
async fn get_user(Path(user_id): Path<uuid::Uuid>) -> impl IntoResponse {
    // Handle user_id
}

// Multiple parameters  
async fn get_user_project(
    Path((user_id, project_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> impl IntoResponse {
    // Handle both IDs
}

// Named path parameters with struct
#[derive(serde::Deserialize)]
struct ProjectPath {
    user_id: uuid::Uuid,
    project_name: String,
}

async fn get_user_project_by_name(
    Path(ProjectPath { user_id, project_name }): Path<ProjectPath>,
) -> impl IntoResponse {
    // Handle structured path
}

// Wildcard/remainder paths
async fn serve_static_files(
    Path(path): Path<String>,
) -> Result<Response<Body>, AppError> {
    let file_path = format!("static/{}", path);
    serve_file(&file_path).await
}
```

## Part 3: Middleware Mastery

### 3.1 Custom Middleware Patterns

**Active Recall Challenge**: Implement production middleware:

```rust
use axum::{
    middleware::Next,
    http::{Request, HeaderValue},
};
use tower::ServiceBuilder;

// Request/Response middleware
async fn request_logging_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await?;
    let duration = start.elapsed();
    
    tracing::info!(
        method = %method,
        uri = %uri,
        status = response.status().as_u16(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );
    
    Ok(response)
}

// Authentication middleware
async fn auth_middleware<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let user = validate_jwt_token(&state.config.jwt_secret, token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add user to request extensions
    request.extensions_mut().insert(user);
    
    Ok(next.run(request).await?)
}

// Rate limiting middleware
async fn rate_limiting_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    // Check rate limit (implement with Redis or in-memory store)
    if is_rate_limited(client_ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    let response = next.run(request).await?;
    Ok(response)
}

// Applying middleware layers
fn create_middleware_stack() -> ServiceBuilder<
    tower::layer::util::Stack<
        axum::middleware::FromFnLayer<fn(Request<Body>, Next<Body>) -> _>,
        tower::layer::util::Stack<
            axum::middleware::FromFnLayer<fn(Request<Body>, Next<Body>) -> _>,
            tower::layer::util::Identity,
        >,
    >,
> {
    ServiceBuilder::new()
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn(cors_middleware))
        .layer(timeout::TimeoutLayer::new(Duration::from_secs(30)))
}
```

### 3.2 Error Handling Architecture

**Active Recall Challenge**: Implement comprehensive error handling:

```rust
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Insufficient permissions")]
    Forbidden,
    
    #[error("Rate limit exceeded")]
    RateLimited,
    
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(_) => {
                tracing::error!("Database error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
            AppError::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            _ => {
                tracing::error!("Unexpected error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            },
        };
        
        let body = Json(serde_json::json!({
            "error": error_message,
            "type": error_type(&self),
        }));
        
        (status, body).into_response()
    }
}

fn error_type(error: &AppError) -> &'static str {
    match error {
        AppError::Database(_) => "database_error",
        AppError::Redis(_) => "redis_error", 
        AppError::UserNotFound => "not_found",
        AppError::Validation(_) => "validation_error",
        AppError::Forbidden => "forbidden",
        AppError::RateLimited => "rate_limited",
        _ => "internal_error",
    }
}
```

## Part 4: Production Patterns

### 4.1 Server Configuration

**Active Recall Challenge**: Configure production-ready server:

```rust
use axum::Server;
use hyper::server::conn::AddrIncoming;
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
};

async fn start_server(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .merge(api_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(CorsLayer::permissive()) // Configure properly for production
                .layer(middleware::from_fn(request_id_middleware))
                .layer(timeout::TimeoutLayer::new(Duration::from_secs(30)))
        )
        .with_state(state);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    
    tracing::info!("Starting server on {}", addr);
    
    Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
        
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    
    tracing::info!("Received shutdown signal, starting graceful shutdown...");
}
```

### 4.2 Testing Patterns

**Active Recall Challenge**: Test Axum applications effectively:

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_user_endpoints() {
    let state = create_test_app_state().await;
    let app = create_app(state);
    
    // Test GET /users
    let response = app
        .oneshot(
            Request::builder()
                .uri("/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test POST /users
    let create_user_body = serde_json::to_string(&CreateUserRequest {
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
    }).unwrap();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("content-type", "application/json")
                .body(Body::from(create_user_body))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

## Active Recall Questions

Test your mastery:

1. **Handler Types**: What's the difference between `impl IntoResponse` and `Result<impl IntoResponse, E>`?
2. **Extractors**: What's the correct order for multiple extractors in a handler?
3. **State**: How do you share database connections across handlers?
4. **Middleware**: When should you use `from_fn` vs `from_fn_with_state`?
5. **Routing**: How do you apply middleware to only specific routes?
6. **Error Handling**: What traits must your error type implement for proper Axum integration?
7. **Testing**: How do you test handlers that require authentication middleware?

## Production Checklist

- [ ] Structured error handling with proper HTTP status codes
- [ ] Request logging and tracing integration
- [ ] Authentication/authorization middleware
- [ ] Rate limiting implementation
- [ ] CORS configuration for production
- [ ] Request timeout configuration
- [ ] Graceful shutdown handling
- [ ] Comprehensive test coverage
- [ ] Metrics and health check endpoints
- [ ] Security headers middleware

Master these patterns and you'll build robust, scalable web services with Axum.