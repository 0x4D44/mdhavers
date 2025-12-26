#![cfg(coverage)]

use inkwell::context::Context;

use mdhavers::error::HaversError;
use mdhavers::llvm::codegen::CodeGen;

#[test]
fn codegen_current_function_none_branch_is_exercised_for_coverage() {
    let context = Context::create();
    let codegen = CodeGen::new(&context, "coverage_current_function_none_branch");

    let err = codegen
        .coverage_current_function_none_branch()
        .expect_err("expected compile error due to missing current function");
    assert!(matches!(err, HaversError::CompileError(_)));
}

