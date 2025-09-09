// JSON Parsing Benchmark Challenge
use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use serde::{Deserialize, Serialize};

// Test data structure for owned deserialization
#[derive(Serialize, Deserialize, Clone, Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    metadata: Vec<String>,
}

// Zero-copy version with borrowed fields
#[derive(Deserialize, Debug)]
struct LogEntryBorrowed<'a> {
    #[serde(borrow)]
    timestamp: &'a str,
    #[serde(borrow)]
    level: &'a str,
    #[serde(borrow)]
    message: &'a str,
    #[serde(borrow)]
    metadata: Vec<&'a str>,
}

// Approach 1: Standard owned deserialization
fn parse_owned(json: &str) -> Result<LogEntry, serde_json::Error> {
    serde_json::from_str(json)
}

// Approach 2: Zero-copy with borrowed fields
fn parse_borrowed(json: &str) -> Result<LogEntryBorrowed<'_>, serde_json::Error> {
    serde_json::from_str(json)
}

// Approach 3: Manual parsing (potentially faster but less safe)
fn parse_manual(json: &str) -> Option<(String, String, String)> {
    // Simple manual parser - finds key fields by string search
    let json = json.trim_start_matches('{').trim_end_matches('}');
    
    let mut timestamp = String::new();
    let mut level = String::new();
    let mut message = String::new();
    
    for part in json.split(',') {
        let part = part.trim();
        if part.starts_with(r#""timestamp":"#) {
            timestamp = part
                .strip_prefix(r#""timestamp":"#)?
                .trim_matches('"')
                .to_string();
        } else if part.starts_with(r#""level":"#) {
            level = part
                .strip_prefix(r#""level":"#)?
                .trim_matches('"')
                .to_string();
        } else if part.starts_with(r#""message":"#) {
            message = part
                .strip_prefix(r#""message":"#)?
                .trim_matches('"')
                .to_string();
        }
    }
    
    Some((timestamp, level, message))
}

fn benchmark_json_parsing(c: &mut Criterion) {
    let json_data = r#"{"timestamp":"2023-01-01T12:00:00Z","level":"INFO","message":"User login successful","metadata":["user_id:123","session:abc","ip:192.168.1.1"]}"#;
    
    let mut group = c.benchmark_group("json_parsing");
    
    // Benchmark owned parsing
    group.bench_function("owned", |b| {
        b.iter(|| parse_owned(black_box(json_data)).unwrap())
    });
    
    // Benchmark borrowed parsing
    group.bench_function("borrowed", |b| {
        b.iter(|| parse_borrowed(black_box(json_data)).unwrap())
    });
    
    // Benchmark manual parsing
    group.bench_function("manual", |b| {
        b.iter(|| parse_manual(black_box(json_data)).unwrap())
    });
    
    // Test with different JSON sizes
    let large_json = format!(
        r#"{{"timestamp":"2023-01-01T12:00:00Z","level":"INFO","message":"{}","metadata":{}}}"#,
        "Very long message with lots of text that should stress the parser".repeat(10),
        serde_json::to_string(&vec!["data"; 100]).unwrap()
    );
    
    group.bench_function("owned_large", |b| {
        b.iter(|| parse_owned(black_box(&large_json)).unwrap())
    });
    
    group.bench_function("borrowed_large", |b| {
        b.iter(|| parse_borrowed(black_box(&large_json)).unwrap())
    });
    
    // Advanced: Benchmark with allocation tracking
    group.bench_function("owned_with_setup", |b| {
        b.iter_batched(
            || json_data.to_string(), // Setup: create owned string each time
            |owned_json| parse_owned(black_box(&owned_json)).unwrap(),
            BatchSize::SmallInput
        )
    });
    
    group.finish();
}

// Test parsing many small JSON objects vs few large ones
fn benchmark_json_batching(c: &mut Criterion) {
    let single_json = r#"{"timestamp":"2023-01-01T12:00:00Z","level":"INFO","message":"Test message","metadata":["key:value"]}"#;
    
    let many_small: Vec<String> = (0..1000)
        .map(|i| single_json.replace("Test message", &format!("Message {}", i)))
        .collect();
    
    let few_large: Vec<String> = (0..10)
        .map(|i| {
            let large_metadata: Vec<String> = (0..100).map(|j| format!("key{}:value{}", i, j)).collect();
            format!(
                r#"{{"timestamp":"2023-01-01T12:00:00Z","level":"INFO","message":"Large message {}","metadata":{}}}"#,
                i,
                serde_json::to_string(&large_metadata).unwrap()
            )
        })
        .collect();
    
    let mut group = c.benchmark_group("json_batching");
    
    group.bench_function("many_small_owned", |b| {
        b.iter(|| {
            for json in &many_small {
                black_box(parse_owned(json).unwrap());
            }
        })
    });
    
    group.bench_function("many_small_borrowed", |b| {
        b.iter(|| {
            for json in &many_small {
                black_box(parse_borrowed(json).unwrap());
            }
        })
    });
    
    group.bench_function("few_large_owned", |b| {
        b.iter(|| {
            for json in &few_large {
                black_box(parse_owned(json).unwrap());
            }
        })
    });
    
    group.bench_function("few_large_borrowed", |b| {
        b.iter(|| {
            for json in &few_large {
                black_box(parse_borrowed(json).unwrap());
            }
        })
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_json_parsing, benchmark_json_batching);
criterion_main!(benches);