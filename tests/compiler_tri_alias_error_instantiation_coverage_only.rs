#![cfg(coverage)]

use mdhavers::compiler::compile;

#[test]
fn compiler_tri_import_requires_alias_error_is_covered_in_dependency_instance() {
    let err = compile("fetch \"tri\"").expect_err("expected tri alias error");
    assert!(err.to_string().contains("requires an alias"));
}
