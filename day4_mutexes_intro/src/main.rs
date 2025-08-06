use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
// Let's check if numbers are prime, splitting up the numbers across 4 threads, writing to an
// atomically-locked Vector.
fn is_prime(num: i32) -> bool {
    // prime is only divisible by 1, so 2 is next lowest, i.e., discard checking anything with a
    // remainder
    for i in 2..=num/2 {
        if num % i == 0 {
            return false
        }
    }
    if num <= 1 {
        return false
    } else {
        return true
    }
}
fn main() {

    let total = 1_000;
    let threads = 4;
    // why aren't these mutable? 
    let iterations: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let primes: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![]));
    let mut handles: Vec<JoinHandle<()>> = vec![];
    let mut min = 1;
    for num in 1..threads+1 {
        let max = min + (total / threads);
        let primes_clone = Arc::clone(&primes);
        let iterations_clone = Arc::clone(&iterations);
        handles.push(std::thread::spawn( move ||
            for num in min..max {
                let mut iterations_lock = iterations_clone.lock().unwrap();
                *iterations_lock += 1;
                if is_prime(num) {
                    let mut primes_lock = primes_clone.lock().unwrap();
                    primes_lock.push(num);
                }   
            }
        ));
        min = (total / threads * num) + 1
    }
    
    for jh in handles {
        let _ = jh.join();
    }
    let primes_lock = primes.lock().unwrap();
    println!("{:?}", primes_lock);
    let total_iterations = iterations.lock().unwrap();
    println!("{}", total_iterations);
}
