#![cfg(feature = "llvm")]

use std::fs;
use std::process::Command;

use mdhavers::{parse, LLVMCompiler};

fn compile_and_run(dir: &std::path::Path, main_path: &std::path::Path) -> String {
    let program = parse(&fs::read_to_string(main_path).unwrap()).unwrap();

    let exe_path = dir.join("out_exe");
    LLVMCompiler::new()
        .compile_to_native_with_source(&program, &exe_path, 0, Some(main_path))
        .unwrap();

    let output = Command::new(&exe_path).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn llvm_imported_class_is_callable_via_alias_and_methods_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("mymod.braw"),
        r#"
kin Box {
    dae init(v) {
        masel.v = v
    }
    dae get() {
        gie masel.v
    }
}
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "mymod" tae m
ken b = m.Box(41)
blether b.get()
"#,
    )
    .unwrap();

    assert_eq!(compile_and_run(dir.path(), &main_path), "41");
}

