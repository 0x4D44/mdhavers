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
    let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
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
    let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
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
    let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
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
        err.contains("Captured variable 'masel' not found in scope"),
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
