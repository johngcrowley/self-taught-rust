// Day 25: Async Basics - From Callbacks to async/await  
// Time Limit: 15 minutes
// Core Concept: Converting callback-based code to async/await

use tokio::time::{sleep, Duration};
use std::thread;

#[tokio::main]
async fn main() {
    println!("Day 25: Async Basics Challenge");
    
    // Challenge 1: Convert callback to async
    challenge_1().await;
    
    // Challenge 2: Concurrent async operations
    challenge_2().await;
    
    // Challenge 3: Error handling in async
    challenge_3().await;
}

// CHALLENGE 1: Convert this callback-style code to async/await
fn old_fetch_data<F>(url: String, callback: F) 
where
    F: Fn(Result<String, &'static str>) + Send + 'static,
{
    thread::spawn(move || {
        // Simulate network delay
        thread::sleep(Duration::from_millis(100));
        
        // Simulate success/failure based on URL
        if url.contains("valid") {
            callback(Ok(format!("Data from {}", url)));
        } else {
            callback(Err("Network error"));
        }
    });
}

async fn challenge_1() {
    println!("\n=== Challenge 1: Callback to Async ===");
    
    // TODO: Implement new_fetch_data as async function
    // let result = new_fetch_data("https://api.valid.com".to_string()).await;
    // println!("Async result: {:?}", result);
    
    println!("Challenge 1 - TODO: Implement new_fetch_data");
}

// TODO: Implement this async version of fetch_data
async fn new_fetch_data(url: String) -> Result<String, &'static str> {
    // Simulate async work (use tokio::time::sleep instead of thread::sleep)
    // Return Result directly instead of using callback
    todo!()
}

// CHALLENGE 2: Fetch from multiple URLs concurrently
async fn challenge_2() {
    println!("\n=== Challenge 2: Concurrent Fetching ===");
    
    let urls = vec![
        "https://api.valid.com/users".to_string(),
        "https://api.valid.com/posts".to_string(),
        "https://api.valid.com/comments".to_string(),
    ];
    
    // TODO: Fetch all URLs concurrently and collect results
    // HINT: Use tokio::join! or futures::future::join_all
    
    let start = std::time::Instant::now();
    
    // Sequential version (slow):
    let mut results = Vec::new();
    for url in &urls {
        let result = new_fetch_data(url.clone()).await;
        results.push(result);
    }
    
    let sequential_time = start.elapsed();
    println!("Sequential time: {:?}", sequential_time);
    
    // TODO: Implement concurrent version here
    // let concurrent_results = fetch_all_concurrent(urls).await;
    // Should be much faster than sequential!
}

// TODO: Implement concurrent fetching
async fn fetch_all_concurrent(urls: Vec<String>) -> Vec<Result<String, &'static str>> {
    // Use tokio::join! or spawn multiple tasks
    todo!()
}

// CHALLENGE 3: Error handling with timeout
async fn challenge_3() {
    println!("\n=== Challenge 3: Async Error Handling ===");
    
    let slow_url = "https://api.slow.com".to_string();
    
    // TODO: Implement fetch_with_timeout that:
    // 1. Has a timeout of 50ms
    // 2. Returns different error types for timeout vs network error
    // 3. Uses tokio::time::timeout
    
    match fetch_with_timeout(slow_url, Duration::from_millis(50)).await {
        Ok(data) => println!("Success: {}", data),
        Err(TimeoutError::Timeout) => println!("Request timed out"),
        Err(TimeoutError::Network(e)) => println!("Network error: {}", e),
    }
}

// TODO: Define error type for timeout handling
#[derive(Debug)]
enum TimeoutError {
    Timeout,
    Network(&'static str),
}

// TODO: Implement fetch with timeout
async fn fetch_with_timeout(url: String, timeout: Duration) -> Result<String, TimeoutError> {
    // Use tokio::time::timeout to wrap the fetch operation
    // Convert timeout and network errors to appropriate enum variants
    todo!()
}

// SOLUTION TEMPLATE (uncomment and complete):
/*
async fn new_fetch_data(url: String) -> Result<String, &'static str> {
    // Simulate async work
    sleep(Duration::from_millis(100)).await;
    
    if url.contains("valid") {
        Ok(format!("Data from {}", url))
    } else {
        Err("Network error")
    }
}

async fn fetch_all_concurrent(urls: Vec<String>) -> Vec<Result<String, &'static str>> {
    let futures: Vec<_> = urls.into_iter()
        .map(|url| new_fetch_data(url))
        .collect();
    
    futures::future::join_all(futures).await
}

async fn fetch_with_timeout(url: String, timeout_duration: Duration) -> Result<String, TimeoutError> {
    match tokio::time::timeout(timeout_duration, new_fetch_data(url)).await {
        Ok(Ok(data)) => Ok(data),
        Ok(Err(network_err)) => Err(TimeoutError::Network(network_err)),
        Err(_timeout_err) => Err(TimeoutError::Timeout),
    }
}
*/

// ACTIVE RECALL QUESTIONS:
// 1. What is the difference between async fn and fn that returns impl Future?
// 2. When should you use tokio::spawn vs just .await?
// 3. How do you handle errors in async contexts?
// 4. What's the difference between concurrent and parallel execution?
// 5. Why can't you use std::thread::sleep in async functions?

// INTEGRATION WITH PREVIOUS CONCEPTS:
// - Ownership: async functions can move or borrow data
// - Error handling: Result<T, E> works the same in async context
// - Traits: async functions use Future trait under the hood

// PREPARATION FOR INTERMEDIATE ASSESSMENT:
// This prepares you for "Section III: Error Handling & Async Patterns"
// - Problem 3.2: Intermediate Async Patterns (10 points)
// - Async task coordination with channels and timeouts