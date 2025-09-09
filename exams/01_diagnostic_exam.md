# 01. Diagnostic Exam: Rust Fundamentals  
## Individual Concept Assessment
**Time Limit: 2 hours | Total Points: 100**

**Purpose:** This diagnostic exam tests individual Rust concepts in isolation to assess foundational knowledge. 

**Note:** For comprehensive systems integration assessment, take the **Mastery Exam** in `03_mastery_exam.md`.

---

## Section I: Basic Ownership & Borrowing (25 points)

### Problem 1.1 (10 points)
Explain why this code fails to compile and provide a working fix:

```rust
fn main() {
    let mut data = vec![1, 2, 3, 4, 5];
    let first = &data[0];
    data.push(6);
    println!("First element: {}", first);
}
```

### Problem 1.2 (15 points)
Complete this function to return references to the longest string:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// Your task: Implement a function that works with different lifetime parameters
fn longest_multiple</* Your lifetime parameters */>(
    strings: Vec</* Your parameter types */>
) -> /* Your return type */ {
    // Your implementation
}
```

---

## Section II: Basic Error Handling (25 points)

### Problem 2.1 (15 points) 
Implement a custom error type using standard library traits:

```rust
#[derive(Debug)]
enum CalculatorError {
    DivisionByZero,
    InvalidInput(String),
    Overflow,
}

// Your task: Implement Display and Error traits
```

### Problem 2.2 (10 points)
Convert this panic-based code to use proper error handling:

```rust
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division by zero!");
    }
    a / b
}

// Convert to: fn divide(a: i32, b: i32) -> Result<i32, CalculatorError>
```

---

## Section III: Basic Async Programming (25 points)

### Problem 3.1 (15 points)
Complete this async function to process items concurrently:

```rust
use tokio::task::JoinSet;

async fn process_items(items: Vec<i32>) -> Vec<i32> {
    let mut join_set = JoinSet::new();
    
    // Your implementation: spawn tasks for each item
    for item in items {
        // TODO: spawn async task to double the item
    }
    
    let mut results = Vec::new();
    // Your implementation: collect results
    
    results
}
```

### Problem 3.2 (10 points)
Explain the difference between `spawn` and `spawn_blocking` and when to use each.

---

## Section IV: Basic Serde Usage (25 points)

### Problem 4.1 (15 points)
Create a deserializable struct and demonstrate both owned and borrowed deserialization:

```rust
use serde::{Deserialize, Serialize};

// Your task: Create a struct that can deserialize this JSON:
// {"name": "Alice", "age": 30, "emails": ["alice@example.com", "alice@work.com"]}

#[derive(Deserialize)]
struct Person {
    // Your fields
}

// Implement both:
fn parse_owned(json: &str) -> Result<Person, serde_json::Error> {
    // Parse to owned data
}

fn parse_borrowed(json: &str) -> Result</* Your borrowed type */, serde_json::Error> {
    // Parse with borrowing
}
```

### Problem 4.2 (10 points)
Explain when you would choose borrowed deserialization vs owned deserialization.

---

## Answer Key & Assessment Rubric

### Scoring Guidelines

**Excellent (90-100%)**
- Correct implementations with proper error handling
- Clear explanations demonstrating understanding of underlying concepts
- Recognition of performance and safety tradeoffs

**Good (80-89%)**
- Mostly correct implementations with minor issues
- Good explanations with some gaps in deeper understanding
- Shows awareness of Rust's safety guarantees

**Satisfactory (70-79%)**
- Basic implementations that compile and work for common cases
- Surface-level explanations without deep insight
- Demonstrates ability to use Rust features but not design with them

**Needs Improvement (60-69%)**
- Partial implementations or implementations with significant issues
- Explanations show gaps in fundamental understanding
- Difficulty applying concepts to novel situations

**Insufficient (<60%)**
- Non-working implementations or major conceptual errors
- Little evidence of understanding core Rust concepts
- Cannot adapt basic patterns to new contexts

---

## Next Steps

### If you scored 80%+:
- You're ready for the comprehensive **Mastery Exam** (`03_mastery_exam.md`)
- Consider reviewing any weak areas before advanced coursework

### If you scored 60-79%:
- Review homework exercises for areas where you struggled
- Focus on understanding concepts rather than memorizing patterns
- Retake this exam after additional practice

### If you scored <60%:
- Work through homework assignments systematically
- Consider additional Rust learning resources
- Focus on one concept at a time before moving to integration

### Advanced Path:
After mastering fundamentals, progress to:
1. **Mastery Exam** (`03_mastery_exam.md`) - Comprehensive integration assessment
2. **Bridge Course** (`04_production_exam.md`) - Production skills development  
3. **Capstone Projects** - Real-world systems engineering

---

*This diagnostic exam validates individual concept mastery. Production Rust development requires systems thinking and concept integration, assessed in our comprehensive curriculum.*
