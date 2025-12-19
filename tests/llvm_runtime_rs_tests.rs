//! Focused tests for the Rust-FFI runtime helpers (JSON + regex).

#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn compile_and_run(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("test_exe");

    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_native(&program, &exe_path, 2)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Executable failed with exit code: {:?}, stderr: {}",
            output.status.code(),
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run(source: &str) -> String {
    compile_and_run(source).expect("Should compile and run successfully")
}

#[test]
fn llvm_json_parse_unknown_escape_matches_interpreter_leniency() {
    // The interpreter's JSON parser treats unknown escapes like `\\q` as the literal char `q`.
    let out = run(r#"blether json_parse("\"\\q\"")"#);
    assert_eq!(out.trim(), "q");
}

#[test]
fn llvm_json_parse_error_is_catchable() {
    let out = run(r#"
hae_a_bash {
    json_parse("{")
    blether "unreachable"
} gin_it_gangs_wrang e {
    blether "caught"
}
"#);
    assert_eq!(out.trim(), "caught");
}

#[test]
fn llvm_regex_invalid_pattern_is_catchable() {
    let out = run(r#"
hae_a_bash {
    regex_test("hello", "*")
    blether "unreachable"
} gin_it_gangs_wrang e {
    blether "caught"
}
"#);
    assert_eq!(out.trim(), "caught");
}

#[test]
fn llvm_regex_replace_supports_capture_expansion() {
    // POSIX-ERE based implementations usually treat `$1` literally; interpreter uses the Rust regex crate.
    let out = run(r#"blether regex_replace("abc123def", "([0-9]+)", "[$1]")"#);
    assert_eq!(out.trim(), "abc[123]def");
}
