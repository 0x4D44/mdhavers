use std::time::Instant;

fn sieve(n: usize) -> Vec<usize> {
    let mut is_prime = vec![true; n + 1];
    if n >= 0 {
        is_prime[0] = false;
    }
    if n >= 1 {
        is_prime[1] = false;
    }

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

    is_prime
        .iter()
        .enumerate()
        .filter(|(_, &is_p)| is_p)
        .map(|(i, _)| i)
        .collect()
}

fn is_prime(n: i64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

pub fn run() {
    // Test correctness
    println!("Testing primality...");
    println!("is_prime(2) = {}", is_prime(2));
    println!("is_prime(17) = {}", is_prime(17));
    println!("is_prime(18) = {}", is_prime(18));
    println!("is_prime(97) = {}", is_prime(97));

    // Sieve benchmark
    println!("\nSieve of Eratosthenes up to 100...");
    let start = Instant::now();
    let primes100 = sieve(100);
    let elapsed = start.elapsed();
    println!("Found {} primes ({:?})", primes100.len(), elapsed);
    println!(
        "First 10: {:?}",
        primes100.iter().take(10).collect::<Vec<_>>()
    );

    println!("\nSieve up to 1000...");
    let start = Instant::now();
    let primes1000 = sieve(1000);
    let elapsed = start.elapsed();
    println!("Found {} primes up to 1000 ({:?})", primes1000.len(), elapsed);

    println!("\nSieve up to 5000...");
    let start = Instant::now();
    let primes5000 = sieve(5000);
    let elapsed = start.elapsed();
    println!("Found {} primes up to 5000 ({:?})", primes5000.len(), elapsed);
}
