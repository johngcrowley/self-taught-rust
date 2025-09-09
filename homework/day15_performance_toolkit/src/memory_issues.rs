// Memory Issues Challenge - Find the leaks with valgrind and profilers!
use std::collections::HashMap;
use std::rc::Rc;

struct LeakyProcessor {
    cache: HashMap<String, Rc<Vec<u8>>>,
    temp_buffers: Vec<Vec<u8>>,
    leaked_strings: Vec<String>, // Another memory leak source
}

impl LeakyProcessor {
    fn new() -> Self {
        println!("Creating new LeakyProcessor");
        LeakyProcessor {
            cache: HashMap::new(),
            temp_buffers: Vec::new(),
            leaked_strings: Vec::new(),
        }
    }
    
    fn process_data(&mut self, key: &str, data: &[u8]) -> Rc<Vec<u8>> {
        println!("Processing data for key: {}", key);
        
        // Memory issue 1: Temp buffers keep growing without cleanup
        let mut temp = Vec::with_capacity(data.len() * 2);
        temp.extend_from_slice(data);
        temp.extend_from_slice(data); // Double the data
        
        // BUG: We keep all temp buffers forever!
        self.temp_buffers.push(temp.clone());
        
        // Memory issue 2: String leaks
        for i in 0..100 {
            let leaked_string = format!("temp_string_{}_{}", key, i);
            self.leaked_strings.push(leaked_string); // Never cleared!
        }
        
        let result = Rc::new(temp);
        
        // Memory issue 3: Cache grows indefinitely 
        self.cache.insert(key.to_string(), result.clone());
        
        // Simulate some more allocations
        self.allocate_temporary_data();
        
        result
    }
    
    fn allocate_temporary_data(&mut self) {
        // Creates temporary data that should be freed but accumulates
        let _temp_map: HashMap<i32, String> = (0..1000)
            .map(|i| (i, format!("value_{}", i)))
            .collect();
        
        // This map gets dropped, but we're doing this repeatedly
        // which can show allocation patterns in profilers
    }
    
    fn process_many(&mut self, count: usize) {
        println!("Processing {} items", count);
        
        for i in 0..count {
            let key = format!("key_{}", i);
            let data: Vec<u8> = (0..1000).map(|x| (x % 256) as u8).collect();
            self.process_data(&key, &data);
            
            if i % 1000 == 0 {
                println!("Processed {} items, cache size: {}, temp_buffers: {}", 
                        i, self.cache.len(), self.temp_buffers.len());
            }
        }
    }
    
    // Method to cleanup - but it's never called in main()
    #[allow(dead_code)]
    fn cleanup(&mut self) {
        println!("Cleaning up processor");
        self.temp_buffers.clear();
        self.leaked_strings.clear();
        
        // Keep only the 100 most recent cache entries
        if self.cache.len() > 100 {
            let keys_to_remove: Vec<String> = self.cache.keys()
                .take(self.cache.len() - 100)
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }
    }
    
    fn get_memory_stats(&self) -> (usize, usize, usize) {
        let cache_size = self.cache.len();
        let temp_buffers_count = self.temp_buffers.len();
        let leaked_strings_count = self.leaked_strings.len();
        
        (cache_size, temp_buffers_count, leaked_strings_count)
    }
}

impl Drop for LeakyProcessor {
    fn drop(&mut self) {
        let (cache, temp, strings) = self.get_memory_stats();
        println!("Dropping LeakyProcessor with {} cache entries, {} temp buffers, {} leaked strings", 
                cache, temp, strings);
    }
}

// Function that creates and discards processors - more memory churn
fn create_and_discard_processors(count: usize) {
    println!("Creating and discarding {} processors", count);
    
    for i in 0..count {
        let mut processor = LeakyProcessor::new();
        processor.process_many(100); // Each processor leaks memory
        
        // processor gets dropped here, but its memory issues accumulate
        if i % 10 == 0 {
            println!("Created and dropped {} processors", i);
        }
    }
}

fn main() {
    println!("Starting memory leak demonstration");
    println!("This program will consume increasing amounts of memory");
    println!("\nTo profile memory usage:");
    println!("1. cargo build --release");
    println!("2. valgrind --tool=memcheck --leak-check=full target/release/memory-issues");
    println!("3. Or use: heaptrack target/release/memory-issues");
    println!("4. Monitor with: /usr/bin/time -v target/release/memory-issues\n");
    
    let args: Vec<String> = std::env::args().collect();
    let count = if args.len() > 1 {
        args[1].parse().unwrap_or(1000)
    } else {
        1000
    };
    
    // Test 1: Single processor with growing memory usage
    println!("=== Test 1: Single processor ===");
    let mut processor = LeakyProcessor::new();
    processor.process_many(count);
    let (cache, temp, strings) = processor.get_memory_stats();
    println!("Final stats - Cache: {}, Temp buffers: {}, Leaked strings: {}", 
            cache, temp, strings);
    
    // Test 2: Multiple processors - more memory churn
    println!("\n=== Test 2: Multiple processors ===");
    create_and_discard_processors(50);
    
    println!("\nMemory leak demonstration complete");
    println!("Check your system memory usage - it should be high!");
    
    // The processor gets dropped here, but the damage is done
}