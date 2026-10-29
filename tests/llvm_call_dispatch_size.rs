//! Calls through function values must not grow IR with (call sites x functions)
//! (MDH-BUG-FLU-00157: examples/stdlib/game_demo.braw emitted 640k lines of IR).

#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

const FUNCS: usize = 60;
const SITES: usize = 40;

fn source() -> String {
    let mut s = String::new();
    for i in 0..FUNCS {
        s.push_str(&format!("dae f{i}(x) {{\n    gie x + {i}\n}}\n"));
    }
    s.push_str("ken fns = [");
    s.push_str(
        &(0..FUNCS)
            .map(|i| format!("f{i}"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    s.push_str("]\nken total = 0\n");
    for k in 0..SITES {
        s.push_str(&format!("total = fns[{}](total)\n", k % FUNCS));
    }
    s.push_str("blether total\n");
    s
}

fn expected() -> i64 {
    (0..SITES).map(|k| (k % FUNCS) as i64).sum()
}

#[test]
fn function_value_calls_do_not_blow_up_ir() {
    let program = parse(&source()).expect("parse");
    // This size bound measures dispatch code without reference-counting instrumentation.
    let ir = LLVMCompiler::new()
        .with_refcounting(false)
        .compile_to_ir(&program)
        .expect("ir");
    let lines = ir.lines().count();
    // Measured: about 49k lines with an inline chain at every site, about 28k with the
    // shared dispatcher (most of the rest is per-statement code, not dispatch).
    assert!(lines < 36000, "IR is {lines} lines");
}

#[test]
fn function_value_calls_still_work_through_the_shared_dispatcher() {
    let program = parse(&source()).expect("parse");
    let dir = tempdir().expect("tempdir");
    let exe = dir.path().join("exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe, 0)
        .expect("compile");
    let out = Command::new(&exe).output().expect("run");
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        expected().to_string()
    );
}
