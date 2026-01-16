#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn expect_compile_error(src: &str) {
    let program = parse(src).expect("parse");
    let _ = LLVMCompiler::new().compile_to_ir(&program).unwrap_err();
}

#[test]
fn llvm_while_condition_compile_error_is_propagated_for_coverage() {
    expect_compile_error(
        r#"
whiles __undef { brak }
"#,
    );
}

#[test]
fn llvm_while_body_compile_error_is_propagated_for_coverage() {
    expect_compile_error(
        r#"
whiles aye { blether __undef }
"#,
    );
}
