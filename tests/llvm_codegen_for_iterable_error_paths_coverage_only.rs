#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn expect_compile_error(src: &str) {
    let program = parse(src).expect("parse");
    let _ = LLVMCompiler::new().compile_to_ir(&program).unwrap_err();
}

#[test]
fn llvm_for_iterable_compile_error_is_propagated_for_coverage() {
    expect_compile_error(
        r#"
fer x in __undef {
  blether x
}
"#,
    );
}

#[test]
fn llvm_for_body_compile_error_is_propagated_for_coverage() {
    expect_compile_error(
        r#"
fer x in "hi" {
  blether __undef
}
"#,
    );
}
