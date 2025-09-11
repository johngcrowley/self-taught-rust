# Intermediate Rust Readiness Assessment

**Prerequisite Test for Advanced Rust Learning Path**

*Time Limit: 2 hours | Pass Threshold: 80% | Gateway to: Performance & Systems Programming*

---

## Mission: Prove You're Ready for Advanced Rust

**Critical Gap Assessment**: You know basic Rust, but are you ready for performance optimization, unsafe code, and systems programming? This assessment verifies you have the intermediate Rust skills needed for advanced courses.

**What This Tests:**
- Advanced ownership, borrowing, and lifetime patterns
- Trait system mastery beyond the basics
- Error handling strategies and patterns
- Intermediate async programming
- Generic programming and type system usage
- Testing and API design
- Basic concurrent programming patterns

**If You Pass**: You're ready for the bridge courses (01-04) and the advanced learning path
**If You Don't Pass**: Review intermediate Rust concepts before attempting advanced material

---

## Section I: Advanced Ownership & Lifetimes (25 points)

### Problem 1.1: Complex Borrowing Scenarios (15 points)

Analyze and fix these borrowing scenarios that go beyond basic ownership:

```rust
// Scenario A: Multiple mutable borrows pattern
struct DataProcessor {
    buffer: Vec<u8>,
    position: usize,
}

impl DataProcessor {
    // Fix this method to compile while maintaining functionality
    fn process_chunk(&mut self) -> Result<&[u8], ProcessError> {
        let start = self.position;
        let chunk_size = 64;
        
        if start + chunk_size > self.buffer.len() {
            self.buffer.resize(start + chunk_size, 0);
        }
        
        self.position += chunk_size;
        
        // This doesn't compile - fix it
        Ok(&self.buffer[start..self.position])
    }
    
    // This method should be callable after process_chunk
    fn reset(&mut self) {
        self.position = 0;
    }
}

// Scenario B: Lifetime elision understanding
// Add explicit lifetimes where needed, explain where elision works

fn find_longest_word(text: &str) -> &str {
    text.split_whitespace()
        .max_by_key(|word| word.len())
        .unwrap_or("")
}

fn combine_strings(a: &str, b: &str, use_first: bool) -> &str {
    if use_first { a } else { b }
}

// Scenario C: Self-referential data (without Pin)
struct TextAnalyzer {
    text: String,
    words: Vec<&str>, // This won't compile - fix it
}

impl TextAnalyzer {
    fn new(text: String) -> Self {
        // How do you make this work safely?
        todo!()
    }
}
```

**Your Tasks:**
1. Fix DataProcessor::process_chunk to compile and work correctly
2. Add explicit lifetimes to functions where needed, explain where elision applies
3. Design a working TextAnalyzer that safely references its own data
4. Explain the borrowing rules being violated and your solutions

### Problem 1.2: Lifetime Bounds and Advanced Patterns (10 points)

```rust
// Complex lifetime scenario
struct Cache<'a> {
    data: std::collections::HashMap<String, &'a str>,
}

impl<'a> Cache<'a> {
    fn get_or_compute<F>(&mut self, key: &str, compute: F) -> &str 
    where
        F: FnOnce() -> &'a str,
    {
        // Implement get-or-compute with proper lifetime handling
        todo!()
    }
}

// Higher-Ranked Trait Bounds challenge
fn process_closure<F>(f: F) -> String 
where
    // Fix this bound to work with closures that take any lifetime
    F: Fn(&str) -> String,
{
    let data = "test data";
    f(data)
}

// Usage that should work:
fn test_hrtb() {
    let result = process_closure(|s: &str| s.to_uppercase());
    println!("{}", result);
}
```

**Your Tasks:**
1. Implement Cache::get_or_compute with correct lifetime handling
2. Fix the HRTB to work with the test case
3. Explain when you need higher-ranked trait bounds

---

## Section II: Advanced Trait System (25 points)

### Problem 2.1: Associated Types and Complex Bounds (15 points)

