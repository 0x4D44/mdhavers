#![cfg(all(feature = "llvm", coverage))]

use std::fs;

use mdhavers::{parse, LLVMCompiler};

#[test]
fn llvm_reimport_exercises_cached_import_rebind_for_functions_and_vars_for_coverage() {
    let dir = tempfile::tempdir().expect("tempdir");

    fs::write(
        dir.path().join("a.braw"),
        r#"
ken x = 1
dae f(a = 1) { gie a }
dae g() { gie 2 }
hurl "boom"
ken y = 2
"#,
    )
    .expect("write module");

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "a"
fetch "a"
"#,
    )
    .expect("write main");

    let program = parse(&fs::read_to_string(&main_path).expect("read main")).expect("parse main");
    let obj_path = dir.path().join("out.o");

    LLVMCompiler::new()
        .compile_to_object_with_source(&program, &obj_path, Some(&main_path))
        .expect("compile");
}
