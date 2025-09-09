// String Concatenation Benchmark Challenge
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

// Approach 1: String concatenation with push_str
fn concat_strings_owned(inputs: &[&str]) -> String {
    let mut result = String::new();
    for input in inputs {
        result.push_str(input);
        result.push(' ');
    }
    result.pop(); // Remove trailing space
    result
}

// Approach 2: Collect with join
fn concat_strings_join(inputs: &[&str]) -> String {
    inputs.join(" ")
}

// Approach 3: Pre-allocated capacity  
fn concat_strings_capacity(inputs: &[&str]) -> String {
    let total_len: usize = inputs.iter().map(|s| s.len()).sum::<usize>() + inputs.len().saturating_sub(1);
    let mut result = String::with_capacity(total_len);
    for (i, input) in inputs.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        result.push_str(input);
    }
    result
}

// Approach 4: Using format! macro
fn concat_strings_format(inputs: &[&str]) -> String {
    inputs.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ")
}

// Approach 5: Iterator with fold
fn concat_strings_fold(inputs: &[&str]) -> String {
    inputs.iter().skip(1).fold(inputs[0].to_string(), |mut acc, s| {
        acc.push(' ');
        acc.push_str(s);
        acc
    })
}

fn benchmark_string_concat(c: &mut Criterion) {
    let inputs = vec!["hello", "world", "this", "is", "a", "test", "string"];
    
    let mut group = c.benchmark_group("string_concatenation");
    
    group.bench_function("owned_concat", |b| {
        b.iter(|| concat_strings_owned(black_box(&inputs)))
    });
    
    group.bench_function("join", |b| {
        b.iter(|| concat_strings_join(black_box(&inputs)))
    });
    
    group.bench_function("with_capacity", |b| {
        b.iter(|| concat_strings_capacity(black_box(&inputs)))  
    });
    
    group.bench_function("format_macro", |b| {
        b.iter(|| concat_strings_format(black_box(&inputs)))
    });
    
    group.bench_function("fold", |b| {
        b.iter(|| concat_strings_fold(black_box(&inputs)))
    });
    
    group.finish();
}

// Scaling benchmark - test performance with different input sizes
fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_concat_scaling");
    
    for size in [10, 100, 1000, 10000].iter() {
        let inputs: Vec<String> = (0..*size)
            .map(|i| format!("string_{}", i))
            .collect();
        let input_refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
        
        group.bench_with_input(BenchmarkId::new("owned", size), &input_refs, |b, inputs| {
            b.iter(|| concat_strings_owned(black_box(inputs)))
        });
        
        group.bench_with_input(BenchmarkId::new("join", size), &input_refs, |b, inputs| {
            b.iter(|| concat_strings_join(black_box(inputs)))
        });
        
        group.bench_with_input(BenchmarkId::new("with_capacity", size), &input_refs, |b, inputs| {
            b.iter(|| concat_strings_capacity(black_box(inputs)))
        });
        
        group.bench_with_input(BenchmarkId::new("format", size), &input_refs, |b, inputs| {
            b.iter(|| concat_strings_format(black_box(inputs)))
        });
        
        group.bench_with_input(BenchmarkId::new("fold", size), &input_refs, |b, inputs| {
            b.iter(|| concat_strings_fold(black_box(inputs)))
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_string_concat, benchmark_scaling);
criterion_main!(benches);