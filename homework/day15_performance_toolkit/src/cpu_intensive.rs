// CPU Intensive Challenge - Find the hotspots with flamegraphs!
use std::collections::HashMap;

fn expensive_computation(n: usize) -> u64 {
    println!("Starting expensive computation for {} iterations", n);
    let mut result = 0;
    
    for i in 0..n {
        // Each iteration does multiple expensive operations
        result += fibonacci(i % 30);           // Recursive fibonacci
        result += prime_check(i % 1000 + 2);   // Inefficient prime check  
        result += hash_operations(i % 50);     // HashMap operations
        
        if i % 100 == 0 {
            println!("Processed {} iterations", i);
        }
    }
    
    println!("Computation complete");
    result
}

// Inefficient recursive Fibonacci - major CPU hotspot
fn fibonacci(n: usize) -> u64 {
    if n <= 1 {
        n as u64
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

// Inefficient prime check - another hotspot
fn prime_check(n: usize) -> u64 {
    if n < 2 { 
        return 0; 
    }
    
    // Bug: checking all numbers up to n instead of sqrt(n)
    for i in 2..n {
        if n % i == 0 {
            return 0;
        }
    }
    1
}

// Inefficient HashMap operations - creates new map every call
fn hash_operations(n: usize) -> u64 {
    let mut map = HashMap::new(); // BUG: New HashMap every call
    
    // Fill with data
    for i in 0..n {
        map.insert(format!("key_{}", i), i * 2);
    }
    
    // Do some operations
    let mut sum = 0;
    for value in map.values() {
        sum += value;
    }
    
    sum as u64
}

// More efficient versions for comparison
#[allow(dead_code)]
fn fibonacci_iterative(n: usize) -> u64 {
    if n <= 1 {
        return n as u64;
    }
    
    let mut a = 0;
    let mut b = 1;
    
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    
    b
}

#[allow(dead_code)]
fn prime_check_optimized(n: usize) -> u64 {
    if n < 2 {
        return 0;
    }
    if n == 2 {
        return 1;
    }
    if n % 2 == 0 {
        return 0;
    }
    
    let sqrt_n = (n as f64).sqrt() as usize;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return 0;
        }
    }
    1
}

// Global HashMap for reuse
lazy_static::lazy_static! {
    static ref GLOBAL_MAP: std::sync::Mutex<HashMap<String, usize>> = 
        std::sync::Mutex::new(HashMap::new());
}

#[allow(dead_code)]
fn hash_operations_optimized(n: usize) -> u64 {
    let mut map = GLOBAL_MAP.lock().unwrap();
    map.clear();
    
    for i in 0..n {
        map.insert(format!("key_{}", i), i * 2);
    }
    
    map.values().sum::<usize>() as u64
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations = if args.len() > 1 {
        args[1].parse().unwrap_or(1000)
    } else {
        1000
    };
    
    let start = std::time::Instant::now();
    let result = expensive_computation(iterations);
    let duration = start.elapsed();
    
    println!("Result: {}", result);
    println!("Time taken: {:?}", duration);
    println!("\nTo profile this:");
    println!("1. cargo build --release");
    println!("2. sudo flamegraph target/release/cpu-intensive {}", iterations);
    println!("3. Open flamegraph.svg to see CPU hotspots");
}