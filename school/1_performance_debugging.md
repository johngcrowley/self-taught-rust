# Performance Debugging Primer
*Learn GDB, Criterion, Profiling Tools Through Problem-Solving*

---

## Course Philosophy

This primer gives you the essential background knowledge and then presents real-world problems for you to solve independently. You'll learn the tools by using them, not by reading exhaustive tutorials.

**Method**: Brief primers on each tool, then increasingly challenging problems for you to tackle. Research solutions yourself using documentation and experimentation.

---

## Section I: GDB Debugging Primer

### GDB Basics You Need
- `gdb -p <pid>` - attach to running process  
- `info threads` - list all threads
- `thread <n>` - switch to thread n
- `bt` - backtrace current thread
- `info locals` - local variables
- `print <var>` - examine variable

### Problem 1: The Hanging Service
**Situation**: Production service consuming 100% CPU but not responding to requests. Process PID is 12345.

**Your Task**: 
1. Write a deadlocking Rust program with 2 threads acquiring locks in opposite order
2. Attach GDB and identify which threads are stuck and where
3. Determine the deadlock pattern from the stack traces

**Questions to Answer**:
- Which GDB commands reveal the deadlock?
- How can you identify the specific mutex objects involved?
- What's the fix for this locking pattern?

### Problem 2: The Segfaulting Parser
**Situation**: Parser crashes with SIGSEGV but only on specific inputs. Stack trace points to valid-looking code.

**Your Task**:
1. Create a parser that uses unsafe code with a lifetime bug
2. Find the crash using GDB and identify the invalid memory access
3. Correlate the crash with the source code line

**Questions to Answer**:
- How do you get GDB to break on SIGSEGV?
- What commands show you the memory that caused the fault?
- How can you examine the memory layout around the crash?

---

## Section II: Criterion Benchmarking Primer

### Criterion Basics You Need
- `cargo bench` - run benchmarks
- `criterion::Criterion` - main benchmarking struct
- `bench_function()` - benchmark a single function  
- `iter()` - measure iterations with black_box
- HTML reports in `target/criterion/`

### Problem 3: The Slow "Fast" Parser
**Situation**: Your zero-copy JSON parser benchmarks slower than the naive version. Results are inconsistent.

**Your Task**:
1. Create two JSON parsers: one zero-copy with `#[serde(borrow)]`, one owned
2. Write proper Criterion benchmarks for both with different input sizes
3. Identify why zero-copy might be slower in certain scenarios

**Questions to Answer**:
- What makes a good benchmark input?
- How do you ensure consistent benchmark conditions? 
- When does allocation overhead actually matter?

### Problem 4: The Microbenchmark Trap
**Situation**: Your optimized hash function shows 10x improvement in microbenchmarks but no real-world speedup.

**Your Task**:
1. Write a "fast" hash function that benchmarks well but performs poorly in practice
2. Design benchmarks that reveal the real-world performance characteristics
3. Identify the measurement distortions in microbenchmarks

**Questions to Answer**:
- What's the difference between throughput and latency benchmarks?
- How do branch prediction and caching affect microbenchmarks?
- How do you benchmark with realistic data patterns?

### Problem 5: The Scaling Mystery
**Situation**: Your O(n) string processing shows O(n²) benchmark behavior.

**Your Task**:
1. Write a seemingly linear function that has hidden quadratic growth
2. Use Criterion to measure scaling behavior with different input sizes
3. Identify the source of unexpected scaling

**Questions to Answer**:
- How do you test algorithmic complexity with benchmarks?
- What Vec operations can cause quadratic behavior?
- How do you distinguish between algorithmic and constant factor issues?

---

## Section III: Profiling Primer

### Profiling Tools You Need
- `cargo install flamegraph` - CPU flame graphs
- `perf record` / `perf report` - Linux profiling
- `valgrind --tool=massif` - memory profiling  
- `cargo-profdata` - PGO analysis

### Problem 6: The CPU Hotspot Hunt
**Situation**: Your server handles 1000 RPS but CPU usage is 80%. Need to find the bottleneck.

**Your Task**:
1. Create a program with an unexpected CPU hotspot (like expensive string operations in logging)
2. Use flamegraph to identify the hotspot
3. Measure the improvement after optimization

**Questions to Answer**:
- How do you get meaningful flame graphs from Rust programs?
- What flame graph patterns indicate different types of bottlenecks?
- How do you validate that optimizations actually help?

### Problem 7: The Memory Leak Detective  
**Situation**: Your long-running service's memory usage grows from 100MB to 2GB over 24 hours.

**Your Task**:
1. Create a program with a subtle memory leak (like caching without bounds)
2. Use memory profiling tools to identify the leak source
3. Verify the fix with before/after memory profiles

**Questions to Answer**:
- What's the difference between memory leaks and memory growth?
- How do you profile memory allocation patterns in Rust?
- What tools help identify different types of memory issues?

### Problem 8: The Cache Miss Investigation
**Situation**: Your data processing pipeline is memory-bound despite having plenty of available memory.

**Your Task**:
1. Create a data structure with poor cache locality
2. Use `perf` to measure cache miss rates
3. Redesign for better cache performance and measure the improvement

**Questions to Answer**:
- What `perf` events indicate cache performance issues?
- How do data structure layouts affect cache performance?
- How do you balance memory usage vs cache locality?

---

## Section IV: Production Debugging Scenarios

### Problem 9: The Intermittent Deadlock
**Real-World Scenario**: Customer reports that your service "sometimes" becomes unresponsive during peak load.

**Your Mission**:
1. Design a reproduction case that triggers the issue under load
2. Set up monitoring to catch the deadlock when it happens
3. Create tooling to diagnose the issue in production

**Research Areas**:
- How do you debug issues that only occur in production?
- What observability do you need to catch intermittent issues?
- How do you safely gather debugging info from a live service?

### Problem 10: The Performance Regression Hunt
**Real-World Scenario**: After a deployment, p99 latency increased from 50ms to 200ms. Git blame shows 47 commits.

**Your Mission**:
1. Design a git bisect strategy to find the problematic commit
2. Create automated performance tests to validate fixes
3. Implement monitoring to catch future regressions

**Research Areas**:
- How do you automate performance regression detection?
- What metrics best indicate real performance problems?
- How do you balance test coverage vs test runtime?

---

## Mastery Goals

After working through these problems, you should be able to:

**Debug Production Issues**:
- Attach to running processes and diagnose problems
- Use stack traces and memory dumps effectively
- Correlate symptoms with root causes

**Measure Performance Accurately**:
- Design benchmarks that reflect real usage
- Identify measurement artifacts and confounding factors  
- Validate that optimizations provide real benefits

**Profile and Optimize Systems**:
- Find CPU and memory bottlenecks in complex programs
- Understand the performance characteristics of your code
- Make data-driven optimization decisions

**Prevent Future Problems**:
- Set up monitoring and alerting for performance issues
- Design systems that are inherently observable
- Create reproducible test cases for debugging

---

## Next Steps

- Work through each problem systematically
- Research tools and techniques as needed
- Build your own debugging and profiling toolkit
- Practice on your own projects to reinforce learning

The key is hands-on experience - reading about these tools won't make you proficient. You need to use them on real problems to develop debugging intuition.