use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use std::sync::mpsc;
use std::time::Instant; 

// i'm going to demonstrate 3 methods to reverse strings, timing them all:
// 1. ordered channel (slow)
// 2. unordered channel (fast)
// 3. OS threads we wait for one-at-a-time (slow)
// 4. OS threads we kick off, then wait for collective (fast)

fn ordered_channel(sentences: &[&'static str]) {
    
    let start = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();
    for (i, &s) in sentences.iter().enumerate() {
        let tx_clone = tx.clone(); 
        std::thread::spawn(move || {
            let reversed = s.chars().rev().collect::<String>() + " [ordered-channel]";
            tx_clone.send((i, reversed)).unwrap();
        });
    }
    drop(tx);
    
    // `.iter()` evaluated by `.collect` so we're waiting for all messages and sorting them. 
    let mut sorted = rx.iter().collect::<Vec<(usize, String)>>();
    sorted.sort_by(|a,b| a.0.cmp(&b.0));
    
    println!("ordered_channel took:   {:?}", start.elapsed());
}

fn unordered_channel(sentences: &[&'static str]) {
    
    let (tx, rx) = std::sync::mpsc::channel();
    let start = Instant::now();
    // sentences is already a reference to a vec, so the implied `.into_iter` here doesn't consume
    for &s in sentences {
        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            let reversed =  s.chars().rev().collect::<String>() + " [unordered-channel]";
            tx_clone.send(reversed).unwrap();
       });
    }
    drop(tx);

    // `println1` blocks, so just drain channel to show how this is faster than `ordered_channel`
    for _ in rx.iter() {}
        
    println!("unordered_channel took: {:?}", start.elapsed());
    
}

fn spawned_threads_parallel(sentences: &[&'static str]) {
    
    // all threads are running in parallel then joined after.
    let start = Instant::now();
    let handles: Vec<JoinHandle<()>> = sentences
        // &&'static str
        .iter()
        // &s matches first &, so that s is of type &'static str inside closure
        // if we just did map(|s|), `spawn` receives an &&'static str, meaning, the outer &,
        // so a non-static lifetime that won't live as long as the thread!
        .map(|&s| {
           let jh = std::thread::spawn(move || {
              let reversed = s.chars().rev().collect::<String>() + " [parallel-threads]";
           });
           jh
        })
        .collect();

    let _ = handles.into_iter().map(|jh| jh.join().unwrap());
    println!("spawned_threads_parallel took:   {:?}", start.elapsed());
}

fn spawned_threads_sequential(sentences: &[&'static str]) {
    
    // each `.join()` blocks next loop iteration.
    let start = Instant::now();
    for &s in sentences {
        let jh = std::thread::spawn(move || {
            let reversed = s.chars().rev().collect::<String>() + " [sequential-threads]";
        });
        jh.join().unwrap();
    }
    println!("spawned_threads_sequential took:   {:?}", start.elapsed());
}


fn main() {
    
    let sentences_to_reverse: Vec<&str> = vec![
        "...litnu yps a eb ot desu i"
        ,"degnahc gnihtyreve litnu ,ynomrah ni"
        ,"rehtegot devil snoitan ruof eht ,oga gnol"
        ,"notsew leahcim si eman ym"
        ,"dekcatta noitan erif eht neht"
    ];

    spawned_threads_parallel(&sentences_to_reverse);
    spawned_threads_sequential(&sentences_to_reverse);
    unordered_channel(&sentences_to_reverse);
    ordered_channel(&sentences_to_reverse);
    
}



