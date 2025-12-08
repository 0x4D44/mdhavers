// Primes Stress Test - Rust Implementation

use std::time::Instant;

fn sieve(n: usize) -> Vec<usize> {
    let mut is_prime = vec![true; n + 1];
    if n >= 0 { is_prime[0] = false; }
    if n >= 1 { is_prime[1] = false; }
    
    let mut p = 2;
    while p * p <= n {
        if is_prime[p] {
            let mut j = p * p;
            while j <= n {
                is_prime[j] = false;
                j += p;
            }
        }
        p += 1;
    }
    
    (2..=n).filter(|&k| is_prime[k]).collect()
}

fn main() {
    println!("=== Primes Stress Test (Rust) ===");
    
    // Correctness
    println!("Correctness check:");
    let p100 = sieve(100);
    println!("  Primes to 100: {} primes", p100.len());
    
    // Scale tests
    println!("\nScalability tests:");
    
    let start1 = Instant::now();
    let p1k = sieve(1000);
    let t1 = start1.elapsed();
    println!("  sieve(1000): {} primes in {} us", p1k.len(), t1.as_micros());
    
    let start2 = Instant::now();
    let p5k = sieve(5000);
    let t2 = start2.elapsed();
    println!("  sieve(5000): {} primes in {} us", p5k.len(), t2.as_micros());
    
    let start3 = Instant::now();
    let p10k = sieve(10000);
    let t3 = start3.elapsed();
    println!("  sieve(10000): {} primes in {} us", p10k.len(), t3.as_micros());
    
    let start4 = Instant::now();
    let p20k = sieve(20000);
    let t4 = start4.elapsed();
    println!("  sieve(20000): {} primes in {} us", p20k.len(), t4.as_micros());
    
    // Scaling
    println!("\nScaling analysis:");
    println!("  5K/1K time ratio: {:.2}", t2.as_nanos() as f64 / t1.as_nanos() as f64);
    println!("  10K/5K time ratio: {:.2}", t3.as_nanos() as f64 / t2.as_nanos() as f64);
    println!("  20K/10K time ratio: {:.2}", t4.as_nanos() as f64 / t3.as_nanos() as f64);
    
    println!("\n=== Primes stress complete ===");
}