Design a flexible serialization system using advanced trait features:

```rust
// Design a trait system for different serialization formats
trait Serialize {
    type Output;
    type Error: std::error::Error;
    
    fn serialize<T>(&self, value: &T) -> Result<Self::Output, Self::Error>
    where
        T: ?Sized + SerializeValue<Self>;
}

trait SerializeValue<S: Serialize> {
    fn serialize_with(&self, serializer: &S) -> Result<S::Output, S::Error>;
}

// Implement for JSON format
struct JsonSerializer;

impl Serialize for JsonSerializer {
    type Output = String;
    type Error = JsonError;
    
    fn serialize<T>(&self, value: &T) -> Result<String, JsonError>
    where
        T: ?Sized + SerializeValue<Self>,
    {
        value.serialize_with(self)
    }
}

// Implement for Binary format  
struct BinarySerializer;

impl Serialize for BinarySerializer {
    type Output = Vec<u8>;
    type Error = BinaryError;
    
    fn serialize<T>(&self, value: &T) -> Result<Vec<u8>, BinaryError>
    where
        T: ?Sized + SerializeValue<Self>,
    {
        value.serialize_with(self)
    }
}

// Your task: Implement SerializeValue for common types
impl SerializeValue<JsonSerializer> for i32 {
    fn serialize_with(&self, _serializer: &JsonSerializer) -> Result<String, JsonError> {
        Ok(self.to_string())
    }
}

impl SerializeValue<BinarySerializer> for i32 {
    fn serialize_with(&self, _serializer: &BinarySerializer) -> Result<Vec<u8>, BinaryError> {
        Ok(self.to_le_bytes().to_vec())
    }
}

// Challenge: Make this work for Vec<T> where T: SerializeValue
impl<S, T> SerializeValue<S> for Vec<T> 
where 
    S: Serialize,
    T: SerializeValue<S>,
{
    fn serialize_with(&self, serializer: &S) -> Result<S::Output, S::Error> {
        // How do you serialize a Vec when you don't know the Output type?
        todo!()
    }
}

#[derive(Debug)]
struct JsonError(String);
#[derive(Debug)] 
struct BinaryError(String);

// Implement std::error::Error for both
```

**Your Tasks:**
1. Complete the error type implementations
2. Solve the Vec<T> serialization challenge
3. Design a way to compose/combine serialized values
4. Show how to use this system polymorphically

### Problem 2.2: Trait Objects and Dynamic Dispatch (10 points)

```rust
// Design a plugin system using trait objects
trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    
    // Challenge: How do you handle plugin-specific configuration?
    fn configure(&mut self, config: &dyn std::any::Any) -> Result<(), ConfigError>;
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
    
    fn register_plugin<P: Plugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }
    
    fn process_data(&self, data: &[u8]) -> Result<Vec<u8>, ProcessingError> {
        // Chain all plugins together
        todo!()
    }
    
    // Challenge: How do you configure specific plugins by type?
    fn configure_plugin<P: Plugin + 'static>(&mut self, config: P::Config) -> Result<(), ConfigError> {
        // This doesn't work with trait objects - fix it
        todo!()
    }
}

// Example plugins
struct CompressionPlugin {
    level: u8,
}

struct EncryptionPlugin {
    key: [u8; 32],
}

impl Plugin for CompressionPlugin {
    fn name(&self) -> &str { "compression" }
    
    fn process(&self, input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Simulate compression
        Ok(input.to_vec())
    }
    
    fn configure(&mut self, config: &dyn std::any::Any) -> Result<(), ConfigError> {
        // How do you safely downcast and configure?
        todo!()
    }
}
```

**Your Tasks:**
1. Implement the plugin system with proper error handling
2. Solve the configuration challenge for trait objects
3. Implement both example plugins
4. Design a type-safe way to configure plugins

---

## Section III: Error Handling & Async Patterns (25 points)

### Problem 3.1: Advanced Error Handling (15 points)

Design a comprehensive error handling strategy for a file processing system:

