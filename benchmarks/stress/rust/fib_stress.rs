// Fibonacci Stress Test - Rust Implementation

use std::time::Instant;

fn fib_naive(n: u64) -> u64 {
    if n <= 1 { return n; }
    fib_naive(n - 1) + fib_naive(n - 2)
}

fn fib_iter(n: u64) -> u64 {
    if n <= 1 { return n; }
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

fn main() {
    println!("=== Fibonacci Stress Test (Rust) ===");
    
    // Correctness
    println!("Correctness check:");
    println!("  fib(10) = {}", fib_iter(10));
    println!("  fib(20) = {}", fib_iter(20));
    
    // Iterative stress
    println!("\nIterative stress test:");
    let start = Instant::now();
    let result = fib_iter(10000);
    let elapsed = start.elapsed();
    println!("  fib_iter(10000) completed in {} us", elapsed.as_micros());
    
    // Many iterations
    let start2 = Instant::now();
    for _ in 0..1000 {
        let _ = fib_iter(1000);
    }
    let elapsed2 = start2.elapsed();
    println!("  1000x fib_iter(1000) in {} ms", elapsed2.as_millis());
    
    // Recursive stress
    println!("\nRecursive stress test:");
    let start3 = Instant::now();
    let r30 = fib_naive(30);
    let elapsed3 = start3.elapsed();
    println!("  fib_naive(30) = {} in {} ms", r30, elapsed3.as_millis());
    
    println!("\n=== Fibonacci stress complete ===");
}
