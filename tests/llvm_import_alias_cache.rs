#![cfg(feature = "llvm")]

use std::fs;
use std::process::Command;

use mdhavers::{parse, LLVMCompiler};

#[test]
fn llvm_importing_the_same_module_twice_with_alias_remains_callable() {
    let dir = tempfile::tempdir().unwrap();

    fs::write(
        dir.path().join("mymod.braw"),
        r#"
ken a = 10
dae f() { gie 32 }
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "mymod" tae m
fetch "mymod" tae m2

blether m["a"]
blether m2["a"]
blether m.f()
blether m2.f()

fetch "mymod"
blether a
blether f()
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
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "10\n10\n32\n32\n10\n32"
    );
}