```rust
use std::io;
use thiserror::Error;

// Design an error hierarchy that preserves context
#[derive(Error, Debug)]
pub enum FileProcessError {
    // Your error types here - think about:
    // 1. I/O errors with file path context
    // 2. Parse errors with line/column information
    // 3. Validation errors with field details
    // 4. Network errors with retry information
}

struct FileProcessor {
    max_retries: usize,
}

impl FileProcessor {
    async fn process_file(&self, path: &std::path::Path) -> Result<ProcessedData, FileProcessError> {
        // Multi-step process that can fail at each step:
        // 1. Read file (I/O error)
        // 2. Parse content (Parse error) 
        // 3. Validate data (Validation error)
        // 4. Upload to server (Network error with retries)
        
        let content = self.read_file_with_context(path).await?;
        let parsed = self.parse_content(&content, path).await?;
        let validated = self.validate_data(parsed, path)?;
        let uploaded = self.upload_with_retries(validated).await?;
        
        Ok(uploaded)
    }
    
    async fn read_file_with_context(&self, path: &std::path::Path) -> Result<String, FileProcessError> {
        // Add rich context to I/O errors
        todo!()
    }
    
    async fn parse_content(&self, content: &str, path: &std::path::Path) -> Result<ParsedData, FileProcessError> {
        // Handle parse errors with line/column info
        todo!()
    }
    
    fn validate_data(&self, data: ParsedData, path: &std::path::Path) -> Result<ValidatedData, FileProcessError> {
        // Handle validation errors with field context
        todo!()
    }
    
    async fn upload_with_retries(&self, data: ValidatedData) -> Result<ProcessedData, FileProcessError> {
        // Handle network errors with retry logic
        todo!()
    }
}

// Supporting types
struct ParsedData { /* fields */ }
struct ValidatedData { /* fields */ }
struct ProcessedData { /* fields */ }
```

**Your Tasks:**
1. Design a comprehensive error hierarchy using thiserror
2. Implement context-preserving error handling for each step
3. Add retry logic with exponential backoff for network errors
4. Show how to use ? operator while preserving context
5. Design error recovery strategies for different error types

### Problem 3.2: Intermediate Async Patterns (10 points)

```rust
use tokio::{sync::{mpsc, oneshot}, time::{timeout, Duration}};

// Design an async task coordinator
struct TaskCoordinator {
    task_tx: mpsc::UnboundedSender<TaskRequest>,
    result_rx: mpsc::UnboundedReceiver<TaskResult>,
}

struct TaskRequest {
    id: u64,
    work: Box<dyn FnOnce() -> Result<String, TaskError> + Send>,
    timeout: Duration,
    response_tx: oneshot::Sender<Result<String, TaskError>>,
}

struct TaskResult {
    id: u64,
    result: Result<String, TaskError>,
}

#[derive(Debug)]
enum TaskError {
    Timeout,
    Execution(String),
    Cancelled,
}

impl TaskCoordinator {
    fn new(max_concurrent: usize) -> Self {
        // Your implementation:
        // 1. Set up channels
        // 2. Spawn worker tasks (up to max_concurrent)
        // 3. Handle task distribution and timeouts
        todo!()
    }
    
    async fn submit_task<F, T>(&self, work: F, timeout: Duration) -> Result<T, TaskError>
    where
        F: FnOnce() -> Result<T, TaskError> + Send + 'static,
        T: Send + 'static,
    {
        // Submit work to be executed by worker tasks
        // Handle timeout and cancellation
        todo!()
    }
    
    async fn submit_batch<F, T>(&self, tasks: Vec<(F, Duration)>) -> Vec<Result<T, TaskError>>
    where
        F: FnOnce() -> Result<T, TaskError> + Send + 'static,
        T: Send + 'static,
    {
        // Execute multiple tasks concurrently
        // Return results in original order
        todo!()
    }
}
```

