#![cfg(feature = "llvm")]
//! Native backend: a call to a known user function, method or constructor with the wrong
//! number of arguments is the interpreter's arity error, raised at compile time like a
//! builtin's (decision 148), not an LLVM verifier failure (MDH-BUG-FLU-00166).

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn compile_error(source: &str) -> String {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let err = LLVMCompiler::new()
        .compile_to_native(&program, &dir.path().join("arity_exe"), 2)
        .expect_err("a wrong argument count must not compile");
    format!("{err}")
}

fn run_ok(source: &str) -> Vec<String> {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("arity_ok_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn too_many_arguments_is_a_scots_compile_error() {
    let err = compile_error("dae f(a) { gie a }\nblether f(1, 2)\n");
    assert!(
        err.contains("Function 'f' expects 1 arguments but ye gave it 2 at line 2"),
        "unexpected error: {err}"
    );
    assert!(
        !err.contains("verification"),
        "verifier error leaked: {err}"
    );
}

#[test]
fn too_few_arguments_is_a_scots_compile_error() {
    let err = compile_error("dae f(a, b) { gie a }\nblether f(1)\n");
    assert!(
        err.contains("Function 'f' expects 2 arguments but ye gave it 1 at line 2"),
        "unexpected error: {err}"
    );
}

#[test]
fn defaults_set_the_range_of_allowed_counts() {
    let err = compile_error("dae g(a, b = 2) { gie a + b }\nblether g()\n");
    assert!(
        err.contains("Function 'g' expects 1 tae 2 arguments but ye gave it 0"),
        "unexpected error: {err}"
    );
    let err = compile_error("dae g(a, b = 2) { gie a + b }\nblether g(1, 2, 3)\n");
    assert!(
        err.contains("Function 'g' expects 1 tae 2 arguments but ye gave it 3"),
        "unexpected error: {err}"
    );
}

#[test]
fn nested_functions_methods_and_constructors_are_checked() {
    let err = compile_error(
        "dae outer() {\n    dae inner(p) { gie p }\n    gie inner()\n}\nblether outer()\n",
    );
    assert!(
        err.contains("Function 'inner' expects 1 arguments but ye gave it 0 at line 3"),
        "unexpected error: {err}"
    );
    let err = compile_error(
        "kin K {\n    dae add(x, y = 1) { gie x + y }\n}\nken k = K()\nblether k.add(1, 2, 3)\n",
    );
    assert!(
        err.contains("Function 'add' expects 1 tae 2 arguments but ye gave it 3"),
        "unexpected error: {err}"
    );
    let err = compile_error(
        "kin K {\n    dae init(n) {\n        masel.n = n\n    }\n}\nken k = K()\nblether k.n\n",
    );
    assert!(
        err.contains("Function 'init' expects 1 arguments but ye gave it 0 at line 6"),
        "unexpected error: {err}"
    );
}

#[test]
fn valid_calls_with_defaults_and_spreads_still_compile() {
    let out = run_ok(
        r#"
dae g(a, b = 2, c = 3) { gie a + b + c }
blether g(1)
blether g(1, 10)
blether g(1, 10, 100)
kin K {
    dae init(n, m = 5) {
        masel.n = n + m
    }
    dae add(x, y = 1) { gie masel.n + x + y }
}
ken k = K(1)
blether k.add(1)
blether k.add(1, 2)
blether K(1, 1).n
dae outer() {
    dae inner(p, q = "q") { gie p + q }
    gie inner("p") + inner("p", "r")
}
blether outer()
dae two(a, b) { gie a + b }
ken xs = [1, 2]
blether two(...xs)
"#,
    );
    assert_eq!(out, vec!["6", "14", "111", "8", "9", "2", "pqpr", "3"]);
}
