# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an active recall Rust programming course focused on building advanced asynchronous data processing and streaming systems. The course is built around the revolutionary insight that **lifetimes are spatial concepts** - they're labels for stack frame regions where memory is allocated, not temporal duration.

**Key Innovation**: The breakthrough understanding discovered in `homework/day13_lifetimes_grok/lifetimes.md` that lifetimes are spatial stack frame labels transforms how you think about Rust's ownership system and unlocks advanced systems programming patterns.

## Repository Structure

### Core Directories
- `homework/` - Daily programming exercises organized by topic (day1-day14)
- `school/` - Course curriculum and project specifications
  - `0_course.md` - **PRIMARY**: Comprehensive Mastery Exam (6 hours, systems integration)
  - `1_introduction.md` - Production Bridge Course (4-day intensive)
  - `2_midterm_project.md` - Neon Storage Backend Project (6-day advanced)
  - `3_capstone.md` - Rust Performance Observatory (local k3s deployment)
- `exams/` - Supplementary assessments
  - `exam1.md` - Diagnostic exam for individual concepts
  - `midterm.md` - Integration-focused intermediate assessment

### Homework Organization
Each homework directory follows the pattern `dayX_topic_name/` with:
- Individual `Cargo.toml` files (edition 2024)
- `src/main.rs` with exercise implementations
- Focus areas include:
  - Async concurrency (JoinSet, channels, mutexes)
  - Error handling (thiserror, anyhow)
  - Serde serialization and lifetimes
  - Iterator chains and combinators
  - Tokio streams and async I/O

## Build and Development Commands

### Building Individual Exercises
```bash
# Navigate to specific day directory
cd homework/day1_joinset_basics
cargo build
cargo run

# Or build/run from project root
cargo build --manifest-path homework/day1_joinset_basics/Cargo.toml
cargo run --manifest-path homework/day1_joinset_basics/Cargo.toml
```

### Testing (if tests exist)
```bash
# From homework directory
cargo test

# From specific exercise
cd homework/dayX_topic_name
cargo test
```

## Key Technical Concepts

### The Spatial Model of Lifetimes (Revolutionary Breakthrough)
**Discovered in**: `homework/day13_lifetimes_grok/lifetimes.md`

The course is built around this transformative understanding:
- **Lifetimes are NOT temporal concepts** (duration)
- **Lifetimes ARE spatial labels** for stack frame regions where memory is allocated
- **References declare which memory region they borrow from**
- **When stack frames pop, all references to that region become invalid**

This spatial model explains:
- Why zero-copy serde needs `#[serde(borrow)]`
- How async move semantics work with task state machines
- Why certain lifetime bounds are required (`'b: 'a`)
- How to reason about complex multi-lifetime scenarios

### Course Focus Areas
1. **Spatial Memory Reasoning**: Stack frames, lifetime regions, zero-copy patterns
2. **Async Programming**: Tokio runtime, JoinSet, channels (mpsc), mutexes, and streaming
3. **Error Handling**: Custom error types, thiserror derive macros, anyhow for application errors
4. **Production Systems**: Tracing/observability, configuration management, graceful shutdown
5. **Performance Engineering**: Benchmarking with criterion, memory profiling, system optimization

### Common Dependencies
- `tokio` - Async runtime with full feature set
- `serde` - Serialization/deserialization with derive macros  
- `anyhow` - Application-level error handling
- `thiserror` - Library error type derivation
- `async-stream` - Async iterator utilities
- `reqwest` - HTTP client with streaming support

### Architecture Patterns
The course emphasizes building production-ready streaming data processors with:
- Decoupled async task communication via channels
- Comprehensive error context propagation  
- Zero-allocation data processing where possible
- Observable systems with structured logging
- Fault-tolerant distributed system design

## Development Notes

### Course Methodology
Uses **Active Recall** learning - each day combines theoretical primers with hands-on "Fix the Code" challenges that require debugging and repair of broken implementations.

### Performance Focus
Exercises emphasize memory-efficient patterns:
- Borrowed data (`&str`) over owned (`String`) where possible
- Iterator chains over explicit loops for zero-cost abstractions
- Conditional compilation for debug vs release builds
- Benchmark-driven optimization decisions

### Error Handling Strategy
- Libraries use `thiserror` for specific, matchable error types
- Applications use `anyhow` for context-rich error reporting
- Comprehensive error chain preservation and reporting

When working in this codebase, prioritize learning through implementation rather than just reading documentation. Each exercise builds on previous concepts, creating a cumulative understanding of production Rust development.