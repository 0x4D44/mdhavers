use std::time::Instant;

fn factorial(n: u64) -> u64 {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}

fn factorial_iter(n: u64) -> u64 {
    let mut result = 1u64;
    for i in 2..=n {
        result *= i;
    }
    result
}

pub fn run() {
    // Test correctness
    println!("Testing correctness...");
    println!("0! = {}", factorial(0));
    println!("1! = {}", factorial(1));
    println!("5! = {}", factorial(5));
    println!("10! = {}", factorial(10));

    // Benchmark
    println!("\nComputing 20!...");
    let start = Instant::now();
    let result = factorial(20);
    let elapsed = start.elapsed();
    println!("20! = {} ({:?})", result, elapsed);

    // Iterative
    println!("\nComputing factorial iteratively...");
    let start = Instant::now();
    for i in 1..=20 {
        let f = factorial_iter(i);
        println!("{}! = {}", i, f);
    }
    let elapsed = start.elapsed();
    println!("Total time: {:?}", elapsed);
}
