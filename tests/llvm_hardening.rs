#![cfg(feature = "llvm")]
//! Native-backend hardening: user functions named like libc helpers
//! (MDH-BUG-FLU-00087), amortised O(1) dict inserts (00053) and runtime errors that
//! carry the interpreter's wording and " at line N" (00086, 00089, 00090, 00093),
//! including forward globals used before their `ken` (00095).

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    stdout: String,
    stderr: String,
    code: Option<i32>,
    elapsed: Duration,
}

/// Compile `source` natively and run it, killing it after `deadline` (code is then None).
fn compile_and_run_within(source: &str, deadline: Duration) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("hardening_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let stdout_path = dir.path().join("stdout.txt");
    let stderr_path = dir.path().join("stderr.txt");
    let start = Instant::now();
    let mut child = Command::new(&exe_path)
        .stdin(Stdio::null())
        .stdout(std::fs::File::create(&stdout_path).expect("stdout file"))
        .stderr(std::fs::File::create(&stderr_path).expect("stderr file"))
        .spawn()
        .expect("run executable");
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait on executable") {
            break Some(status);
        }
        if start.elapsed() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    Run {
        stdout: std::fs::read_to_string(&stdout_path).unwrap_or_default(),
        stderr: std::fs::read_to_string(&stderr_path).unwrap_or_default(),
        code: status.and_then(|s| s.code()),
        elapsed: start.elapsed(),
    }
}

