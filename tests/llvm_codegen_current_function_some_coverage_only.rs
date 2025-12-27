#![cfg(all(feature = "llvm", coverage))]

use inkwell::context::Context;

use mdhavers::llvm::codegen::CodeGen;

#[test]
fn llvm_codegen_current_function_some_branch_is_covered_in_dependency_instance() {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "coverage_current_function_some_branch");
    codegen.coverage_current_function_some_branch();
}
