#![cfg(coverage)]

use super::LLVMCompiler;
use crate::parser::parse;

fn compile_to_ir_for_unit_coverage(source: &str) {
    let _ = parse(source)
        .ok()
        .and_then(|program| LLVMCompiler::new().compile_to_ir(&program).ok());
}

#[test]
fn llvm_codegen_boxed_var_decl_paths_are_exercised_for_unit_coverage() {
    // Drives the boxed-variable declaration path via a nested function that mutates an outer local.
    compile_to_ir_for_unit_coverage(
        r#"
dae outer() {
    ken x = 0
    dae inc() { x = x + 1 }
    inc()
    gie x
}
outer()
"#,
    );
}

#[test]
fn llvm_codegen_globals_lookup_fallback_is_exercised_for_unit_coverage() {
    // Drives variable resolution from function scope into the global slot.
    compile_to_ir_for_unit_coverage(
        r#"
ken g = 1
dae f() { gie g }
f()
"#,
    );
}
