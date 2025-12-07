// Rust benchmark implementations for comparison with mdhavers
// Run with: cargo run --release

use std::time::Instant;

mod fibonacci;
mod factorial;
mod gcd;
mod primes;
mod quicksort;
mod mergesort;

fn main() {
    println!("=== Rust Benchmarks ===\n");

    println!("--- Fibonacci ---");
    fibonacci::run();
    println!();

    println!("--- Factorial ---");
    factorial::run();
    println!();

    println!("--- GCD ---");
    gcd::run();
    println!();

    println!("--- Primes ---");
    primes::run();
    println!();

    println!("--- Quicksort ---");
    quicksort::run();
    println!();

    println!("--- Mergesort ---");
    mergesort::run();
    println!();

    println!("=== Rust Benchmarks Complete ===");
}
