use tokio::time::{Duration, sleep};
use rand::Rng;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    io_bottlenecked_work().await;
    cpu_intensive_work().await;
}

// htop shows 4-6% cpu usage and many cores 
// ---
// I/O bound ... CPU isn't being used heavily. Sleep is just waiting. No Computational work is being
// done besides string formatting.
// the string formatting _is_ CPU work, but so fast you don't see it. The stdout stream is
// blocking. Println writes to one stdout so its not garbled output. global lock around stdout used
// by println macro.
async fn io_bottlenecked_work() {
    
    let mut rand_num_gen = rand::rng();
    let mut join_set = JoinSet::new();
    for task_number in 1..1_000_000 {
        let duration = Duration::from_millis(rand_num_gen.random_range(1000..25000));
        join_set.spawn(async move {
            sleep(duration).await;
            println!("task {} complete. duration: {:?}", task_number, duration);
        }); 
    }
    join_set.join_all().await;
}

// htop shows 100% cpu usage and many cores 
// ---
// CPU bound ... CPU work is being done.
async fn cpu_intensive_work() {
    
    let mut rand_num_gen = rand::rng();
    let mut join_set = JoinSet::new();
    let iterations = rand_num_gen.random_range(1_000_000..10_000_000);
    for task_number in 1..1_000_000 {
        join_set.spawn(async move {
            // CPU-intensive work instead of sleeping
            let mut sum = 0u64;
            for i in 0..iterations {
                sum = sum.wrapping_add(i * i);
            }
            println!("task {} complete. sum: {}", task_number, sum);
        }); 
    }
    join_set.join_all().await;
}
