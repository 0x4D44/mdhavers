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
fn test_grup_up_basic() {
    let out = run(r#"
ken xs = [1, 2, 3, 4]
ken g = grup_up(xs, |x| x % 2)
blether g
"#);
    assert_eq!(out.trim(), "{\"1\": [1, 3], \"0\": [2, 4]}");
}

#[test]
fn test_pairt_by_basic() {
    let out = run(r#"
ken xs = [1, 2, 3, 4]
ken p = pairt_by(xs, |x| x % 2 == 0)
blether p
"#);
    assert_eq!(out.trim(), "[[2, 4], [1, 3]]");
}
