#![cfg(all(feature = "llvm", coverage))]

use std::fs;

use mdhavers::{HaversError, LLVMCompiler};

const UNDEFINED_VAR: &str = "__mdh_coverage_undefined_default__";

fn compile_to_ir_err(source: &str) -> HaversError {
    let program = mdhavers::parser::parse(source).expect("parse source");
    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_ir(&program)
        .expect_err("expected compile error")
}

fn assert_compile_error_contains(err: HaversError, needle: &str) {
    match err {
        HaversError::CompileError(message) => assert!(
            message.contains(needle),
            "unexpected error message: {message}"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn llvm_codegen_covers_method_default_error_in_dyn_dispatch_when_param_names_present_for_coverage() {
    let err = compile_to_ir_err(&format!(
        r#"
kin C {{
    dae m(x = {UNDEFINED_VAR}) {{ }}
}}

C().m()
"#
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}

#[test]
fn llvm_codegen_covers_method_default_error_in_direct_call_when_param_names_present_for_coverage() {
    let err = compile_to_ir_err(&format!(
        r#"
kin C {{
    dae m(x = {UNDEFINED_VAR}) {{ }}
}}

ken x = C()
x.m()
"#
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}

#[test]
fn llvm_codegen_covers_init_default_error_when_param_names_present_for_coverage() {
    let err = compile_to_ir_err(&format!(
        r#"
kin C {{
    dae init(x = {UNDEFINED_VAR}) {{ }}
}}

C()
"#
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}

#[test]
fn llvm_codegen_covers_method_default_error_in_dyn_dispatch_when_param_names_missing_for_coverage() {
    let dir = tempfile::Builder::new()
        .prefix("method_defaults_no_param_names_dyn")
        .tempdir_in("target")
        .expect("tempdir");

    let module_path = dir.path().join("m.braw");
    fs::write(
        &module_path,
        format!(
            r#"
kin C {{
    dae m(x = {UNDEFINED_VAR}) {{ }}
}}
"#
        ),
    )
    .expect("write module");

    let err = compile_to_ir_err(&format!(
        r#"
fetch "{}" tae mod

mod.C().m()
"#,
        module_path.to_string_lossy()
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}

#[test]
fn llvm_codegen_covers_method_default_error_in_direct_call_when_param_names_missing_for_coverage() {
    let dir = tempfile::Builder::new()
        .prefix("method_defaults_no_param_names_direct")
        .tempdir_in("target")
        .expect("tempdir");

    let module_path = dir.path().join("m.braw");
    fs::write(
        &module_path,
        format!(
            r#"
kin C {{
    dae m(x = {UNDEFINED_VAR}) {{ }}
}}
"#
        ),
    )
    .expect("write module");

    let err = compile_to_ir_err(&format!(
        r#"
fetch "{}" tae mod

ken x = mod.C()
x.m()
"#,
        module_path.to_string_lossy()
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}

#[test]
fn llvm_codegen_covers_field_call_fallback_propagates_call_function_value_error_for_coverage() {
    let err = compile_to_ir_err(&format!(
        r#"
dae f(x = {UNDEFINED_VAR}) {{ }}

kin A {{
    dae callback() {{ }}
}}

kin B {{
    dae init() {{
        masel.callback = f
    }}
}}

B().callback()
"#
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}

#[test]
fn llvm_codegen_covers_init_default_error_when_param_names_missing_for_coverage() {
    let dir = tempfile::Builder::new()
        .prefix("init_defaults_no_param_names")
        .tempdir_in("target")
        .expect("tempdir");

    let module_path = dir.path().join("m.braw");
    fs::write(
        &module_path,
        format!(
            r#"
kin C {{
    dae init(x = {UNDEFINED_VAR}) {{ }}
}}
"#
        ),
    )
    .expect("write module");

    let err = compile_to_ir_err(&format!(
        r#"
fetch "{}" tae mod

mod.C()
"#,
        module_path.to_string_lossy()
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}

#[test]
fn llvm_codegen_covers_method_dict_path_call_function_value_error_for_coverage() {
    let err = compile_to_ir_err(&format!(
        r#"
dae f(x = {UNDEFINED_VAR}) {{ }}

kin C {{
    dae m() {{ }}
}}

ken x = C()
x.m()
"#
    ));

    assert_compile_error_contains(err, UNDEFINED_VAR);
}
