// Mystery Segfault Challenge - Find the bug with GDB!
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct DataProcessor {
    cache: HashMap<String, Vec<u8>>,
    buffer: Vec<u8>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            buffer: Vec::with_capacity(1024),
        }
    }
    
    fn process_data(&mut self, key: &str, data: &[u8]) -> &[u8] {
        // This function has a subtle bug that causes crashes under certain conditions
        self.buffer.clear();
        self.buffer.extend_from_slice(data);
        
        // Process the data (double each byte)
        for byte in &mut self.buffer {
            *byte = byte.wrapping_mul(2);
        }
        
        // Cache the result - BUG IS HERE
        self.cache.insert(key.to_string(), self.buffer.clone());
        
        // Return reference to processed data - BUG CONTINUES HERE
        &self.buffer
    }
}

fn main() {
    let mut processor = DataProcessor::new();
    
    let data1 = vec![1, 2, 3, 4, 5];
    let result1 = processor.process_data("test1", &data1);
    println!("Result 1: {:?}", result1);
    
    let data2 = vec![10, 20, 30];
    let result2 = processor.process_data("test2", &data2);
    println!("Result 2: {:?}", result2);
    
    // This line will cause problems - result1 is now invalid!
    println!("Previous result: {:?}", result1); // CRASH HAPPENS HERE
}