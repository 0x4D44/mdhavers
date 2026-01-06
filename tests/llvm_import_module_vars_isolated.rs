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
fn llvm_aliased_imports_keep_module_level_vars_separate() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("a.braw"),
        r#"
ken x = 1

dae get() { gie x }

dae set(v) { x = v }
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("b.braw"),
        r#"
ken x = 2

dae get() { gie x }

dae set(v) { x = v }
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "a" tae a
fetch "b" tae b

blether a.get()
blether b.get()

a.set(10)
blether a.get()
blether b.get()

b.set(20)
blether a.get()
blether b.get()
"#,
    )
    .unwrap();

    assert_eq!(compile_and_run(dir.path(), &main_path), "1\n2\n10\n2\n10\n20");
}

#[test]
fn llvm_reimporting_a_module_with_vars_under_alias_uses_the_cached_module_vars() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("a.braw"),
        r#"
ken x = 1
"#,
    )
    .unwrap();
    fs::write(
        dir.path().join("b.braw"),
        r#"
ken x = 2
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "a" tae a
fetch "b" tae b
blether b["x"]
fetch "b" tae b2
blether b2["x"]
"#,
    )
    .unwrap();

    assert_eq!(compile_and_run(dir.path(), &main_path), "2\n2");
}

#[test]
fn llvm_unaliased_import_injects_module_level_vars_into_scope() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("a.braw"),
        r#"
ken x = 41

dae bump() { x = x + 1 }
dae get() { gie x }
"#,
    )
    .unwrap();

    let main_path = dir.path().join("main.braw");
    fs::write(
        &main_path,
        r#"
fetch "a"

blether x
bump()
blether get()
"#,
    )
    .unwrap();

    assert_eq!(compile_and_run(dir.path(), &main_path), "41\n42");
}
