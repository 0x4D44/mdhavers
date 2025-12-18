use mdhavers::{parse, LLVMCompiler};
use std::process::Command;
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

fn main() {
    // Test 1: inner doesn't use outer's param
    let code = r#"
        dae outer(x) {
            dae inner(y) {
                gie y * 2
            }
            gie inner(10)
        }
        blether outer(5)
    "#;
    println!(
        "Test 1 (no capture): {}",
        compile_and_run(code)
            .unwrap_or_else(|e| format!("ERROR: {}", e))
            .trim()
    );

    // Test 2: inner uses outer's param
    let code = r#"
        dae outer(x) {
            dae inner(y) {
                gie x + y
            }
            gie inner(10)
        }
        blether outer(5)
    "#;
    println!(
        "Test 2 (with capture): {}",
        compile_and_run(code)
            .unwrap_or_else(|e| format!("ERROR: {}", e))
            .trim()
    );
}
