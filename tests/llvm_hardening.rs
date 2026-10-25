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

// ---------- MDH-BUG-FLU-00091: heave_oot leaves tombstones no reader sees ----------

#[test]
fn creel_removals_are_invisible_to_every_reader() {
    // Ten keys, so lookups go through the hash index. heave_oot tombstones in place; len,
    // listing, display, membership, equality, the set operations and truthiness must all
    // see only the live keys, and a removed key can come back.
    let out = run_ok(
        r#"
ken c = creel(["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"])
heave_oot(c, "c")
heave_oot(c, "zz")
blether len(c)
blether creel_tae_list(c)
blether c
blether is_in_creel(c, "c")
blether is_in_creel(c, "d")
toss_in(c, "c")
blether len(c)
blether is_in_creel(c, "c")
ken fresh = creel(["j", "i", "h", "g", "f", "e", "d", "c", "b", "a"])
blether c == fresh
heave_oot(c, "a")
blether c == fresh
blether fresh == c
blether creels_thegither(c, creel(["a"])) == fresh
blether creel_tae_list(creels_baith(c, creel(["a", "b", "zz"])))
blether creel_tae_list(creels_differ(c, creel(["b", "j"])))
fer k in ["b", "c", "d", "e", "f", "g", "h", "i", "j"] {
    heave_oot(c, k)
}
blether len(c)
blether c
gin c {
    blether "full"
} ither {
    blether "toom"
}
toss_in(c, "q")
blether creel_tae_list(c)
"#,
    );
    // The interpreter prints the same.
    assert_eq!(
        out,
        vec![
            "9",
            "[a, b, d, e, f, g, h, i, j]",
            "creel{\"a\", \"b\", \"d\", \"e\", \"f\", \"g\", \"h\", \"i\", \"j\"}",
            "nae",
            "aye",
            "10",
            "aye",
            "aye",
            "nae",
            "nae",
            "aye",
            "[b]",
            "[c, d, e, f, g, h, i]",
            "0",
            "creel{}",
            "toom",
            "[q]",
        ]
    );
}

#[test]
fn a_hundred_thousand_creel_removals_run_in_linear_time() {
    // A copy per heave_oot is quadratic: about 25 s for this on a 2026 Mac. Tombstones
    // make it milliseconds. The churn loop keeps 1000 live keys while 100000 pass
    // through, so the payload compacts again and again.
    let run = compile_and_run_within(
        r#"
ken n = 100000
ken c = empty_creel()
fer i in 0..n {
    toss_in(c, i)
}
fer i in 0..n {
    heave_oot(c, i)
}
blether len(c)
blether c
ken w = empty_creel()
fer i in 0..n {
    toss_in(w, i)
    gin i >= 1000 {
        heave_oot(w, i - 1000)
    }
}
blether len(w)
blether is_in_creel(w, n - 1001)
blether is_in_creel(w, n - 1000)
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
        vec!["0", "creel{}", "1000", "nae", "aye"]
    );
}

// ---------- MDH-BUG-FLU-00128: fer over a dict or creel is a type error ----------

#[test]
fn fer_over_a_dict_or_creel_raises_the_interpreters_type_error() {
    // Only lists and strings iterate. The creel's third loop runs after removals, so its
    // payload holds tombstones; strings and lists still iterate.
    let out = run_ok(
        r#"
ken d = {"a": 1, "b": 2}
hae_a_bash {
    fer k in d {
        blether k
    }
} gin_it_gangs_wrang e {
    blether e
}
ken c = creel(["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"])
hae_a_bash {
    fer k in c {
        blether k
    }
} gin_it_gangs_wrang e {
    blether e
}
heave_oot(c, "b")
heave_oot(c, "e")
hae_a_bash {
    fer k in c {
        blether k
    }
} gin_it_gangs_wrang e {
    blether e
}
fer ch in "ab" {
    blether ch
}
fer x in [1, 2] {
    blether x
}
"#,
    );
    // The interpreter prints the same.
    assert_eq!(
        out,
        vec![
            "That's pure mince! Type error at line 4: Cannae iterate ower a dict",
            "That's pure mince! Type error at line 12: Cannae iterate ower a creel",
            "That's pure mince! Type error at line 21: Cannae iterate ower a creel",
            "a",
            "b",
            "1",
            "2",
        ]
    );
}

#[test]
fn uncaught_fer_over_a_dict_exits_with_the_type_error() {
    let run = compile_and_run(
        r#"
ken d = {"a": 1}
fer k in d {
    blether k
}
blether "after"
"#,
    );
    assert_ne!(run.code, Some(0));
    assert_eq!(run.stdout.trim(), "");
    assert!(
        run.stderr
            .contains("That's pure mince! Type error at line 3: Cannae iterate ower a dict"),
        "stderr={:?}",
        run.stderr
    );
}

