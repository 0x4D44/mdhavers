// Large-Scale Fibonacci Benchmark (Rust)
use std::time::Instant;

fn fib_iter(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 2..=n {
        let temp = a.wrapping_add(b);
        a = b;
        b = temp;
    }
    b
}

fn main() {
    println!("=== Large-Scale Fibonacci Benchmark (Rust) ===");

    println!("Correctness check:");
    println!("  fib(10) = {}", fib_iter(10));
    println!("  fib(20) = {}", fib_iter(20));
    println!("  fib(50) = {}", fib_iter(50));

    println!("\nLarge-scale iterative tests:");

    let start = Instant::now();
    let _ = fib_iter(1000);
    let elapsed = start.elapsed();
    println!("  fib_iter(1000): {} us", elapsed.as_micros());

    let start = Instant::now();
    let _ = fib_iter(10000);
    let elapsed = start.elapsed();
    println!("  fib_iter(10000): {} us", elapsed.as_micros());

    let start = Instant::now();
    let _ = fib_iter(50000);
    let elapsed = start.elapsed();
    println!("  fib_iter(50000): {} us", elapsed.as_micros());

    let start = Instant::now();
    let _ = fib_iter(100000);
    let elapsed = start.elapsed();
    println!("  fib_iter(100000): {} us", elapsed.as_micros());

    println!("\nStress test (10000 calls to fib_iter(100)):");
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = fib_iter(100);
    }
    let elapsed = start.elapsed();
    println!("  10000x fib_iter(100): {} ms", elapsed.as_millis());

    println!("\n=== Fibonacci large-scale complete ===");
}
