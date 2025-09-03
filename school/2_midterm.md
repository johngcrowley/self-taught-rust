# Neon Storage Backend Engineering Bootcamp

*Welcome to the deep end. This bootcamp will transform you from a Rust developer into a storage systems engineer ready to work on Neon's distributed PostgreSQL storage backend.*

## Prerequisites

You should already be comfortable with:
- Rust's ownership model and lifetimes
- Basic async/await
- PostgreSQL concepts (WAL, pages, LSN)

## Initial Setup

Create your workspace:

```bash
cargo new --bin neon-bootcamp
cd neon-bootcamp
```

Replace `Cargo.toml` with:

```toml
[package]
name = "neon-bootcamp"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
reqwest = { version = "0.11", features = ["json"] }
clap = { version = "4.4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
bytes = "1.5"
futures = "0.3"
async-trait = "0.1"
dashmap = "5.5"
uuid = { version = "1.6", features = ["serde", "v4"] }
thiserror = "1.0"
pin-project = "1.1"

[[bin]]
name = "day1"
path = "src/day1.rs"

[[bin]]
name = "day2"
path = "src/day2.rs"

[[bin]]
name = "day3"
path = "src/day3.rs"

[[bin]]
name = "day4"
path = "src/day4.rs"

[[bin]]
name = "day5"
path = "src/day5.rs"

[[bin]]
name = "day6"
path = "src/day6.rs"

[[bin]]
name = "final"
path = "src/final.rs"

[dev-dependencies]
criterion = "0.5"
```

Create `src/common.rs` with shared types:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use bytes::Bytes;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lsn(pub u64);

