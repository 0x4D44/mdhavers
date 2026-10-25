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
fn lists_and_their_items_die_with_their_last_holder() {
    // Lists counted (the default from stage R1 checkpoint 4): pushes, pops, index
    // stores, nested lists, slices, map/filter/fold and sort all let go of everything.
    // Default kinds: no MDH_RC_KINDS, so this also checks the default counts them.
    let src = r#"
dae mk(i) {
    gie ["a" + tae_string(i), [i, "b" + tae_string(i)], i]
}
ken xs = []
ken i = 0
whiles i < 40 {
    shove(xs, mk(i))
    i = i + 1
}
blether yank(xs)
yank(xs)
xs[0] = mk(100)
xs[1][1][1] = "replaced" + ""
blether xs[1]
ken ys = xs
xs = [mk(7)]
blether len(ys)
blether gaun(ys, |r| r[0] + "!")[3]
blether len(sieve(ys, |r| r[2] % 5 == 0))
blether tumble(ys, "", |acc, r| acc + r[0])[0:6]
ken total = 0
fer r in ys {
    total = total + r[2]
}
blether total
ken nested = [[["deep" + "est"]]]
ken inner = nested[0][0]
nested = naething
blether inner[0]
blether sort(["pear" + "", "apple" + "", "fig" + ""])
blether reverse([1, "two" + "", [3]])
ken sl = ys[2:5]
ys = naething
blether sl[0][0]
"#;
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "[a39, [39, b39], 39]\n[a1, [1, replaced], 1]\n38\na3!\n8\na100a1\n803\ndeepest\n[apple, fig, pear]\n[[3], two, 1]\na2\n");
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
}

#[test]
fn dicts_and_creels_release_removed_and_replaced_entries() {
    // Dicts and creels counted: overwritten values, removals (tombstones, then
    // compaction past half dead) and a creel churned through its index all let go.
    // Default kinds: no MDH_RC_KINDS, so this also checks the default counts them.
    let src = r#"
ken d = {}
ken i = 0
whiles i < 30 {
    d["k" + tae_string(i)] = ["v" + tae_string(i), {"inner": "x" + tae_string(i)}]
    i = i + 1
}
d["k3"] = "overwritten" + "!"
blether d["k3"]
blether d["k4"][1]["inner"]
i = 0
whiles i < 25 {
    d = dict_remove(d, "k" + tae_string(i))
    i = i + 1
}
blether keys(d)
fer k in keys(d) {
    blether k + "=" + tae_string(len(d[k]))
}
ken vals = values(d)
d = naething
blether vals[0][0]
ken e = {"a": "one" + "", "b": ["two" + ""]}
ken e2 = e
e["a"] = "uno" + ""
blether e2["a"]
ken c = creel(["x" + "", "y" + "", "z" + ""])
toss_in(c, "w" + "")
heave_oot(c, "x")
heave_oot(c, "y")
toss_in(c, "y" + "2")
toss_in(c, "w" + "")
blether len(c)
ken big = creel([])
i = 0
whiles i < 40 {
    toss_in(big, "item" + tae_string(i))
    i = i + 1
}
i = 0
whiles i < 35 {
    heave_oot(big, "item" + tae_string(i))
    toss_in(big, "again" + tae_string(i % 3))
    i = i + 1
}
blether len(big)
ken nested = {"l": [{"m": "deep" + ""}]}
ken got = nested["l"][0]
nested = {}
blether got["m"]
"#;
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "overwritten!\nx4\n[k25, k26, k27, k28, k29]\nk25=2\nk26=2\nk27=2\nk28=2\nk29=2\nv25\nuno\n3\n8\ndeep\n");
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "dict"), 0, "{}", run.stderr);
}

#[test]
fn instances_and_their_fields_die_with_their_last_holder() {
    // Instances counted: a chain of nodes, methods that return masel, inheritance,
    // instances held by dicts and lists, and a field store that drops the old one.
    // Default kinds: no MDH_RC_KINDS, so this also checks the default counts them.
    let src = r#"
kin Node {
    dae init(name, nxt) {
        masel.name = name
        masel.nxt = nxt
        masel.tags = []
    }
    dae tag(t) {
        shove(masel.tags, t)
        gie masel
    }
    dae rename(n) {
        masel.name = n + "!"
        gie masel
    }
    dae describe() {
        gie masel.name + tae_string(len(masel.tags))
    }
}
kin Leaf fae Node {
    dae init(name) {
        masel.name = name
        masel.nxt = naething
        masel.tags = ["leaf" + ""]
    }
}
ken head = naething
ken i = 0
whiles i < 20 {
    head = Node("n" + tae_string(i), head)
    i = i + 1
}
ken walk = head
ken count = 0
whiles walk != naething {
    count = count + 1
    walk = walk.nxt
}
blether count
blether head.tag("a" + "").tag("b" + "").rename("top").describe()
ken second = head.nxt
head = naething
blether second.name
ken lf = Leaf("leafy" + "")
blether lf.describe()
ken holder = {"obj": Node("held" + "", lf), "list": [Leaf("l2" + "")]}
lf = naething
blether holder["obj"].nxt.describe()
blether holder["list"][0].describe()
holder["obj"].nxt = Node("swapped" + "", naething)
blether holder["obj"].nxt.name
holder = naething
"#;
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "20\ntop!2\nn18\nleafy1\nleafy1\nl21\nswapped\n");
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "dict"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "instance"), 0, "{}", run.stderr);
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
