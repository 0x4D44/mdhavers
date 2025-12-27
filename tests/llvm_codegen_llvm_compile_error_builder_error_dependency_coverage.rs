#![cfg(all(feature = "llvm", coverage))]

use inkwell::context::Context;

use mdhavers::llvm::codegen::CodeGen;

#[test]
fn codegen_llvm_compile_error_builder_error_is_covered_in_dependency_instance() {
    let context = Context::create();
    let codegen = CodeGen::new(&context, "coverage_llvm_compile_error_builder_error");

    codegen
        .coverage_llvm_compile_error_builder_error()
        .expect("expected helper to exercise BuilderError -> CompileError mapping");
}
