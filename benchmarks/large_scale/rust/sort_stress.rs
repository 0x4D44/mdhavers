// Sort Stress Test Benchmark (Rust)
use std::time::Instant;

struct Rng {
    seed: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { seed }
    }

    fn next(&mut self) -> u64 {
        self.seed = (self.seed.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
        self.seed % 10000
    }
}

fn make_random_list(n: usize, rng: &mut Rng) -> Vec<u64> {
    (0..n).map(|_| rng.next()).collect()
}

fn is_sorted(lst: &[u64]) -> bool {
    lst.windows(2).all(|w| w[0] <= w[1])
}

fn main() {
    println!("=== Sort Stress Test Benchmark (Rust) ===");

    println!("Correctness check:");
    let mut rng = Rng::new(12345);
    let small: Vec<u64> = (0..10).map(|_| rng.next()).collect();
    println!("  Unsorted: {:?}", small);
    let mut sorted_small = small.clone();
    sorted_small.sort();
    println!("  Sorted: {:?}", sorted_small);
    println!("  Is sorted: {}", is_sorted(&sorted_small));

    println!("\nScaling tests:");

    let mut rng = Rng::new(12345);
    let start = Instant::now();
    let mut lst = make_random_list(1000, &mut rng);
    let gen_time = start.elapsed();
    let start = Instant::now();
    lst.sort();
    let sort_time = start.elapsed();
    println!("  1K elements: gen={}us, sort={}us, valid={}",
             gen_time.as_micros(), sort_time.as_micros(), is_sorted(&lst));

    let mut rng = Rng::new(12345);
    let start = Instant::now();
    let mut lst = make_random_list(5000, &mut rng);
    let gen_time = start.elapsed();
    let start = Instant::now();
    lst.sort();
    let sort_time = start.elapsed();
    println!("  5K elements: gen={}us, sort={}us, valid={}",
             gen_time.as_micros(), sort_time.as_micros(), is_sorted(&lst));

    let mut rng = Rng::new(12345);
    let start = Instant::now();
    let mut lst = make_random_list(10000, &mut rng);
    let gen_time = start.elapsed();
    let start = Instant::now();
    lst.sort();
    let sort_time = start.elapsed();
    println!("  10K elements: gen={}us, sort={}us, valid={}",
             gen_time.as_micros(), sort_time.as_micros(), is_sorted(&lst));

    let mut rng = Rng::new(12345);
    let start = Instant::now();
    let mut lst = make_random_list(50000, &mut rng);
    let gen_time = start.elapsed();
    let start = Instant::now();
    lst.sort();
    let sort_time = start.elapsed();
    println!("  50K elements: gen={}us, sort={}ms, valid={}",
             gen_time.as_micros(), sort_time.as_millis(), is_sorted(&lst));

    let mut rng = Rng::new(12345);
    let start = Instant::now();
    let mut lst = make_random_list(100000, &mut rng);
    let gen_time = start.elapsed();
    let start = Instant::now();
    lst.sort();
    let sort_time = start.elapsed();
    println!("  100K elements: gen={}us, sort={}ms, valid={}",
             gen_time.as_micros(), sort_time.as_millis(), is_sorted(&lst));

    println!("\n=== Sort stress complete ===");
}
