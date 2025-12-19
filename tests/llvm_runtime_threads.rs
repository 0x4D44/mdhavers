//! Threading smoke test for LLVM runtime.

#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn compile_and_run(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("thread_test_exe");

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

#[test]
fn llvm_thread_spawn_and_atomic() {
    let out = compile_and_run(
        r#"
dae worker(a) {
    atomic_add(a, 1)
    gie naething
}

ken a = atomic_new(0)
ken t = thread_spawn(worker, [a])
thread_join(t)
blether atomic_load(a)
"#,
    )
    .expect("compile/run failed");
    assert_eq!(out.trim(), "1");
}
