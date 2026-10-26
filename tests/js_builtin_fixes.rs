//! JS backend: builtin behaviours that must match the interpreter (MDH-BUG-FLU-00167, -00170,
//! -00173). The shared cases live in tests/golden (run by js_golden_parity); these cover the
//! error paths and JS-only wording that a golden cannae pin across every backend.

use std::io::Write;
use std::process::{Command, Stdio};

fn node_available() -> bool {
    Command::new("node").arg("--version").output().is_ok()
}

/// Compile `source` tae JS an' run it under node; returns stdout.
fn run_js(source: &str) -> String {
    let js = mdhavers::compile_to_js(source).expect("compile");
    let mut child = Command::new("node")
        .arg("-e")
        .arg("eval(require('fs').readFileSync(0,'utf8'))")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start node");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(js.as_bytes())
        .expect("write js");
    let out = child.wait_with_output().expect("node output");
    assert!(
        out.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The message a `hae_a_bash` handler sees when `expr` raises.
fn error_of(expr: &str) -> String {
    let source = format!(
        "hae_a_bash {{\n    blether {expr}\n}} gin_it_gangs_wrang e {{\n    blether e\n}}\n"
    );
    run_js(&source).trim().to_string()
}

#[test]
fn join_rejects_a_number_separator() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    assert_eq!(
        error_of("join([3, 1, 2], 2)"),
        "Jings! Something went awfy wrang: join() expects a list and a string"
    );
    assert_eq!(
        error_of("join(\"abc\", \",\")"),
        "Jings! Something went awfy wrang: join() expects a list and a string"
    );
    assert_eq!(run_js("blether join([3, 1, 2], \"-\")").trim(), "3-1-2");
}

#[test]
fn stacktrace_is_the_no_frames_placeholder() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    assert_eq!(run_js("blether stacktrace()").trim(), "(no stack trace)");
}

#[test]
fn tak_and_drap_take_strings_an_clamp_negative_counts() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    let out = run_js(
        "blether tak(\"Hello\", 2)\nblether drap(\"Hello\", 6)\nblether tak([3, 1, 2], -1)\nblether drap([3, 1, 2], -1)\n",
    );
    assert_eq!(out, "[H, e]\n[]\n[]\n[3, 1, 2]\n");
    // A non-list, non-string is the usual type error (the interpreter's, MDH-BUG-FLU-00171).
    assert_eq!(
        error_of("tak(7, 0)"),
        "Jings! Something went awfy wrang: tak() needs a list and an integer"
    );
    assert_eq!(
        error_of("drap(7, 1)"),
        "Jings! Something went awfy wrang: drap() needs a list and an integer"
    );
}

#[test]
fn every_new_scots_builtin_is_bound() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    // None o' these may be a ReferenceError (the random an' clock ones are just called).
    let out = run_js(
        "blether len(haver()) > 0\nblether len(slainte()) > 0\nblether len(braw_time()) > 0\nblether jammy(1, 2)\nblether dram([7])\nblether len(bampot_mode([1, 2, 3]))\nblether stopwatch(|x| x) != naething\nblether set_crash_handling(aye)\n",
    );
    assert_eq!(out, "aye\naye\naye\n1\n7\n3\naye\nnaething\n");
}

#[test]
fn new_builtins_raise_the_interpreters_errors() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    let wrap = "Jings! Something went awfy wrang: ";
    for (expr, message) in [
        ("stoater([])", "Cannae find a stoater in an empty list!"),
        ("stoater(1)", "stoater() needs a list"),
        (
            "dicht([1, 2], 5)",
            "Index 5 oot o' bounds fer list o' length 2",
        ),
        ("dicht(1, 0)", "dicht() needs a list"),
        ("dicht([1], \"a\")", "dicht() needs an integer index"),
        ("is_a(1, 2)", "is_a() needs a type name string"),
        ("redd_up(1)", "redd_up() needs a list"),
        ("scottify(1)", "scottify() needs a string"),
        ("sclaff(1)", "sclaff needs a list"),
        ("drookit(1)", "drookit() needs a list"),
        ("wrang_sort(1, 2)", "Second arg must be a type name string"),
        (
            "scunner_check(1, 2)",
            "scunner_check() needs type name as second arg",
        ),
        (
            "backside_forrit(1)",
            "backside_forrit() expects a list or string",
        ),
        ("jammy(3, 3)", "jammy() needs min < max, ya numpty!"),
    ] {
        assert_eq!(error_of(expr), format!("{wrap}{message}"), "{expr}");
    }
}

#[test]
fn middle_rest_destructures_a_string() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    let out =
        run_js("ken [p, q, ...tail, z] = \"abcdef\"\nblether p + q\nblether tail\nblether z\n");
    assert_eq!(out, "ab\n[c, d, e]\nf\n");
}

#[test]
fn wee_sclaff_and_dicht_match_the_interpreter() {
    if !node_available() {
        eprintln!("SKIPPED: `node` is no' on PATH");
        return;
    }
    let out = run_js(
        "blether wee(10, 25)\nblether wee(7, 2)\nblether sclaff([1, [2, [3, [4]]], 5])\nblether dicht([1, 2, 3], -3)\n",
    );
    assert_eq!(out, "10\n2\n[1, 2, 3, 4, 5]\n[2, 3]\n");
    // A too-negative index wraps tae a huge usize, as in the interpreter.
    assert_eq!(
        error_of("dicht([1, 2, 3], -4)"),
        "Jings! Something went awfy wrang: Index 18446744073709551615 oot o' bounds fer list o' length 3"
    );
}
