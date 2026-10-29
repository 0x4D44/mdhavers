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
# Naming every variable in a function keeps them all globals (RC7 phase B
# makes a top-level name no function mentions a local of `main`).
dae keep_global() {
    gie [main, stdout, printf, malloc]
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

// ---------- MDH-BUG-FLU-00102: an indexed value in a condition uses full truthiness ----------

#[test]
fn indexed_condition_uses_full_truthiness() {
    let out = run_ok(
        r#"
dae t(c) { gin c { gie "aye" } gie "nae" }
ken d = {"l": [], "m": {}, "s": "", "z": 0, "n": naething, "b": nae, "f": [1], "g": {"k": 1}, "t": "x"}
gin d["l"] { blether "l aye" } ither { blether "l nae" }
gin d["m"] { blether "m aye" } ither { blether "m nae" }
gin d["s"] { blether "s aye" } ither { blether "s nae" }
gin d["z"] { blether "z aye" } ither { blether "z nae" }
gin d["n"] { blether "n aye" } ither { blether "n nae" }
gin d["b"] { blether "b aye" } ither { blether "b nae" }
gin d["f"] { blether "f aye" } ither { blether "f nae" }
gin d["g"] { blether "g aye" } ither { blether "g nae" }
gin d["t"] { blether "t aye" } ither { blether "t nae" }
ken xs = [[], {}, "", [0]]
gin xs[0] { blether "x0 aye" } ither { blether "x0 nae" }
gin xs[1] { blether "x1 aye" } ither { blether "x1 nae" }
gin xs[2] { blether "x2 aye" } ither { blether "x2 nae" }
gin xs[3] { blether "x3 aye" } ither { blether "x3 nae" }
ken nest = {"a": {"b": []}, "c": [[[]]]}
gin nest["a"]["b"] { blether "ab aye" } ither { blether "ab nae" }
gin nest["c"][0][0] { blether "c00 aye" } ither { blether "c00 nae" }
gin nest["c"][0] { blether "c0 aye" } ither { blether "c0 nae" }
gin d["l"] or d["s"] { blether "or aye" } ither { blether "or nae" }
gin d["f"] an d["m"] { blether "an aye" } ither { blether "an nae" }
dae pick(ys, i) {
    gin ys[i] { gie "aye" }
    gie "nae"
}
blether pick([[]], 0)
blether pick([[1]], 0)
ken q = [[1], [2], [], [3]]
ken i = 0
whiles q[i] { i = i + 1 }
blether i
blether t(d["l"])
"#,
    );
    assert_eq!(
        out,
        vec![
            "l nae", "m nae", "s nae", "z nae", "n nae", "b nae", "f aye", "g aye", "t aye",
            "x0 nae", "x1 nae", "x2 nae", "x3 aye", "ab nae", "c00 nae", "c0 aye", "or nae",
            "an nae", "nae", "aye", "2", "nae",
        ]
    );
}

// ---------- MDH-BUG-FLU-00103: a capture on one branch boxes for the whole function ----------

#[test]
fn capture_on_one_branch_is_sound_on_every_path() {
    let out = run_ok(
        r#"
dae f(v) {
    gin v == 0 {
        gie gaun(["a"], |key| v)
    }
    gie v
}
blether f(5)
blether f(0)
dae g(v) {
    ken w = v * 2
    gin v > 100 {
        gie gaun([1], |k| w + k)
    }
    gie w
}
blether g(3)
blether g(200)
dae h(v) {
    ken out = 0
    gin v == 0 {
        out = 1
    } ither {
        ken fs = gaun([1, 2], |k| k + v)
        out = fs[1]
    }
    gie out + v
}
blether h(0)
blether h(4)
dae k(v) {
    ken w = v + 1
    gin v == 0 {
        blether "zero"
    } ither {
        ken fs = gaun([1], |q| q + w)
        blether fs[0]
    }
    gie w
}
blether k(0)
blether k(5)
dae loopy(v) {
    ken total = 0
    ken i = 0
    whiles i < 3 {
        ken fs = gaun([1], |n| n + v)
        total = total + fs[0]
        i = i + 1
    }
    gie total + v
}
blether loopy(2)
ken lam = |v| {
    gin v == 0 { gie gaun([1], |n| v) }
    v + 1
}
blether lam(4)
blether lam(0)
kin Bx {
    dae init() {}
    dae pick(v) {
        gin v == 0 { gie gaun([1], |n| v) }
        gie v
    }
}
ken b = Bx()
blether b.pick(7)
blether b.pick(0)
"#,
    );
    assert_eq!(
        out,
        vec![
            "5", "[0]", "6", "[401]", "1", "10", "zero", "1", "7", "6", "11", "5", "[0]", "7",
            "[0]",
        ]
    );
}
