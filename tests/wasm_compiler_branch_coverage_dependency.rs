#![cfg(coverage)]

use mdhavers::wasm_compiler::wasm_compiler_dependency_branch_matrix_for_coverage;

#[test]
fn wasm_compiler_dependency_branches_are_covered() {
    wasm_compiler_dependency_branch_matrix_for_coverage();
}
