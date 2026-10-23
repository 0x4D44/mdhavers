#![cfg(feature = "llvm")]
//! MDH-BUG-FLU-00078: native match arms wi' inclusive range patterns (`1..=3`).

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn run(source: &str) -> String {
    let program = parse(source).unwrap_or_else(|e| panic!("should parse: {e:?}"));
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("test_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .unwrap_or_else(|e| panic!("should compile: {e:?}"));
    let output = Command::new(&exe_path).output().expect("run executable");
    assert!(
        output.status.success(),
        "executable failed: {:?} {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn llvm_inclusive_range_pattern_includes_its_end() {
    let out = run(r#"
fer n in 0..8 {
    keek n {
        whan 1..=3 -> { blether "low" }
        whan 4..6 -> { blether "mid" }
        whan _ -> { blether "other" }
    }
}
"#);
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(
        lines,
        vec!["other", "low", "low", "low", "mid", "mid", "other", "other"]
    );
}
