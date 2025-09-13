# K3s + Postgres Production Primer - Active Recall Master Class

## Core Competency: Production Kubernetes Deployment Pipeline

Master the complete stack from k3s cluster setup to production-ready Rust web services with Postgres integration.

## Part 1: K3s Cluster Foundation

### 1.1 K3s Installation and Configuration

**Active Recall Challenge**: Set up production-ready k3s cluster:

```bash
# Install k3s server (control plane)
curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC="--write-kubeconfig-mode 644 --disable traefik --disable servicelb" sh -

# Get node token for workers
sudo cat /var/lib/rancher/k3s/server/node-token

# Install k3s agent (worker nodes)
curl -sfL https://get.k3s.io | K3S_URL=https://MASTER_IP:6443 K3S_TOKEN=TOKEN sh -

# Verify cluster status
kubectl get nodes -o wide
kubectl get pods --all-namespaces

# Install essential tools
kubectl apply -f https://raw.githubusercontent.com/metallb/metallb/v0.13.7/config/manifests/metallb-native.yaml
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml
```

**Essential k3s Configuration** (`/etc/systemd/system/k3s.service.d/override.conf`):

```ini
[Service]
ExecStart=
ExecStart=/usr/local/bin/k3s server \
    --write-kubeconfig-mode 644 \
    --disable traefik \
    --disable servicelb \
    --kube-controller-manager-arg bind-address=0.0.0.0 \
    --kube-proxy-arg metrics-bind-address=0.0.0.0 \
    --kube-scheduler-arg bind-address=0.0.0.0 \
    --kubelet-arg containerd=/run/k3s/containerd/containerd.sock
```

### 1.2 MetalLB Configuration

**Active Recall Challenge**: Configure load balancer for bare metal:

```yaml
# metallb-config.yaml
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: production-pool
  namespace: metallb-system
spec:
  addresses:
  - 192.168.1.240-192.168.1.250  # Adjust to your network
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: production-l2
  namespace: metallb-system
spec:
  ipAddressPools:
  - production-pool
```

```bash
kubectl apply -f metallb-config.yaml
```

## Part 2: Postgres Production Deployment

### 2.1 Postgres StatefulSet with Persistence

**Active Recall Challenge**: Deploy production Postgres with proper persistence:

```yaml
# postgres-storage.yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: postgres-pv
spec:
  capacity:
    storage: 20Gi
  accessModes:
    - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: local-storage
  local:
    path: /opt/postgres-data
  nodeAffinity:
    required:
      nodeSelectorTerms:
      - matchExpressions:
        - key: kubernetes.io/hostname
          operator: In
          values:
          - your-node-name
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: database
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
  storageClassName: local-storage
```

```yaml
# postgres-secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: postgres-secret
  namespace: database
type: Opaque
data:
  POSTGRES_DB: bXlkYXRhYmFzZQ==        # myatabase (base64)
  POSTGRES_USER: cG9zdGdyZXM=          # postgres (base64)  
  POSTGRES_PASSWORD: c2VjdXJlcGFzcw==   # securepass (base64)
```

```yaml
# postgres-deployment.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: database
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15-alpine
        ports:
        - containerPort: 5432
        envFrom:
        - secretRef:
            name: postgres-secret
        env:
        - name: PGDATA
          value: /var/lib/postgresql/data/pgdata
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        - name: postgres-config
          mountPath: /etc/postgresql/postgresql.conf
          subPath: postgresql.conf
          readOnly: true
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
      - name: postgres-config
        configMap:
          name: postgres-config
---
apiVersion: v1
kind: Service
metadata:
  name: postgres-service
  namespace: database
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  type: ClusterIP
```

### 2.2 Postgres Configuration Tuning

**Active Recall Challenge**: Optimize Postgres for production workloads:

