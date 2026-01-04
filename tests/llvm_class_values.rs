//! LLVM backend parity tests for classes as first-class values.

#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn compile_and_run(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("test_exe");

    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 0)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Executable failed (code {:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run(source: &str) -> String {
    compile_and_run(source).expect("program should compile and run")
}

#[test]
fn llvm_class_is_a_first_class_value_and_callable() {
    let out = run(
        r#"
kin Box {
    dae init(v) {
        masel.v = v
    }
    dae get() {
        gie masel.v
    }
}

ken C = Box
ken b = C(41)
blether b.get()
"#,
    );
    assert_eq!(out.trim(), "41");
}

#[test]
fn llvm_class_value_roundtrips_through_dict_and_is_callable() {
    let out = run(
        r#"
kin Box {
    dae init(v) { masel.v = v }
    dae get() { gie masel.v }
}

ken d = {"C": Box}
ken C = d.C
blether C(7).get()
"#,
    );
    assert_eq!(out.trim(), "7");
}