**Your Tasks:**
1. Implement the task coordinator with proper async patterns
2. Handle timeouts, cancellation, and backpressure
3. Implement batch processing with ordered results
4. Add graceful shutdown capabilities

---

## Section IV: Generic Programming & Collections (25 points)

### Problem 4.1: Advanced Generics and Type System (15 points)

```rust
use std::marker::PhantomData;

// Design a type-safe builder pattern with compile-time state verification
struct ConfigBuilder<State> {
    name: Option<String>,
    port: Option<u16>,
    timeout: Option<Duration>,
    _state: PhantomData<State>,
}

// States
struct Incomplete;
struct HasName;
struct HasNameAndPort;
struct Complete;

impl ConfigBuilder<Incomplete> {
    fn new() -> Self {
        Self {
            name: None,
            port: None,
            timeout: None,
            _state: PhantomData,
        }
    }
}

impl ConfigBuilder<Incomplete> {
    fn name(self, name: impl Into<String>) -> ConfigBuilder<HasName> {
        // Transition to HasName state
        todo!()
    }
}

impl ConfigBuilder<HasName> {
    fn port(self, port: u16) -> ConfigBuilder<HasNameAndPort> {
        // Transition to HasNameAndPort state
        todo!()
    }
}

impl ConfigBuilder<HasNameAndPort> {
    fn timeout(self, timeout: Duration) -> ConfigBuilder<Complete> {
        // Optional field - transition to Complete state
        todo!()
    }
    
    fn build(self) -> Config {
        // Can build without timeout
        todo!()
    }
}

impl ConfigBuilder<Complete> {
    fn build(self) -> Config {
        // Can build with all fields
        todo!()
    }
}

struct Config {
    name: String,
    port: u16,
    timeout: Option<Duration>,
}

// Challenge: Design a generic container with type-level constraints
trait TypeConstraint {
    type Value;
    fn validate(value: &Self::Value) -> bool;
}

struct PositiveNumber;
impl TypeConstraint for PositiveNumber {
    type Value = i32;
    fn validate(value: &i32) -> bool {
        *value > 0
    }
}

struct NonEmptyString;
impl TypeConstraint for NonEmptyString {
    type Value = String;
    fn validate(value: &String) -> bool {
        !value.is_empty()
    }
}

struct ValidatedVec<C: TypeConstraint> {
    inner: Vec<C::Value>,
    _constraint: PhantomData<C>,
}

impl<C: TypeConstraint> ValidatedVec<C> {
    fn new() -> Self {
        Self {
            inner: Vec::new(),
            _constraint: PhantomData,
        }
    }
    
    fn push(&mut self, value: C::Value) -> Result<(), ValidationError> {
        // Only add if validation passes
        todo!()
    }
    
    fn get(&self, index: usize) -> Option<&C::Value> {
        self.inner.get(index)
    }
}

#[derive(Debug)]
struct ValidationError(String);
```

**Your Tasks:**
1. Complete the type-state builder pattern implementation
2. Implement the validated container with compile-time constraints
3. Show usage examples that demonstrate type safety
4. Explain how phantom types enable zero-cost abstractions

### Problem 4.2: Custom Collections and Iterators (10 points)

