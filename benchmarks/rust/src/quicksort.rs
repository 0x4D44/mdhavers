use std::time::Instant;

fn quicksort(arr: &mut [i64]) {
    if arr.len() <= 1 {
        return;
    }

    let pivot_idx = partition(arr);
    let (left, right) = arr.split_at_mut(pivot_idx);
    quicksort(left);
    quicksort(&mut right[1..]);
}

fn partition(arr: &mut [i64]) -> usize {
    let pivot = arr[arr.len() - 1];
    let mut i = 0;
    for j in 0..arr.len() - 1 {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, arr.len() - 1);
    i
}

fn is_sorted(arr: &[i64]) -> bool {
    arr.windows(2).all(|w| w[0] <= w[1])
}

fn make_test_array(size: usize) -> Vec<i64> {
    let mut arr = Vec::with_capacity(size);
    let mut seed: i64 = 12345;
    for _ in 0..size {
        seed = (seed.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
        arr.push(seed % 1000);
    }
    arr
}

pub fn run() {
    // Test correctness
    println!("Testing correctness...");
    let mut small = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
    println!("Original: {:?}", small);
    quicksort(&mut small);
    println!("Sorted: {:?}", small);
    println!("Is sorted: {}", is_sorted(&small));

    // Medium benchmark
    println!("\nSorting 100 elements...");
    let mut medium = make_test_array(100);
    let start = Instant::now();
    quicksort(&mut medium);
    let elapsed = start.elapsed();
    println!("Sorted 100 elements, is_sorted: {} ({:?})", is_sorted(&medium), elapsed);

    // Large benchmark
    println!("\nSorting 500 elements...");
    let mut large = make_test_array(500);
    let start = Instant::now();
    quicksort(&mut large);
    let elapsed = start.elapsed();
    println!("Sorted 500 elements, is_sorted: {} ({:?})", is_sorted(&large), elapsed);
}
