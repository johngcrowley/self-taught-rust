use std::time::{Instant};
use std::thread::JoinHandle;
use std::sync::mpsc::{Sender};
use tokio::task::JoinSet;
use rand::prelude::*;

// show use of OS threads you wait for
// show use of async/await tasks you wait for
fn calculator(mut n: i64, rng: &mut ThreadRng ) -> i64 {
    for _ in 1..1_000_000 {
        n = n.wrapping_add(rng.random_range(0..100)).wrapping_mul(rng.random_range(1..100)) / rng.random_range(1..100);
    }
    n
}

fn mpsc_example(numbers_vec: &[i64], tx: Sender<i64>) {
    for &num in numbers_vec {
        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            let mut rng = rand::rng(); 
            tx_clone.send(calculator(num, &mut rng)).unwrap();
        });
    }
}

fn os_example(numbers_vec: &[i64]) {
    
    let start = Instant::now();
    let join_handles: Vec<JoinHandle<i64>> = numbers_vec
        .iter()
        .map(|&n| {
            let jh: JoinHandle<i64> = std::thread::spawn(move || {
                let mut rng = rand::rng(); 
                calculator(n, &mut rng)
            });
            jh
        })
        .collect();
    
    let sum: i64 = join_handles.into_iter().map(|jh| jh.join().unwrap()).sum();
    println!("os sum: {} | took: {:?}", sum, start.elapsed());
    
}

async fn async_example(numbers_vec: &[i64]) {
    
    let start = Instant::now();
    let mut js = JoinSet::new();
    for &n in numbers_vec {
        js.spawn_blocking(move || {
            let mut rng = rand::rng(); 
            calculator(n, &mut rng)
        });
    }
    let async_sum: i64 = js.join_all().await.iter().sum();
    println!("async sum: {} | took: {:?}", async_sum, start.elapsed());
}

#[tokio::main]
async fn main() {
    
    let numbers = vec![1; 25];

    // show use of MPSC channels
    let start = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();
    mpsc_example(&numbers, tx);
    let mpsc_sum: i64 = rx.iter().sum(); 
    println!("mpsc sum: {} | took: {:?}", mpsc_sum, start.elapsed());

    // show use of OS threads
    os_example(&numbers);
    
    // show JoinSet in `async` with tokio
    async_example(&numbers).await;
}

