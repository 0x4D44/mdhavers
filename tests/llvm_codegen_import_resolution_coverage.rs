#![cfg(all(feature = "llvm", coverage))]

use std::fs;
use std::path::Path;

use mdhavers::{llvm::LLVMCompiler, parse};
use tempfile::tempdir;

fn compile_to_object_with_source_path(source: &str, source_path: &Path) -> Result<(), String> {
    let program = parse(source).map_err(|e| format!("Parse error: {e:?}"))?;
    let dir = tempdir().map_err(|e| format!("tempdir failed: {e}"))?;
    let obj_path = dir.path().join("out.o");

    LLVMCompiler::new()
        .compile_to_object_with_source(&program, &obj_path, Some(source_path))
        .map_err(|e| format!("Compile error: {e:?}"))
}

#[test]
fn llvm_codegen_import_resolution_hits_grandparent_path_branch() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Layout:
    //   root/
    //     stdlib/main.braw
    //     lib/bar.braw
    fs::create_dir_all(root.join("stdlib")).unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("lib").join("bar.braw"), "ken x = 1").unwrap();
    let main_path = root.join("stdlib").join("main.braw");
    fs::write(&main_path, "fetch \"lib/bar\"").unwrap();

    // Import uses grandparent search (stdlib/ -> root/).
    let src = r#"
fetch "lib/bar"
fetch "lib/bar"  # exercise already-imported short-circuit
blether 1
"#;
    compile_to_object_with_source_path(src, &main_path)
        .unwrap_or_else(|e| panic!("expected import compile to succeed, got: {e}"));
}

#[test]
fn llvm_codegen_import_read_failure_path_is_exercised() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("stdlib")).unwrap();
    let main_path = root.join("stdlib").join("main.braw");
    fs::write(&main_path, "fetch \"unreadable\"").unwrap();

    // Create a directory named `unreadable.braw` so path resolution succeeds but read_to_string fails.
    fs::create_dir_all(root.join("stdlib").join("unreadable.braw")).unwrap();

    let err = compile_to_object_with_source_path(r#"fetch "unreadable""#, &main_path)
        .expect_err("expected compile error when import is not readable");
    assert!(!err.is_empty());
}
