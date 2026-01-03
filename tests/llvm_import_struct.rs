#![cfg(feature = "llvm")]

use std::fs;
use std::process::Command;

use mdhavers::{parse, LLVMCompiler};

#[test]
fn llvm_can_import_modules_that_define_structs() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(
        dir.path().join("mymod.braw"),
        r#"
thing Point { x, y }
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "mymod"
ken p = Point(10, 20)
blether p.x + p.y
"#,
    )
    .unwrap();

    let program = parse(&fs::read_to_string(&main_path).unwrap()).unwrap();

    let exe_path = dir.path().join("out_exe");
    LLVMCompiler::new()
        .compile_to_native_with_source(&program, &exe_path, 0, Some(&main_path))
        .unwrap();

    let output = Command::new(&exe_path).output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "30");
}

