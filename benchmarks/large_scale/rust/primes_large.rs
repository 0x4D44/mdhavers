// Large-Scale Prime Sieve Benchmark (Rust)
use std::time::Instant;

fn count_primes(limit: usize) -> usize {
    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    if limit > 0 {
        is_prime[1] = false;
    }

    let mut p = 2;
    while p * p <= limit {
        if is_prime[p] {
            let mut multiple = p * p;
            while multiple <= limit {
                is_prime[multiple] = false;
                multiple += p;
            }
        }
        p += 1;
    }

    is_prime.iter().filter(|&&x| x).count()
}

fn sieve(limit: usize) -> Vec<usize> {
    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    if limit > 0 {
        is_prime[1] = false;
    }

    let mut p = 2;
    while p * p <= limit {
        if is_prime[p] {
            let mut multiple = p * p;
            while multiple <= limit {
                is_prime[multiple] = false;
                multiple += p;
            }
        }
        p += 1;
    }

    is_prime.iter().enumerate()
        .filter(|(_, &is_p)| is_p)
        .map(|(i, _)| i)
        .collect()
}

fn main() {
    println!("=== Large-Scale Prime Sieve Benchmark (Rust) ===");

    println!("Correctness check:");
    let small = sieve(100);
    println!("  Primes to 100: {} primes", small.len());
    println!("  First 10: {:?}", &small[..10]);

    println!("\nScaling tests:");

    let start = Instant::now();
    let c = count_primes(10000);
    let elapsed = start.elapsed();
    println!("  sieve(10K): {} primes in {} us", c, elapsed.as_micros());

    let start = Instant::now();
    let c = count_primes(50000);
    let elapsed = start.elapsed();
    println!("  sieve(50K): {} primes in {} us", c, elapsed.as_micros());

    let start = Instant::now();
    let c = count_primes(100000);
    let elapsed = start.elapsed();
    println!("  sieve(100K): {} primes in {} ms", c, elapsed.as_millis());

    let start = Instant::now();
    let c = count_primes(500000);
    let elapsed = start.elapsed();
    println!("  sieve(500K): {} primes in {} ms", c, elapsed.as_millis());

    let start = Instant::now();
    let c = count_primes(1000000);
    let elapsed = start.elapsed();
    println!("  sieve(1M): {} primes in {} ms", c, elapsed.as_millis());

    println!("\n=== Prime sieve large-scale complete ===");
}
