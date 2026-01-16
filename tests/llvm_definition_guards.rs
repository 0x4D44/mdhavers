#![cfg(feature = "llvm")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mdhavers::{parse, LLVMCompiler};

fn compile_to_exe(main_path: &Path, out_dir: &Path) -> PathBuf {
    let program = parse(&fs::read_to_string(main_path).unwrap()).unwrap();

    let exe_path = out_dir.join("out_exe");
    LLVMCompiler::new()
        .compile_to_native_with_source(&program, &exe_path, 0, Some(main_path))
        .unwrap();
    exe_path
}

fn run_exe(exe: &Path) -> std::process::Output {
    Command::new(exe).output().unwrap()
}

#[test]
fn llvm_errors_when_calling_function_before_definition() {
    let dir = tempfile::tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
blether f()
dae f() { gie 2 }
"#,
    )
    .unwrap();

    let exe = compile_to_exe(&main_path, dir.path());
    let out = run_exe(&exe);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("'f' hasnae been defined yet"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn llvm_errors_when_instantiating_class_before_definition() {
    let dir = tempfile::tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
blether Foo()
kin Foo { }
"#,
    )
    .unwrap();

    let exe = compile_to_exe(&main_path, dir.path());
    let out = run_exe(&exe);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("'Foo' hasnae been defined yet"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn llvm_errors_when_instantiating_struct_before_definition() {
    let dir = tempfile::tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
blether Point(1, 2)
thing Point { x, y }
"#,
    )
    .unwrap();

    let exe = compile_to_exe(&main_path, dir.path());
    let out = run_exe(&exe);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("'Point' hasnae been defined yet"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn llvm_errors_when_class_superclass_is_not_defined_yet() {
    let dir = tempfile::tempdir().unwrap();
    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
kin B fae A { }
kin A { }
blether B()
"#,
    )
    .unwrap();

    let exe = compile_to_exe(&main_path, dir.path());
    let out = run_exe(&exe);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("'A' hasnae been defined yet"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn llvm_errors_when_imported_module_uses_function_before_definition() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("mymod.braw"),
        r#"
blether f()
dae f() { gie 2 }
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "mymod"
"#,
    )
    .unwrap();

    let exe = compile_to_exe(&main_path, dir.path());
    let out = run_exe(&exe);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("'f' hasnae been defined yet"),
        "stderr:\n{stderr}"
    );
}
