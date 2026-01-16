#![cfg(all(feature = "llvm", coverage))]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn compile_to_object_with_source_ok(source_path: &Path, source: &str) {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let out = source_path.with_extension("o");
    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_object_with_source(&program, &out, Some(source_path))
        .unwrap_or_else(|e| panic!("object compile failed for:\n{source}\nerr={e:?}"));
    assert!(out.exists(), "expected object output to exist: {out:?}");
}

fn compile_to_object_with_source_err(source_path: &Path, source: &str) -> String {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let out = source_path.with_extension("o");
    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_object_with_source(&program, &out, Some(source_path))
        .expect_err("expected object compile to fail");
    format!("{err:?}")
}

fn unique_module_name(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}_{}_{}", std::process::id(), nanos)
}

#[test]
fn llvm_codegen_import_alias_call_falls_through_when_export_is_not_a_function() {
    let dir = tempfile::tempdir().unwrap();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    fs::write(
        stdlib_dir.join("mod.braw"),
        r#"
ken a = 1
dae f() { gie 2 }
"#,
    )
    .unwrap();

    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = r#"
fetch "lib/mod" tae m
blether m.a()
"#;
    fs::write(&source_path, source).unwrap();

    compile_to_object_with_source_ok(&source_path, source);
}

#[test]
fn llvm_codegen_resolve_import_path_finds_module_next_to_test_exe() {
    let module = unique_module_name("cov_exe_mod");
    let exe_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let module_path = exe_dir.join(format!("{module}.braw"));

    fs::write(&module_path, "ken a = 1\n").unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _cleanup = Cleanup(module_path.clone());

    let dir = tempfile::tempdir().unwrap();
    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = format!(
        r#"
fetch "{module}" tae m
blether m["a"]
"#
    );
    fs::write(&source_path, &source).unwrap();

    compile_to_object_with_source_ok(&source_path, &source);
}

#[test]
fn llvm_codegen_resolve_import_path_finds_module_in_exe_stdlib_dir() {
    let module = unique_module_name("cov_exe_stdlib_mod");
    let exe_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let stdlib_dir = exe_dir.join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    let module_path = stdlib_dir.join(format!("{module}.braw"));

    fs::write(&module_path, "ken a = 1\n").unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _cleanup = Cleanup(module_path.clone());

    let dir = tempfile::tempdir().unwrap();
    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = format!(
        r#"
fetch "{module}" tae m
blether m["a"]
"#
    );
    fs::write(&source_path, &source).unwrap();

    compile_to_object_with_source_ok(&source_path, &source);
}

#[test]
fn llvm_codegen_resolve_import_path_supports_lib_stripped_next_to_exe_stdlib_dir() {
    let module = unique_module_name("cov_exe_lib_mod");
    let exe_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let stdlib_dir = exe_dir.join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    let module_path = stdlib_dir.join(format!("{module}.braw"));

    fs::write(&module_path, "ken a = 1\n").unwrap();
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _cleanup = Cleanup(module_path.clone());

    let dir = tempfile::tempdir().unwrap();
    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = format!(
        r#"
fetch "lib/{module}" tae m
blether m["a"]
"#
    );
    fs::write(&source_path, &source).unwrap();

    compile_to_object_with_source_ok(&source_path, &source);
}

#[test]
fn llvm_codegen_resolve_import_path_exe_search_runs_before_error_for_coverage() {
    let module = unique_module_name("cov_exe_missing_mod");

    let dir = tempfile::tempdir().unwrap();
    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = format!(
        r#"
fetch "lib/{module}" tae m
blether m
"#
    );
    fs::write(&source_path, &source).unwrap();

    let err = compile_to_object_with_source_err(&source_path, &source);
    assert!(
        err.contains("Cannot find module to import"),
        "unexpected error: {err}"
    );
}

#[test]
fn llvm_codegen_injects_masel_for_nested_functions_and_errors_when_called_without_masel_in_scope() {
    let err = compile_to_ir_err(
        r#"
ken x = 7

kin C {
    dae m(x) {
        dae outer() {
            blether masel
            dae inner2() {
                blether masel
                gie x
            }
            gie inner2()
        }
        gie 0
    }
}

outer()
"#,
    );
    assert!(
        err.contains("Undefined variable: outer"),
        "unexpected error: {err}"
    );
}

#[test]
fn llvm_codegen_list_index_fast_paths_cover_missing_shadow_and_non_int_expr_fallbacks() {
    compile_to_ir_ok(
        r#"
ken xs = [1, 2, 3]
ken i = 1

# idx type infers as Int, but `compile_int_expr` returns None for unary expressions.
# Also: `xs` is a top-level list, so it has no list pointer shadow.
blether xs[-i]
"#,
    );
}

