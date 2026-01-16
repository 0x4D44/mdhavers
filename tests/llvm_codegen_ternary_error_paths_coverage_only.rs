#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn expect_compile_error(src: &str) {
    let program = parse(src).expect("parse");
    let _ = LLVMCompiler::new().compile_to_ir(&program).unwrap_err();
}

#[test]
fn llvm_ternary_condition_compile_error_is_propagated_for_coverage() {
    expect_compile_error("blether gin __undef than 1 ither 2\n");
}

#[test]
fn llvm_ternary_then_compile_error_is_propagated_for_coverage() {
    expect_compile_error("blether gin aye than __undef ither 0\n");
}

#[test]
fn llvm_ternary_else_compile_error_is_propagated_for_coverage() {
    expect_compile_error("blether gin aye than 1 ither __undef\n");
}