impl Lsn {
    pub fn as_u64(self) -> u64 { self.0 }
    pub fn checked_sub<T: Into<u64>>(self, other: T) -> Option<Lsn> {
        self.0.checked_sub(other.into()).map(Lsn)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub field1: u32,  // tablespace
    pub field2: u32,  // database  
    pub field3: u32,  // relation
    pub field4: u32,  // block number
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct TenantId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct TimelineId(pub uuid::Uuid);

#[derive(Debug, thiserror::Error)]
pub enum PageReconstructError {
    #[error("Layer not found")]
    LayerNotFound,
    #[error("Cancelled")]
    Cancelled,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum WalDecodeError {
    #[error("Invalid WAL record format")]
    InvalidFormat,
    #[error("Unexpected end of buffer")]
    UnexpectedEof,
}

#[derive(Debug, thiserror::Error)]
pub enum CompactionError {
    #[error("I/O error during compaction: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compaction cancelled")]
    Cancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Tenant not found: {0:?}")]
    TenantNotFound(TenantId),
    #[error("Timeline not found: {0:?}")]
    TimelineNotFound(TimelineId),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoteStorageError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum ReconcileError {
    #[error("Reconciliation failed: {0}")]
    Failed(String),
}

pub struct RequestContext {
    pub request_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub timeline_id: TimelineId,
}

pub type RmgrId = u8;
```

---

## Day 1: The Storage Model & Your First Crisis

### The Mental Model

Think of Neon's architecture like a library that separated its catalog system from its physical books. Traditional databases (like vanilla PostgreSQL) are like old libraries where the catalog cards are stored with the books. If you want to look up a book, you go to the physical location. 

Neon split these apart. The "catalog" (compute nodes running PostgreSQL) can be anywhere, even multiple copies running simultaneously. The "books" (actual data pages) live in a completely separate distributed storage system. When PostgreSQL wants to read a page, instead of going to disk, it asks our storage layer over the network.

The magic? The storage layer speaks PostgreSQL's language - it understands WAL (Write-Ahead Logging) and can reconstruct any page at any point in time by replaying history. It's like having a library that can show you exactly how any book looked at any moment in the past.

### Your First Crisis

We have a production bug. Under high load with concurrent compaction, we're occasionally serving stale pages. Here's the buggy code:

```rust
// src/day1.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use bytes::Bytes;

mod common;
use common::*;

pub struct Layer {
    pub start_lsn: Lsn,
    pub end_lsn: Lsn,
    data: Arc<HashMap<Key, Bytes>>,
}

impl Layer {
    pub async fn get_value(
        &self,
        key: Key,
        lsn: Lsn,
        _ctx: &RequestContext,
    ) -> Result<Bytes, PageReconstructError> {
        // Simulate some async I/O
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        
        self.data
            .get(&key)
            .cloned()
            .ok_or(PageReconstructError::LayerNotFound)
    }
}

pub struct LayerMap {
    layers: Vec<Arc<Layer>>,
}

impl LayerMap {
    pub fn find_layer(&self, key: Key, lsn: Lsn) -> Result<Arc<Layer>, PageReconstructError> {
        self.layers
            .iter()
            .find(|l| l.start_lsn <= lsn && lsn < l.end_lsn)
            .cloned()
            .ok_or(PageReconstructError::LayerNotFound)
    }
}

pub struct Timeline {
    layers: Arc<RwLock<LayerMap>>,
    last_record_lsn: AtomicU64,
}

impl Timeline {
    pub async fn get_page_at_lsn(
        &self,
        key: Key,
        request_lsn: Lsn,
        ctx: &RequestContext,
    ) -> Result<Bytes, PageReconstructError> {
        // BUG: What happens if compaction runs between these two lines?
        let layers = self.layers.read().await;
        let layer = layers.find_layer(key, request_lsn)?;
        
        // The read lock is held while we call into layer
        // But what if compaction removed this layer?
        layer.get_value(key, request_lsn, ctx).await
    }
    
    pub async fn compact(&self) {
        // Compaction might create new layers and remove old ones
        let mut layers = self.layers.write().await;
        // ... compaction logic that replaces the entire LayerMap
    }
}

fn main() {
    // Your task: Fix the race condition
    // Hint: Think about the lifetime of the Arc<Layer> vs the RwLock guard
    // The problem is subtle but critical in production
    
    println!("Fix the race condition in get_page_at_lsn!");
}
```

### An Aside

**Arc<RwLock<T>> vs RwLock<Arc<T>>**: Think of `Arc` as a backstage pass that multiple people can hold copies of, and `RwLock` as a bouncer who controls access. `Arc<RwLock<LayerMap>>` means everyone shares the same bouncer who guards one LayerMap. The bug here: we get past the bouncer (acquire read lock), grab a reference to something inside (the Arc<Layer>), then leave (drop the lock). But the bouncer doesn't know we're still holding that reference - compaction could swap everything out while we're using it.

**Key concepts to understand:**
• `Arc<T>`: Atomically Reference Counted pointer - thread-safe shared ownership
• `RwLock<T>`: Read-Write lock - multiple readers OR one writer
• Lock guards: RAII types that release locks when dropped
• `.clone()` on Arc: Just increments refcount, doesn't clone the data

**AtomicU64 with Ordering**: Atomics are like bank transactions - they complete fully or not at all, no half-states. `Ordering::Relaxed` is saying "I don't care about synchronization with other memory operations, just make this atomic." It's the fastest but offers no guarantees about what other threads see when. `Ordering::SeqCst` is paranoid mode - everything happens in a globally consistent order, like having a universal timestamp on every operation.

**Memory ordering options:**
• `Relaxed`: No synchronization, just atomicity
• `Acquire`: All subsequent reads see at least this value
• `Release`: All previous writes are visible before this
• `AcqRel`: Both acquire and release
• `SeqCst`: Total order across all threads (expensive)

**The async sleep in get_value**: This simulates real I/O and forces a yield point. In async Rust, `.await` is where your task can be paused and another task can run. This is exactly where race conditions love to hide - between the time you check something and the time you use it.

**Async yield points:**
• Every `.await` is a potential task switch
• The executor can move your task to a different thread
• Local variables persist, but the world can change

**The Challenge**: The bug is that we're holding onto the read lock while doing async I/O. This seems fine, but what if compaction runs and completely replaces the layer map? The Arc keeps the old layer alive, but we might be reading from an outdated layer that doesn't reflect recent writes.

**Your Goal**: Fix this without sacrificing performance. Think about when you actually need the lock.

---

## Day 2: Zero-Copy WAL Processing

### The Mental Model

Imagine you're a court stenographer, but instead of transcribing speech, you're recording every change to a database. That's WAL (Write-Ahead Logging). Every modification is first written to this log before touching the actual data.

Now imagine you need to process millions of these records per second. Every time you copy data, it's like making the stenographer write everything twice. In systems programming, copying data is often the hidden performance killer. Zero-copy techniques are like giving the stenographer a stamp instead of making them rewrite - you're just marking where things are, not duplicating them.

### The Challenge

```rust
// src/day2.rs
use bytes::{Bytes, BytesMut};
use tokio::io::AsyncReadExt;

mod common;
use common::*;

pub struct WalRecord {
    pub lsn: Lsn,
    pub will_init: bool,
    pub rmgr: RmgrId,
    pub record: Bytes,  // Changed from &'a [u8]
}

impl WalRecord {
    // Problem: WAL records can span multiple network packets
    // We might get the header in one read() and the body in another
    // How do we handle this without copying everything into one big buffer?
    
    pub async fn decode_from_stream<R: tokio::io::AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Self, WalDecodeError> {
        // Your implementation here
        // 
        // Requirements:
        // 1. Read the header (24 bytes)
        // 2. Header tells you the body size
        // 3. Read the body (size from header)
        // 4. Handle partial reads (TCP might give you less than requested)
        // 5. Don't copy unnecessarily - use Bytes for zero-copy
        
        todo!("Implement zero-copy WAL decoding")
    }
}

// Here's a helper to test your implementation
async fn test_your_decoder() {
    let wal_data = vec![
        0x01, 0x00, 0x00, 0x00, // LSN low
        0x00, 0x00, 0x00, 0x00, // LSN high  
        0x01,                   // will_init
        0x0A,                   // rmgr
        0x00, 0x00,            // padding
        0x10, 0x00, 0x00, 0x00, // record length (16 bytes)
        0x00, 0x00, 0x00, 0x00, // more header...
        // Record body (16 bytes)
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
    ];
    
    let mut cursor = std::io::Cursor::new(wal_data);
    let record = WalRecord::decode_from_stream(&mut cursor).await.unwrap();
    
    assert_eq!(record.lsn, Lsn(1));
    assert_eq!(record.will_init, true);
    assert_eq!(record.rmgr, 0x0A);
    assert_eq!(record.record.len(), 16);
}

#[tokio::main]
async fn main() {
    println!("Implement zero-copy WAL decoding!");
    // test_your_decoder().await;
}
```

### An Aside

**Bytes vs BytesMut**: `BytesMut` is a growable buffer - think of it as a `Vec<u8>` optimized for network buffers. When you call `freeze()`, it becomes `Bytes` - an immutable, reference-counted slice. The magic: multiple `Bytes` can share the same underlying memory. Calling `slice()` on Bytes doesn't copy - it just adjusts pointers. It's like having multiple windows looking at different parts of the same wall.

**The Bytes ecosystem:**
• `BytesMut`: Mutable, growable buffer (like Vec<u8>)
• `freeze()`: Convert BytesMut → Bytes (zero-copy)
• `slice()`: Create sub-view of Bytes (zero-copy)
• `clone()`: Just increments refcount (cheap)
• Automatically deallocates when last reference dropped

**AsyncRead + Unpin**: The `AsyncRead` trait is for things you can read from asynchronously. The `Unpin` bound means "this type can be moved freely in memory." Most types are `Unpin`, but futures that are currently being polled might not be. It's Rust's way of saying "I promise not to create self-referential nightmares."

**Trait bounds explained:**
• `AsyncRead`: Can read bytes asynchronously
• `Unpin`: Safe to move in memory (not self-referential)
• `Send`: Can be transferred between threads
• `Sync`: Can be shared between threads
• `'static`: No non-static references (can live forever)

**read_exact vs read**: `read()` is like asking for a glass of water - you might get less than a full glass. `read_exact()` keeps asking until the glass is full or the tap runs dry. In TCP, packets arrive in chunks, so `read()` might return 10 bytes when you asked for 100. Always handle partial reads in network code.

**Network reading patterns:**
• `read()`: Returns whatever is available (1 to n bytes)
• `read_exact()`: Loops until exactly n bytes or EOF
• `read_buf()`: Reads into uninitialized memory (advanced)
• `read_to_end()`: Reads until EOF (dangerous for streams)

**The tokio::io::Cursor**: It's a fake "file" that reads from memory - perfect for testing. It implements `AsyncRead` just like a real TCP socket would, but it's deterministic and doesn't require actual networking.

**Testing utilities:**
• `Cursor`: In-memory AsyncRead/AsyncWrite
• `DuplexStream`: Bidirectional in-memory pipe
• `timeout()`: Wrap any future with a deadline
• `yield_now()`: Force a yield point for testing races

**Hint**: Use `BytesMut` as an accumulator, but convert to `Bytes` (via `freeze()`) for zero-copy sharing. The tricky part is handling partial reads - you might get 10 bytes when you asked for 24.

---

## Day 3: The Delta Layer Compaction Problem

### The Mental Model

Think of our storage like a stack of transparent sheets, each with some changes drawn on them. To see the current state, you look down through all the sheets. Over time, you get too many sheets, making lookups slow.

Compaction is like taking multiple sheets and redrawing them onto fewer sheets, preserving all the information but making it faster to look through. The challenge? You're doing this while people are still looking at and adding new sheets. It's like reorganizing a library while it's open - you can't just dump everything on the floor and start over.

Delta layers specifically store changes between versions, like "git diffs" for database pages. Compacting them means merging these diffs efficiently without loading everything into memory.

### The Challenge

```rust
// src/day3.rs
use std::sync::Arc;
use futures::stream::{self, Stream, StreamExt};
use tokio_util::sync::CancellationToken;
use bytes::Bytes;

mod common;
use common::*;

pub struct DeltaEntry {
    pub key: Key,
    pub lsn: Lsn,
    pub value: Bytes,
}

pub struct DeltaLayer {
    pub start_lsn: Lsn,
    pub end_lsn: Lsn,
    entries: Vec<DeltaEntry>,  // In production, this would be a file
}

impl DeltaLayer {
    // Returns a stream of entries in key-lsn order
    pub fn stream_entries(&self) -> impl Stream<Item = DeltaEntry> + '_ {
        stream::iter(self.entries.iter().cloned())
    }
    
    pub fn size_bytes(&self) -> u64 {
        self.entries.iter().map(|e| e.value.len() as u64).sum()
    }
}

pub async fn compact_deltas(
    deltas: Vec<Arc<DeltaLayer>>,
    target_file_size: u64,
    cancel: CancellationToken,
) -> Result<Vec<Arc<DeltaLayer>>, CompactionError> {
    // Your task: Implement k-way merge of sorted streams
    //
    // Requirements:
    // 1. Input layers are sorted by (key, lsn)
    // 2. Output layers must maintain this sorting
    // 3. Each output layer should be close to target_file_size
    // 4. Must handle cancellation - check cancel.is_cancelled()
    // 5. Use streaming to avoid loading all data in memory
    // 6. Track progress with tracing::info!
    //
    // Metaphor: You're merging multiple sorted decks of cards into
    // new decks of a target size, while someone might tell you to stop
    // at any moment. You can only look at the top card of each deck.
    
    tracing::info!("Starting compaction of {} layers", deltas.len());
    
    // Skeleton to get you started:
    use futures::stream::select_all;
    
    // Create streams from all input layers
    let streams = deltas.iter().map(|layer| {
        layer.stream_entries()
    }).collect::<Vec<_>>();
    
    // Merge all streams...
    let mut merged = select_all(streams);
    
    let mut current_layer_entries = Vec::new();
    let mut current_size = 0u64;
    let mut result_layers = Vec::new();
    
    while let Some(entry) = merged.next().await {
        if cancel.is_cancelled() {
            return Err(CompactionError::Cancelled);
        }
        
        // Your logic here:
        // - Add entry to current_layer_entries
        // - Track current_size
        // - When current_size >= target_file_size, create new layer
        // - Handle the last partial layer
        
        todo!("Implement the compaction logic")
    }
    
    Ok(result_layers)
}

#[tokio::main]
async fn main() {
    println!("Implement k-way merge compaction!");
    
    // Test data: Create some delta layers
    let layer1 = Arc::new(DeltaLayer {
        start_lsn: Lsn(0),
        end_lsn: Lsn(100),
        entries: vec![
            DeltaEntry { key: Key { field1: 1, field2: 0, field3: 0, field4: 0 }, lsn: Lsn(10), value: Bytes::from(vec![1; 100]) },
            DeltaEntry { key: Key { field1: 2, field2: 0, field3: 0, field4: 0 }, lsn: Lsn(20), value: Bytes::from(vec![2; 100]) },
        ],
    });
    
    let cancel = CancellationToken::new();
    let result = compact_deltas(vec![layer1], 150, cancel).await;
    
    match result {
        Ok(layers) => println!("Created {} compacted layers", layers.len()),
        Err(e) => println!("Compaction failed: {}", e),
    }
}
```

### An Aside

**impl Stream<Item = T> + '_**: This returns an anonymous type that implements Stream. The `'_` lifetime means "figure it out yourself, compiler" - it's borrowing from `self` for as long as the stream lives. Streams are async iterators - instead of `next()` returning `Option<T>`, it returns `Future<Output = Option<T>>`. Each call to `next().await` might suspend while waiting for the next item.

**Stream fundamentals:**
• `Stream`: Async version of `Iterator`
• `next()`: Returns `Future<Option<T>>`
• `poll_next()`: Low-level polling interface
• `StreamExt`: Extension trait with combinators
• Backpressure built-in: consumer controls pace

**CancellationToken**: This is cooperative cancellation - like a fire alarm that politely asks everyone to leave rather than forcibly ejecting them. When `cancel()` is called, `is_cancelled()` returns true, but running code must check it. It's not like killing a thread; it's setting a flag that says "please stop when convenient."

**Cancellation patterns:**
• `CancellationToken`: Shared flag approach
• `tokio::select!`: Race cancellation against work
• `abortable()`: Creates abortable future wrapper
• Drop-based: Dropping future cancels it
• Always make cancellation points explicit

**select_all**: Takes multiple streams and merges them into one stream that yields items from whichever stream has data ready first. Warning: it doesn't preserve ordering across streams! If you need sorted output from sorted inputs, `select_all` is wrong - you need a proper k-way merge with a `BinaryHeap`.

**Stream merging strategies:**
• `select_all`: First ready wins (unordered)
• `futures::join!`: Wait for all (parallel)
• `stream::iter().then()`: Sequential processing
• `buffer_unordered(n)`: Process n items concurrently
• `try_join!`: Like join! but short-circuits on error

**StreamExt trait**: Adds combinators to streams, like `map`, `filter`, `buffer_unordered`. Think of it as the async version of `Iterator` methods. The `buffer_unordered(n)` method is particularly powerful - it processes up to `n` futures concurrently, yielding results as they complete.

**Common StreamExt methods:**
• `map()`: Transform each item
• `filter()`: Skip items conditionally
• `then()`: Async transformation
• `collect()`: Gather into collection
• `fold()`: Reduce to single value
• `buffer_unordered(n)`: Concurrent processing

**K-way merge pattern (the correct approach):**
```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

// Keep track of which stream produced each item
struct HeapEntry {
    value: DeltaEntry,
    stream_idx: usize,
}

// Use BinaryHeap to always get the smallest item
let mut heap = BinaryHeap::new();
// Initialize with first item from each stream
// Pop minimum, push next from same stream
```

**Key Insight**: The `select_all` approach above is actually wrong for maintaining sort order! You need a proper k-way merge. Consider using a `BinaryHeap` to track which stream has the next smallest entry.

---

## Day 4: Request Routing with Axum

### The Mental Model

Building an API is like being an air traffic controller. Requests come in, you need to route them to the right handler, make sure they have proper identification (auth), log their journey (tracing), and handle any emergencies (errors) gracefully.

The twist in production systems: you need to track context across async boundaries. When a request fails three function calls deep, you need to know which tenant and timeline it was for. This is where structured logging with tracing spans becomes critical.

### The Challenge

```rust
// src/day4.rs
use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    http::StatusCode,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use dashmap::DashMap;
use tracing::{info, error, Instrument};

mod common;
use common::*;

#[derive(Serialize)]
pub struct TimelineInfo {
    pub timeline_id: TimelineId,
    pub last_lsn: Lsn,
    pub size_bytes: u64,
}

pub struct TenantState {
    pub timelines: DashMap<TimelineId, TimelineInfo>,
}

pub struct AppState {
    pub tenants: Arc<DashMap<TenantId, Arc<TenantState>>>,
}

// Your task: Implement proper error handling and tracing
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        // Map errors to appropriate HTTP status codes
        let (status, message) = match self {
            ApiError::TenantNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::TimelineNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };
        
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

async fn timeline_detail(
    Path((tenant_id, timeline_id)): Path<(TenantId, TimelineId)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<TimelineInfo>, ApiError> {
    // Your implementation:
    // 1. Create a tracing span with tenant_id and timeline_id
    // 2. Look up tenant, return proper error if not found
    // 3. Look up timeline within tenant
    // 4. Log successful lookups at info level
    // 5. Log failures at error level with context
    
    let span = tracing::info_span!(
        "timeline_detail",
        tenant_id = ?tenant_id,
        timeline_id = ?timeline_id
    );
    
    async move {
        info!("Looking up timeline");
        
        // Your code here
        todo!("Implement timeline lookup with proper error handling")
    }
    .instrument(span)
    .await
}

async fn create_timeline(
    Path(tenant_id): Path<TenantId>,
    State(state): State<Arc<AppState>>,
    Json(create_req): Json<TimelineCreateRequest>,
) -> Result<Json<TimelineInfo>, ApiError> {
    // Similar pattern: Add tracing, handle errors
    todo!("Implement timeline creation")
}

#[derive(Deserialize)]
struct TimelineCreateRequest {
    timeline_id: Option<TimelineId>,  // Optional - generate if not provided
    ancestor_lsn: Option<Lsn>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/tenants/:tenant_id/timelines/:timeline_id", 
            axum::routing::get(timeline_detail))
        .route("/tenants/:tenant_id/timelines",
            axum::routing::post(create_timeline))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    let state = Arc::new(AppState {
        tenants: Arc::new(DashMap::new()),
    });
    
    // Add some test data
    let tenant_id = TenantId(uuid::Uuid::new_v4());
    let tenant_state = Arc::new(TenantState {
        timelines: DashMap::new(),
    });
    tenant_state.timelines.insert(
        TimelineId(uuid::Uuid::new_v4()),
        TimelineInfo {
            timeline_id: TimelineId(uuid::Uuid::new_v4()),
            last_lsn: Lsn(1000),
            size_bytes: 1024 * 1024,
        },
    );
    state.tenants.insert(tenant_id, tenant_state);
    
    let app = create_router(state);
    
    println!("Server starting on http://localhost:3000");
    println!("Try: curl http://localhost:3000/tenants/{}/timelines/{{timeline_id}}", tenant_id.0);
    
    // Uncomment to run:
    // axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();
}
```

### An Aside

**Path extractors**: Axum's `Path` extractor destructures URL segments directly into your types. `Path((a, b))` with double parens extracts a tuple. The types must implement `Deserialize`. It's compile-time routing - if the types don't match the route definition, it won't compile.

**Axum extractors:**
• `Path`: Extract URL segments
• `Query`: Extract query parameters
• `Json`: Parse request body as JSON
• `State`: Clone app state (cheap with Arc)
• `Header`: Extract specific headers
• Order matters: consume body last

**State extractor**: `State<T>` clones `T` on each request (that's why we wrap everything in `Arc`). It's how you share app-wide resources. The actual cloning is just incrementing an atomic reference count - cheap. Without `Arc`, you'd be deep-cloning your entire application state on every request.

**State management patterns:**
• Always wrap state in `Arc`
• Use `RwLock` for rarely-changing data
• Use `DashMap` for concurrent mutations
• Consider `ArcSwap` for config hot-reloading
• State must be `Clone + Send + Sync + 'static`

**DashMap**: A concurrent HashMap that shards itself internally. Unlike `RwLock<HashMap>`, different threads can modify different keys simultaneously. Each shard has its own lock. It's like having multiple separate HashMaps with a router in front - much better concurrency for hot paths.

**DashMap internals:**
• Default: 16 shards (configurable)
• Key hash determines shard
• Each shard has its own RwLock
• Iteration locks shards sequentially
• Entry API for atomic updates

**tracing::Instrument**: Attaches a span to a future. Every log message inside that future automatically includes the span's fields. The `?` in `tenant_id = ?tenant_id` means "use Debug formatting." Without `?`, it would need Display. Spans form a tree - child spans inherit parent context.

**Tracing span features:**
• Fields become structured log data
• `?` prefix: Debug formatting
• `%` prefix: Display formatting
• No prefix: requires Display
• Fields visible to all child spans
• Async-aware: follows futures across threads

**IntoResponse trait**: Axum's way of converting your types into HTTP responses. Implement it for your error types to get automatic error handling. The framework calls `into_response()` on whatever you return. Even `Json<T>` implements `IntoResponse`, setting the content-type header automatically.

**Response conversion chain:**
```rust
YourType 
  → IntoResponse::into_response() 
  → Response<BoxBody>
  → HTTP response
```

**Error handling patterns in Axum:**
• Return `Result<T, E>` where both T and E implement `IntoResponse`
• Use `thiserror` for error types
• Map domain errors to HTTP status codes
• Include request ID in error responses
• Log errors with full context, return sanitized messages

**Pro tip**: The tracing span pattern shown above ensures that every log message within the request includes the tenant_id and timeline_id automatically. This is invaluable when debugging production issues.

---

## Day 5: The Connection Pool Problem

### The Mental Model

Connection pools are like a fleet of taxis. You don't want every person to own their own taxi (too expensive), but you also don't want just one taxi for the whole city (too slow). You need a pool of reusable connections that can be shared efficiently.

The complications: 
- Some taxis break down (connection failures)
- Rush hour happens (surge in requests)  
- Some destinations are temporarily closed (backend outages)

This is where patterns like circuit breakers come in - if a destination keeps failing, stop sending taxis there for a while.

### The Challenge

```rust
// src/day5.rs
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::{sleep, timeout};
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

mod common;
use common::*;

#[async_trait]
pub trait RemoteStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Bytes, RemoteStorageError>;
    async fn put(&self, key: &str, data: Bytes) -> Result<(), RemoteStorageError>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>, RemoteStorageError>;
}

pub struct ConnectionPoolConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: Duration,
}

pub struct ConnectionPool<T: RemoteStorage> {
    storage: Arc<T>,
    semaphore: Arc<Semaphore>,
    config: ConnectionPoolConfig,
    
    // Circuit breaker state
    consecutive_failures: AtomicU32,
    circuit_opened_at: AtomicU64,  // Timestamp in millis
    
    // Metrics
    total_requests: AtomicU64,
    failed_requests: AtomicU64,
}

impl<T: RemoteStorage> ConnectionPool<T> {
    pub fn new(storage: T, config: ConnectionPoolConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        
        Self {
            storage: Arc::new(storage),
            semaphore,
            config,
            consecutive_failures: AtomicU32::new(0),
            circuit_opened_at: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
        }
    }
    
    async fn is_circuit_open(&self) -> bool {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures < self.config.circuit_breaker_threshold {
            return false;
        }
        
        let opened_at = self.circuit_opened_at.load(Ordering::Relaxed);
        if opened_at == 0 {
            // Just opened
            self.circuit_opened_at.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                Ordering::Relaxed
            );
            return true;
        }
        
        // Check if timeout has passed
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        let timeout_ms = self.config.circuit_breaker_timeout.as_millis() as u64;
        
        if now - opened_at > timeout_ms {
            // Reset circuit breaker
            self.consecutive_failures.store(0, Ordering::Relaxed);
            self.circuit_opened_at.store(0, Ordering::Relaxed);
            false
        } else {
            true
        }
    }
    
    pub async fn get_with_retries(&self, key: &str) -> Result<Bytes, RemoteStorageError> {
        // Your implementation here:
        // 1. Check circuit breaker
        // 2. Acquire semaphore permit
        // 3. Implement exponential backoff retry logic
        // 4. Update metrics
        // 5. Handle circuit breaker state transitions
        
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if self.is_circuit_open().await {
            return Err(RemoteStorageError::Network("Circuit breaker open".to_string()));
        }
        
        let mut attempt = 0;
        let mut backoff = self.config.initial_backoff;
        
        loop {
            // Your retry logic here
            let _permit = self.semaphore.acquire().await.unwrap();
            
            match timeout(self.config.connection_timeout, self.storage.get(key)).await {
                Ok(Ok(data)) => {
                    // Success - reset consecutive failures
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                    return Ok(data);
                }
                Ok(Err(e)) if attempt < self.config.max_retries => {
                    // Failure but can retry
                    attempt += 1;
                    self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
                    
                    tracing::warn!(
                        "Attempt {} failed for key {}: {}", 
                        attempt, key, e
                    );
                    
                    // Exponential backoff
                    sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, self.config.max_backoff);
                    continue;
                }
                Ok(Err(e)) => {
                    // Final failure
                    self.failed_requests.fetch_add(1, Ordering::Relaxed);
                    self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
                    return Err(e);
                }
                Err(_) => {
                    // Timeout
                    self.failed_requests.fetch_add(1, Ordering::Relaxed);
                    self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
                    
                    if attempt < self.config.max_retries {
                        attempt += 1;
                        sleep(backoff).await;
                        backoff = std::cmp::min(backoff * 2, self.config.max_backoff);
                        continue;
                    } else {
                        return Err(RemoteStorageError::Timeout);
                    }
                }
            }
        }
    }
    
    pub async fn metrics(&self) -> ConnectionPoolMetrics {
        ConnectionPoolMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            circuit_breaker_open: self.is_circuit_open().await,
        }
    }
}

pub struct ConnectionPoolMetrics {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub circuit_breaker_open: bool,
}

// Mock storage for testing
struct MockStorage {
    failure_rate: f32,
}

#[async_trait]
impl RemoteStorage for MockStorage {
    async fn get(&self, _key: &str) -> Result<Bytes, RemoteStorageError> {
        // Simulate network delay
        sleep(Duration::from_millis(10)).await;
        
        // Simulate failures
        if rand::random::<f32>() < self.failure_rate {
            Err(RemoteStorageError::Network("Simulated failure".to_string()))
        } else {
            Ok(Bytes::from("data"))
        }
    }
    
    async fn put(&self, _key: &str, _data: Bytes) -> Result<(), RemoteStorageError> {
        sleep(Duration::from_millis(10)).await;
        Ok(())
    }
    
    async fn list(&self, _prefix: &str) -> Result<Vec<String>, RemoteStorageError> {
        Ok(vec![])
    }
}

#[tokio::main]
async fn main() {
    let storage = MockStorage { failure_rate: 0.3 };  // 30% failure rate
    
    let config = ConnectionPoolConfig {
        max_connections: 10,
        connection_timeout: Duration::from_secs(5),
        max_retries: 3,
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(10),
        circuit_breaker_threshold: 5,
        circuit_breaker_timeout: Duration::from_secs(30),
    };
    
    let pool = Arc::new(ConnectionPool::new(storage, config));
    
    // Simulate concurrent requests
    let mut handles = vec![];
    for i in 0..20 {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            match pool.get_with_retries(&format!("key{}", i)).await {
                Ok(_) => println!("Request {} succeeded", i),
                Err(e) => println!("Request {} failed: {}", i, e),
            }
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let metrics = pool.metrics().await;
    println!("Total requests: {}", metrics.total_requests);
    println!("Failed requests: {}", metrics.failed_requests);
    println!("Circuit breaker open: {}", metrics.circuit_breaker_open);
}
```

### An Aside

**async_trait**: Rust doesn't natively support async functions in traits (yet). This macro rewrites your async trait methods to return `Pin<Box<dyn Future>>`. It adds a heap allocation per call, but makes async traits ergonomic. The alternative is manually writing associated types for each future - painful.

**How async_trait works:**
```rust
// You write:
async fn get(&self) -> Result<T>

// Macro generates:
fn get(&self) -> Pin<Box<dyn Future<Output = Result<T>>>>
```

**Why we need it:**
• Trait methods can't be async (no associated type for the future)
• Return type must be known at compile time
• Box provides a known size
• Pin ensures the future won't move

**Semaphore**: Not your OS semaphore - this is an async-aware counting semaphore. `acquire()` returns a future that completes when a permit is available. The permit is RAII - dropping it releases back to the semaphore. It's like a bouncer counting people: "Room for 10, you're number 11, wait outside."

**Semaphore patterns:**
• `acquire()`: Wait for permit (async)
• `try_acquire()`: Non-blocking attempt
• `acquire_many(n)`: Get n permits at once
• `add_permits(n)`: Dynamically increase capacity
• Permit drop automatically releases

**Ordering on Atomics**: 
• `Relaxed`: "Just make it atomic, I don't care about synchronization"
• `Acquire/Release`: Forms happens-before relationships (like mutex lock/unlock)
• `SeqCst`: Global total order, most expensive, usually overkill

**When to use each:**
• Counters: `Relaxed` (just need atomicity)
• Flags: `Acquire/Release` (synchronize with data)
• Complex state machines: `SeqCst` (when in doubt)

**timeout() wrapper**: Creates a new future that races your future against a timer. If the timer wins, you get `Err(Elapsed)`. If your future wins, you get `Ok(your_result)`. The original future is cancelled (dropped) if it times out. Always put timeouts on network operations.

**Timeout composition:**
```rust
timeout(duration, future).await
// Returns Result<T, Elapsed>
// T = future's output type
// Elapsed = timeout error
```

**Exponential backoff math**: Start at 100ms, double each time, cap at 10s. This prevents thundering herd - if a service is down and 1000 clients all retry immediately, they'll overwhelm it when it comes back. Spreading retries over time gives the service a chance to recover.

**Backoff strategies:**
• Linear: delay = attempt * base_delay
• Exponential: delay = base_delay * 2^attempt
• Jittered: Add random variation to prevent synchronization
• Capped: Never exceed max_delay

**Circuit breaker states:**
• **Closed**: Normal operation, requests flow through
• **Open**: Too many failures, reject immediately
• **Half-open**: After timeout, allow one test request

**Key insight**: The circuit breaker pattern prevents cascading failures. When a backend is struggling, continuously hammering it with retries makes things worse. Better to fail fast and give it time to recover.

---

## Day 6: The Vectored Read Optimization

### The Mental Model

Imagine you're at a library and need pages 1, 2, 3, 7, 8, and 20 from a book. You have two strategies:
1. Make 6 trips to get each page individually
2. Make 2 trips: get pages 1-8 in one go (even though you don't need 4-6), and page 20 separately

The second strategy reads some unnecessary data but makes fewer trips. This is the vectored read optimization - grouping nearby reads together even if it means reading some data you don't need.

The art is in deciding when to merge reads. Too aggressive and you're reading lots of unnecessary data. Too conservative and you're making too many I/O operations.

### The Challenge

```rust
// src/day6.rs
use std::ops::Range;

mod common;
use common::*;

#[derive(Debug, Clone)]
pub struct ReadOp {
    pub key_range: Range<Key>,
    pub lsn: Lsn,
}

pub struct VectoredReadPlanner {
    max_gap: usize,      // Maximum gap between reads to merge (in key space)
    max_size: usize,     // Maximum size of merged read (in keys)
    max_ops: usize,      // Maximum number of operations to return
}

impl VectoredReadPlanner {
    pub fn new(max_gap: usize, max_size: usize, max_ops: usize) -> Self {
        Self { max_gap, max_size, max_ops }
    }
    
    pub fn plan_reads(&self, mut requests: Vec<(Key, Lsn)>) -> Vec<ReadOp> {
        if requests.is_empty() {
            return vec![];
        }
        
        // Sort by key first, then by LSN
        requests.sort_by(|a, b| {
            a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1))
        });
        
        let mut ops = Vec::new();
        let mut current_op: Option<ReadOp> = None;
        
        for (key, lsn) in requests {
            match current_op.as_mut() {
                None => {
                    // Start first operation
                    current_op = Some(ReadOp {
                        key_range: key..self.next_key(key),
                        lsn,
                    });
                }
                Some(op) => {
                    // Decide whether to merge or start new operation
                    let gap = self.key_distance(&op.key_range.end, &key);
                    let current_size = self.key_distance(&op.key_range.start, &op.key_range.end);
                    
                    if gap <= self.max_gap && 
                       current_size + gap < self.max_size &&
                       op.lsn == lsn  // Only merge if same LSN
                    {
                        // Extend current operation
                        op.key_range.end = self.next_key(key);
                    } else {
                        // Start new operation
                        ops.push(current_op.take().unwrap());
                        
                        if ops.len() >= self.max_ops {
                            break;  // Hit operation limit
                        }
                        
                        current_op = Some(ReadOp {
                            key_range: key..self.next_key(key),
                            lsn,
                        });
                    }
                }
            }
        }
        
        // Don't forget the last operation
        if let Some(op) = current_op {
            ops.push(op);
        }
        
        ops
    }
    
    fn key_distance(&self, k1: &Key, k2: &Key) -> usize {
        // Simplified distance calculation
        // In reality, this would consider the hierarchical nature of keys
        if k1.field1 != k2.field1 || k1.field2 != k2.field2 || k1.field3 != k2.field3 {
            usize::MAX  // Different relations, infinite distance
        } else {
            (k2.field4.saturating_sub(k1.field4)) as usize
        }
    }
    
    fn next_key(&self, key: Key) -> Key {
        Key {
            field1: key.field1,
            field2: key.field2,
            field3: key.field3,
            field4: key.field4 + 1,
        }
    }
}

// Your advanced challenge: Implement cost-based planning
pub struct CostBasedReadPlanner {
    io_cost_per_op: f64,      // Fixed cost per I/O operation
    io_cost_per_byte: f64,    // Variable cost per byte read
    page_size: usize,          // Size of each page in bytes
}

impl CostBasedReadPlanner {
    pub fn plan_reads(&self, requests: Vec<(Key, Lsn)>) -> Vec<ReadOp> {
        // Your implementation:
        // Instead of fixed thresholds, use cost modeling
        // 
        // Cost = (num_ops * io_cost_per_op) + (total_bytes * io_cost_per_byte)
        //
        // For each potential merge, calculate:
        // - Cost of separate reads
        // - Cost of merged read (including wasted bytes)
        // - Choose the option with lower cost
        
        todo!("Implement cost-based read planning")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_merging() {
        let planner = VectoredReadPlanner::new(5, 100, 10);
        
        let requests = vec![
            (Key { field1: 1, field2: 1, field3: 1, field4: 1 }, Lsn(100)),
            (Key { field1: 1, field2: 1, field3: 1, field4: 2 }, Lsn(100)),
            (Key { field1: 1, field2: 1, field3: 1, field4: 3 }, Lsn(100)),
            (Key { field1: 1, field2: 1, field3: 1, field4: 10 }, Lsn(100)),
            (Key { field1: 1, field2: 1, field3: 1, field4: 11 }, Lsn(100)),
            (Key { field1: 1, field2: 1, field3: 1, field4: 100 }, Lsn(100)),
        ];
        
        let ops = planner.plan_reads(requests);
        
        // Should merge 1-3 (consecutive), 10-11 (gap of 1), leave 100 alone
        assert_eq!(ops.len(), 3);
        
        // First op should cover keys 1-3
        assert_eq!(ops[0].key_range.start.field4, 1);
        assert_eq!(ops[0].key_range.end.field4, 4);
        
        // Second op should cover keys 10-11  
        assert_eq!(ops[1].key_range.start.field4, 10);
        assert_eq!(ops[1].key_range.end.field4, 12);
        
        // Third op should be key 100 alone
        assert_eq!(ops[2].key_range.start.field4, 100);
    }
    
    #[test]
    fn test_different_lsns() {
        let planner = VectoredReadPlanner::new(5, 100, 10);
        
        let requests = vec![
            (Key { field1: 1, field2: 1, field3: 1, field4: 1 }, Lsn(100)),
            (Key { field1: 1, field2: 1, field3: 1, field4: 2 }, Lsn(200)),  // Different LSN
            (Key { field1: 1, field2: 1, field3: 1, field4: 3 }, Lsn(100)),
        ];
        
        let ops = planner.plan_reads(requests);
        
        // Should not merge across different LSNs
        assert_eq!(ops.len(), 3);
    }
}

fn main() {
    println!("Implement vectored read optimization!");
    
    // Run tests with: cargo test --bin day6
    
    // Example usage:
    let planner = VectoredReadPlanner::new(5, 100, 10);
    
    let requests = vec![
        (Key { field1: 1, field2: 1, field3: 1, field4: 1 }, Lsn(100)),
        (Key { field1: 1, field2: 1, field3: 1, field4: 2 }, Lsn(100)),
        (Key { field1: 1, field2: 1, field3: 1, field4: 10 }, Lsn(100)),
        (Key { field1: 1, field2: 1, field3: 1, field4: 50 }, Lsn(100)),
    ];
    
    let ops = planner.plan_reads(requests);
    
    for op in ops {
        println!("Read operation: keys {:?} at LSN {:?}", 
                 op.key_range, op.lsn);
    }
}
```

### An Aside

**Range<T>**: Rust's half-open interval type - `start..end` includes start but excludes end. This makes adjacent ranges naturally non-overlapping: `0..5` and `5..10` don't overlap. Perfect for representing key ranges. Implements `Iterator` if T is numeric.

**Range types in Rust:**
• `start..end`: Half-open (excludes end)
• `start..=end`: Closed (includes end)
• `..end`: From beginning to end
• `start..`: From start to end of collection
• `..`: Full range

**Option::take()**: Moves the value out of an Option, leaving None behind. It's like removing something from a box and leaving the box empty. `current_op.take().unwrap()` says "I know there's something in here, give it to me and leave None." Useful for state machines where you transition by consuming the old state.

**Option manipulation methods:**
• `take()`: Move out value, leave None
• `replace()`: Swap in new value, return old
• `as_ref()`: Borrow inside without moving
• `as_mut()`: Mutably borrow inside
• `map()`: Transform if Some

**saturating_sub**: Subtraction that clamps to 0 instead of panicking on underflow. `5u32.saturating_sub(10)` gives 0, not a panic. Essential for defensive programming with untrusted input. There's also `wrapping_sub` (wraps around) and `checked_sub` (returns Option).

**Arithmetic safety methods:**
• `saturating_*`: Clamp to min/max
• `wrapping_*`: Allow overflow/underflow
• `checked_*`: Return None on overflow
• `overflowing_*`: Return (result, did_overflow)
• Default ops panic in debug, wrap in release

**#[cfg(test)]**: Conditional compilation - this code only exists when running tests. The test module and its imports disappear in release builds. Keeps your binary lean. You can also use `#[cfg(feature = "...")]` for feature flags.

**Conditional compilation attributes:**
• `#[cfg(test)]`: Only in test builds
• `#[cfg(debug_assertions)]`: Only in debug
• `#[cfg(target_os = "linux")]`: Platform-specific
• `#[cfg(feature = "foo")]`: Feature-gated
• `#[cfg(not(...))]`: Negation

**then_with() in sorting**: Chains comparison operations. `a.cmp(&b).then_with(|| c.cmp(&d))` means "sort by a/b first, break ties with c/d." The closure is only called if the first comparison is Equal. More efficient than tuple comparison when the secondary key is expensive to compute.

**Comparison chaining methods:**
• `then()`: Chain with another Ordering
• `then_with()`: Lazy evaluation with closure
• `reverse()`: Flip the ordering
• Useful for multi-level sorting

**Cost-based optimization hints:**
```rust
// Consider these factors:
const SEEK_COST: f64 = 1.0;      // Fixed cost per I/O
const READ_COST: f64 = 0.001;    // Cost per byte
const PAGE_SIZE: usize = 8192;   // Typical page size

// Merge if:
// cost(separate) > cost(merged)
// where cost = SEEK_COST + (bytes * READ_COST)
```

**Real-world note**: This optimization can reduce I/O operations by 10x in workloads with spatial locality. The key is tuning the parameters based on your storage backend's characteristics.

---

## Final Challenge: The Reconciliation Loop

### The Mental Model

Reconciliation is like being an accountant who needs to match two sets of books that are being modified while you're reading them. You have local state (what you think you have) and remote state (the source of truth). Your job is to make them match without stopping business operations.

The complexity comes from:
1. **Scale**: Millions of files to check
2. **Concurrency**: Other processes are modifying both sets while you work
3. **Failures**: Network issues, partial downloads, crashes mid-reconciliation
4. **Performance**: Can't block foreground operations or use too much memory

This is the kind of unglamorous but critical work that keeps distributed systems running.

### The Challenge

```rust
// src/final.rs
use std::sync::Arc;
use futures::stream::{self, StreamExt};
use tokio_util::sync::CancellationToken;
use dashmap::DashMap;
use bytes::Bytes;
use std::collections::HashSet;

mod common;
use common::*;

#[async_trait::async_trait]
trait RemoteStorage: Send + Sync {
    async fn list_files(&self, prefix: &str) -> Result<Vec<String>, RemoteStorageError>;
    async fn download(&self, key: &str) -> Result<Bytes, RemoteStorageError>;
    async fn upload(&self, key: &str, data: Bytes) -> Result<(), RemoteStorageError>;
}

pub struct LayerFileName {
    pub tenant_id: TenantId,
    pub timeline_id: TimelineId,
    pub start_lsn: Lsn,
    pub end_lsn: Lsn,
    pub generation: u32,  // Incremented on each modification
}

impl LayerFileName {
    fn parse(name: &str) -> Option<Self> {
        // Parse format: tenant_timeline_startlsn_endlsn_generation
        todo!("Parse layer file name")
    }
    
    fn to_string(&self) -> String {
        format!("{}_{}_{}_{}_{}",
            self.tenant_id.0,
            self.timeline_id.0, 
            self.start_lsn.0,
            self.end_lsn.0,
            self.generation
        )
    }
}

pub struct Tenant {
    pub tenant_id: TenantId,
    pub timelines: DashMap<TimelineId, Arc<Timeline>>,
}

pub struct Timeline {
    pub timeline_id: TimelineId,
    
    // Local layers - being actively modified by other tasks
    pub local_layers: DashMap<String, Arc<Layer>>,
    
    // Layers we know exist in remote storage
    pub remote_index: Arc<RwLock<HashSet<String>>>,
    
    // Layers currently being uploaded/downloaded
    pub in_flight: DashMap<String, InFlightOp>,
}

pub struct Layer {
    pub file_name: String,
    pub size: u64,
    pub data: Bytes,
}

pub enum InFlightOp {
    Upload,
    Download,
}

pub struct ReconciliationProgress {
    pub files_checked: AtomicU64,
    pub files_downloaded: AtomicU64,
    pub files_deleted: AtomicU64,
    pub bytes_downloaded: AtomicU64,
    pub last_error: Arc<RwLock<Option<String>>>,
}

pub async fn reconciliation_loop(
    tenant: Arc<Tenant>,
    remote: Arc<dyn RemoteStorage>,
    cancel: CancellationToken,
    progress: Arc<ReconciliationProgress>,
) -> Result<(), ReconcileError> {
    // Your implementation must handle:
    //
    // 1. List all remote files for this tenant
    // 2. For each timeline in the tenant:
    //    a. Compare local vs remote layers
    //    b. Download missing layers
    //    c. Delete local layers not in remote (unless they're being uploaded)
    // 3. Handle concurrent modifications:
    //    - New layers being created locally
    //    - Layers being deleted locally
    //    - Layers being uploaded by other tasks
    // 4. Resumability:
    //    - Track progress so we can resume after a crash
    //    - Don't re-download files we already have
    // 5. Resource management:
    //    - Limit concurrent downloads
    //    - Stream the file listing (don't load all in memory)
    //    - Backpressure when local storage is full
    // 6. Observability:
    //    - Update progress metrics
    //    - Log important events with context
    //    - Record errors without stopping
    
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Reconciliation cancelled");
                return Ok(());
            }
            _ = interval.tick() => {
                if let Err(e) = reconcile_once(
                    tenant.clone(),
                    remote.clone(),
                    progress.clone(),
                    &cancel
                ).await {
                    tracing::error!("Reconciliation iteration failed: {}", e);
                    
                    let mut last_error = progress.last_error.write().await;
                    *last_error = Some(e.to_string());
                    
                    // Continue despite errors - we'll retry next iteration
                }
            }
        }
    }
}

async fn reconcile_once(
    tenant: Arc<Tenant>,
    remote: Arc<dyn RemoteStorage>,
    progress: Arc<ReconciliationProgress>,
    cancel: &CancellationToken,
) -> Result<(), ReconcileError> {
    tracing::info!("Starting reconciliation for tenant {}", tenant.tenant_id.0);
    
    // List all remote files for this tenant
    let prefix = format!("{}/", tenant.tenant_id.0);
    let remote_files = remote.list_files(&prefix).await
        .map_err(|e| ReconcileError::Failed(format!("Failed to list remote files: {}", e)))?;
    
    tracing::info!("Found {} remote files", remote_files.len());
    
    // Group files by timeline
    let mut files_by_timeline: HashMap<TimelineId, Vec<String>> = HashMap::new();
    
    for file_name in remote_files {
        if let Some(parsed) = LayerFileName::parse(&file_name) {
            files_by_timeline.entry(parsed.timeline_id)
                .or_default()
                .push(file_name);
        }
    }
    
    // Process each timeline
    for timeline in tenant.timelines.iter() {
        let timeline_id = timeline.key();
        let timeline = timeline.value();
        
        let remote_files = files_by_timeline.get(timeline_id).cloned().unwrap_or_default();
        
        reconcile_timeline(
            timeline.clone(),
            remote_files,
            remote.clone(),
            progress.clone(),
            cancel
        ).await?;
    }
    
    Ok(())
}

async fn reconcile_timeline(
    timeline: Arc<Timeline>,
    remote_files: Vec<String>,
    remote: Arc<dyn RemoteStorage>,
    progress: Arc<ReconciliationProgress>,
    cancel: &CancellationToken,
) -> Result<(), ReconcileError> {
    // Your implementation here:
    // 1. Update remote_index with the list from remote storage
    // 2. Find files to download (in remote but not local)
    // 3. Find files to delete (in local but not remote)
    // 4. Execute downloads with concurrency limit
    // 5. Delete obsolete local files (carefully!)
    
    // Update remote index
    {
        let mut index = timeline.remote_index.write().await;
        *index = remote_files.iter().cloned().collect();
    }
    
    // Find files to download
    let mut to_download = Vec::new();
    for remote_file in &remote_files {
        if !timeline.local_layers.contains_key(remote_file) &&
           !timeline.in_flight.contains_key(remote_file) {
            to_download.push(remote_file.clone());
        }
    }
    
    // Download files with concurrency limit
    const MAX_CONCURRENT_DOWNLOADS: usize = 3;
    
    let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_DOWNLOADS));
    
    let download_tasks = to_download.into_iter().map(|file_name| {
        let timeline = timeline.clone();
        let remote = remote.clone();
        let progress = progress.clone();
        let semaphore = semaphore.clone();
        let cancel = cancel.clone();
        
        async move {
            let _permit = semaphore.acquire().await.unwrap();
            
            if cancel.is_cancelled() {
                return Ok(());
            }
            
            // Mark as in-flight
            timeline.in_flight.insert(file_name.clone(), InFlightOp::Download);
            
            // Download
            match remote.download(&file_name).await {
                Ok(data) => {
                    let size = data.len() as u64;
                    
                    // Store locally
                    timeline.local_layers.insert(
                        file_name.clone(),
                        Arc::new(Layer {
                            file_name: file_name.clone(),
                            size,
                            data,
                        })
                    );
                    
                    // Update metrics
                    progress.files_downloaded.fetch_add(1, Ordering::Relaxed);
                    progress.bytes_downloaded.fetch_add(size, Ordering::Relaxed);
                    
                    tracing::info!("Downloaded {} ({} bytes)", file_name, size);
                }
                Err(e) => {
                    tracing::error!("Failed to download {}: {}", file_name, e);
                    // Don't fail the whole reconciliation for one file
                }
            }
            
            // Remove from in-flight
            timeline.in_flight.remove(&file_name);
            
            Ok::<(), ReconcileError>(())
        }
    });
    
    // Execute all downloads
    let results: Vec<_> = stream::iter(download_tasks)
        .buffer_unordered(MAX_CONCURRENT_DOWNLOADS)
        .collect()
        .await;
    
    // Check for errors
    for result in results {
        result?;
    }
    
    // Find and delete obsolete local files
    let remote_set: HashSet<String> = remote_files.into_iter().collect();
    
    let mut to_delete = Vec::new();
    for entry in timeline.local_layers.iter() {
        let file_name = entry.key();
        
        // Don't delete if:
        // - It exists in remote
        // - It's being uploaded
        if !remote_set.contains(file_name) &&
           !timeline.in_flight.contains_key(file_name) {
            to_delete.push(file_name.clone());
        }
    }
    
    for file_name in to_delete {
        timeline.local_layers.remove(&file_name);
        progress.files_deleted.fetch_add(1, Ordering::Relaxed);
        tracing::info!("Deleted obsolete local file: {}", file_name);
    }
    
    progress.files_checked.fetch_add(
        timeline.local_layers.len() as u64,
        Ordering::Relaxed
    );
    
    Ok(())
}

// Mock implementation for testing
struct MockRemoteStorage {
    files: Arc<RwLock<HashMap<String, Bytes>>>,
}

#[async_trait::async_trait]
impl RemoteStorage for MockRemoteStorage {
    async fn list_files(&self, prefix: &str) -> Result<Vec<String>, RemoteStorageError> {
        let files = self.files.read().await;
        Ok(files.keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
    
    async fn download(&self, key: &str) -> Result<Bytes, RemoteStorageError> {
        let files = self.files.read().await;
        files.get(key)
            .cloned()
            .ok_or_else(|| RemoteStorageError::NotFound(key.to_string()))
    }
    
    async fn upload(&self, key: &str, data: Bytes) -> Result<(), RemoteStorageError> {
        let mut files = self.files.write().await;
        files.insert(key.to_string(), data);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    println!("Implement the reconciliation loop!");
    
    // Test setup
    let tenant_id = TenantId(uuid::Uuid::new_v4());
    let timeline_id = TimelineId(uuid::Uuid::new_v4());
    
    let tenant = Arc::new(Tenant {
        tenant_id,
        timelines: DashMap::new(),
    });
    
    let timeline = Arc::new(Timeline {
        timeline_id,
        local_layers: DashMap::new(),
        remote_index: Arc::new(RwLock::new(HashSet::new())),
        in_flight: DashMap::new(),
    });
    
    tenant.timelines.insert(timeline_id, timeline);
    
    let remote = Arc::new(MockRemoteStorage {
        files: Arc::new(RwLock::new(HashMap::new())),
    });
    
    let cancel = CancellationToken::new();
    let progress = Arc::new(ReconciliationProgress {
        files_checked: AtomicU64::new(0),
        files_downloaded: AtomicU64::new(0),
        files_deleted: AtomicU64::new(0),
        bytes_downloaded: AtomicU64::new(0),
        last_error: Arc::new(RwLock::new(None)),
    });
    
    // Run reconciliation in background
    let handle = tokio::spawn(reconciliation_loop(
        tenant.clone(),
        remote.clone(),
        cancel.clone(),
        progress.clone(),
    ));
    
    // Let it run for a bit
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Cancel and wait
    cancel.cancel();
    let _ = handle.await;
    
    // Print metrics
    println!("Files checked: {}", progress.files_checked.load(Ordering::Relaxed));
    println!("Files downloaded: {}", progress.files_downloaded.load(Ordering::Relaxed));
    println!("Files deleted: {}", progress.files_deleted.load(Ordering::Relaxed));
    println!("Bytes downloaded: {}", progress.bytes_downloaded.load(Ordering::Relaxed));
}
```

### An Aside

**tokio::select!**: The Swiss Army knife of async concurrency. It races multiple futures and proceeds with whichever completes first, cancelling the others. Here we're racing cancellation against timer ticks. The `_ =` syntax means "I don't care about the value." Think of it as a switch statement for futures - first match wins, others are dropped.

**select! patterns:**
```rust
select! {
    // Biased: Check branches in order
    biased;
    
    // Pattern matching on results
    Ok(val) = future1 => { },
    
    // With timeout
    _ = sleep(timeout) => { /* timeout */ },
    
    // Else branch (always ready)
    else => { },
}
```

**select! semantics:**
• Dropped futures are cancelled
• Can be biased (check in order) or fair (random)
• All branches must have compatible types
• Often used with loops for state machines

**buffer_unordered(n)**: Transforms a stream of futures into a stream of results, running up to `n` futures concurrently. Unlike `join_all`, it yields results as they complete, not in order. This is key for throughput - slow downloads don't block fast ones. The semaphore inside each future provides additional backpressure control.

**Stream buffering methods:**
• `buffer_unordered(n)`: Run n concurrently, yield as ready
• `buffered(n)`: Run n concurrently, maintain order
• `for_each_concurrent(n, f)`: Process n at a time
• `try_for_each_concurrent`: With early exit on error

**DashMap iteration**: When you call `.iter()` on a DashMap, each iteration briefly locks that shard. The iterator holds a guard that's dropped between iterations. This means the map can be modified between iterations - it's not a snapshot. If you need consistency, collect to a Vec first (but that defeats the purpose of sharding).

**DashMap iteration gotchas:**
• Not a snapshot - can see concurrent modifications
• Each iteration locks one shard briefly
• `into_iter()` consumes the map
• Use `entry()` API for atomic read-modify-write

**Arc<dyn Trait>**: Dynamic dispatch through a vtable. Every call to a trait method goes through a pointer indirection. We use `Arc` because `dyn Trait` is unsized - you can't put it directly in a struct. The `Send + Sync` bounds ensure the trait object can be shared across threads safely.

**Trait object requirements:**
• Must be object-safe (no generics, no Self)
• Stored behind pointer (Box, Arc, &)
• Dynamic dispatch has runtime cost
• Can't recover concrete type

**Generation numbers**: A versioning scheme to handle concurrent modifications. Each write increments the generation. If two processes try to write generation 5, one wins and becomes generation 6. The loser must retry with generation 7. This prevents the "lost update" problem without distributed locking.

**Generation number pattern:**
```rust
loop {
    let current = read_current_generation();
    let new_data = prepare_update(current);
    
    match compare_and_swap(current.gen, new_data, current.gen + 1) {
        Ok(_) => break,
        Err(newer_gen) => continue, // Retry with newer generation
    }
}
```

**RAII pattern with in_flight**: We insert into `in_flight` before starting the operation and remove after completion (success or failure). If we crash, the entry remains, signaling incomplete work. This is crash-safety 101 - mark your intent before acting, clean up after. The next reconciliation run can detect and retry incomplete operations.

**Crash-safety checklist:**
• Mark operations as in-progress before starting
• Use write-ahead logging for critical changes
• Make operations idempotent
• Design for partial failure
• Test crash recovery explicitly

**Production considerations for reconciliation:**
• **Streaming file listings**: Use pagination APIs to handle millions of files
• **Checksums**: Verify downloaded files match expected hash
• **Exponential backoff**: Don't hammer remote storage
• **Metrics**: Track reconciliation lag and errors
• **Partial progress**: Save state periodically for resumability

**This is your final test**: Make this production-ready. Consider:
- What happens if we crash mid-download?
- How do we avoid downloading the same file twice?
- What if the remote listing is so large it doesn't fit in memory?
- How do we ensure we don't delete a file that's being uploaded?
- How do we prevent thundering herd when many tenants reconcile simultaneously?

---

## Essential Patterns Reference

### Pin and Self-Referential Structures

**The Problem**: When you have a struct that contains pointers to itself, moving it breaks those pointers.

```rust
// BROKEN: Self-referential struct that breaks when moved
struct BrokenFuture {
    data: String,
    ptr_to_data: *const String,  // Points to `data` field above
}

// After moving, ptr_to_data points to the OLD location!
```

**The Solution**: Pin guarantees the value won't move in memory.

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct CorrectFuture {
    data: String,
    ptr_to_data: *const String,
    _pin: PhantomPinned,  // Makes this !Unpin
}

// Now it can only exist as Pin<Box<CorrectFuture>>
// The compiler ensures it never moves
```

### Arc vs Rc: Concurrency Matters

```rust
// Rc: Single-threaded reference counting
// Arc: Atomic reference counting (thread-safe)

// Choose your lock based on access patterns:
Arc<RwLock<T>>     // Many readers, few writers
Arc<Mutex<T>>      // Frequent writes or need fairness
Arc<DashMap<K,V>>  // Concurrent access to different keys
```

### Zero-Copy with Bytes

```rust
use bytes::{Bytes, BytesMut};

// BAD: Copies data
let subset = buffer[0..100].to_vec();

// GOOD: Just adjusts pointers
let subset = bytes_buffer.slice(0..100);
```

### Stream Backpressure

```rust
// Control concurrency with Semaphore
let semaphore = Arc::new(Semaphore::new(3));

stream::iter(items)
    .map(|item| {
        let sem = semaphore.clone();
        async move {
            let _permit = sem.acquire().await?;
            process(item).await  // Permit dropped when done
        }
    })
    .buffer_unordered(10)
    .collect()
```

### Structured Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Timeline {timeline_id} not found")]
    TimelineNotFound { timeline_id: TimelineId },
}

// Auto-convert to HTTP responses
impl IntoResponse for StorageError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::TimelineNotFound { .. } => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        (status, message).into_response()
    }
}
```

### Tracing Context

```rust
// Create span with context
let span = tracing::info_span!(
    "operation",
    tenant_id = ?tenant_id,
    timeline_id = ?timeline_id
);

// All logs within include the context
async move {
    info!("Processing");  // Automatically includes tenant_id, timeline_id
    // ... work ...
}
.instrument(span)
.await
```

---

## Graduation Criteria

You're ready to work on the Neon storage backend when you can:

1. **Fix race conditions** without adding unnecessary locks
2. **Implement zero-copy patterns** for high-throughput data processing  
3. **Design async systems** with proper cancellation and backpressure
4. **Build resilient APIs** with structured error handling and tracing
5. **Create connection pools** with circuit breakers and retries
6. **Optimize I/O patterns** with vectored reads and cost modeling
7. **Handle distributed state** reconciliation with concurrent modifications

Each challenge you've completed represents a real problem we've solved in production. The patterns you've learned - from careful lock management to zero-copy optimizations to reconciliation loops - are the daily tools of a storage engineer.

Welcome to Neon. You're ready to build the future of PostgreSQL storage.

---

## Next Steps

1. Review the actual Neon codebase at https://github.com/neondatabase/neon
2. Study the pageserver implementation in `/pageserver/src`
3. Read the architecture docs in `/docs/architecture`
4. Join the discussions in GitHub issues and Discord

Remember: In production, every microsecond matters, every byte counts, and every edge case will eventually happen. Code accordingly.
