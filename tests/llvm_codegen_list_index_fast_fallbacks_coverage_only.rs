#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn expect_compile_error(src: &str) {
    let program = parse(src).expect("parse");
    let _ = LLVMCompiler::new().compile_to_ir(&program).unwrap_err();
}

#[test]
fn llvm_list_index_fast_fallback_paths_are_exercised_for_coverage() {
    let program = parse(
        r#"
ken xs = [1, 2, 3]
blether xs[0]
blether [1, 2, 3][(0)]
"#,
    )
    .expect("parse");

    LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect("compile");
}

#[test]
fn llvm_list_index_fast_object_compile_error_is_propagated_for_coverage() {
    expect_compile_error("blether [__undef][0]\n");
}
