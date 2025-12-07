use std::time::Instant;

fn fib_naive(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    fib_naive(n - 1) + fib_naive(n - 2)
}

fn fib_iter(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

pub fn run() {
    // Test correctness
    println!("Testing correctness...");
    println!("fib(0) = {}", fib_iter(0));
    println!("fib(1) = {}", fib_iter(1));
    println!("fib(10) = {}", fib_iter(10));
    println!("fib(20) = {}", fib_iter(20));

    // Iterative benchmark
    println!("\nIterative fib(40)...");
    let start = Instant::now();
    let result = fib_iter(40);
    let elapsed = start.elapsed();
    println!("fib_iter(40) = {} ({:?})", result, elapsed);

    // Naive recursive benchmark
    println!("\nNaive recursive fib(30)...");
    let start = Instant::now();
    let result = fib_naive(30);
    let elapsed = start.elapsed();
    println!("fib_naive(30) = {} ({:?})", result, elapsed);
}
