#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn expect_compile_error(src: &str) {
    let program = parse(src).expect("parse");
    let _ = LLVMCompiler::new()
        .compile_to_ir(&program)
        .unwrap_err();
}

#[test]
fn llvm_fstring_expr_first_part_propagates_compile_error_for_coverage() {
    expect_compile_error("blether f\"{__undef}\"\n");
}

#[test]
fn llvm_fstring_expr_later_part_propagates_compile_error_for_coverage() {
    expect_compile_error("blether f\"ok {__undef}\"\n");
}

#[test]
fn llvm_pipe_left_compile_error_is_propagated_for_coverage() {
    expect_compile_error("blether __undef |> tae_int\n");
}

#[test]
fn llvm_pipe_lambda_body_compile_error_is_propagated_for_coverage() {
    expect_compile_error("blether 1 |> |x| __undef\n");
}

