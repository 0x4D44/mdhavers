use std::fs;

use mdhavers::{parse, HaversError, Interpreter};

#[test]
fn interpreter_detects_circular_imports_and_reports_the_chain() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.braw"), "fetch \"b\"\nken a = 1\n").unwrap();
    fs::write(dir.path().join("b.braw"), "fetch \"a\"\nken b = 2\n").unwrap();

    let program = parse("fetch \"a\"").unwrap();
    let mut interp = Interpreter::new();
    interp.set_current_dir(dir.path());

    let err = interp.interpret(&program).unwrap_err();
    let HaversError::CircularImport { path } = err else {
        panic!("expected CircularImport, got: {err:?}");
    };
    assert!(path.contains("a.braw"), "chain should mention a.braw, got: {path}");
    assert!(path.contains("b.braw"), "chain should mention b.braw, got: {path}");
}