fn compile_and_run(source: &str) -> Run {
    compile_and_run_within(source, Duration::from_secs(120))
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

// ---------- MDH-BUG-FLU-00087: user functions named like libc helpers ----------

#[test]
fn user_functions_named_like_libc_helpers_do_not_collide() {
    let out = run_ok(
        r#"
dae pow(a, b) { gie 42 }
dae atan2(a, b) { gie 7 }
dae strtod(s) { gie 1 }
dae strncmp(a, b, c) { gie 2 }
dae malloc(n) { gie 3 }
dae printf(s) { gie 4 }
dae sqrt(x) { gie 9 }
dae exit(code) { gie 5 }
kin pthread { dae create() { gie 6 } }
blether pooer(2.0, 0.5)
blether pooer(2, 10)
blether starts_wi("haggis", "hag")
blether is_nummer("3.5")
blether pow(1, 2)
blether atan2(1, 2)
blether strtod("x")
blether strncmp(1, 2, 3)
blether malloc(1)
blether printf("x")
blether sqrt(4)
blether exit(1)
ken f = pow
blether f(3, 4)
ken p = pthread()
blether p.create()
ken d = {"a": 1}
d["b"] = 2
blether d
"#,
    );
    assert_eq!(
        out,
        vec![
            "1.4142135623730951",
            "1024",
            "aye",
            "aye",
            "42",
            "7",
            "1",
            "2",
            "3",
            "4",
            "9",
            "5",
            "42",
            "6",
            r#"{"a": 1, "b": 2}"#,
        ]
    );
}

// ---------- MDH-BUG-FLU-00053: dict inserts are amortised O(1) ----------

#[test]
fn two_hundred_thousand_dict_inserts_finish_quickly() {
    // Quadratic inserts take minutes (and leak every copy), so cap the run.
    let run = compile_and_run_within(
        r#"
ken d = {}
fer i in 0..200000 {
    d[tae_string(i)] = i
}
blether len(d)
blether d["199999"]
ken s = empty_creel()
fer i in 0..200000 {
    toss_in(s, i)
}
blether len(s)
"#,
        Duration::from_secs(10),
    );
    assert_eq!(
        run.code,
        Some(0),
        "after {:?}: stderr={:?}",
        run.elapsed,
        run.stderr
    );
    assert_eq!(
        run.stdout.trim().lines().collect::<Vec<_>>(),
        vec!["200000", "199999", "200000"]
    );
}

#[test]
fn growing_dicts_keep_order_updates_aliases_and_key_identity() {
    let out = run_ok(
        r#"
ken d = {"b": 1, "a": 2}
fer i in 0..20 {
    d["k" + tae_string(i)] = i
}
d["a"] = 99
d["k3"] = 333
blether len(d)
blether d["a"]
blether d["k3"]
blether d["k19"]
blether keys(d)[0]
blether keys(d)[1]
blether keys(d)[2]
blether keys(d)[21]
blether dict_has(d, "k7")
blether dict_has(d, "nope")
ken alias = d
alias["shared"] = 1
blether len(d)
ken m = {}
fer i in 0..50 {
    m[i] = i * 2
}
m[1.0] = "float"
blether len(m)
blether m[49]
blether m[1]
blether m[1.0]
ken s = empty_creel()
fer i in 0..30 {
    toss_in(s, i % 20)
}
blether len(s)
blether is_in_creel(s, 19)
blether is_in_creel(s, 20)
"#,
    );
    // Same output as the interpreter.
    assert_eq!(
        out,
        vec![
            "22", "99", "333", "19", "b", "a", "k0", "k19", "aye", "nae", "23", "51", "98", "2",
            "float", "20", "aye", "nae",
        ]
    );
}

// ---------- MDH-BUG-FLU-00086: runtime errors carry the interpreter's text and line ----------

#[test]
fn caught_runtime_errors_match_the_interpreter_text() {
    let out = run_ok(
        r#"
hae_a_bash {
    ken x = 10 / 0
} gin_it_gangs_wrang e {
    blether e
}
ken z = 0
hae_a_bash {
    ken x = 10 / z
} gin_it_gangs_wrang e {
    blether e
}
ken l = [1, 2, 3]
hae_a_bash {
    blether l[5]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    l[7] = 1
} gin_it_gangs_wrang e {
    blether e
}
ken d = {"a": 1}
hae_a_bash {
    blether d["zz"]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether "a" - 1
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether 7 % z
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether l[-9]
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether "abc"[9]
} gin_it_gangs_wrang e {
    blether e
}
ken fz = 0.0
hae_a_bash {
    blether 1.0 / fz
} gin_it_gangs_wrang e {
    blether e
}
"#,
    );
    // Captured from the interpreter on the same program.
    assert_eq!(
        out,
        vec![
            "Ye numpty! Tryin' tae divide by zero at line 3",
            "Ye numpty! Tryin' tae divide by zero at line 9",
            "Hoachin'! Index 5 is oot o' bounds (size is 3) at line 15",
            "Hoachin'! Index 7 is oot o' bounds (size is 3) at line 20",
            "Awa' an bile yer heid! 'zz' hasnae been defined yet at line 26",
            "That's pure mince! Type error at line 31: Cannae subtract integer fae string",
            "Ye numpty! Tryin' tae divide by zero at line 36",
            "Hoachin'! Index -9 is oot o' bounds (size is 3) at line 41",
            "Hoachin'! Index 9 is oot o' bounds (size is 3) at line 46",
            "Ye numpty! Tryin' tae divide by zero at line 52",
        ]
    );
}

#[test]
fn runtime_error_lines_follow_calls_in_and_out_of_functions() {
    let out = run_ok(
        r#"
dae step(x) {
    ken y = x + 1
    gie y
}
dae boom(x) {
    ken y = step(x)
    gie y / 0
}
hae_a_bash {
    ken r = step(1) / 0
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    boom(1)
} gin_it_gangs_wrang e {
    blether e
}
"#,
    );
    assert_eq!(
        out,
        vec![
            "Ye numpty! Tryin' tae divide by zero at line 11",
            "Ye numpty! Tryin' tae divide by zero at line 8",
        ]
    );
}

#[test]
fn uncaught_runtime_error_reports_its_line() {
    let run = compile_and_run(
        r#"
blether "before"
ken l = [1]
blether l[3]
"#,
    );
    assert_ne!(run.code, Some(0));
    assert!(
        run.stderr
            .contains("Hoachin'! Index 3 is oot o' bounds (size is 1) at line 4"),
        "stderr={:?}",
        run.stderr
    );
}

// ---------- MDH-BUG-FLU-00093: integer overflow carries the interpreter's line ----------

#[test]
fn integer_overflow_error_carries_its_line() {
    // Both the inline checked arithmetic (known ints) and an untyped parameter.
    let out = run_ok(
        r#"
ken big = 9223372036854775807
hae_a_bash {
    blether big + 1
} gin_it_gangs_wrang e {
    blether e
}
dae dbl(x) {
    gie x * 2
}
hae_a_bash {
    blether dbl(big)
} gin_it_gangs_wrang e {
    blether e
}
"#,
    );
    // The interpreter's IntegerOverflow text: the line sits mid-message.
    assert_eq!(
        out,
        vec![
            "Wheesht! Yer number's too muckle at line 4 - it's overflowed!",
            "Wheesht! Yer number's too muckle at line 9 - it's overflowed!",
        ]
    );
}

// ---------- MDH-BUG-FLU-00089: modulo by an integer zero divides by zero ----------

#[test]
fn modulo_by_integer_zero_divides_by_zero_whatever_the_left_type() {
    let out = run_ok(
        r#"
ken f = 1.5
ken z = 0
hae_a_bash {
    blether f % z
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether f % 0
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    blether 7 % 2.0
} gin_it_gangs_wrang e {
    blether e
}
"#,
    );
    // The interpreter checks for an integer zero divisor before the operand types;
    // a mixed int/float remainder stays a type error.
    assert_eq!(
        out,
        vec![
            "Ye numpty! Tryin' tae divide by zero at line 5",
            "Ye numpty! Tryin' tae divide by zero at line 10",
            "That's pure mince! Type error at line 15: Cannae get remainder o' integer by float",
        ]
    );
}

// ---------- MDH-BUG-FLU-00090 (in part): the loop-condition line ----------

#[test]
fn error_in_a_loop_condition_reports_the_loop_line() {
    let out = run_ok(
        r#"
ken d = 2
hae_a_bash {
    whiles 10 / d > 0 {
        d = d - 1
        blether d
    }
} gin_it_gangs_wrang e {
    blether e
}
"#,
    );
    assert_eq!(
        out,
        vec!["1", "0", "Ye numpty! Tryin' tae divide by zero at line 4"]
    );
}

// ---------- MDH-BUG-FLU-00095: a forward global used before its ken has run ----------

#[test]
fn forward_global_used_before_its_ken_is_undefined() {
    let out = run_ok(
        r#"
dae f() {
    gie later
}
dae g() {
    later = 5
}
hae_a_bash {
    blether f()
} gin_it_gangs_wrang e {
    blether e
}
hae_a_bash {
    g()
} gin_it_gangs_wrang e {
    blether e
}
ken later = 1
blether f()
g()
blether f()
"#,
    );
    // As the interpreter: reading or assigning the global before its `ken` is an
    // undefined variable; afterwards both work.
    assert_eq!(
        out,
        vec![
            "Awa' an bile yer heid! 'later' hasnae been defined yet at line 3",
            "Awa' an bile yer heid! 'later' hasnae been defined yet at line 6",
            "1",
            "5",
        ]
    );
}

#[test]
fn forward_global_read_before_its_ken_fails_uncaught() {
    let run = compile_and_run(
        r#"
dae f() {
    gie later
}
blether f()
ken later = 1
"#,
    );
    assert_ne!(run.code, Some(0));
    assert!(
        run.stderr
            .contains("Awa' an bile yer heid! 'later' hasnae been defined yet at line 3"),
        "stdout={:?} stderr={:?}",
        run.stdout,
        run.stderr
    );
}

// ---------- MDH-BUG-FLU-00091: dict reads use the hash index ----------

#[test]
fn two_hundred_thousand_dict_reads_finish_quickly_and_dict_remove_copies() {
    // An inline linear scan per read is quadratic (minutes), so cap the run.
    let run = compile_and_run_within(
        r#"
ken d = {}
fer i in 0..200000 {
    d[tae_string(i)] = i
}
ken total = 0
fer i in 0..200000 {
    total = total + d[tae_string(i)]
}
blether total
ken small = {"a": 1, "b": 2, "c": 3}
ken smaller = dict_remove(small, "b")
blether len(smaller)
blether smaller["a"] + smaller["c"]
blether dict_has(smaller, "b")
blether len(small)
blether len(dict_remove(small, "zz"))
blether len(dict_remove({"a": 1}, "a"))
ken big = dict_remove(d, "5")
blether len(big)
blether big["199999"]
hae_a_bash {
    blether big["5"]
} gin_it_gangs_wrang e {
    blether e
}
"#,
        Duration::from_secs(10),
    );
    assert_eq!(
        run.code,
        Some(0),
        "after {:?}: stderr={:?}",
        run.elapsed,
        run.stderr
    );
    assert_eq!(
        run.stdout.trim().lines().collect::<Vec<_>>(),
        vec![
            "19999900000",
            "2",
            "4",
            "nae",
            "3",
            "3",
            "0",
            "199999",
            "199999",
            "Awa' an bile yer heid! '5' hasnae been defined yet at line 23",
        ]
    );
}
