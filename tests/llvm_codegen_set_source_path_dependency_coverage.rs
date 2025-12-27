#![cfg(all(feature = "llvm", coverage))]

use inkwell::context::Context;

use mdhavers::llvm::codegen::CodeGen;

#[test]
fn llvm_codegen_set_source_path_is_covered_in_dependency_instance() {
    let context = Context::create();
    let mut codegen = CodeGen::new(&context, "coverage_set_source_path");

    let abs = std::env::current_dir()
        .expect("cwd")
        .join("does_not_need_to_exist_for_coverage_abs.braw");
    codegen.set_source_path(&abs);

    codegen.set_source_path(std::path::Path::new("does_not_need_to_exist_for_coverage_rel.braw"));
}

