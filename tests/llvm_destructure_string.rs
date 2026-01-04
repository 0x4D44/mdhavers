#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn compile_and_run(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("test_exe");

    LLVMCompiler::new()
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
fn llvm_destructure_string_splits_into_char_strings() {
    let out = run(
        r#"
ken [a, b, c] = "abc"
blether a
blether b
blether c
"#,
    );
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines, vec!["a", "b", "c"]);
}

#[test]
fn llvm_destructure_string_supports_rest_pattern() {
    let out = run(
        r#"
ken [first, ...mid, last] = "hełło"
blether first
blether len(mid)
blether mid[0]
blether last
"#,
    );
    let lines: Vec<&str> = out.trim().lines().collect();
    assert_eq!(lines[0], "h");
    assert_eq!(lines[1], "3");
    assert_eq!(lines[2], "e");
    assert_eq!(lines[3], "o");
}

#[test]
fn llvm_destructure_non_list_non_string_throws_catchable_error() {
    let out = run(
        r#"
hae_a_bash {
    ken [a] = 1
    blether "unreachable"
} gin_it_gangs_wrang e {
    blether "caught"
}
"#,
    );
    assert_eq!(out.trim(), "caught");
}

#[test]
fn llvm_destructure_too_few_elements_throws_catchable_error() {
    let out = run(
        r#"
hae_a_bash {
    ken [a, b] = "a"
    blether "unreachable"
} gin_it_gangs_wrang e {
    blether "caught"
}
"#,
    );
    assert_eq!(out.trim(), "caught");
}

