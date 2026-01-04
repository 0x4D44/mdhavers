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
fn llvm_import_executes_module_top_level_statements_once() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("mymod.braw"),
        r#"
blether "loaded"

dae one() {
    gie 1
}
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "mymod" tae m
blether m.one()

fetch "mymod" tae m2
blether m2.one()
"#,
    )
    .unwrap();

    assert_eq!(compile_and_run(dir.path(), &main_path), "loaded\n1\n1");
}