```rust
// Implement a custom collection with efficient iteration
struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    head: usize,
    tail: usize,
    full: bool,
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![None; capacity],
            head: 0,
            tail: 0,
            full: false,
        }
    }
    
    fn push(&mut self, item: T) -> Option<T> {
        // Return evicted item if buffer is full
        todo!()
    }
    
    fn pop(&mut self) -> Option<T> {
        // Remove and return oldest item
        todo!()
    }
    
    fn is_full(&self) -> bool {
        self.full
    }
    
    fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }
}

// Implement Iterator for RingBuffer
impl<T> IntoIterator for RingBuffer<T> {
    type Item = T;
    type IntoIter = RingBufferIterator<T>;
    
    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

struct RingBufferIterator<T> {
    buffer: RingBuffer<T>,
}

impl<T> Iterator for RingBufferIterator<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

// Challenge: Implement a custom iterator adapter
trait IteratorExt: Iterator {
    fn sliding_window<const N: usize>(self) -> SlidingWindow<Self, N>
    where
        Self: Sized;
}

impl<I: Iterator> IteratorExt for I {
    fn sliding_window<const N: usize>(self) -> SlidingWindow<Self, N> {
        SlidingWindow::new(self)
    }
}

struct SlidingWindow<I: Iterator, const N: usize> {
    iter: I,
    window: [Option<I::Item>; N],
    pos: usize,
    started: bool,
}

impl<I: Iterator, const N: usize> SlidingWindow<I, N>
where
    I::Item: Clone,
{
    fn new(iter: I) -> Self {
        todo!()
    }
}

impl<I: Iterator, const N: usize> Iterator for SlidingWindow<I, N>
where
    I::Item: Clone,
{
    type Item = [I::Item; N];
    
    fn next(&mut self) -> Option<Self::Item> {
        // Return sliding windows of size N
        todo!()
    }
}
```

**Your Tasks:**
1. Complete the RingBuffer implementation with proper iterator
2. Implement the sliding window iterator adapter
3. Add size_hint() implementations for efficiency
4. Show usage examples demonstrating the functionality

---

## Assessment Criteria

### **Pass Requirements (80% minimum - 80/100 points):**

**Advanced Ownership (20/25 points required):**
- Correctly handle complex borrowing scenarios
- Understand lifetime bounds and HRTB
- Design safe alternatives to self-referential structs

**Trait System Mastery (20/25 points required):**
- Use associated types and complex bounds correctly
- Design flexible trait-based systems
- Handle trait objects and dynamic dispatch properly

**Error & Async Patterns (20/25 points required):**
- Design comprehensive error hierarchies
- Implement proper async coordination patterns
- Handle timeouts, cancellation, and backpressure

**Generic Programming (20/25 points required):**
- Use type-state patterns and phantom types
- Implement custom collections and iterators
- Design zero-cost abstractions

### **If You Pass This Assessment:**

🎉 **You're ready for the bridge courses!**

- **01_performance_measurement_bridge.md** - Learn to measure and optimize
- **02_unsafe_rust_bridge.md** - Master unsafe superpowers safely  
- **03_systems_programming_bridge.md** - Build production-ready systems
- **04_concurrency_bridge.md** - Advanced concurrent programming

### **If You Don't Pass:**

📚 **Focus on these areas before retrying:**

- **Rust Book Chapter 10 (Generics, Traits, Lifetimes)** - Deep dive
- **Rust Book Chapter 15 (Smart Pointers)** - Understanding ownership patterns
- **Async Book** - async/await fundamentals
- **Rust by Example** - Trait objects, error handling patterns
- **Practice projects** - Build intermediate-complexity applications

### **Key Concepts You Must Master:**

✅ Complex ownership and borrowing patterns  
✅ Trait system beyond basics (associated types, bounds, objects)  
✅ Proper error handling strategies and context preservation  
✅ Intermediate async patterns (channels, coordination, timeouts)  
✅ Generic programming and type-level computation  
✅ Custom collections and iterator implementation  
✅ API design and zero-cost abstractions

---

## Next Steps by Skill Level

**Expert (90-100 points)**: Jump directly into any bridge course. You have strong intermediate Rust skills.

**Proficient (80-89 points)**: Ready for bridge courses, but consider extra practice with your weaker areas.

**Developing (70-79 points)**: Review specific topics you struggled with, then retry. Focus on the Rust Book advanced chapters.

**Novice (<70 points)**: Build more intermediate projects. Consider working through "Programming Rust" or "Rust in Action" before retrying.

---

*"Intermediate Rust is where you stop fighting the borrow checker and start leveraging Rust's type system to build robust, efficient software."*

**This assessment ensures you have the foundation needed for advanced Rust systems programming. Master these concepts, and you'll be ready to tackle performance optimization, unsafe code, and low-level systems programming with confidence.**