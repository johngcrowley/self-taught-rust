// Day 1: Basic Ownership Rules
// Time Limit: 15 minutes
// Core Concept: Move semantics and basic ownership

fn main() {
    println!("Day 1: Basic Ownership Challenge");
    
    // Challenge 1: Fix this code to make it compile
    challenge_1();
    
    // Challenge 2: Understanding moves
    challenge_2();
    
    // Challenge 3: When to clone vs reference
    challenge_3();
}

// CHALLENGE 1: Fix this function to print the vector twice
fn challenge_1() {
    println!("\n=== Challenge 1: Fix the Move ===");
    let data = vec![1, 2, 3];
    print_vec(data);
    // print_vec(data); // UNCOMMENT THIS - it should work after your fix
    
    // TODO: Fix the code above without changing print_vec signature
    // HINT: Think about the difference between moving and borrowing
}

fn print_vec(v: Vec<i32>) {
    println!("{:?}", v);
}

// CHALLENGE 2: Understanding what gets moved
fn challenge_2() {
    println!("\n=== Challenge 2: What Gets Moved? ===");
    let s1 = String::from("hello");
    let s2 = s1; // What happens to s1 here?
    
    // println!("{}", s1); // Will this compile? Why or why not?
    println!("{}", s2);
    
    // TODO: Explain in comments what happened to s1 and why
    // TODO: How would you fix this to use both s1 and s2?
}

// CHALLENGE 3: Choose the right parameter type
fn challenge_3() {
    println!("\n=== Challenge 3: Efficient Function Parameters ===");
    let owned_string = String::from("Hello, World!");
    let string_slice = "Hello, Rust!";
    
    // TODO: Implement process_text function that can handle both types efficiently
    // process_text should NOT take ownership unnecessarily
    
    let result1 = process_text(&owned_string);
    let result2 = process_text(string_slice);
    
    println!("Result 1: {}", result1);
    println!("Result 2: {}", result2);
    
    // These should still be accessible:
    println!("Original owned_string: {}", owned_string);
    println!("Original string_slice: {}", string_slice);
}

// TODO: Implement this function with the right parameter type
fn process_text(text: &str) -> String {
    // Return the text with "Processed: " prefix
    format!("Processed: {}", text)
}

// SOLUTION TEMPLATE (uncomment and implement):

/*
// SOLUTION FOR CHALLENGE 1:
fn challenge_1_solution() {
    let data = vec![1, 2, 3];
    print_vec(data.clone()); // Clone for first use
    print_vec(data);         // Move on second use
    
    // Alternative: use references
    // print_vec_ref(&data);
    // print_vec_ref(&data);
}

fn print_vec_ref(v: &Vec<i32>) {
    println!("{:?}", v);
}
*/

// ACTIVE RECALL QUESTIONS:
// 1. What is the difference between move and copy semantics?
// 2. Which types implement Copy trait by default?  
// 3. When should you use clone() vs references?
// 4. What happens when you assign a non-Copy type to another variable?
// 5. How does Rust prevent use-after-free errors at compile time?

// SPACED REPETITION SCHEDULE:
// - Review this concept tomorrow (Day 2)
// - Review again in 3 days (Day 4) 
// - Review again in 1 week (Day 8)
// - Final review in 2 weeks (Day 15)