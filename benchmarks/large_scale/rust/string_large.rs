// Large-Scale String Processing Benchmark (Rust)
use std::time::Instant;

fn build_string(n: usize) -> String {
    let mut result = String::new();
    for _ in 0..n {
        result.push('x');
    }
    result
}

fn build_pattern_string(n: usize) -> String {
    let mut result = String::new();
    for _ in 0..n {
        result.push_str("abc,");
    }
    result
}

fn main() {
    println!("=== Large-Scale String Processing Benchmark (Rust) ===");

    println!("Correctness check:");
    let small = build_string(10);
    println!("  build_string(10): len={}", small.len());
    let pattern = build_pattern_string(5);
    println!("  build_pattern_string(5): {}", pattern);

    println!("\nString building scaling:");

    let start = Instant::now();
    let s = build_string(100);
    let elapsed = start.elapsed();
    println!("  build_string(100): {} us, len={}", elapsed.as_micros(), s.len());

    let start = Instant::now();
    let s = build_string(500);
    let elapsed = start.elapsed();
    println!("  build_string(500): {} us, len={}", elapsed.as_micros(), s.len());

    let start = Instant::now();
    let s = build_string(1000);
    let elapsed = start.elapsed();
    println!("  build_string(1000): {} us, len={}", elapsed.as_micros(), s.len());

    let start = Instant::now();
    let s = build_string(2000);
    let elapsed = start.elapsed();
    println!("  build_string(2000): {} us, len={}", elapsed.as_micros(), s.len());

    println!("\nSplit/join on pattern strings:");

    let start = Instant::now();
    let pattern = build_pattern_string(100);
    let elapsed = start.elapsed();
    println!("  Build 100 patterns: {} us, len={}", elapsed.as_micros(), pattern.len());

    let start = Instant::now();
    let parts: Vec<&str> = pattern.split(',').collect();
    let elapsed = start.elapsed();
    println!("  Split 100 patterns: {} us, parts={}", elapsed.as_micros(), parts.len());

    let start = Instant::now();
    let joined = parts.join("-");
    let elapsed = start.elapsed();
    println!("  Join 100 parts: {} us, len={}", elapsed.as_micros(), joined.len());

    let start = Instant::now();
    let pattern = build_pattern_string(500);
    let elapsed = start.elapsed();
    println!("  Build 500 patterns: {} us, len={}", elapsed.as_micros(), pattern.len());

    let start = Instant::now();
    let parts: Vec<&str> = pattern.split(',').collect();
    let elapsed = start.elapsed();
    println!("  Split 500 patterns: {} us, parts={}", elapsed.as_micros(), parts.len());

    let start = Instant::now();
    let joined = parts.join("-");
    let elapsed = start.elapsed();
    println!("  Join 500 parts: {} us, len={}", elapsed.as_micros(), joined.len());

    println!("\nString operations on large strings:");

    let mut large = String::from("The quick brown fox jumps over the lazy dog. ");
    for _ in 0..5 {
        large = format!("{}{}", large, large.clone());
    }
    println!("  Built string of length: {}", large.len());

    let start = Instant::now();
    let u = large.to_uppercase();
    let elapsed = start.elapsed();
    println!("  upper(): {} us", elapsed.as_micros());
    drop(u);

    let start = Instant::now();
    let l = large.to_lowercase();
    let elapsed = start.elapsed();
    println!("  lower(): {} us", elapsed.as_micros());
    drop(l);

    let start = Instant::now();
    let found = large.contains("fox");
    let elapsed = start.elapsed();
    println!("  contains('fox'): {} in {} us", found, elapsed.as_micros());

    let start = Instant::now();
    let found = large.contains("xyz");
    let elapsed = start.elapsed();
    println!("  contains('xyz'): {} in {} us", found, elapsed.as_micros());

    println!("\n=== String large-scale complete ===");
}
