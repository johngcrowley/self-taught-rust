# SQLx Master Class - Active Recall Primer

## Core Competency: Deep SQLx Mastery for Production Systems

This exam tests your mastery of SQLx's core types, query mechanics, and production patterns. Master these concepts through active recall.

## Part 1: Essential Types Mastery

### 1.1 Row Type Deep Dive

**Active Recall Challenge**: Without looking, implement these Row operations:

```rust
use sqlx::{Row, FromRow, postgres::PgRow};

// Challenge 1: Extract values from PgRow using different methods
fn extract_from_row(row: &PgRow) -> Result<(i32, String, Option<chrono::DateTime<chrono::Utc>>), sqlx::Error> {
    // Method 1: Index-based extraction
    let id: i32 = row.try_get(0)?;
    
    // Method 2: Column name extraction  
    let name: String = row.try_get("name")?;
    
    // Method 3: Optional field handling
    let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
    
    Ok((id, name, created_at))
}

// Challenge 2: Implement custom FromRow for complex types
#[derive(Debug)]
struct User {
    id: uuid::Uuid,
    email: String,
    profile: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(User {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            profile: row.try_get("profile")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
```

**Key Row Methods You Must Know**:
- `try_get::<T>()` - Safe extraction with type conversion
- `get::<T>()` - Panics on error, use sparingly
- `len()` - Number of columns
- `is_empty()` - Check if row has columns
- `columns()` - Get column metadata

### 1.2 Pool Management Mastery

**Active Recall Challenge**: Configure production-ready connection pools:

```rust
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

async fn create_production_pool() -> Result<Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)           // Max concurrent connections
        .min_connections(5)            // Keep minimum alive
        .acquire_timeout(Duration::from_secs(30))  // Connection timeout
        .idle_timeout(Duration::from_secs(600))    // Close idle connections
        .max_lifetime(Duration::from_secs(1800))   // Connection max age
        .test_before_acquire(true)     // Validate before use
        .connect(&database_url)
        .await
}

// Pool usage patterns
async fn use_pool_correctly(pool: &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error> {
    // Pattern 1: Direct query (auto-acquires connection)
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(pool)
        .await?;
    
    // Pattern 2: Manual connection management for transactions
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO audit_log (action) VALUES ('user_fetch')")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    
    Ok(users)
}
```

### 1.3 Query Builder Mastery

**Active Recall Challenge**: Build dynamic, safe queries:

```rust
use sqlx::{QueryBuilder, Postgres, Arguments};

async fn dynamic_user_search(
    pool: &Pool<Postgres>,
    filters: UserFilters,
) -> Result<Vec<User>, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT id, email, profile, created_at FROM users WHERE 1=1");
    
    if let Some(email_pattern) = filters.email_pattern {
        query.push(" AND email ILIKE ");
        query.push_bind(format!("%{}%", email_pattern));
    }
    
    if let Some(created_after) = filters.created_after {
        query.push(" AND created_at > ");
        query.push_bind(created_after);
    }
    
    if !filters.ids.is_empty() {
        query.push(" AND id = ANY(");
        query.push_bind(filters.ids);
        query.push(")");
    }
    
    query.push(" ORDER BY created_at DESC LIMIT ");
    query.push_bind(filters.limit.unwrap_or(100));
    
    query.build_query_as::<User>()
        .fetch_all(pool)
        .await
}
```

## Part 2: Query Execution Patterns

### 2.1 Fetch Variants Mastery

**Active Recall Challenge**: Know when to use each fetch method:

```rust
// fetch_one() - Exactly one row, error if 0 or >1
let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
    .bind(user_id)
    .fetch_one(pool)
    .await?;

// fetch_optional() - 0 or 1 row
let maybe_user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = $1")
    .bind(&email)
    .fetch_optional(pool)
    .await?;

// fetch_all() - All matching rows (memory intensive)
let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
    .fetch_all(pool)
    .await?;

// Stream for large datasets (memory efficient)
use futures::StreamExt;
let mut stream = sqlx::query_as::<_, User>("SELECT * FROM users").fetch(pool);
while let Some(user_result) = stream.next().await {
    let user = user_result?;
    // Process one user at a time
    process_user(user).await?;
}
```