// ---------- MDH-BUG-FLU-00090: each thread's errors report its own line ----------

#[test]
fn worker_thread_error_reports_its_own_line_while_main_runs_on() {
    // The worker starts line 4, says so on `ready` and blocks inside the same statement
    // until main, having run lines 9 to 15 meanwhile, sends the index. One line shared by
    // every thread would report main's line instead of the worker's.
    let out = run_ok(
        r#"
dae worker(ready, c) {
    ken l = [1]
    gie l[[chan_send(ready, 1), chan_recv(c)][1]]
}
ken ready = chan_new(0)
ken c = chan_new(0)
ken t = thread_spawn(worker, [ready, c])
chan_recv(ready)
ken a = 1
ken b = a + 1
blether b
chan_send(c, 5)
hae_a_bash {
    thread_join(t)
} gin_it_gangs_wrang e {
    blether e
}
"#,
    );
    assert_eq!(
        out,
        vec![
            "2",
            "Hoachin'! Index 5 is oot o' bounds (size is 1) at line 4"
        ]
    );
}

#[test]
fn main_thread_error_keeps_its_line_while_a_worker_runs_statements() {
    // Main starts line 16, lets the worker go and blocks inside the same statement; the
    // worker runs lines 4 to 7, sends the index back and waits on line 8 (so it cannot
    // return and put back the line it started with) while main raises.
    let out = run_ok(
        r#"
dae worker(go, back) {
    chan_recv(go)
    ken x = 1
    ken y = x + 1
    ken z = y * 3
    chan_send(back, z)
    chan_recv(go)
    gie z
}
ken go = chan_new(0)
ken back = chan_new(0)
ken t = thread_spawn(worker, [go, back])
ken l = [1, 2]
hae_a_bash {
    blether l[[chan_send(go, 1), chan_recv(back)][1]]
} gin_it_gangs_wrang e {
    blether e
}
chan_send(go, 2)
blether thread_join(t)
"#,
    );
    assert_eq!(
        out,
        vec![
            "Hoachin'! Index 6 is oot o' bounds (size is 2) at line 16",
            "6",
        ]
    );
}

// ---------- MDH-BUG-FLU-00132: chunks() with a huge size must not overflow ----------

#[test]
fn chunks_with_a_huge_size_does_not_overflow() {
    let out = run_ok(
        r#"
blether chunks([1, 2, 3], 9223372036854775807)
blether chunks([1, 2, 3], 9223372036854775806)
blether chunks([1, 2, 3, 4, 5], 2)
blether chunks([1, 2, 3, 4], 2)
blether chunks([], 3)
blether chunks([1, 2, 3], 3)
"#,
    );
    assert_eq!(
        out,
        vec![
            "[[1, 2, 3]]",
            "[[1, 2, 3]]",
            "[[1, 2], [3, 4], [5]]",
            "[[1, 2], [3, 4]]",
            "[]",
            "[[1, 2, 3]]",
        ]
    );
}

// ---------- MDH-BUG-FLU-00135: a dae nested in a lambda reads the lambda's scope ----------

#[test]
fn dae_nested_in_a_lambda_reads_the_lambdas_scope() {
    let out = run_ok(
        r#"
ken f1 = |p| { ken out = p + 1  dae inner() { gie out }  gie out }
blether f1(5)
ken f2 = |p| { ken out = p + 1  dae inner() { gie out }  gie inner() }
blether f2(5)
ken f3 = |p| { dae inner() { gie p * 2 }  gie inner() }
blether f3(5)
dae outerfn() {
    ken z = 7
    ken f = |p| { dae inner() { gie z + p }  gie inner() }
    gie f(1)
}
blether outerfn()
ken g = 40
ken f4 = |p| { dae inner() { gie g + p }  gie inner() }
blether f4(2)
ken f5 = |p| {
    ken out = p + 1
    dae inner(q) {
        ken k = |x| x + out + q + p
        gie k(100)
    }
    gie inner(10)
}
blether f5(5)
dae mk() {
    ken base = 3
    ken f6 = |p| { ken c = 0  dae bump() { c = c + base + p  gie c }  bump()  gie bump() }
    gie f6(1)
}
blether mk()
dae a() {
    ken z = 9
    dae b() {
        dae c() { gie z + 1 }
        gie c()
    }
    gie b()
}
blether a()
"#,
    );
    assert_eq!(out, vec!["6", "6", "10", "8", "42", "121", "8", "10"]);
}

// ---------- MDH-BUG-FLU-00137: cannie is a validity check, not a trim ----------

