#![cfg(feature = "llvm")]
//! Native backend: reference counting, stage R1 (`wrk_docs/2026.10.25 - DES - R1 refcount
//! insertion plan.md`). Counts are kept and an object reaching zero is marked DEAD instead
//! of freed, so a missing retain shows up as an abort naming the object's kind.

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Compiles natively at -O2 with reference counting on and runs with the given environment.
fn run_native(source: &str, envs: &[(&str, &str)]) -> Run {
    run_native_rc(source, true, envs)
}

fn run_native_rc(source: &str, refcounting: bool, envs: &[(&str, &str)]) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("rc_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let mut cmd = Command::new(&exe_path);
    cmd.stdin(Stdio::null());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// The live count the harness reports for `kind` (MDH_RC_REPORT=1).
fn reported_live(stderr: &str, kind: &str) -> i64 {
    let line = stderr
        .lines()
        .find(|l| l.starts_with("mdhavers rc: live at exit:"))
        .unwrap_or_else(|| panic!("no rc report in {stderr:?}"));
    let key = format!(" {kind}=");
    let start = line
        .find(&key)
        .unwrap_or_else(|| panic!("no {kind} in {line}"))
        + key.len();
    line[start..]
        .split(' ')
        .next()
        .unwrap()
        .parse()
        .expect("count")
}

const STRINGS: &str = "0x2";

#[test]
fn strings_die_when_their_last_holder_lets_go() {
    // Globals, locals, parameters, returns, reassignment, self-append (in place and
    // copying), f-strings, a loop and a try: every string made is released by the end.
    let src = r#"
dae greet(name) {
    ken msg = "Hullo, " + name
    gie msg + "!"
}
dae pick(a, b) {
    gin len(a) > len(b) {
        gie a
    }
    gie b
}
ken s = "a" + "b"
s = s + "c"
ken t = s
s = "lit"
blether t
blether greet("Arthur")
ken acc = ""
ken i = 0
whiles i < 50 {
    acc = acc + "x"
    ken tmp = upper(acc) + lower("Y")
    i = i + 1
}
blether len(acc)
blether pick(acc, t)
blether f"interp {t} and {greet(t)}"
"#;
    let run = run_native(src, &[("MDH_RC_KINDS", STRINGS), ("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "abc\nHullo, Arthur!\n50\nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n\
         interp abc and Hullo, abc!\n"
    );
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
}

#[test]
fn a_hurl_leaks_at_most_its_error_and_never_frees_early() {
    // Exception paths may leak (DEC round 2, F6 Q4) but must never release twice: the
    // handler's frame keeps its slots consistent across the longjmp, at -O2 too.
    let src = r#"
dae risky(n) {
    ken label = "risk" + "y"
    gin n > 2 {
        hurl label + " " + tae_string(n)
    }
    gie label
}
ken kept = "kept" + "!"
ken i = 0
whiles i < 5 {
    hae_a_bash {
        ken got = risky(i)
        kept = got + kept
    } gin_it_gangs_wrang e {
        blether e
    }
    i = i + 1
}
blether kept
"#;
    let run = run_native(src, &[("MDH_RC_KINDS", STRINGS), ("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "Hurled at line 5: risky 3\nHurled at line 5: risky 4\nriskyriskyriskykept!\n"
    );
    // Each of the two hurls abandons at most its frame's label, the message's two
    // operands and the message; one more for the error kept after the last handler.
    assert!(
        reported_live(&run.stderr, "string") <= 2 * 4 + 1,
        "{}",
        run.stderr
    );
}

#[test]
fn borrowed_reads_and_parameters_take_a_reference_once_written() {
    // Parameters and local reads stay borrowed until something stores into their slot:
    // a read of `x` still in use when `x` is reassigned in the same statement, and
    // parameters reassigned in the body, must not let the old string die early.
    let src = r#"
dae pair(a, b) {
    gie [a, b]
}
dae f(p) {
    ken x = "a" + p
    ken y = pair(x, (x = "z" + p))
    gie y[0] + y[1] + x
}
dae g(s, n) {
    s = s + "!"
    whiles n > 0 {
        s = s + tae_string(n)
        n = n - 1
    }
    gie s
}
ken keep = "kee" + "p"
blether f("b")
blether g(keep, 3)
blether keep
"#;
    let run = run_native(src, &[("MDH_RC_KINDS", STRINGS), ("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "abzbzb\nkeep!321\nkeep\n");
    // Lists are not counted yet, so the pair's two strings live on with it.
    assert_eq!(reported_live(&run.stderr, "string"), 2, "{}", run.stderr);
}

#[test]
fn static_strings_carry_the_header_magic_only_when_counting() {
    // The runtime's header check looks for the magic; without counting, the IR is as
    // before reference counting existed (flags = IMMORTAL).
    let program = parse("blether \"literal\"\n").expect("parse");
    let flags = |rc: bool| {
        let ir = LLVMCompiler::new()
            .with_refcounting(rc)
            .compile_to_ir(&program)
            .expect("compile");
        let tail = ", [8 x i8] c\"literal\\00\" }";
        let at = ir
            .find(tail)
            .unwrap_or_else(|| panic!("no literal in {ir}"));
        let head = &ir[..at];
        head[head.rfind("i32 ").expect("flags") + 4..].to_string()
    };
    assert_eq!(flags(true), (0x4D44_0000u32 | 1).to_string());
    assert_eq!(flags(false), "1");
}

#[test]
fn counting_stays_off_unless_compiled_in() {
    // Off by default (counts are not atomic yet): the runtime counts nothing, even when
    // MDH_RC_KINDS asks it to.
    let src = r#"
ken s = "a" + "b"
s = s + "c"
blether s
"#;
    let run = run_native_rc(
        src,
        false,
        &[("MDH_RC_KINDS", "0x1FE"), ("MDH_RC_REPORT", "1")],
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "abc\n");
    assert!(run.stderr.contains("(counted kinds 0x0)"), "{}", run.stderr);
}

#[test]
fn rc_report_prints_live_objects_at_exit() {
    let src = r#"
ken s = "a" + "b"
blether s
"#;
    let quiet = run_native(src, &[]);
    assert_eq!(quiet.code, Some(0), "{}", quiet.stderr);
    assert!(!quiet.stderr.contains("mdhavers rc:"), "{}", quiet.stderr);
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "ab\n");
    assert!(reported_live(&run.stderr, "string") >= 0, "{}", run.stderr);
    assert!(run.stderr.contains(" total="), "{}", run.stderr);
}
