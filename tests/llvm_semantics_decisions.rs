#![cfg(feature = "llvm")]
//! Native backend: Arthur's 2026-10-23 design decisions (see `wrk_docs/2026.10.23 - DEC - ...`).
//! F1 structural dict/creel equality, F2 empty dicts are falsy, F3 a fresh `fer` binding
//! per iteration, a block's value is its last expression (MDH-BUG-FLU-00064), functions
//! may use globals declared later (00072) and globals get mangled symbols (00088).

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    stdout: String,
    stderr: String,
    code: Option<i32>,
}

fn compile_and_run(source: &str) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("decisions_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        code: output.status.code(),
    }
}

/// Runs a program that must exit cleanly; returns its trimmed stdout lines.
fn run_ok(source: &str) -> Vec<String> {
    let run = compile_and_run(source);
    assert_eq!(
        run.code,
        Some(0),
        "program failed: stdout={:?} stderr={:?}",
        run.stdout,
        run.stderr
    );
    run.stdout.trim().lines().map(str::to_string).collect()
}

// ---------- F1: dict and creel equality is structural ----------

#[test]
fn dict_equality_is_structural() {
    let out = run_ok(
        r#"
blether {"a": 1} == {"a": 1}
blether {"a": 1, "b": 2} == {"b": 2, "a": 1}
blether {"a": 1} == {"a": 2}
blether {"a": 1} == {"b": 1}
blether {"a": 1} == {"a": 1, "b": 2}
blether {"a": 1} != {"a": 1}
blether {"a": [1, {"x": 2}]} == {"a": [1, {"x": 2}]}
blether [{"a": 1}] == [{"a": 1}]
blether {} == {}
blether {"a": 1} == [1]
"#,
    );
    assert_eq!(
        out,
        vec!["aye", "aye", "nae", "nae", "nae", "nae", "aye", "aye", "aye", "nae"]
    );
}

#[test]
fn cyclic_containers_compare_in_finite_time() {
    let out = run_ok(
        r#"
ken a = {"n": 1}
a["me"] = a
ken b = {"n": 1}
b["me"] = b
blether a == b
ken c = {"n": 2}
c["me"] = c
blether a == c
ken xs = [1]
shove(xs, xs)
ken ys = [1]
shove(ys, ys)
blether xs == ys
"#,
    );
    assert_eq!(out, vec!["aye", "nae", "aye"]);
}

#[test]
fn creel_equality_is_structural() {
    let out = run_ok(
        r#"
blether creel([1, 2]) == creel([2, 1])
blether creel([1, 2]) == creel([1, 2, 3])
blether creel([1, 2]) == creel([1, 3])
blether creel([]) == creel([])
blether creel(["a"]) != creel(["a"])
"#,
    );
    assert_eq!(out, vec!["aye", "nae", "nae", "aye", "nae"]);
}

// ---------- F2: an empty dict is falsy ----------

#[test]
fn empty_dict_is_falsy() {
    let out = run_ok(
        r#"
gin {} { blether "truthy" } ither { blether "falsy" }
gin {"a": 1} { blether "truthy" } ither { blether "falsy" }
ken d = {}
blether nae d
d["k"] = 1
blether nae d
blether {} or "fallback"
ken n = 0
whiles d {
    n = n + 1
    d = {}
}
blether n
"#,
    );
    assert_eq!(out, vec!["falsy", "truthy", "aye", "nae", "fallback", "1"]);
}

// ---------- F3: each fer iteration gets a fresh binding ----------

#[test]
fn closures_capture_each_iterations_loop_value() {
    let out = run_ok(
        r#"
ken fs = []
fer i in 0..3 { shove(fs, || i) }
blether gaun(fs, |f| f())
ken gs = []
fer x in [10, 20, 30] { shove(gs, || x) }
blether gaun(gs, |f| f())
ken cs = []
fer c in "ab" { shove(cs, || c) }
blether gaun(cs, |f| f())
dae mk() {
    ken hs = []
    fer j in 0..3 { shove(hs, || j * 2) }
    gie gaun(hs, |f| f())
}
blether mk()
"#,
    );
    assert_eq!(
        out,
        vec!["[0, 1, 2]", "[10, 20, 30]", "[a, b]", "[0, 2, 4]"]
    );
}

#[test]
fn loop_variable_captured_on_some_iterations_only() {
    let out = run_ok(
        r#"
ken fs = []
fer i in 0..4 {
    gin i % 2 == 0 {
        shove(fs, || i)
    }
    blether i
}
blether gaun(fs, |f| f())
"#,
    );
    assert_eq!(out, vec!["0", "1", "2", "3", "[0, 2]"]);
}

// ---------- MDH-BUG-FLU-00064: a block's value is its last expression ----------

