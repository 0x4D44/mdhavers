#![cfg(feature = "llvm")]
//! Native backend: a keek arm's identifier pattern binds a name seen only inside that arm
//! (MDH-BUG-FLU-00178), and a nested function reads an enclosing function's local that
//! hides a global of the same name (MDH-BUG-FLU-00179), as in the interpreter.

use std::process::{Command, Stdio};

use mdhavers::{parse, Interpreter, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_native(source: &str, refcounting: bool) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("block_scope_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .env("MDH_RC_REPORT", "1")
        .output()
        .expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// The interpreter's output lines, on a thread with room for its deep frames.
fn run_interpreter(source: &str) -> Vec<String> {
    let source = source.to_string();
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            let program = parse(&source).expect("parse");
            let mut interp = Interpreter::new();
            interp.interpret(&program).expect("interpreter run");
            interp.get_output().to_vec()
        })
        .expect("spawn interpreter thread")
        .join()
        .expect("interpreter thread")
}

/// Native output (plain and with reference counting) matches the interpreter's, and the
/// counted build frees everything it made.
fn assert_parity(source: &str) {
    let expected: Vec<String> = run_interpreter(source)
        .join("\n")
        .lines()
        .map(str::to_string)
        .collect();
    for refcounting in [false, true] {
        let run = run_native(source, refcounting);
        assert_eq!(
            run.code,
            Some(0),
            "native (rc={refcounting}) failed: stdout={:?} stderr={:?}",
            run.stdout,
            run.stderr
        );
        let got: Vec<String> = run.stdout.lines().map(str::to_string).collect();
        assert_eq!(got, expected, "native (rc={refcounting}) differs");
        if refcounting {
            assert!(
                run.stderr.contains(" total=0 "),
                "counted build leaked: {}",
                run.stderr
            );
        }
    }
}

#[test]
fn match_arm_binding_does_not_leak_at_top_level() {
    assert_parity(
        r#"
ken x = 1
ken s = "out"
keek 5 {
    whan 1 -> { blether "one" }
    whan x -> { blether x }
}
blether x
keek "hi" {
    whan s -> { ken t = s + "!" blether t }
}
blether s
keek [1, 2] {
    whan x -> { blether len(x) }
}
blether x
keek 7 {
    whan 1 -> { blether "one" }
    whan 2..9 -> { blether "range" }
    whan s -> { blether "never" }
}
blether s
"#,
    );
}

#[test]
fn match_arm_binding_is_scoped_to_its_arm_in_the_interpreter() {
    let out = run_interpreter("ken x = 1\nkeek 5 {\n    whan x -> { blether x }\n}\nblether x\n");
    assert_eq!(out.join("\n").lines().collect::<Vec<_>>(), ["5", "1"]);
}

#[test]
fn match_arm_binding_does_not_leak_in_functions() {
    assert_parity(
        r#"
ken g = 100
dae f(p) {
    ken x = 1
    keek p {
        whan 0 -> { blether "zero" }
        whan x -> { blether x + 1 }
    }
    keek p {
        whan g -> { blether g }
    }
    keek p {
        whan p -> { blether p }
    }
    blether g
    gie x * 10 + p
}
blether f(5)
blether f(0)
blether g
dae h(v) {
    keek v {
        whan y -> { blether y }
    }
    keek v {
        whan y -> { blether y }
    }
    gie v
}
blether h("str")
blether h([1, 2])
ken fs = []
fer i in 0..3 {
    keek "n" + tae_string(i) {
        whan i -> { shove(fs, || i) }
    }
}
fer q in fs { blether q() }
blether g
"#,
    );
}

#[test]
fn nested_function_reads_enclosing_local_that_hides_a_global() {
    assert_parity(
        r#"
ken x = 1
dae f() {
    ken x = 2
    dae g() { gie x }
    gie g()
}
blether f()
blether x
"#,
    );
}

#[test]
fn nested_function_and_lambda_capture_a_local_hiding_a_global() {
    assert_parity(
        r#"
ken x = 1
ken name = "glob"
dae f(n) {
    ken x = n * 10
    ken name = "loc" + tae_string(n)
    dae g() { gie x + 1 }
    ken h = || name + "!" + tae_string(x)
    dae deep() {
        dae deeper() { gie x + len(name) }
        gie deeper()
    }
    gie [g(), h(), deep()]
}
blether f(2)
blether f(3)
blether x
blether name
dae counter() {
    ken x = 0
    dae bump() { x = x + 1 gie x }
    bump()
    bump()
    gie x
}
blether counter()
blether x
dae uses_global() {
    dae g() { gie x }
    gie g()
}
blether uses_global()
ken total = 100
dae mk() {
    ken x = 0
    ken total = 10
    ken inc = || { x = x + 1 gie x }
    inc()
    inc()
    dae add(n) { total = total + n gie total }
    add(5)
    gie [x, total, inc()]
}
blether mk()
blether total
dae reads_first() {
    blether x
    ken x = 5
    dae g() { gie x }
    gie g()
}
blether reads_first()
"#,
    );
}

/// A block-level `ken` that hides a global is what a nested function or lambda in that
/// block reads, until the block ends (MDH-BUG-FLU-00180).
#[test]
fn block_level_local_hiding_a_global_is_captured() {
    assert_parity(
        r#"
ken x = 1
ken total = 100
dae blk(c) {
    gin c {
        ken x = 7
        dae g() { gie x }
        gie g()
    }
    gie x
}
blether blk(aye)
blether blk(nae)
dae lam(c) {
    gin c {
        ken x = 8
        ken h = || x + 1
        gie h()
    }
    gie x
}
blether lam(aye)
dae mut(c) {
    blether x
    gin c {
        ken x = 0
        ken inc = || { x = x + 1 gie x }
        dae bump() { x = x + 10 gie x }
        inc()
        bump()
        inc()
        blether x
    }
    gie x
}
blether mut(aye)
blether x
dae loops() {
    ken out = []
    ken i = 0
    whiles i < 3 {
        ken total = i * 2
        dae g() { total = total + 1 gie total }
        g()
        shove(out, g())
        ken x = i
        ken h = || x * 100
        shove(out, h())
        i = i + 1
    }
    fer k in [1, 2] {
        ken x = k + 50
        dae g2() { gie x }
        shove(out, g2())
    }
    shove(out, x)
    shove(out, total)
    gie out
}
blether loops()
dae nested(c) {
    gin c {
        ken x = 3
        gin c {
            ken x = 4
            dae g() { gie x }
            blether g()
        }
        dae g() { gie x }
        blether g()
    }
    dae last() { gie x }
    gie last()
}
blether nested(aye)
blether x
blether total
"#,
    );
}
