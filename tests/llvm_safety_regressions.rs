#![cfg(feature = "llvm")]
//! Native-backend safety regressions: shadow caches (MDH-BUG-FLU-00048), try/catch
//! stack discipline (00049), checked integer arithmetic (00050), list growth (00051),
//! checked string-repeat sizes (00052) and long `speir` lines (00055).

use std::io::Write;
use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    stdout: String,
    stderr: String,
    code: Option<i32>,
}

fn compile_and_run_with_input(source: &str, input: &str) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("safety_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");

    let mut child = Command::new(&exe_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run executable");
    {
        let mut stdin = child.stdin.take().expect("stdin");
        stdin.write_all(input.as_bytes()).expect("write stdin");
    }
    let output = child.wait_with_output().expect("wait for executable");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        code: output.status.code(),
    }
}

fn compile_and_run(source: &str) -> Run {
    compile_and_run_with_input(source, "")
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

// ---------- MDH-BUG-FLU-00048: shadow caches follow reassignment ----------

#[test]
fn list_shadow_follows_reassignment() {
    let out = run_ok(
        r#"
dae f() {
    ken xs = [1, 2, 3]
    xs = [9]
    blether xs[0]
    shove(xs, 4)
    blether xs
}
f()
"#,
    );
    assert_eq!(out, vec!["9", "[9, 4]"]);
}

#[test]
fn int_variable_retyped_to_string_is_not_read_as_int() {
    let out = run_ok(
        r#"
ken x = 1
x = "hello"
blether x + 1
dae f() {
    ken y = 1
    y = "hullo"
    blether y + 2
}
f()
"#,
    );
    assert_eq!(out, vec!["hello1", "hullo2"]);
}

#[test]
fn string_variable_retyped_to_int_is_not_read_as_string() {
    let out = run_ok(
        r#"
ken s = "abc"
s = 42
blether s
dae f() {
    ken t = "abc"
    t = 7
    blether t
}
f()
"#,
    );
    assert_eq!(out, vec!["42", "7"]);
}

#[test]
fn string_append_after_literal_reassignment_copies_the_literal() {
    let out = run_ok(
        r#"
ken s = ""
s = s + "a"
s = "hello"
s = s + "!"
blether s
dae f() {
    ken t = ""
    t = t + "a"
    t = "hullo"
    t = t + "?"
    blether t
}
f()
"#,
    );
    assert_eq!(out, vec!["hello!", "hullo?"]);
}

#[test]
fn string_append_after_small_runtime_string_does_not_overflow() {
    let out = run_ok(
        r#"
ken s = ""
s = s + "a"
s = tae_string(7)
s = s + "xxxxxxxxxxxxxxxxxxxxxxxx"
blether s
"#,
    );
    assert_eq!(out, vec!["7xxxxxxxxxxxxxxxxxxxxxxxx"]);
}

#[test]
fn string_append_does_not_mutate_an_alias() {
    let out = run_ok(
        r#"
ken s = ""
s = s + "a"
ken t = s
s = s + "b"
blether t
blether s
fer i in 0..100 {
    s = s + "zzzzzzzz"
}
blether len(t)
ken xs = [s]
s = s + "q"
blether len(xs[0])
blether len(s)
"#,
    );
    assert_eq!(out, vec!["a", "ab", "1", "802", "803"]);
}

#[test]
fn parameter_assignment_keeps_the_assigned_value() {
    let out = run_ok(
        r#"
dae f(a, b) {
    a = b
    blether a
}
f(1, "hi")
dae g(xs) {
    ken i = 0
    whiles i < 2 {
        i = i + 1
    }
    blether xs
}
g([1, 2])
"#,
    );
    assert_eq!(out, vec!["hi", "[1, 2]"]);
}

#[test]
fn string_parameter_survives_a_loop_in_the_function() {
    let out = run_ok(
        r#"
dae f(t) {
    ken i = 0
    whiles i < 2 { i = i + 1 }
    gie t
}
blether f("str")
"#,
    );
    assert_eq!(out, vec!["str"]);
}

#[test]
fn list_reassigned_inside_gin_is_indexed_fresh() {
    let out = run_ok(
        r#"
ken size = [96, 54]
gin size != naething { size = [640, 360] }
blether size[0]
dae g() {
    ken sz = [96, 54]
    gin sz != naething { sz = [640, 360] }
    blether sz[0]
}
g()
"#,
    );
    assert_eq!(out, vec!["640", "640"]);
}

#[test]
fn globals_stay_in_step_with_functions() {
    let out = run_ok(
        r#"
ken n = 0
dae inc() {
    n = n + 1
}
inc()
inc()
blether n + 1
ken m = 0
dae show() {
    blether m
}
fer i in 0..3 {
    m = m + 1
    show()
}
"#,
    );
    assert_eq!(out, vec!["3", "1", "2", "3"]);
}

// ---------- MDH-BUG-FLU-00049: try/catch handler stack ----------

#[test]
fn gie_inside_try_unregisters_the_handler() {
    let run = compile_and_run(
        r#"
dae f() {
    hae_a_bash {
        gie 1
    } gin_it_gangs_wrang e {
        blether "f caught: " + e
    }
    gie 0
}
blether f()
hurl "boom"
"#,
    );
    assert_eq!(run.stdout.trim(), "1");
    assert_eq!(run.code, Some(1), "stderr={:?}", run.stderr);
    assert!(run.stderr.contains("boom"), "stderr={:?}", run.stderr);
}

#[test]
fn brak_and_haud_inside_try_unregister_the_handler() {
    let run = compile_and_run(
        r#"
fer i in 0..3 {
    hae_a_bash {
        brak
    } gin_it_gangs_wrang e {
        blether "brak caught: " + e
    }
}
fer i in 0..3 {
    hae_a_bash {
        haud
    } gin_it_gangs_wrang e {
        blether "haud caught: " + e
    }
}
blether "done"
hurl "boom"
"#,
    );
    assert_eq!(run.stdout.trim(), "done");
    assert_eq!(run.code, Some(1), "stderr={:?}", run.stderr);
    assert!(run.stderr.contains("boom"), "stderr={:?}", run.stderr);
}

#[test]
fn deeply_nested_try_blocks_catch_at_the_right_depth() {
    let out = run_ok(
        r#"
dae deep(n) {
    hae_a_bash {
        gin n == 0 { hurl "bottom" }
        deep(n - 1)
    } gin_it_gangs_wrang e {
        blether "caught at depth " + tae_string(n)
    }
}
deep(70)
"#,
    );
    assert_eq!(out, vec!["caught at depth 0"]);
}

#[test]
fn try_in_a_long_loop_does_not_grow_the_stack() {
    let out = run_ok(
        r#"
ken n = 0
fer i in 0..1000000 {
    hae_a_bash {
        n = n + 1
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether n
"#,
    );
    assert_eq!(out, vec!["1000000"]);
}

#[test]
fn worker_thread_hurl_is_reraised_by_join_not_in_main_handlers() {
    let out = run_ok(
        r#"
dae worker() {
    hurl "worker failed"
}
hae_a_bash {
    ken h = thread_spawn(worker, [])
    thread_join(h)
    blether "main ok"
} gin_it_gangs_wrang e {
    blether "main caught: " + e
}
blether "after"
"#,
    );
    assert_eq!(out, vec!["main caught: worker failed", "after"]);
}

// ---------- MDH-BUG-FLU-00054: native handles are validated ----------

#[test]
fn forged_native_handles_are_catchable_errors() {
    let out = run_ok(
        r#"
hae_a_bash {
    mutex_lock(42)
} gin_it_gangs_wrang e {
    blether "mutex: " + e
}
hae_a_bash {
    thread_join(7)
} gin_it_gangs_wrang e {
    blether "thread: " + e
}
ken a = atomic_new(5)
hae_a_bash {
    mutex_lock(a)
} gin_it_gangs_wrang e {
    blether "wrong kind: " + e
}
ken m = mutex_new()
mutex_lock(m)
mutex_unlock(m)
blether atomic_load(a)
"#,
    );
    assert_eq!(
        out,
        vec![
            "mutex: Unknown mutex handle",
            "thread: Unknown thread handle",
            "wrong kind: Unknown mutex handle",
            "5",
        ]
    );
}

// ---------- MDH-BUG-FLU-00050: checked integer arithmetic ----------

#[test]
fn integer_overflow_hurls_instead_of_wrapping() {
    let out = run_ok(
        r#"
ken big = 9223372036854775807
hae_a_bash {
    blether big + 1
} gin_it_gangs_wrang e {
    blether "add caught"
}
ken small = -9223372036854775807 - 1
hae_a_bash {
    blether small - 1
} gin_it_gangs_wrang e {
    blether "sub caught"
}
hae_a_bash {
    blether big * 2
} gin_it_gangs_wrang e {
    blether "mul caught"
}
hae_a_bash {
    blether -small
} gin_it_gangs_wrang e {
    blether "neg caught"
}
ken xs = [big]
hae_a_bash {
    blether xs[0] + 1
} gin_it_gangs_wrang e {
    blether "dynamic add caught"
}
dae f(n) {
    ken total = n
    total = total + n
    gie total
}
hae_a_bash {
    blether f(big)
} gin_it_gangs_wrang e {
    blether "function add caught"
}
blether big - 1
blether 6 * 7
"#,
    );
    assert_eq!(
        out,
        vec![
            "add caught",
            "sub caught",
            "mul caught",
            "neg caught",
            "dynamic add caught",
            "function add caught",
            "9223372036854775806",
            "42",
        ]
    );
}

#[test]
fn int_min_divided_by_minus_one_hurls() {
    let out = run_ok(
        r#"
ken a = -9223372036854775807 - 1
ken b = -1
hae_a_bash {
    blether a / b
} gin_it_gangs_wrang e {
    blether "div caught"
}
hae_a_bash {
    blether a % b
} gin_it_gangs_wrang e {
    blether "mod caught"
}
dae g() {
    ken c = -9223372036854775807 - 1
    ken d = -1
    ken q = c / d
    gie q
}
hae_a_bash {
    blether g()
} gin_it_gangs_wrang e {
    blether "fn div caught"
}
blether 7 / -1
blether 7 % -1
"#,
    );
    assert_eq!(
        out,
        vec!["div caught", "mod caught", "fn div caught", "-7", "0"]
    );
}

#[test]
fn bool_arithmetic_is_a_type_error() {
    let out = run_ok(
        r#"
hae_a_bash {
    blether aye + 1
} gin_it_gangs_wrang e {
    blether "add caught"
}
hae_a_bash {
    blether nae * 3
} gin_it_gangs_wrang e {
    blether "mul caught"
}
blether "x" + aye
"#,
    );
    assert_eq!(out, vec!["add caught", "mul caught", "xaye"]);
}

// ---------- MDH-BUG-FLU-00051: lists with capacity 0 grow ----------

#[test]
fn empty_slice_grows_on_push() {
    let out = run_ok(
        r#"
ken xs = [1, 2]
ken ys = xs[5:]
shove(ys, 1)
shove(ys, 2)
shove(ys, 3)
blether ys
"#,
    );
    assert_eq!(out, vec!["[1, 2, 3]"]);
}

// ---------- MDH-BUG-FLU-00052: checked string repeat ----------

#[test]
fn huge_string_repeat_hurls_instead_of_overflowing() {
    let out = run_ok(
        r#"
ken s = "abcd"
hae_a_bash {
    blether len(s * 4611686018427387905)
} gin_it_gangs_wrang e {
    blether "repeat caught"
}
blether "ab" * 3
blether len("" * 4611686018427387905)
"#,
    );
    assert_eq!(out, vec!["repeat caught", "ababab", "0"]);
}

// ---------- MDH-BUG-FLU-00055: speir reads whole lines ----------

#[test]
fn speir_reads_lines_longer_than_1023_bytes() {
    let line = "x".repeat(5000);
    let run = compile_and_run_with_input(
        r#"
ken first = speir ""
ken second = speir ""
blether len(first)
blether second
"#,
        &format!("{line}\nnext\n"),
    );
    assert_eq!(run.code, Some(0), "stderr={:?}", run.stderr);
    let lines: Vec<&str> = run.stdout.trim().lines().collect();
    assert_eq!(lines, vec!["5000", "next"]);
}
