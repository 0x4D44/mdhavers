// String Stress Test - Rust Implementation

use std::time::Instant;

fn main() {
    println!("=== String Stress Test (Rust) ===");
    
    // String concatenation
    println!("String concatenation:");
    let start1 = Instant::now();
    let mut s = String::new();
    for _ in 0..1000 {
        s.push('x');
    }
    let t1 = start1.elapsed();
    println!("  1000 single-char concats: {} us, len={}", t1.as_micros(), s.len());
    
    let start2 = Instant::now();
    let mut s2 = String::new();
    for _ in 0..100 {
        s2.push_str("hello world ");
    }
    let t2 = start2.elapsed();
    println!("  100 multi-char concats: {} us, len={}", t2.as_micros(), s2.len());
    
    // String operations
    println!("\nString operations:");
    let test_str = "The quick brown fox jumps over the lazy dog";
    
    let start3 = Instant::now();
    for _ in 0..1000 {
        let _ = test_str.to_uppercase();
    }
    let t3 = start3.elapsed();
    println!("  1000x upper(): {} us", t3.as_micros());
    
    let start4 = Instant::now();
    for _ in 0..1000 {
        let _ = test_str.to_lowercase();
    }
    let t4 = start4.elapsed();
    println!("  1000x lower(): {} us", t4.as_micros());
    
    // Split/join
    println!("\nSplit/Join operations:");
    let csv_line = "field1,field2,field3,field4,field5,field6,field7,field8,field9,field10";
    
    let start5 = Instant::now();
    for _ in 0..1000 {
        let _: Vec<&str> = csv_line.split(',').collect();
    }
    let t5 = start5.elapsed();
    println!("  1000x split(): {} us", t5.as_micros());
    
    let words = vec!["one", "two", "three", "four", "five"];
    let start6 = Instant::now();
    for _ in 0..1000 {
        let _ = words.join("-");
    }
    let t6 = start6.elapsed();
    println!("  1000x join(): {} us", t6.as_micros());
    
    // Contains
    println!("\nContains operations:");
    let long_str = "Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor";
    
    let start7 = Instant::now();
    for _ in 0..1000 {
        let _ = long_str.contains("tempor");
    }
    let t7 = start7.elapsed();
    println!("  1000x contains(): {} us", t7.as_micros());
    
    println!("\n=== String stress complete ===");
}
