# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-focused active recall learning course designed for skill retention and mastery. The repository contains practical coding exercises (kata) and advanced capstone projects that demonstrate production-level systems engineering skills.

## Repository Structure

- `kata/` - Daily coding exercises organized by day (day1_joinset_basics, day2_os_threads_channels, etc.)
- `exams/` - Advanced capstone projects including:
  - `midterm.md` - Submarine ML operations system (classified military simulation)
  - `capstone.md` - Neon storage backend engineering bootcamp
- `README.md` - Project goals and learning philosophy

## Build and Test Commands

### Individual Kata Exercises

Each kata directory contains its own `Cargo.toml` with specific dependencies. To work with individual exercises:

```bash
# Run a specific day's exercise
cd kata/day2_os_threads_channels
cargo run

# Test a specific exercise
cargo test

# Check code without building
cargo check
```

### Project-wide Operations

Since this is a multi-crate workspace with individual kata exercises:

```bash
# Check all kata exercises for compilation errors
find kata -name "Cargo.toml" -execdir cargo check \;

# Run all tests across kata exercises  
find kata -name "Cargo.toml" -execdir cargo test \;

# Build all exercises
find kata -name "Cargo.toml" -execdir cargo build \;
```

## Code Architecture

### Learning Philosophy

This is an **active recall** based learning system focused on:
- Retention through deliberate practice
- Progressive complexity building
- Real-world systems engineering problems

### Kata Structure

Each daily kata focuses on specific Rust concepts:
- **OS threads and channels** - Concurrency primitives and performance comparison
- **Tokio streams** - Async programming and stream processing
- **Lifetimes** - Memory management and borrowing rules
- **Error handling** - thiserror, anyhow, and graceful error propagation
- **Serde integration** - Serialization with complex lifetime management

### Capstone Projects

The repository includes two advanced engineering simulations:

1. **Submarine ML Operations System** (`exams/midterm.md`)
   - Distributed ML inference pipeline
   - Kubernetes auto-scaling
   - Real-time data processing (10,000+ ops/second)
   - Combat-grade reliability requirements

2. **Neon Storage Backend** (`exams/capstone.md`)  
   - PostgreSQL storage layer implementation
   - Async I/O optimization
   - Distributed state reconciliation
   - Production-ready connection pooling

## Development Workflow

### Working with Exercises

1. Navigate to specific kata directory
2. Read the exercise requirements
3. Implement solutions focusing on:
   - Memory safety
   - Performance optimization
   - Error handling patterns
   - Concurrent programming techniques

### Code Standards

- Follow Rust idioms and best practices
- Prioritize zero-copy patterns where applicable
- Use structured error handling with `thiserror`
- Implement comprehensive tracing for debugging
- Focus on production-ready patterns over academic exercises

### Testing Approach

Each kata includes its own test scenarios. Run tests frequently during development:

```bash
cargo test --verbose
```

Some exercises include benchmarking capabilities using the `criterion` crate.

## Key Dependencies

Common dependencies across kata exercises:
- `tokio` - Async runtime and utilities
- `serde` - Serialization/deserialization
- `thiserror`/`anyhow` - Error handling
- `uuid` - Unique identifiers
- `chrono` - Time handling
- `bytes` - Efficient byte buffer handling
- `futures` - Stream processing utilities

## Learning Objectives

This course develops skills in:
- **Systems Programming** - Low-level performance optimization
- **Distributed Systems** - Multi-service coordination and failure handling  
- **Async Programming** - Non-blocking I/O and concurrency patterns
- **Database Engineering** - Storage layer implementation
- **Production Readiness** - Monitoring, observability, and resilience patterns

## Notes for AI Assistants

- Each kata is self-contained with its own dependencies
- Focus on practical, production-ready implementations
- Emphasize performance and memory efficiency
- The capstone projects are comprehensive system design exercises
- Code should demonstrate deep Rust expertise suitable for infrastructure engineering roles