#[test]
fn llvm_codegen_dict_index_set_updates_variable_binding_for_coverage() {
    compile_to_ir_ok(
        r#"
dae main() {
    ken d = {"a": 1}
    d["a"] = 2
    blether d["a"]
}
main()
"#,
    );
}

#[test]
fn llvm_codegen_list_index_set_fast_paths_cover_missing_shadow_and_non_int_expr_fallbacks() {
    compile_to_ir_ok(
        r#"
ken xs = [1, 2, 3]
ken i = 1

# Fast path picks list+int, but the unary index forces a compile_int_expr() fallback.
xs[-i] = 9
blether xs[2]
"#,
    );
}

#[test]
fn llvm_codegen_method_param_boxing_happens_when_captured_by_nested_function() {
    compile_to_ir_ok(
        r#"
kin C {
    dae init(x) {
        dae inner() { gie x }
        gie inner()
    }
}

blether C().init(1)
"#,
    );
}

#[test]
fn llvm_codegen_falls_back_to_prefixed_function_lookup_for_method_calls() {
    compile_to_ir_ok(
        r#"
kin C {
    dae init() { masel.v = 1 }
}

dae C_ext(it) { gie it.v }

blether C().ext()
"#,
    );
}

#[test]
fn llvm_codegen_prefixed_function_lookup_uses_best_match_when_arity_mismatches_for_coverage() {
    compile_to_ir_ok(
        r#"
kin C {
    dae init() { masel.v = 1 }
}

# Wrong arity: call is `ext()` but function is `C_ext(instance, extra)`.
dae C_ext(it, extra) { gie it }

blether C().ext()
"#,
    );
}

#[test]
fn llvm_codegen_method_value_dispatch_supports_multiple_candidate_classes_for_coverage() {
    compile_to_ir_ok(
        r#"
kin A {
    dae init() { }
    dae foo() { gie 1 }
}

kin B {
    dae init() { }
    dae foo() { gie 2 }
}

ken f = A().foo
blether f()
"#,
    );
}

#[test]
fn llvm_codegen_nested_function_capture_scan_runs_for_free_functions_for_coverage() {
    compile_to_ir_ok(
        r#"
dae outer(x) {
    dae inner() { gie x }
    gie inner()
}

blether outer(1)
"#,
    );
}

#[test]
fn llvm_codegen_nested_nested_function_inherits_masel_capture_for_coverage() {
    compile_to_ir_ok(
        r#"
kin C {
    dae init(v) { masel.v = v }
    dae m() {
        dae outer() {
            blether masel.v
            dae inner2() {
                blether masel.v
                gie 0
            }
            gie inner2()
        }
        gie outer()
    }
}

blether C(7).m()
"#,
    );
}

#[test]
fn llvm_codegen_nested_nested_function_captures_masel_from_default_param_for_coverage() {
    compile_to_ir_ok(
        r#"
kin C {
    dae init(v) { masel.v = v }
    dae m() {
        dae outer() {
            blether masel.v
            dae inner2(x = masel.v) { gie x }
            gie inner2()
        }
        gie outer()
    }
}

blether C(7).m()
"#,
    );
}

#[test]
fn llvm_codegen_nested_function_defaults_evaluate_in_closure_env_for_coverage() {
    compile_to_ir_ok(
        r#"
dae outer(x) {
    dae inner(y = x) { gie y }
    ken f = inner
    gie f()
}

blether outer(7)
"#,
    );
}

#[test]
fn llvm_codegen_nested_function_as_value_materializes_closure_captures_for_coverage() {
    compile_to_ir_ok(
        r#"
dae outer(x) {
    dae inner(y) { gie x + y }
    ken f = inner
    gie f(2)
}

blether outer(40)
"#,
    );
}

#[test]
fn llvm_codegen_test_globals_default_values_are_available_for_coverage() {
    compile_to_ir_ok(
        r#"
blether __current_suite()
blether _tick_counter()
blether _global_bus()
"#,
    );
}

#[test]
fn llvm_codegen_import_rebind_applies_cached_defaults_and_superclass_guards_for_coverage() {
    let dir = tempfile::tempdir().unwrap();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    fs::write(
        stdlib_dir.join("mod.braw"),
        r#"
dae with_default(x = 7) { gie x }
dae without_default(x) { gie x }

kin Base {
    dae init() { masel.x = 1 }
}

kin Sub fae Base {
    dae init() { masel.y = 2 }
}
"#,
    )
    .unwrap();

    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = r#"
fetch "lib/mod" tae m1
fetch "lib/mod" tae m2
blether m1.with_default()
blether m2.without_default(2)
"#;
    fs::write(&source_path, source).unwrap();

    compile_to_object_with_source_ok(&source_path, source);
}

