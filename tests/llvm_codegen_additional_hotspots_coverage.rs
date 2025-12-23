#![cfg(feature = "llvm")]

use std::fs;

use mdhavers::{parse, LLVMCompiler};

fn compile_to_ir(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {e:?}"))?;
    LLVMCompiler::new()
        .compile_to_ir(&program)
        .map_err(|e| format!("Compile error: {e:?}"))
}

#[test]
fn llvm_codegen_exercises_additional_inline_and_condition_fast_paths() {
    let cases: &[&str] = &[
        // builtin sweep: keep a handful of cold builtins reachable for coverage.
        r#"
dae f() { gie 1 }
ken xs = [1]
ken d = {"a": 1}
blether whit_kind(naething)
blether whit_kind(aye)
blether whit_kind(1)
blether whit_kind(1.0)
blether whit_kind("hi")
blether whit_kind(xs)
blether whit_kind(d)
blether whit_kind(f)
"#,
        // compile_condition_direct: list index truthiness fast path.
        r#"
ken xs = [1]
gin xs[0] {
    blether 1
} ither {
    blether 0
}
"#,
        // compile_string_concat_fast: nested concat forces strlen on non-variable RHS.
        r#"
ken a = "a"
blether a + ("b" + "c")
"#,
        // string len bookkeeping: s = s + t (var + var) should compute new_len from shadows.
        r#"
ken s = "a"
ken t = "b"
s = s + t
blether s
"#,
        // predeclare_locals_for_capture: destructure bindings visible to nested functions.
        r#"
dae outer() {
    ken [a, ...rest] = [1, 2, 3]
    dae inner() { gie a + len(rest) }
    gie inner()
}
blether outer()
"#,
        // assert without message: hits the default message branch.
        r#"
hae_a_bash {
    mak_siccar 1 == 2
} gin_it_gangs_wrang e {
    blether "caught"
}
"#,
    ];

    for src in cases {
        compile_to_ir(src)
            .unwrap_or_else(|e| panic!("expected IR compile success for:\n{src}\n{e}"));
    }
}

#[test]
fn llvm_codegen_rejects_native_method_calls_over_8_args() {
    let src = r#"
ken x = 1
x.foo(1,2,3,4,5,6,7,8,9)
"#;
    let err = compile_to_ir(src).expect_err("expected IR compile to fail");
    assert!(
        err.contains("Native method call supports up to 8 arguments"),
        "unexpected error: {err}"
    );
}

#[test]
fn llvm_codegen_import_resolution_walks_ancestors_when_source_path_is_nested() {
    let dir = tempfile::tempdir().unwrap();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    fs::write(
        stdlib_dir.join("functional.braw"),
        r#"
ken a = 10
dae f() { gie 32 }
"#,
    )
    .unwrap();

    let nested_dir = dir.path().join("nested").join("deeper");
    fs::create_dir_all(&nested_dir).unwrap();
    let source_path = nested_dir.join("main.braw");
    fs::write(
        &source_path,
        r#"
fetch "lib/functional" tae m
blether m["a"]
blether m["f"]()
"#,
    )
    .unwrap();

    let source = fs::read_to_string(&source_path).unwrap();
    let program = parse(&source).unwrap();

    let output = dir.path().join("out.o");
    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_object_with_source(&program, &output, Some(&source_path))
        .unwrap_or_else(|e| panic!("expected object compile success: {e:?}"));

    assert!(output.exists(), "expected object file to be written");
}
