//! MDH-BUG-FLU-00129: the benchmark programs print `tick()` deltas (nanoseconds)
//! labelled ms or us, and `fib_large` overflowed i64. Two cheap guards.

use mdhavers::{parse, Interpreter};
use std::path::{Path, PathBuf};

fn braw_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            braw_files(&path, out);
        } else if path.extension().map(|e| e == "braw").unwrap_or(false) {
            out.push(path);
        }
    }
}

/// `tick()` counts nanoseconds, so a delta labelled ms needs `/ 1000000` and one
/// labelled us needs `/ 1000` (a line using `mono_ms` is already in ms).
#[test]
fn benchmark_timing_labels_match_their_divisors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("benchmarks");
    let mut files = Vec::new();
    braw_files(&root, &mut files);
    assert!(files.len() > 10, "expected the benchmark programs");

    let mut bad = Vec::new();
    let mut labelled = 0;
    for file in &files {
        let text = std::fs::read_to_string(file).expect("read benchmark");
        for (n, line) in text.lines().enumerate() {
            let t = line.trim();
            if t.starts_with('#') || t.starts_with("//") || !t.contains("tae_string(") {
                continue;
            }
            let unit_ms = t.ends_with(" ms\"");
            let unit_us = t.ends_with(" us\"");
            if !(unit_ms || unit_us) || t.contains("mono_ms") {
                continue;
            }
            labelled += 1;
            let ok = if unit_ms {
                t.contains("/ 1000000)")
            } else {
                t.contains("/ 1000)")
            };
            if !ok {
                bad.push(format!("{}:{}: {t}", file.display(), n + 1));
            }
        }
    }
    assert!(labelled > 20, "found only {labelled} labelled timing lines");
    assert!(
        bad.is_empty(),
        "timing lines whose unit does not match their divisor (tick() is ns):\n{}",
        bad.join("\n")
    );
}

/// `fib_iter(100000)` from fib_large must stay inside i64 (a plain `a + b`
/// overflows at fib(93), natively and in the interpreter).
#[test]
fn fib_large_function_does_not_overflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("benchmarks/large_scale/fib_large.braw");
    let text = std::fs::read_to_string(path).expect("read fib_large");
    let (defs, _) = text
        .split_once("# Test correctness first")
        .expect("fib_large layout changed");
    let program = parse(&format!(
        "{defs}\nblether fib_iter(50)\nblether fib_iter(100000)\n"
    ))
    .expect("parse");
    let mut interp = Interpreter::new();
    if let Err(e) = interp.interpret(&program) {
        panic!("fib_iter(100000) failed: {e}");
    }
    let out = interp.get_output();
    assert_eq!(out[out.len() - 2], "12586269025");
}
