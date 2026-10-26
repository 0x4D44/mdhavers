//! WASM backend: `tak` and `drap` take a string as a list o' its characters, like the CLI
//! interpreter (MDH-BUG-FLU-00170). The WASM host runs the interpreter's native builtins, which
//! want a list, so the host converts the string first.

#![cfg(feature = "wasm_runner")]

use mdhavers::wasm_compiler::compile_to_wat;
use mdhavers::wasm_runner::run_wat_captured;

fn run(source: &str) -> mdhavers::wasm_runner::WasmRunOutput {
    let wat = compile_to_wat(source).expect("compile_to_wat");
    run_wat_captured(&wat).expect("module should load")
}

#[test]
fn tak_and_drap_accept_strings() {
    let out = run(
        "blether tak(\"Hello\", 2)\nblether drap(\"Hello\", 3)\nblether drap(\"Hello\", 6)\nblether tak(\"\", 3)\nblether tak([1, 2, 3, 4], 2)\nblether drap([1, 2, 3, 4], 2)\nblether tak([1, 2], 5)\n",
    );
    assert_eq!(out.error, None);
    assert_eq!(
        out.stdout,
        "[H, e]\n[l, o]\n[]\n[]\n[1, 2]\n[3, 4]\n[1, 2]\n"
    );
}

#[test]
fn tak_and_drap_still_reject_other_types() {
    let out = run("blether tak(7, 1)\n");
    let err = out.error.expect("tak(7, 1) should fail");
    assert!(err.contains("tak() needs a list and an integer"), "{err}");
}
