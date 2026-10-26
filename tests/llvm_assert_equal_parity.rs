#![cfg(feature = "llvm")]
//! MDH-BUG-FLU-00192: native `assert_equal` raises the interpreter's exact text,
//! "Assertion failed: expected <a> but got <b>", and returns aye when the values are equal.

use std::process::{Command, Stdio};

use mdhavers::{parse, Interpreter, LLVMCompiler};
use tempfile::tempdir;

fn run_interpreter(source: &str) -> String {
    let program = parse(source).expect("parse");
    let mut interp = Interpreter::new();
    interp.interpret(&program).expect("interpret");
    interp.get_output().join("\n")
}

fn run_native(source: &str) -> String {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("assert_equal_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string()
}

const SOURCE: &str = r#"
dae tryit(f) {
    hae_a_bash {
        blether f()
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether assert_equal(1, 1.0)
blether assert_equal([1, {"a": 2}], [1, {"a": 2}])
tryit(|| assert_equal(1, 2))
tryit(|| assert_equal("a", "b"))
tryit(|| assert_equal([1, "x"], [2]))
tryit(|| assert_equal({"k": 1}, {"k": 2}))
tryit(|| assert_equal(naething, 0))
tryit(|| assert_equal(aye, nae))
tryit(|| assert_equal(1.5, 2))
"#;

#[test]
fn native_assert_equal_matches_the_interpreter() {
    let expected = run_interpreter(SOURCE);
    assert!(
        expected.contains("Assertion failed: expected 1 but got 2"),
        "{expected}"
    );
    assert_eq!(run_native(SOURCE), expected);
}
