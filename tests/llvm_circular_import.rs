#![cfg(feature = "llvm")]

use std::fs;

use mdhavers::{parse, HaversError, LLVMCompiler};

#[test]
fn llvm_compiler_detects_circular_imports_and_reports_the_chain() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.braw"), "fetch \"b\"\nken a = 1\n").unwrap();
    fs::write(dir.path().join("b.braw"), "fetch \"a\"\nken b = 2\n").unwrap();

    // Provide a source path so the compiler resolves imports relative to this temp directory.
    let main_path = dir.path().join("main.braw");
    fs::write(&main_path, "fetch \"a\"\n").unwrap();

    let program = parse("fetch \"a\"").unwrap();

    let compiler = LLVMCompiler::new();
    let out = dir.path().join("out.o");
    let err = compiler
        .compile_to_object_with_source(&program, &out, Some(&main_path))
        .unwrap_err();

    let HaversError::CircularImport { path } = err else {
        panic!("expected CircularImport, got: {err:?}");
    };
    assert!(path.contains("a.braw"), "chain should mention a.braw, got: {path}");
    assert!(path.contains("b.braw"), "chain should mention b.braw, got: {path}");
}

