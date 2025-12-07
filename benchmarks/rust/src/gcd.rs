use std::time::Instant;

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        return a;
    }
    gcd(b, a % b)
}

fn gcd_iter(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

fn lcm(a: i64, b: i64) -> i64 {
    (a * b) / gcd(a, b)
}

fn gcd_list(arr: &[i64]) -> i64 {
    if arr.is_empty() {
        return 0;
    }
    if arr.len() == 1 {
        return arr[0];
    }
    let mut result = arr[0];
    for &x in arr.iter().skip(1) {
        result = gcd(result, x);
    }
    result
}

pub fn run() {
    // Test correctness
    println!("Testing correctness...");
    println!("gcd(48, 18) = {}", gcd(48, 18));
    println!("gcd(54, 24) = {}", gcd(54, 24));
    println!("gcd(17, 13) = {}", gcd(17, 13));
    println!("gcd(100, 35) = {}", gcd(100, 35));

    println!("\nTesting LCM...");
    println!("lcm(4, 6) = {}", lcm(4, 6));
    println!("lcm(21, 6) = {}", lcm(21, 6));

    // Stress test
    println!("\nComputing 1000 GCDs...");
    let start = Instant::now();
    let mut sum: i64 = 0;
    for i in 1..=1000 {
        let g = gcd_iter(i * 7, i * 5);
        sum += g;
    }
    let elapsed = start.elapsed();
    println!("Sum of 1000 GCDs: {} ({:?})", sum, elapsed);

    // Large numbers
    println!("\nTesting with large numbers...");
    println!("gcd(123456789, 987654321) = {}", gcd(123456789, 987654321));
    println!("gcd(1000000007, 998244353) = {}", gcd(1000000007, 998244353));

    // GCD of list
    println!("\nGCD of [48, 36, 24, 12]...");
    let numbers = vec![48, 36, 24, 12];
    println!("gcd_list = {}", gcd_list(&numbers));
}