#[test]
fn cannie_checks_validity_like_the_interpreter() {
    // Native used to compile cannie as wheesht (trim), so cannie(42) ran strlen on an
    // integer and segfaulted.
    let out = run_ok(
        r#"
blether cannie(42)
blether cannie(0)
blether cannie(-5)
blether cannie(2.5)
blether cannie(-0.5)
blether cannie("hello")
blether cannie("")
blether cannie([1])
blether cannie([])
blether cannie(naething)
blether cannie(aye)
blether cannie(nae)
blether cannie({"a": 1})
"#,
    );
    assert_eq!(
        out,
        vec![
            "aye", "aye", "nae", "aye", "nae", "aye", "nae", "aye", "nae", "nae", "aye", "nae",
            "aye"
        ]
    );
}

// ---------- MDH-BUG-FLU-00139: tak/drap take strings, as the prelude does ----------

#[test]
fn tak_and_drap_take_strings_as_character_lists() {
    // The interpreter's prelude tak/drap index their argument, so a string comes back as
    // a list of its characters (codepoints, not bytes).
    let out = run_ok(
        r#"
blether tak("Hello", 3)
blether tak("Hello", -1)
blether tak("Hello", 99)
blether tak("héllo", 2)
blether drap("Hello", 2)
blether drap("Hello", 99)
blether drap("héllo", 1)
blether tak([1, 2, 3], 2)
blether drap([1, 2, 3], 1)
"#,
    );
    assert_eq!(
        out,
        vec![
            "[H, e, l]",
            "[]",
            "[H, e, l, l, o]",
            "[h, é]",
            "[l, l, o]",
            "[]",
            "[é, l, l, o]",
            "[1, 2]",
            "[2, 3]",
        ]
    );
}

// ---------- MDH-BUG-FLU-00140: backside_forrit reverses lists ----------

#[test]
fn backside_forrit_reverses_lists() {
    let out = run_ok(
        r#"
blether backside_forrit([1, 2, 3])
blether backside_forrit([])
blether backside_forrit(["a", [1, 2]])
blether backside_forrit("hello")
"#,
    );
    assert_eq!(out, vec!["[3, 2, 1]", "[]", "[[1, 2], a]", "olleh"]);
}

// ---------- MDH-BUG-FLU-00141: a module's own function wins over a nested import's ----------

#[test]
fn module_function_shadowing_a_nested_imports_function_is_defined() {
    let dir = tempdir().expect("temp dir");
    std::fs::write(
        dir.path().join("inner.braw"),
        "dae rep(c, n) {\n    gie \"inner\"\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("outer.braw"),
        "fetch \"inner\"\ndae rep(c, n) {\n    gie c + \"outer\"\n}\ndae banner() {\n    gie rep(\"x\", 2)\n}\n",
    )
    .unwrap();
    let main_path = dir.path().join("main.braw");
    let source = "fetch \"outer\"\nblether banner()\n";
    std::fs::write(&main_path, source).unwrap();
    let program = parse(source).expect("parse");
    let exe_path = dir.path().join("exe");
    LLVMCompiler::new()
        .compile_to_native_with_source(&program, &exe_path, 2, Some(&main_path))
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "xouter");
}

// ---------- MDH-BUG-FLU-00142: prelude higher-order functions ----------

#[test]
fn prelude_higher_order_functions_match_the_interpreter() {
    let source = r#"ken nums = [1, 5, 10]
dae big(x) { gie x > 3 }
dae small(x) { gie x < 8 }
blether ilka(nums, big)
blether ilka(nums, nae_that(big))
dae add_one(x) { gie x + 1 }
dae double(x) { gie x * 2 }
ken f = compose(add_one, double)
blether f(5)
blether apply_n(double, 3, 1)
blether same(7)
ken c = constant(9)
blether c(1)
ken notsmall = nae_that(small)
blether notsmall(10)
blether notsmall(2)
"#;
    assert_eq!(
        run_ok(source),
        vec!["nae", "nae", "11", "8", "7", "9", "aye", "nae"]
    );
}

#[test]
fn prelude_functions_used_from_methods_lambdas_and_fstrings_are_found() {
    let source = r#"kin Holder {
    dae pick(x) {
        gie same(x)
    }
}
ken h = Holder()
blether h.pick(3)
ken twice = |v| apply_n(|n| n + 1, 2, v)
blether twice(5)
blether f"mak: {make_range(1, 3)}"
"#;
    assert_eq!(run_ok(source), vec!["3", "7", "mak: [1, 2, 3]"]);
}

#[test]
fn user_definition_shadows_a_compiled_in_prelude_function() {
    let source = "dae same(x) {\n    gie 42\n}\nblether same(1)\n";
    assert_eq!(run_ok(source), vec!["42"]);
}