#[test]
fn llvm_codegen_operator_overload_default_filling_is_exercised_for_coverage() {
    compile_to_ir_ok(
        r#"
kin NumBox {
    dae init(v) { masel.v = v }
    dae __pit_thegither__(that, extra = 40) { gie masel.v + that + extra }
}

ken a = NumBox(2)
blether a + 3
"#,
    );
}

#[test]
fn llvm_codegen_operator_overload_dispatch_builds_multi_class_chain_for_coverage() {
    compile_to_ir_ok(
        r#"
kin A {
    dae init(v) { masel.v = v }
    dae __pit_thegither__(that) { gie masel.v + that }
}

kin B {
    dae init(v) { masel.v = v }
    dae __pit_thegither__(that) { gie masel.v + that }
}

blether A(1) + 2
"#,
    );
}

#[test]
fn llvm_codegen_aliased_import_method_and_init_defaults_without_param_names_are_covered() {
    let dir = tempfile::tempdir().unwrap();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    fs::write(
        stdlib_dir.join("mod.braw"),
        r#"
kin Box {
    dae init(v, extra = 100) { masel.v = v + extra }
    dae add(x, y = 40) { gie masel.v + x + y }
}
"#,
    )
    .unwrap();

    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = r#"
fetch "lib/mod" tae m
ken a = m.Box(1)
blether a.add(2)
blether m.Box(1).add(2)
"#;
    fs::write(&source_path, source).unwrap();

    compile_to_object_with_source_ok(&source_path, source);
}

#[test]
fn llvm_codegen_aliased_import_default_filling_nil_branches_are_covered() {
    let dir = tempfile::tempdir().unwrap();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    fs::write(
        stdlib_dir.join("mod.braw"),
        r#"
kin Box {
    dae init(v, extra, more = 100) { masel.v = v }
    dae add(x, y, z = 40) { gie masel.v }
    dae nodflt(x, y) { gie masel.v }
}
"#,
    )
    .unwrap();

    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = r#"
fetch "lib/mod" tae m
ken a = m.Box(1)
blether a.add(2)
blether m.Box(1).add(2)
blether a.nodflt()
"#;
    fs::write(&source_path, source).unwrap();

    compile_to_object_with_source_ok(&source_path, source);
}

#[test]
fn llvm_codegen_imported_function_defaults_fill_nil_without_param_names_for_coverage() {
    let dir = tempfile::tempdir().unwrap();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    fs::write(
        stdlib_dir.join("mod.braw"),
        r#"
dae f(x, y = 7) { gie y }
"#,
    )
    .unwrap();

    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    let source = r#"
fetch "lib/mod"
blether f()
"#;
    fs::write(&source_path, source).unwrap();

    compile_to_object_with_source_ok(&source_path, source);
}

#[test]
fn llvm_codegen_struct_decl_declares_ctor_when_not_predeclared_for_coverage() {
    compile_to_ir_ok(
        r#"
dae main() {
    thing Point { x, y }
    ken p = Point(1, 2)
    blether p.x
}
main()
"#,
    );
}

#[test]
fn llvm_codegen_operator_overload_dispatch_covers_builtin_and_default_fill_paths_for_coverage() {
    compile_to_ir_ok(
        r#"
kin Ops {
    dae init(v) { masel.v = v }

    # Has defaults + a missing default slot to exercise nil filling in overload default handling.
    dae __pit_thegither__(that, mid, tail = 1) { gie masel.v }

    # No defaults to exercise the "fill remaining with nil" overload path.
    dae __tak_awa__(that, extra) { gie masel.v }

    dae __times__(that) { gie masel.v }
    dae __pairt__(that) { gie masel.v }
    dae __lave__(that) { gie masel.v }

    dae __same_as__(that) { gie masel.v }
    dae __differs_fae__(that) { gie masel.v }
    dae __wee_er__(that) { gie masel.v }
    dae __wee_er_or_same__(that) { gie masel.v }
    dae __muckle_er__(that) { gie masel.v }
    dae __muckle_er_or_same__(that) { gie masel.v }
}

# Static operator-overload path (tracked class type).
ken a = Ops(10)
blether a - 1
blether a * 2
blether a / 3
blether a % 2
blether a < 2
blether a <= 2
blether a > 2
blether a >= 2

# Dynamic operator-overload path (non-variable left operand).
blether Ops(10) + 1
blether Ops(10) - 1
blether Ops(10) * 2
blether Ops(10) / 3
blether Ops(10) % 2
blether Ops(10) == 2
blether Ops(10) != 2
blether Ops(10) < 2
blether Ops(10) <= 2
blether Ops(10) > 2
blether Ops(10) >= 2
"#,
    );
}
