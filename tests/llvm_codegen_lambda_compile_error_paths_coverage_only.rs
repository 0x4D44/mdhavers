#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn expect_compile_error(src: &str) {
    let program = parse(src).expect("parse");
    let _ = LLVMCompiler::new().compile_to_ir(&program).unwrap_err();
}

#[test]
fn llvm_lambda_block_body_stmt_compile_error_is_propagated_for_coverage() {
    expect_compile_error(
        r#"
ken f = |x| {
    blether __undef
    gie x
}
"#,
    );
}

#[test]
fn llvm_lambda_expr_body_compile_error_is_propagated_for_coverage() {
    expect_compile_error("ken f = |x| __undef\n");
}
