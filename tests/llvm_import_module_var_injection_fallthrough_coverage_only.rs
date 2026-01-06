#![cfg(all(feature = "llvm", coverage))]

use std::fs;

use mdhavers::{parse, LLVMCompiler};

#[test]
fn llvm_import_module_var_injection_skips_unreached_var_decls_for_coverage() {
    let dir = tempfile::Builder::new()
        .prefix("import_modvar_fallthrough")
        .tempdir_in("target")
        .expect("tempdir");

    fs::write(
        dir.path().join("m.braw"),
        r#"
ken a = 1
hurl "boom"
ken b = 2
"#,
    )
    .expect("write module");

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "m"
blether 1
"#,
    )
    .expect("write main");

    let program = parse(&fs::read_to_string(&main_path).expect("read main")).expect("parse main");
    let obj = dir.path().join("out.o");
    LLVMCompiler::new()
        .with_optimization(0)
        .compile_to_object_with_source(&program, &obj, Some(&main_path))
        .expect("compile");
    assert!(obj.exists());
}