#[test]
fn block_expression_value_is_its_last_expression() {
    let out = run_ok(
        r#"
ken a = {0 + {0 + 1}}
blether a
blether {
    ken t = 5
    t * 2
}
ken noval = { blether "side" }
blether noval
blether {}
"#,
    );
    assert_eq!(out, vec!["1", "10", "side", "naething", "{}"]);
}

#[test]
fn lambda_block_body_returns_its_last_expression() {
    let out = run_ok(
        r#"
ken add1 = |x| { x + 1 }
blether add1(4)
ken lam = |x| {
    ken y = x * 3
    y + 1
}
blether lam(2)
ken early = |x| {
    gin x > 0 { gie "pos" }
    "other"
}
blether early(1)
blether early(0)
blether 5 |> |v| { v * 10 }
"#,
    );
    assert_eq!(out, vec!["5", "7", "pos", "other", "50"]);
}

#[test]
fn gie_inside_a_block_expression_returns_from_the_function() {
    let out = run_ok(
        r#"
dae f() {
    ken v = { gie 7 }
    gie 99
}
blether f()
"#,
    );
    assert_eq!(out, vec!["7"]);
}

#[test]
fn gie_in_a_top_level_block_is_return_outside_function() {
    // Since MDH-BUG-FLU-00094 the parser rejects this before codegen, so no
    // backend (native included) ever sees a top-level block gie.
    let err = parse("blether { gie 7 }\n").expect_err("a top-level gie has nae function to leave");
    assert!(
        matches!(
            err,
            mdhavers::HaversError::ReturnOutsideFunction { line: 1 }
        ),
        "unexpected error: {err:?}"
    );
}

// ---------- MDH-BUG-FLU-00072: functions see globals declared later ----------

#[test]
fn function_uses_global_declared_later() {
    let out = run_ok(
        r#"
dae show() {
    gie limit + 1
}
dae bump() {
    counter = counter + 10
}
dae greet() {
    gie greeting + ", " + who
}
ken limit = 41
ken counter = 0
fer i in 0..3 {
    counter = counter + i
    bump()
}
blether show()
blether counter
ken greeting = "Hullo"
ken who = "pal"
blether greet()
"#,
    );
    assert_eq!(out, vec!["42", "33", "Hullo, pal"]);
}

#[test]
fn function_local_ken_does_not_touch_a_later_global() {
    let out = run_ok(
        r#"
dae f() {
    ken x = 2
    gie x
}
ken x = 1
blether f()
blether x
"#,
    );
    assert_eq!(out, vec!["2", "1"]);
}

#[test]
fn later_globals_never_shadow_nearer_bindings() {
    let out = run_ok(
        r#"
dae make() {
    dae get() { gie "inner" }
    gie get()
}
dae twice(x) {
    gie double(double(x))
}
blether len("abc")
ken get = "outer"
ken len = |s| 999
ken double = |n| n * 2
blether make()
blether get
blether len("abc")
blether twice(3)
"#,
    );
    assert_eq!(out, vec!["3", "inner", "outer", "999", "12"]);
}

#[test]
fn top_level_read_before_ken_is_still_undefined() {
    let program = parse("blether early\nken early = 1\n").expect("parse");
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("reading a global before its ken must not compile");
    assert!(
        err.to_string().contains("Undefined variable: early"),
        "unexpected error: {err}"
    );
}

// ---------- A global captured by a top-level lambda stays a plain global ----------

#[test]
fn functions_see_globals_that_top_level_lambdas_mutate() {
    let out = run_ok(
        r#"
dae early() {
    gie cnt
}
ken cnt = 1
dae show() {
    gie cnt * 10
}
ken inc = || { cnt = cnt + 1 }
inc()
inc()
blether show()
blether early()
blether cnt
ken names = []
ken add = |n| shove(names, n)
add("a")
dae count() {
    gie len(names)
}
blether count()
"#,
    );
    assert_eq!(out, vec!["30", "3", "3", "1"]);
}

// ---------- MDH-BUG-FLU-00088: global symbols are mangled ----------

#[test]
fn globals_named_like_c_symbols_work_and_are_mangled() {
    let source = r#"
ken main = 1
ken stdout = "out"
ken printf = 3
ken malloc = [1, 2]
dae f() {
    gie main + printf
}
blether main
blether stdout
blether f()
blether malloc
"#;
    let out = run_ok(source);
    assert_eq!(out, vec!["1", "out", "4", "[1, 2]"]);

    let program = parse(source).expect("parse");
    let ir = LLVMCompiler::new().compile_to_ir(&program).expect("ir");
    for name in ["main", "stdout", "printf", "malloc"] {
        assert!(
            ir.contains(&format!("@__mdh_global_{name} = ")),
            "global `{name}` is not mangled"
        );
        assert!(
            !ir.contains(&format!("@{name} = global")),
            "global `{name}` kept its bare symbol"
        );
    }
}
