use std::time::Instant;

fn mergesort(arr: &[i64]) -> Vec<i64> {
    if arr.len() <= 1 {
        return arr.to_vec();
    }

    let mid = arr.len() / 2;
    let left = mergesort(&arr[..mid]);
    let right = mergesort(&arr[mid..]);

    merge(&left, &right)
}

fn merge(left: &[i64], right: &[i64]) -> Vec<i64> {
    let mut result = Vec::with_capacity(left.len() + right.len());
    let (mut i, mut j) = (0, 0);

    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            result.push(left[i]);
            i += 1;
        } else {
            result.push(right[j]);
            j += 1;
        }
    }

    result.extend_from_slice(&left[i..]);
    result.extend_from_slice(&right[j..]);
    result
}

fn is_sorted(arr: &[i64]) -> bool {
    arr.windows(2).all(|w| w[0] <= w[1])
}

fn make_array(size: usize) -> Vec<i64> {
    let mut arr = Vec::with_capacity(size);
    let mut seed: i64 = 42;
    for _ in 0..size {
        seed = (seed.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
        arr.push(seed % 1000);
    }
    arr
}

pub fn run() {
    // Test correctness
    println!("Testing correctness...");
    let test = vec![38, 27, 43, 3, 9, 82, 10];
    println!("Original: {:?}", test);
    let sorted = mergesort(&test);
    println!("Sorted: {:?}", sorted);

    // Small benchmark
    println!("\nSorting 50 elements...");
    let arr50 = make_array(50);
    let start = Instant::now();
    let sorted50 = mergesort(&arr50);
    let elapsed = start.elapsed();
    println!("Sorted 50 elements, is_sorted: {} ({:?})", is_sorted(&sorted50), elapsed);

    // Medium benchmark
    println!("\nSorting 200 elements...");
    let arr200 = make_array(200);
    let start = Instant::now();
    let sorted200 = mergesort(&arr200);
    let elapsed = start.elapsed();
    println!("Sorted 200 elements, is_sorted: {} ({:?})", is_sorted(&sorted200), elapsed);
}