### 2.2 Transaction Management

**Active Recall Challenge**: Handle complex transactions:

```rust
async fn transfer_funds(
    pool: &Pool<Postgres>,
    from_account: uuid::Uuid,
    to_account: uuid::Uuid,
    amount: rust_decimal::Decimal,
) -> Result<(), TransferError> {
    let mut tx = pool.begin().await?;
    
    // Lock accounts in consistent order to prevent deadlock
    let (first_id, second_id) = if from_account < to_account {
        (from_account, to_account)
    } else {
        (to_account, from_account)
    };
    
    // Acquire locks
    sqlx::query("SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE")
        .bind(first_id)
        .bind(second_id)
        .fetch_all(&mut *tx)
        .await?;
    
    // Check source balance
    let balance: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT balance FROM accounts WHERE id = $1"
    )
    .bind(from_account)
    .fetch_one(&mut *tx)
    .await?;
    
    if balance < amount {
        return Err(TransferError::InsufficientFunds);
    }
    
    // Perform transfer
    sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
        .bind(amount)
        .bind(from_account)
        .execute(&mut *tx)
        .await?;
        
    sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE id = $2")
        .bind(amount)
        .bind(to_account)
        .execute(&mut *tx)
        .await?;
    
    // Log transaction
    sqlx::query(
        "INSERT INTO transfers (from_account, to_account, amount, created_at) VALUES ($1, $2, $3, NOW())"
    )
    .bind(from_account)
    .bind(to_account)
    .bind(amount)
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    Ok(())
}
```

## Part 3: Advanced Patterns

### 3.1 Type Mapping Mastery

**Active Recall Challenge**: Handle complex Postgres types:

```rust
use sqlx::types::Json;

#[derive(Debug, Serialize, Deserialize)]
struct UserPreferences {
    theme: String,
    notifications: bool,
    language: String,
}

// JSON column handling
async fn update_user_preferences(
    pool: &Pool<Postgres>,
    user_id: uuid::Uuid,
    prefs: UserPreferences,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET preferences = $1 WHERE id = $2")
        .bind(Json(prefs))
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// Array handling
async fn get_users_by_tags(
    pool: &Pool<Postgres>,
    tags: Vec<String>,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE tags && $1")
        .bind(&tags)
        .fetch_all(pool)
        .await
}
```

### 3.2 Migration Management

**Active Recall Challenge**: Production migration patterns:

```rust
async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}

// Custom migration validation
async fn validate_schema_version(pool: &Pool<Postgres>) -> Result<bool, sqlx::Error> {
    let version: Option<i64> = sqlx::query_scalar(
        "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;
    
    let expected_version = 20240101000000; // Your latest migration
    Ok(version == Some(expected_version))
}
```

## Active Recall Questions

Test your mastery without looking at the code:

1. **Row Extraction**: What's the difference between `try_get()` and `get()` on a Row?
2. **Pool Configuration**: What are the 5 essential pool settings for production?
3. **Fetch Methods**: When would you use `fetch()` vs `fetch_all()`?
4. **Transactions**: How do you prevent deadlocks in multi-row updates?
5. **Type Mapping**: How do you handle Postgres JSON columns in SQLx?
6. **Error Handling**: What SQLx errors should you always handle explicitly?
7. **Performance**: When should you use prepared statements vs dynamic queries?

## Production Patterns Checklist

- [ ] Connection pool configured with appropriate limits
- [ ] Transaction timeouts configured
- [ ] Prepared statements for hot paths
- [ ] Streaming for large result sets
- [ ] Proper error handling and retries
- [ ] Connection health checking
- [ ] Migration validation on startup
- [ ] Monitoring and observability hooks

Master these patterns and you'll handle any SQLx production scenario with confidence.