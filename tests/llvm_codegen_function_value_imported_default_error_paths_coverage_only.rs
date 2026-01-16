#![cfg(all(feature = "llvm", coverage))]

use std::fs;

use mdhavers::{parse, LLVMCompiler};

#[test]
fn llvm_function_value_imported_default_compile_error_is_propagated_for_coverage() {
    let dir = tempfile::tempdir().expect("tempdir");

    fs::write(
        dir.path().join("a.braw"),
        r#"
dae foo(x = __undef) { gie x }
"#,
    )
    .expect("write module");

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "a"
ken f = foo
blether f()
"#,
    )
    .expect("write main");

    let program = parse(&fs::read_to_string(&main_path).expect("read main")).expect("parse main");
    let obj_path = dir.path().join("out.o");

    let _ = LLVMCompiler::new()
        .compile_to_object_with_source(&program, &obj_path, Some(&main_path))
        .unwrap_err();
}