```yaml
# postgres-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: postgres-config
  namespace: database
data:
  postgresql.conf: |
    # Connection settings
    max_connections = 200
    shared_buffers = 512MB
    effective_cache_size = 1536MB
    maintenance_work_mem = 128MB
    checkpoint_completion_target = 0.9
    wal_buffers = 16MB
    default_statistics_target = 100
    
    # Write-ahead logging
    wal_level = replica
    max_wal_size = 2GB
    min_wal_size = 1GB
    checkpoint_timeout = 15min
    
    # Query tuning
    random_page_cost = 1.1
    effective_io_concurrency = 200
    work_mem = 8MB
    
    # Logging
    log_destination = 'stderr'
    logging_collector = on
    log_directory = '/var/log/postgresql'
    log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
    log_min_messages = warning
    log_min_error_statement = error
    log_statement = 'mod'
    
    # Performance monitoring
    track_activities = on
    track_counts = on
    track_io_timing = on
    track_functions = pl
```

## Part 3: Rust Web Service Deployment

### 3.1 Production Dockerfile

**Active Recall Challenge**: Create optimized multi-stage Docker build:

```dockerfile
# Dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
RUN cargo build --release && rm src/main.rs target/release/deps/web_service*

# Copy source code
COPY src ./src

# Build application
RUN cargo build --release

# Runtime stage  
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false appuser

# Copy binary from builder
COPY --from=builder /app/target/release/web-service ./

# Copy static files if needed
# COPY static ./static

# Set ownership
RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./web-service"]
```

### 3.2 Kubernetes Deployment Configuration

**Active Recall Challenge**: Deploy Rust service with production patterns:

```yaml
# rust-web-service.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: web-service-config
  namespace: default
data:
  DATABASE_URL: "postgresql://postgres:securepass@postgres-service.database.svc.cluster.local:5432/mydatabase"
  RUST_LOG: "info"
  SERVER_PORT: "8080"
  REDIS_URL: "redis://redis-service:6379"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-service
  namespace: default
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: web-service
  template:
    metadata:
      labels:
        app: web-service
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: web-service
        image: your-registry/web-service:latest
        ports:
        - containerPort: 8080
          name: http
        envFrom:
        - configMapRef:
            name: web-service-config
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE  
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 15"]
      terminationGracePeriodSeconds: 30
---
apiVersion: v1
kind: Service
metadata:
  name: web-service
  namespace: default
spec:
  selector:
    app: web-service
  ports:
  - port: 80
    targetPort: 8080
    name: http
  type: LoadBalancer
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-service-hpa
  namespace: default
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web-service
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## Part 4: Database Migration and Initialization

### 4.1 Migration Job

**Active Recall Challenge**: Handle database migrations in Kubernetes:

```yaml
# db-migration-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: db-migration
  namespace: default
spec:
  template:
    spec:
      restartPolicy: OnFailure
      initContainers:
      - name: wait-for-postgres
        image: postgres:15-alpine
        command:
        - sh
        - -c
        - |
          until pg_isready -h postgres-service.database.svc.cluster.local -p 5432 -U postgres; do
            echo "Waiting for postgres..."
            sleep 2
          done
      containers:
      - name: migrate
        image: your-registry/web-service:latest
        command: ["./web-service", "migrate"]
        envFrom:
        - configMapRef:
            name: web-service-config
        resources:
          requests:
            memory: "64Mi"
            cpu: "50m"
          limits:
            memory: "256Mi"
            cpu: "200m"
      backoffLimit: 3
```

### 4.2 Database Initialization Scripts

**Active Recall Challenge**: Implement robust migration handling:

```rust
// src/migrations.rs
use sqlx::{Pool, Postgres, migrate::Migrator};
use std::path::Path;

pub struct MigrationManager {
    pool: Pool<Postgres>,
}

