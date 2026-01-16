//! LLVM backend parity tests for calling function values with default params.

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
fn llvm_calling_named_function_value_applies_defaults_like_interpreter() {
    let out = run(r#"
dae greet(name, greeting = "Hello") {
    gie greeting + " " + name
}

ken f = greet
blether f("World")
"#);
    assert_eq!(out.trim(), "Hello World");
}

#[test]
fn llvm_calling_captured_function_applies_defaults_with_correct_indexing() {
    let out = run(r#"
dae outer(x) {
    dae inner(a = x) {
        gie a
    }
    gie inner()
}

blether outer(5)
"#);
    assert_eq!(out.trim(), "5");
}

#[test]
fn llvm_calling_returned_closure_applies_captured_default() {
    let out = run(r#"
dae outer(x) {
    dae inner(a = x) { gie a }
    gie inner
}

ken f = outer(5)
blether f()
"#);
    assert_eq!(out.trim(), "5");
}

#[test]
fn llvm_bound_method_value_applies_defaults_using_masel() {
    let out = run(r#"
kin A {
    dae init(v) { masel.v = v }
    dae get(a = masel.v) { gie a }
}

ken f = A(5).get
blether f()
"#);
    assert_eq!(out.trim(), "5");
}

#[test]
fn llvm_direct_method_call_applies_defaults_using_masel() {
    let out = run(r#"
kin A {
    dae init(v) { masel.v = v }
    dae get(a = masel.v) { gie a }
}

blether A(5).get()
"#);
    assert_eq!(out.trim(), "5");
}

#[test]
fn llvm_returned_closure_from_method_captures_masel_for_default() {
    let out = run(r#"
kin A {
    dae init(v) { masel.v = v }
    dae maker() {
        dae inner(a = masel.v) { gie a }
        gie inner
    }
}

ken f = A(7).maker()
blether f()
"#);
    assert_eq!(out.trim(), "7");
}

#[test]
fn llvm_defaults_can_reference_prior_param_in_direct_call() {
    let out = run(r#"
dae f(a, b = a) { gie b }
blether f(3)
"#);
    assert_eq!(out.trim(), "3");
}

#[test]
fn llvm_defaults_can_reference_prior_param_in_function_value_call() {
    let out = run(r#"
dae f(a, b = a) { gie b }
ken g = f
blether g(3)
"#);
    assert_eq!(out.trim(), "3");
}

#[test]
fn llvm_defaults_can_reference_prior_param_in_returned_closure_value_call() {
    let out = run(r#"
dae outer(x) {
    dae inner(a, b = a, c = x) { gie b }
    gie inner
}

ken f = outer(9)
blether f(3)
"#);
    assert_eq!(out.trim(), "3");
}
