# Active Recall Rust Systems Programming Course

*Master production-ready Rust through spatial understanding and hands-on systems engineering*

## Course Philosophy

This course is built around **Active Recall** methodology and the breakthrough insight that **lifetimes are spatial concepts** - they're labels for stack frame regions where memory is allocated, not temporal duration. This spatial understanding transforms how you think about Rust's ownership system and unlocks advanced systems programming patterns.

## Learning Progression

### Phase 1: Foundational Practice (Days 1-14)
**Location**: `homework/day1_*` through `homework/day14_*`

Your personal workspace for practicing core competencies:

**Async & Concurrency** (Days 1-5)
- `day1_joinset_basics` - Basic task spawning and coordination
- `day2_os_threads_channels` - OS thread patterns and synchronization  
- `day3_recall` - Integration and pattern reinforcement
- `day4_mutexes_intro` - Shared state and locking
- `day5_tokio_streams` - Streaming data and backpressure

**Error Handling Evolution** (Days 6-9)  
- `day6_error_basics` - Standard library error patterns
- `day7_recall` - Channel-based error propagation
- `day8_custom_error` - Building domain-specific error types
- `day9_thiserror_anyhow` - Production error handling

**Data Processing & Advanced Patterns** (Days 10-14)
- `day10_serde_intro` - Serialization fundamentals
- `day11_iterator_chains` - Functional data processing
- `day12_graceful_combinators` - Error handling with combinators
- `day13_lifetimes_grok` - **THE BREAKTHROUGH**: Spatial lifetime understanding
- `day14_serde_lifetimes` - Zero-copy deserialization patterns

### Phase 2: Active Recall Mastery Courses  
**Location**: `school/`

Intensive courses that push you to mastery through active recall methodology:

**`school/1_performance_debugging.md`** - **Performance Engineering & Debugging Mastery**
Master GDB debugging, criterion benchmarking, CPU profiling, memory analysis, and production performance engineering. Learn to diagnose deadlocks, hunt memory leaks, and optimize for real-world workloads.

**`school/2_midterm_project.md`** - **Neon Storage Backend Project**  
6-day intensive building production storage system patterns including WAL processing, layer management, and distributed reconciliation.

**`school/3_capstone.md`** - **Rust Performance Observatory**
Deploy a complete streaming data pipeline to local k3s, orchestrate with custom operators, and build comprehensive tooling to measure and optimize Rust performance.

### Phase 3: Challenging Examinations
**Location**: `exams/`

Rigorous exams designed to really test your mastery:

**`exams/01_diagnostic_exam.md`** - **Diagnostic Exam** (2 hours)
Individual concept assessment for foundational knowledge

**`exams/02_integration_exam.md`** - **Integration Exam** (4 hours)  
Tests integration of multiple concepts and intermediate systems design

**`exams/03_mastery_exam.md`** - **The Comprehensive Mastery Exam** (6 hours)
Ultimate test of systems integration built around spatial lifetime model

**`exams/04_production_exam.md`** - **Production Bridge Course Exam** (4 hours)
Advanced production patterns and crisis simulation challenges

**`exams/05_systems_specialization_exam.md`** - **Advanced Systems Programming** (4 hours)
Distributed systems, performance engineering, memory management, and error handling

**`exams/06_concurrency_specialization_exam.md`** - **Async & Concurrency Mastery** (3 hours)
Deep dive into async runtime internals, advanced synchronization, and concurrent system design

## Key Learning Breakthroughs

### The Spatial Model of Lifetimes
**Discovered in**: `homework/day13_lifetimes_grok/lifetimes.md`

The revolutionary insight that changes everything:
- **Lifetimes are NOT temporal concepts**  
- **Lifetimes ARE spatial labels for stack frame regions**
- **References declare which memory region they borrow from**
- **When stack frames pop, all references to that region become invalid**

This spatial understanding explains:
- Why zero-copy serde needs `#[serde(borrow)]`
- How async move semantics work with task state machines  
- Why certain lifetime bounds are required
- How to reason about complex multi-lifetime scenarios

### Production Systems Thinking

From homework patterns to production reality:
- **Memory efficiency**: Zero-copy vs allocation patterns
- **Error strategies**: Recovery, fallback, and graceful degradation  
- **Async coordination**: Backpressure, cancellation, and resource management
- **Observability**: Metrics, tracing, and performance profiling
- **Distributed patterns**: Reconciliation, eventual consistency, partial failures

## Assessment Strategy

**Exams are designed to really push you** - they're not just knowledge checks, but challenging scenarios that force you to apply concepts under pressure and integrate multiple areas of expertise.

**Progression Strategy:**
- Start with `exams/01_diagnostic_exam.md` for diagnostic assessment
- Work through `school/` courses for active recall mastery  
- Take `exams/03_mastery_exam.md` as the ultimate comprehensive test
- Challenge yourself with specialized exams like `06_concurrency_specialization_exam.md`

The exams mirror real-world scenarios: debugging production issues, optimizing performance under constraints, designing distributed systems, and handling concurrent programming challenges.

## Getting Started

### For New Students
1. **Complete homework exercises** (days 1-14) in order
2. **Take the Mastery Exam** (`school/0_course.md`) 
3. **Enroll in Bridge Course** (`school/1_introduction.md`)
4. **Choose advanced project** (Neon Backend or Performance Observatory)

### For Assessment
- **Diagnostic**: Try homework day3 or day7 to gauge current level
- **Comprehensive**: Take the full Mastery Exam for complete evaluation
- **Production Readiness**: Complete the Bridge Course challenges

### For Instructors
- **Homework** provides foundational skill building with active recall
- **Mastery Exam** validates deep understanding and integration
- **Bridge Course** develops production engineering capabilities  
- **Advanced Projects** demonstrate professional-level competency

## Technical Requirements

### Development Environment
```bash
# Rust toolchain
rustup update stable
rustup component add clipart rust-src rust-analyzer

# Async runtime  
cargo install tokio

# Performance tooling
cargo install criterion cargo-flamegraph

# Kubernetes (for capstone)
curl -sfL https://get.k3s.io | sh -
```

### Key Dependencies
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] } 
thiserror = "1.0"
anyhow = "1.0"
bytes = "1.5"
tracing = "0.1"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

## Course Outcomes

### Upon Completion, You Will:

**Think Spatially About Memory**
- Reason about stack frames and memory regions
- Design zero-copy data processing pipelines  
- Optimize allocation patterns for performance

**Master Production Error Handling**
- Build comprehensive error type hierarchies
- Implement graceful degradation and recovery
- Design observable error propagation systems

**Architect Concurrent Systems**  
- Coordinate async tasks with proper backpressure
- Handle cancellation and resource cleanup
- Build distributed reconciliation systems

**Engineer for Performance**
- Profile and benchmark Rust applications
- Measure memory usage and allocation patterns
- Deploy and monitor production systems

**Ready for Professional Development**
- Contribute to production Rust codebases
- Design and review system architectures
- Mentor other developers in advanced Rust patterns

---

## Course Statistics

- **14 homework assignments** building foundational skills
- **1 comprehensive exam** validating mastery  
- **4-day bridge course** developing production capabilities
- **2 capstone projects** demonstrating professional competency
- **100+ hours** of hands-on Rust development
- **Production-ready skills** for systems engineering roles

**Success Metric**: Graduates can design, implement, and deploy production Rust systems with confidence in memory safety, performance characteristics, and operational reliability.

---

*"The best way to learn systems programming is to build systems. The best way to master Rust is to understand that lifetimes are spatial, not temporal."*