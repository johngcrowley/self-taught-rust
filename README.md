# Active Recall Rust Systems Programming Course

*Master production-ready Rust through spatial understanding and hands-on systems engineering*

## Course Philosophy

This course is built around **Active Recall** methodology and the breakthrough insight that **lifetimes are spatial concepts** - they're labels for stack frame regions where memory is allocated, not temporal duration. This spatial understanding transforms how you think about Rust's ownership system and unlocks advanced systems programming patterns.

## Learning Progression

### Phase 1: Foundational Skills (Days 1-14)
**Location**: `homework/day1_*` through `homework/day14_*`

Build core competencies through hands-on exercises:

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

### Phase 2: Mastery Assessment
**Location**: `school/0_course.md`

**The Rust Systems Programming Mastery Exam** - A comprehensive 6-hour assessment that validates your understanding of how all concepts integrate. This exam is built around the spatial model of lifetimes and tests your ability to reason about memory regions, async state machines, and production system design.

### Phase 3: Production Bridge Course  
**Location**: `school/1_introduction.md`

**4-Day Intensive Bridge Course** - Transforms homework-level knowledge into production-ready skills:

- **Day 1**: Advanced lifetime management & production race conditions
- **Day 2**: Zero-copy processing & advanced serde patterns  
- **Day 3**: Production error handling & observability
- **Day 4**: Distributed system patterns & integration challenges

**Prerequisites**: Pass the Mastery Exam
**Outcome**: Ready for advanced projects and real production work

### Phase 4: Advanced Applications

**Neon Storage Backend Project** (`school/2_midterm_project.md`)
6-day intensive building production storage system patterns including WAL processing, layer management, and distributed reconciliation.

**Rust Performance Observatory** (`school/3_capstone.md`)  
Deploy a complete streaming data pipeline to local k3s, orchestrate with custom operators, and build comprehensive tooling to measure and optimize Rust performance.

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

### Exams Directory Structure

**`exams/exam1.md`** - Traditional academic-style exam (supplementary)
**`exams/midterm.md`** - Advanced concepts exam (supplementary) 
**`school/0_course.md`** - **PRIMARY ASSESSMENT** - Comprehensive mastery exam

The primary assessment is the comprehensive exam in `school/0_course.md` because it:
- Tests integrated understanding rather than isolated concepts
- Uses the spatial lifetime model consistently
- Connects homework exercises to production scenarios
- Validates systems thinking and architecture skills

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