impl MigrationManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
    
    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        let migrator = Migrator::new(Path::new("./migrations")).await?;
        
        tracing::info!("Starting database migrations...");
        let results = migrator.run(&self.pool).await?;
        
        for result in results {
            tracing::info!(
                "Migration {} applied successfully in {}ms",
                result.version,
                result.elapsed.as_millis()
            );
        }
        
        tracing::info!("All migrations completed successfully");
        Ok(())
    }
    
    pub async fn validate_schema(&self) -> Result<bool, sqlx::Error> {
        // Check critical tables exist
        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.tables 
             WHERE table_schema = 'public' 
             AND table_name IN ('users', 'projects', 'jobs')"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(table_count >= 3)
    }
    
    pub async fn seed_initial_data(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        
        // Insert admin user if not exists
        sqlx::query(
            "INSERT INTO users (id, email, role, created_at) 
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (email) DO NOTHING"
        )
        .bind(uuid::Uuid::new_v4())
        .bind("admin@example.com")
        .bind("admin")
        .execute(&mut *tx)
        .await?;
        
        // Insert default project templates
        sqlx::query(
            "INSERT INTO project_templates (name, config, created_at)
             VALUES ($1, $2, NOW())
             ON CONFLICT (name) DO NOTHING"
        )
        .bind("default-pipeline")
        .bind(serde_json::json!({
            "image": "alpine:latest",
            "command": ["echo", "hello world"]
        }))
        .execute(&mut *tx)
        .await?;
        
        tx.commit().await?;
        Ok(())
    }
}

// Health check integration
pub async fn database_health_check(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(())
}
```

## Part 5: Production Operations

### 5.1 Monitoring and Observability

**Active Recall Challenge**: Implement comprehensive monitoring:

```yaml
# monitoring-stack.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'kubernetes-pods'
      kubernetes_sd_configs:
      - role: pod
      relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
    - job_name: 'postgres-exporter'
      static_configs:
      - targets: ['postgres-exporter:9187']
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres-exporter
  namespace: database
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgres-exporter
  template:
    metadata:
      labels:
        app: postgres-exporter
    spec:
      containers:
      - name: postgres-exporter
        image: prometheuscommunity/postgres-exporter:latest
        ports:
        - containerPort: 9187
        env:
        - name: DATA_SOURCE_NAME
          value: "postgresql://postgres:securepass@postgres-service:5432/mydatabase?sslmode=disable"
        resources:
          requests:
            memory: "64Mi"
            cpu: "50m"
          limits:
            memory: "128Mi"
            cpu: "100m"
```

### 5.2 Backup and Disaster Recovery

**Active Recall Challenge**: Implement automated backup strategy:

```yaml
# postgres-backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: database
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: postgres-backup
            image: postgres:15-alpine
            command:
            - /bin/sh
            - -c
            - |
              BACKUP_FILE="/backup/postgres-$(date +%Y%m%d-%H%M%S).sql"
              pg_dump -h postgres-service -U postgres -d mydatabase > $BACKUP_FILE
              
              # Compress backup
              gzip $BACKUP_FILE
              
              # Clean old backups (keep 30 days)
              find /backup -name "*.sql.gz" -mtime +30 -delete
              
              echo "Backup completed: ${BACKUP_FILE}.gz"
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: postgres-secret
                  key: POSTGRES_PASSWORD
            volumeMounts:
            - name: backup-storage
              mountPath: /backup
          volumes:
          - name: backup-storage
            persistentVolumeClaim:
              claimName: postgres-backup-pvc
          restartPolicy: OnFailure
```

## Active Recall Questions

Test your production readiness:

1. **Cluster Setup**: What are the essential flags for production k3s installation?
2. **Storage**: How do you ensure Postgres data persistence across pod restarts?
3. **Networking**: How does MetalLB provide load balancing for bare metal clusters?
4. **Security**: What are the key security considerations for database secrets?
5. **Monitoring**: How do you expose custom metrics from your Rust application?
6. **Scaling**: When should you use HPA vs VPA for your web service?
7. **Disaster Recovery**: How do you test your backup and restore procedures?

## Production Deployment Checklist

- [ ] K3s cluster with multiple nodes for HA
- [ ] MetalLB configured for external access
- [ ] Postgres deployed with persistent storage
- [ ] Database connection pooling configured
- [ ] SSL/TLS certificates via cert-manager
- [ ] Resource limits and requests defined
- [ ] Health checks implemented
- [ ] Horizontal Pod Autoscaler configured
- [ ] Monitoring and alerting deployed
- [ ] Backup strategy automated
- [ ] Network policies for security
- [ ] RBAC configured appropriately

Master this stack and you'll deploy bullet-proof Rust applications on Kubernetes with confidence.