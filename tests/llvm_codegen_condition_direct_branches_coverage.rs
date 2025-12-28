#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, LLVMCompiler};

fn compile_to_ir_ok(source: &str) {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let ir = LLVMCompiler::new()
        .compile_to_ir(&program)
        .unwrap_or_else(|e| panic!("compile failed for:\n{source}\nerr={e:?}"));
    assert!(!ir.is_empty());
}

fn compile_to_ir_err(source: &str) -> String {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error");
    format!("{err:?}")
}

#[test]
fn llvm_codegen_condition_direct_exercises_more_branches_for_coverage() {
    compile_to_ir_ok(
        r#"
ken nums = [1, 0]
ken idx = 1
ken flag = aye

gin flag { blether 1 }

// Bool comparisons should take the direct-compare fast path, but fall back to boxed extraction
// inside the int-data helper (compile_int_expr returns None for bools).
gin aye == nae { blether 1 }
gin aye < nae { blether 1 }

// Index-condition fast paths:
// - top-level list globals have no list_ptr shadow -> object fallback
// - top-level int globals have no int shadow -> index fallback
gin nums[idx] { blether 1 }

// Non-variable list object path.
gin [1, 0][1] { blether 1 }

// Index-expression fallback (non list/int index shape).
ken d = {"a": 1}
gin d["a"] { blether 1 }

// Logical short-circuit branches where sub-expressions can't be compiled directly.
gin (1 + 2) an (3 + 4) { blether 1 }
gin (1 + 2) or (3 + 4) { blether 1 }
"#,
    );
}

#[test]
fn llvm_codegen_condition_direct_error_paths_for_coverage() {
    // Index-expression fallback compile error path (object fails to compile).
    let err = compile_to_ir_err(
        r#"
gin missing_list[0] { blether 1 }
"#,
    );
    assert!(!err.is_empty());

    // Direct-compare error propagation inside get_int_data closure (left then right).
    let err = compile_to_ir_err(
        r#"
gin missing < 1 == aye { blether 1 }
"#,
    );
    assert!(!err.is_empty());
    let err = compile_to_ir_err(
        r#"
gin aye == missing < 1 { blether 1 }
"#,
    );
    assert!(!err.is_empty());

    // Direct-compare error propagation for ordering comparisons.
    let err = compile_to_ir_err(
        r#"
gin missing < 1 < aye { blether 1 }
"#,
    );
    assert!(!err.is_empty());
    let err = compile_to_ir_err(
        r#"
gin aye < (missing < 1) { blether 1 }
"#,
    );
    assert!(!err.is_empty());

    // Logical condition paths:
    // - compile_condition_direct(left/right)? error propagation
    // - compile_expr(left/right)? error propagation after fallback to full compilation
    let err = compile_to_ir_err(
        r#"
gin missing < 1 == aye an aye { blether 1 }
"#,
    );
    assert!(!err.is_empty());
    let err = compile_to_ir_err(
        r#"
gin missing_func() an aye { blether 1 }
"#,
    );
    assert!(!err.is_empty());
    let err = compile_to_ir_err(
        r#"
gin aye an missing < 1 == aye { blether 1 }
"#,
    );
    assert!(!err.is_empty());
    let err = compile_to_ir_err(
        r#"
gin aye an missing_func() { blether 1 }
"#,
    );
    assert!(!err.is_empty());
}
