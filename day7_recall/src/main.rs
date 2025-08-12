use std::sync::{Arc,Mutex};
use tokio::task::JoinSet;
use std::sync::mpsc::{Sender};
use rand::prelude::*;
use tokio_stream::StreamExt;

// TODO: 
// 1. how could i borrow armies and not copy it into `combat()`
// 2. why isn't my "general crushed army ..." output interleaved?

fn combat(mut army: i32, tx: Sender<(i32, i32)>) {
    let initial_forces = army;
    let mut rng = rand::rng();
    for _wave in 1..1_000_00 {
        let casualties = rng.random_range(1..15);
        army -= casualties;
    }
    tx.send((initial_forces, army));
}

async fn execute_order(general: &'static str, battles_mx: Arc<Mutex<i32>>, armies: Vec<i32>) {
    
    println!("{general} executing order");
    
    // each general's communication line to lieutenants
    let (tx, rx) =  std::sync::mpsc::channel();
    
    // Simulate a stream of armies we're going to battle with 
    let mut armies_stream = tokio_stream::iter(armies);
    while let Some(army) = armies_stream.next().await {
        let mut battles_mx_lock = battles_mx.lock().unwrap();
        *battles_mx_lock += 1;
        let tx_clone = tx.clone();
        tokio::task::spawn_blocking(move || {
            combat(army, tx_clone);
        });
    }

    drop(tx);
    
    for (i, battle) in rx.iter().enumerate() {
        println!("{general} crushed army {i} from {} to {}", battle.0, battle.1);
    }
}


#[tokio::main]
async fn main() {

    let mut rng = rand::rng();
    
    // set up battle board
    let total_battles_fought = Arc::new(Mutex::new(0));
    let generals = vec!["rommel", "caesar", "napoleon", "patton", "hannibal", "alexander"];
    let mut war_room = JoinSet::new();
    // deploy orders
    for general in generals {
       let capacity = rng.random_range(100..1000);
       let mut armies = Vec::with_capacity(capacity);
       for _ in 1..capacity {
          let army_size = rng.random_range(50_000_00..500_000_00);
          armies.push(army_size);
       }
       let battles_mx = Arc::clone(&total_battles_fought);
       war_room.spawn( async move {
            execute_order(&general, battles_mx, armies).await;
       });
    }

    // rather than a channel, which we pull off of willy-nilly, we care about the Result<> as a
    // collection, hence, war room.
    war_room.join_all().await;

    let total_battles_fought_lock = total_battles_fought.lock().unwrap();
    println!("total battles fought by all generals: {total_battles_fought_lock}");
    
}
