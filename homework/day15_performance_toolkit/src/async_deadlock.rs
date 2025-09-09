// Async Deadlock Challenge - Debug the hanging program!
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[derive(Clone)]
struct SharedCounter {
    value1: Arc<Mutex<i32>>,
    value2: Arc<Mutex<i32>>,
}

impl SharedCounter {
    fn new() -> Self {
        SharedCounter {
            value1: Arc::new(Mutex::new(0)),
            value2: Arc::new(Mutex::new(0)),
        }
    }
    
    async fn increment_both(&self) {
        println!("Task A: Acquiring value1 lock...");
        let _guard1 = self.value1.lock().unwrap();
        println!("Task A: Got value1 lock, sleeping...");
        sleep(Duration::from_millis(100)).await; // Give task B time to get value2
        
        println!("Task A: Trying to acquire value2 lock...");
        let _guard2 = self.value2.lock().unwrap(); // DEADLOCK: B has this
        println!("Task A: Incremented both values");
    }
    
    async fn decrement_both(&self) {
        println!("Task B: Acquiring value2 lock...");
        let _guard2 = self.value2.lock().unwrap();
        println!("Task B: Got value2 lock, sleeping...");
        sleep(Duration::from_millis(100)).await; // Give task A time to get value1
        
        println!("Task B: Trying to acquire value1 lock...");
        let _guard1 = self.value1.lock().unwrap(); // DEADLOCK: A has this
        println!("Task B: Decremented both values");
    }
}

#[tokio::main]
async fn main() {
    println!("Starting deadlock demonstration...");
    let counter = SharedCounter::new();
    
    let task_a = {
        let counter = counter.clone();
        tokio::spawn(async move {
            counter.increment_both().await;
        })
    };
    
    let task_b = {
        let counter = counter.clone();
        tokio::spawn(async move {
            counter.decrement_both().await;
        })
    };
    
    println!("Both tasks started, waiting for completion...");
    
    // This will hang - your job is to find where and why using GDB
    let _ = tokio::join!(task_a, task_b);
    println!("All done!"); // This never prints
}