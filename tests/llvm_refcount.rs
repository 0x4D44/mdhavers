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
fn closures_cells_and_bound_methods_die_with_their_last_holder() {
    // Closures, cells and bound methods counted: a counter over a mutable capture,
    // closures in lists and dicts, a bound method outliving its variable.
    // Default kinds: no MDH_RC_KINDS, so this also checks the default counts them.
    let src = r#"
dae counter() {
    ken n = 0
    ken bump = || {
        n = n + 1
        gie "c" + tae_string(n)
    }
    gie bump
}
ken c = counter()
c()
c()
blether c()
c = naething
dae make_adder(n) {
    ken base = "b" + tae_string(n)
    gie |x| base + x
}
ken adders = []
fer k in 0..4 {
    shove(adders, make_adder(k))
}
blether adders[2]("z")
adders = naething
ken fns = {"twice": |x| x + x, "neg": |x| 0 - x}
blether fns["twice"]("ab")
fns = naething
kin Greeter {
    dae init(name) {
        masel.name = name
    }
    dae greet(who) {
        gie masel.name + " greets " + who
    }
}
ken g = Greeter("Hamish" + "")
ken bound = g.greet
g = naething
blether bound("Morag" + "")
ken bounds = [bound, bound]
bound = naething
blether bounds[1]("Ailsa")
bounds = naething
ken label = "lbl" + ""
ken shout = || label + "!"
label = "changed" + ""
blether shout()
shout = naething
"#;
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "c3\nb2z\nabab\nHamish greets Morag\nHamish greets Ailsa\nchanged!\n"
    );
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "dict"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "instance"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "closure"), 0, "{}", run.stderr);
    assert_eq!(
        reported_live(&run.stderr, "bound method"),
        0,
        "{}",
        run.stderr
    );
}

#[test]
fn bytes_die_with_their_last_holder() {
    // Bytes counted (every kind is now): appends, slices, sets, bytes in a list.
    // Default kinds: no MDH_RC_KINDS, so this also checks the default counts them.
    let src = r#"
ken by = bytes_from_string("hello" + "")
bytes_append(by, by)
blether bytes_len(by)
ken by2 = bytes_slice(by, 1, 4)
by = naething
blether bytes_len(by2)
blether bytes_get(by2, 0)
bytes_set(by2, 0, 65)
blether bytes_get(by2, 0)
ken keep = [by2, bytes(3)]
by2 = naething
blether bytes_len(keep[0]) + bytes_len(keep[1])
keep = naething
ken i = 0
ken total = 0
whiles i < 50 {
    ken b = bytes_from_string("x" + tae_string(i))
    total = total + bytes_len(b)
    i = i + 1
}
blether total
"#;
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "10\n3\n101\n65\n6\n140\n");
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "bytes"), 0, "{}", run.stderr);
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
    assert!(
        run.stderr.contains("(counted kinds 0x0, freed kinds 0x0)"),
        "{}",
        run.stderr
    );
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

#[test]
fn showing_a_float_and_splitting_on_a_long_delimiter_leak_nothing() {
    // MDH-BUG-FLU-00150: blether of a float and split's multi-character path each left
    // strings live at exit.
    let src = r#"
blether 1.5
ken parts = split("a--b", "--")
blether len(parts)
ken single = split("a,b,c", ",")
blether len(single)
ken shown = str(2.25)
blether shown
"#;
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "1.5\n2\n3\n2.25\n");
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
}

#[test]
fn mixed_concatenation_and_read_lines_leak_nothing() {
    // Found by the golden sweep for MDH-BUG-FLU-00150: string + non-string made a shown
    // copy nobody released, and read_lines dropped the file text it slurped.
    let src = r#"
blether "a" + 1
blether 1 + "a"
blether [1, 2] + "x"
blether "x" + [1, 2]
blether "n=" + naething
ken text = "ab" + "cd"
text = text + 7
blether text
ken rows = read_lines("tests/golden/fixtures/lines_fixture.txt")
blether len(rows)
"#;
    let run = run_native(src, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(run
        .stdout
        .starts_with("a1\n1a\n[1, 2]x\nx[1, 2]\nn=naething\nabcd7\n"));
    assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
    assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
}

#[test]
fn a_million_deep_nested_list_is_freed_without_recursing() {
    // Stage R2 (DES F6 3.9): letting go of a 10^6-deep nest frees it in a loop, not by
    // recursion, in FREE mode and in the R1 checker (MDH_RC_NOFREE=1) alike.
    let src = r#"
ken xs = ["leaf" + ""]
ken i = 0
whiles i < 1000000 {
    xs = [xs]
    i = i + 1
}
xs = naething
blether "done"
"#;
    for (envs, freed) in [
        (
            &[("MDH_RC_REPORT", "1"), ("MDH_RC_FREE_KINDS", "0x1FE")][..],
            "0xfe",
        ),
        (&[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..], "0x0"),
    ] {
        let run = run_native(src, envs);
        assert_eq!(run.code, Some(0), "{}", run.stderr);
        assert_eq!(run.stdout, "done\n");
        assert!(
            run.stderr.contains(&format!("freed kinds {freed})")),
            "{}",
            run.stderr
        );
        assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
        assert_eq!(reported_live(&run.stderr, "string"), 0, "{}", run.stderr);
    }
}

#[test]
fn thread_spawn_releases_its_function_and_arguments() {
    // MDH-BUG-FLU-00152: the thread record kept func and args for good. A closure and
    // argument lists, all joined (a detached worker releases when its function returns, which
    // can be after exit, so it is not counted on here): none stay live at exit.
    let src = r#"
dae sum_list(list) {
    gie sumaw(list)
}
dae bump(counter, delta) {
    atomic_add(counter, delta)
    gie atomic_load(counter)
}
ken base = 5
ken adder = |x| x + base
ken handles = []
fer t in [[1, 2, 3], [10, 20], [7, 8, 9]] {
    shove(handles, thread_spawn(sum_list, [t]))
}
ken total = 0
fer h in handles {
    total = total + thread_join(h)
}
blether total
ken counter = atomic_new(0)
ken h1 = thread_spawn(bump, [counter, 3])
ken h2 = thread_spawn(adder, [4])
ken h3 = thread_spawn(bump, [counter, 7])
thread_join(h1)
blether thread_join(h2)
thread_join(h3)
ken h4 = thread_spawn(bump, [counter, 1])
thread_join(h4)
"#;
    for _ in 0..5 {
        for envs in [
            &[("MDH_RC_REPORT", "1")][..],
            &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
        ] {
            let run = run_native(src, envs);
            assert_eq!(run.code, Some(0), "{}", run.stderr);
            assert_eq!(run.stdout, "60\n9\n");
            assert_eq!(reported_live(&run.stderr, "list"), 0, "{}", run.stderr);
            assert_eq!(reported_live(&run.stderr, "closure"), 0, "{}", run.stderr);
        }
    }
}

/// R4: a program that never spawns a thread has no thread checks left under counting
/// (every read of `__mdh_line_threaded` folds to 0, and shared-slot calls become plain
/// loads and stores); a program that spawns keeps them.
#[test]
fn a_program_without_threads_carries_no_thread_checks() {
    let ir = |src: &str| {
        LLVMCompiler::new()
            .with_refcounting(true)
            .compile_to_ir(&parse(src).expect("parse"))
            .expect("compile to IR")
    };
    let single = ir(r#"
ken g = "a"
dae f(n) {
    g = g + "b"
    gie len(g) + n
}
ken i = 0
whiles i < 3 {
    ken k = f(i)
    i = i + 1
}
blether g
"#);
    assert!(
        !single.contains("load i32, ptr @__mdh_line_threaded"),
        "{single}"
    );
    assert!(
        !single.contains("call void @__mdh_shared_store"),
        "{single}"
    );
    assert!(
        !single.contains("call { i8, i64 } @__mdh_shared_load("),
        "{single}"
    );
    let threaded = ir(r#"
ken g = "a"
dae w() {
    g = "b"
    gie 0
}
thread_join(thread_spawn(w, []))
blether g
"#);
    assert!(
        threaded.contains("load i32, ptr @__mdh_line_threaded"),
        "{threaded}"
    );
}

/// R4: objects freed at zero are reused for later allocations of their size class, and
/// the cache is dropped at the first thread_spawn: counts stay exact either side of it.
#[test]
fn reused_objects_keep_counts_exact_across_the_first_spawn() {
    let src = r#"
dae churn(n) {
    ken total = 0
    ken i = 0
    whiles i < n {
        ken l = [i, tae_string(i), [i]]
        ken s = "x" + tae_string(i)
        s = s + "yy"
        ken f = |x| x + i
        total = total + len(l) + len(s) + f(1) - i
        i = i + 1
    }
    gie total
}
blether churn(2000)
ken t = thread_spawn(churn, [500])
blether thread_join(t)
blether churn(1000)
"#;
    for envs in [
        &[("MDH_RC_REPORT", "1")][..],
        &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
    ] {
        let run = run_native(src, envs);
        assert_eq!(run.code, Some(0), "{}", run.stderr);
        assert_eq!(run.stdout, "20890\n4890\n9890\n");
        for kind in ["string", "list", "closure"] {
            assert_eq!(reported_live(&run.stderr, kind), 0, "{}", run.stderr);
        }
    }
}

/// R5 borrowing (`wrk_docs/2026.10.26 - DES - RC borrowing.md`): a value read from a
/// holder keeps its reference whenever something could drop the holder's reference before
/// the value's last use: compiled code that reassigns a global or empties a list, an
/// element overwritten in the same expression, a variable reassigned in the same
/// statement. The functions allocate right after letting go, so a string freed too early
/// is reused and the output changes; with MDH_RC_NOFREE=1 a retain or release of a dead
/// object aborts.
#[test]
fn borrowed_values_keep_a_reference_while_their_holder_can_let_go() {
    let src = r#"
ken g = ["a" + "b", "c"]
dae clobber() {
    g = ["z"]
    ken j = "x" + "y"
    gie len(j) - 1
}
dae f(x, n) {
    gie x + tae_string(n)
}
blether f(g[0], clobber())
ken h = "p" + "q"
dae reset() {
    h = "r" + "s"
    ken j = "x" + "y"
    gie len(j) - 2
}
blether h + tae_string(reset())
ken ys = ["d" + "e"]
blether ys[0] + (ys[0] = "f" + "g")
dae local(p) {
    ken x = p + "!"
    ken y = x + (x = "w" + p)
    gie y + x
}
blether local("v")
ken zs = ["m" + "n"]
blether zs[0] + tae_string(len(clear(zs)))
"#;
    for envs in [
        &[("MDH_RC_REPORT", "1")][..],
        &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
    ] {
        let run = run_native(src, envs);
        assert_eq!(run.code, Some(0), "{}", run.stderr);
        assert_eq!(
            run.stdout, "ab1\npq0\ndefg\nv!wvwv\nmn0\n",
            "{}",
            run.stderr
        );
        for kind in ["string", "list"] {
            assert_eq!(reported_live(&run.stderr, kind), 0, "{}", run.stderr);
        }
    }
}

/// R5 borrowing: in a loop that only reads, the element, the left operand and the
/// variable read for its own update take no reference of their own.
#[test]
fn reads_used_up_before_anything_can_drop_them_take_no_reference() {
    let src = r#"
dae total(xs) {
    ken t = 0
    ken i = 0
    whiles i < len(xs) {
        t = t + xs[i] * 2
        i = i + 1
    }
    gie t
}
blether total([1, 2, 3])
"#;
    let ir = LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_ir(&parse(src).expect("parse"))
        .expect("compile to IR");
    let body = ir
        .split("\ndefine ")
        .find(|f| {
            f.lines()
                .next()
                .is_some_and(|l| l.contains("@__mdh_fn_total("))
        })
        .expect("total's body");
    let retains = body.matches("call void @__mdh_retain_fast(").count()
        + body.matches("call void @__mdh_rc_retain_obj_fast(").count();
    assert!(!body.contains("__mdh_rc_borrow_retain"), "{body}");
    // Only the copies `+` makes when it joins two lists, and the returned value's own
    // reference, remain (trunk: 11).
    assert_eq!(retains, 3, "{body}");
    let run = run_native(src, &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "12\n");
}

/// R6: `xs[i] = xs[i] + 1` reads the local `xs` and, on its dict path, stores the
/// updated dict back into `xs`. The runtime's dict and list setters only borrow `xs`, so
/// the read takes no reference of its own; the dict path still keeps its counts exact.
#[test]
fn an_element_update_takes_no_reference_to_its_container() {
    let src = r#"
dae bump(xs) {
    ken i = 0
    whiles i < len(xs) {
        xs[i] = xs[i] + 1
        i = i + 1
    }
    gie xs
}
dae bumpd(d, keys) {
    ken i = 0
    whiles i < len(keys) {
        d[keys[i]] = d[keys[i]] + 1
        i = i + 1
    }
    gie d
}
ken xs = [1, 2, 3]
bump(xs)
blether xs
ken d = {"a" + "b": 1, "c" + "d": 2}
bumpd(d, ["a" + "b", "c" + "d", "a" + "b"])
blether d["ab"] + d["cd"]
"#;
    let ir = LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_ir(&parse(src).expect("parse"))
        .expect("compile to IR");
    let body = ir
        .split("\ndefine ")
        .find(|f| {
            f.lines()
                .next()
                .is_some_and(|l| l.contains("@__mdh_fn_bump("))
        })
        .expect("bump's body");
    let retains = body.matches("call void @__mdh_retain_fast(").count()
        + body.matches("call void @__mdh_rc_retain_obj_fast(").count();
    // The per-element retain of `xs` is gone; the rest are on cold paths (`+` joining
    // lists or strings) and the returned value's own reference (trunk: 6).
    assert_eq!(retains, 5, "{body}");
    for envs in [
        &[("MDH_RC_REPORT", "1")][..],
        &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
    ] {
        let run = run_native(src, envs);
        assert_eq!(run.code, Some(0), "{}", run.stderr);
        assert_eq!(run.stdout, "[2, 3, 4]\n6\n", "{}", run.stderr);
        for kind in ["string", "list", "dict"] {
            assert_eq!(reported_live(&run.stderr, kind), 0, "{}", run.stderr);
        }
    }
}

/// Retains (`__mdh_retain_fast`, `__mdh_rc_retain_obj_fast`) in compiled function `name`.
fn retains_in(src: &str, name: &str) -> usize {
    let ir = LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_ir(&parse(src).expect("parse"))
        .expect("compile to IR");
    let head = format!("@__mdh_fn_{name}(");
    let body = ir
        .split("\ndefine ")
        .find(|f| f.lines().next().is_some_and(|l| l.contains(&head)))
        .unwrap_or_else(|| panic!("{name}'s body"));
    body.matches("call void @__mdh_retain_fast(").count()
        + body.matches("call void @__mdh_rc_retain_obj_fast(").count()
}

/// Runs `src` with MDH_RC_REPORT=1, then also MDH_RC_NOFREE=1: `expected` output, exit 0,
/// nothing live at exit, no use of a dead object.
fn assert_exact_run(src: &str, expected: &str) {
    for envs in [
        &[("MDH_RC_REPORT", "1")][..],
        &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
    ] {
        let run = run_native(src, envs);
        assert_eq!(run.code, Some(0), "{envs:?}: {}", run.stderr);
        assert_eq!(run.stdout, expected, "{envs:?}: {}", run.stderr);
        let total = run
            .stderr
            .lines()
            .find(|l| l.starts_with("mdhavers rc: live at exit:"))
            .and_then(|l| l.split(" total=").nth(1))
            .and_then(|t| t.split(' ').next())
            .unwrap_or_else(|| panic!("no rc report: {}", run.stderr));
        assert_eq!(total, "0", "{}", run.stderr);
    }
}

/// R6 rule (a): `chynge` has an ownership row, so passing it a value loaded from `xs`
/// cannot drop `xs`'s reference; the read of `xs` takes none (two more retains when every
/// call passed a loaded value counts as touching the slot).
#[test]
fn a_runtime_call_that_borrows_a_local_leaves_its_read_borrowed() {
    let src = r#"
dae ins(xs) {
    ken i = 0
    whiles i < 100 {
        xs = chynge(xs, len(xs), "k" + tae_string(i))
        i = i + 1
    }
    gie len(xs)
}
blether ins(["a" + "b"])
"#;
    assert_eq!(retains_in(src, "ins"), 3);
    assert_exact_run(src, "101\n");
}

/// R6 rule (b) stress: `x = x[1]` walks a 500-deep chain whose only holder is `x`; each
/// store drops the old head. (Rule (b) alone changes no retain here; the element-update
/// test above isolates it.)
#[test]
fn walking_a_chain_through_its_only_holder_frees_nothing_early() {
    let src = r#"
dae build(n) {
    ken l = naething
    ken i = 0
    whiles i < n {
        l = ["s" + tae_string(i), l]
        i = i + 1
    }
    gie l
}
dae drain(n) {
    ken x = build(n)
    ken k = 0
    whiles x != naething {
        k = k + len(x[0])
        x = x[1]
    }
    gie k
}
blether drain(500)
"#;
    assert_eq!(retains_in(src, "drain"), 6);
    assert_exact_run(src, "1890\n");
}

/// R6: creel insert and removal (`toss_in`, `heave_oot`: rows `ms` and `mb`) on a global
/// creel keep counts exact.
#[test]
fn creel_insert_and_removal_keep_counts_exact() {
    let src = r#"
ken s = creel(["a" + "b"])
ken i = 0
whiles i < 50 {
    toss_in(s, "k" + tae_string(i))
    heave_oot(s, "k" + tae_string(i - 1))
    i = i + 1
}
blether len(s)
"#;
    assert_exact_run(src, "2\n");
}

/// R6: after the optimiser, each tag read of a merged value gets its own phi of the
/// incoming tags (`rc_split_tag_phis`), so heap tests fold where every path is a number.
/// Values that are a number on some paths and a fresh string or list on others must still
/// be released exactly once, on the paths that made them.
#[test]
fn values_that_are_numbers_on_some_paths_release_the_rest_exactly() {
    let src = r#"
dae mix(n) {
    ken acc = 0
    ken s = 0.5
    ken i = 0
    whiles i < n {
        ken a = i * 2
        ken b = a - s
        gin i % 7 == 3 {
            a = "k" + tae_string(i)
        }
        gin i % 11 == 5 {
            a = [i, "z" + tae_string(i)]
        }
        ken c = a + a
        gin i % 7 == 3 {
            acc = acc + len(c)
        } ither gin i % 11 == 5 {
            acc = acc + len(c) + len(c[1])
        } ither {
            acc = acc + c + b
        }
        i = i + 1
    }
    gie acc
}
blether mix(200)
"#;
    for envs in [
        &[("MDH_RC_REPORT", "1")][..],
        &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
    ] {
        let run = run_native(src, envs);
        assert_eq!(run.code, Some(0), "{}", run.stderr);
        assert_eq!(run.stdout, "93483\n", "{}", run.stderr);
        for kind in ["string", "list"] {
            assert_eq!(reported_live(&run.stderr, kind), 0, "{}", run.stderr);
        }
    }
}

/// R5 list-buffer cache: item buffers of dead lists are reused for new lists of the same
/// capacity; lists that grow, shrink and nest keep their contents and counts exact.
#[test]
fn reused_list_buffers_keep_contents_and_counts_exact() {
    let src = r#"
dae churn(n) {
    ken total = 0
    ken i = 0
    whiles i < n {
        ken a = [i]
        ken b = [i, i + 1, tae_string(i)]
        ken c = []
        ken j = 0
        whiles j < i % 40 {
            shove(c, [j, tae_string(j)])
            j = j + 1
        }
        ken f = |x| x + i
        total = total + len(a) + len(b) + len(c) + f(0) + b[1] - i
        gin len(c) > 3 {
            total = total + c[3][0] + len(c[len(c) - 1][1])
        }
        i = i + 1
    }
    gie total
}
blether churn(3000)
"#;
    for envs in [
        &[("MDH_RC_REPORT", "1")][..],
        &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
    ] {
        let run = run_native(src, envs);
        assert_eq!(run.code, Some(0), "{}", run.stderr);
        assert_eq!(run.stdout, "4584975\n", "{}", run.stderr);
        for kind in ["string", "list", "closure"] {
            assert_eq!(reported_live(&run.stderr, kind), 0, "{}", run.stderr);
        }
    }
}

/// `run_native` for a main program that `fetch`es sibling modules: `files` are written
/// next to it first.
fn run_native_with_modules(source: &str, files: &[(&str, &str)], envs: &[(&str, &str)]) -> Run {
    let dir = tempdir().expect("temp dir");
    for (name, text) in files {
        std::fs::write(dir.path().join(name), text).expect("write module");
    }
    let main_path = dir.path().join("main.braw");
    std::fs::write(&main_path, source).expect("write main");
    let program = parse(source).expect("parse");
    let exe_path = dir.path().join("rc_exe");
    LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_native_with_source(&program, &exe_path, 2, Some(&main_path))
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

/// Checks a borrowing probe (MDH-BUG-FLU-00196): the interpreter's output, nothing live at
/// exit, and with MDH_RC_NOFREE=1 no read of an object that died early (debug builds check
/// concatenation, index and field reads for DEAD objects).
fn assert_probe(run: impl Fn(&[(&str, &str)]) -> Run, expected: &str) {
    for envs in [
        &[("MDH_RC_REPORT", "1")][..],
        &[("MDH_RC_REPORT", "1"), ("MDH_RC_NOFREE", "1")][..],
    ] {
        let run = run(envs);
        assert_eq!(run.code, Some(0), "{envs:?}: {}", run.stderr);
        assert_eq!(run.stdout, expected, "{envs:?}: {}", run.stderr);
        for kind in ["string", "list", "closure"] {
            assert_eq!(reported_live(&run.stderr, kind), 0, "{}", run.stderr);
        }
    }
}

/// An assignment's value is borrowed from the element it stored; the call on the right
/// overwrites that element, so the left operand must keep its own reference.
#[test]
fn an_assigned_element_survives_a_call_that_overwrites_it() {
    let src = r#"
ken ys = ["q" + "r"]
dae zap() {
    ys[0] = "z" + "z"
    ken j = "x" + "y"
    gie j
}
blether (ys[0] = "a" + "b") + zap()
blether ys[0]
"#;
    assert_probe(|envs| run_native(src, envs), "abxy\nzz\n");
}

/// A local captured by a lambda lives in a cell the lambda can write: the value read for
/// `s + inner()` must survive `inner` overwriting the cell.
#[test]
fn a_captured_local_survives_a_lambda_that_overwrites_it() {
    let src = r#"
dae outer() {
    ken s = "a" + "b"
    ken inner = || {
        s = "c" + "d"
        ken j = "x" + "y"
        gie j
    }
    s = s + inner()
    gie s
}
blether outer()
"#;
    assert_probe(|envs| run_native(src, envs), "abxy\n");
}

/// A fetched module's global read before a call to the module's function that writes it.
#[test]
fn a_fetched_global_survives_a_call_that_overwrites_it() {
    let module = r#"
ken mg = "m" + "g"
dae setmg() {
    mg = "n" + "h"
    ken j = "x" + "y"
    gie j
}
"#;
    let src = "fetch \"m\"\nblether mg + setmg()\nblether mg\n";
    assert_probe(
        |envs| run_native_with_modules(src, &[("m.braw", module)], envs),
        "mgxy\nnh\n",
    );
}

// RC7 A: a callee may let go of a heap holder while the caller still borrows it.
#[test]
fn rc7_a1_a_closure_can_remove_its_own_list_holder() {
    let src = r#"
dae install(fs) {
    ken label = "a" + "b"
    shove(fs, || {
        yank(fs)
        gie label + "!"
    })
}
dae run() {
    ken fs = []
    install(fs)
    blether fs[0]()
    blether len(fs)
}
run()
"#;
    assert_exact_run(src, "ab!\n0\n");
}

#[test]
fn rc7_a2_a_closure_can_replace_its_global_holder() {
    let src = r#"
ken held = naething
dae install() {
    ken label = "a" + "b"
    held = || {
        held = naething
        gie label + "!"
    }
}
install()
blether held()
blether held
"#;
    assert_exact_run(src, "ab!\nnaething\n");
}

#[test]
fn rc7_a3_a_plain_element_store_preserves_the_callers_old_value() {
    let src = r#"
dae overwrite(xs) {
    xs[0] = "c" + "d"
    gie "x" + "y"
}
dae run() {
    ken xs = ["a" + "b"]
    blether xs[0] + overwrite(xs)
    blether xs[0]
}
run()
"#;
    assert_exact_run(src, "abxy\ncd\n");
}

#[test]
fn rc7_a4_heap_drops_propagate_through_two_calls() {
    let src = r#"
dae mutate(xs) {
    xs[0] = "c" + "d"
    gie "x" + "y"
}
dae intermediate(xs) { gie mutate(xs) }
dae wrapper(xs) {
    gie intermediate(xs)
}
dae run() {
    ken xs = ["a" + "b"]
    blether xs[0] + wrapper(xs)
    blether xs[0]
}
run()
"#;
    assert_exact_run(src, "abxy\ncd\n");
}

#[test]
fn rc7_a5_heap_drops_propagate_through_mutual_recursion() {
    let src = r#"
dae first(xs, n) {
    gin n == 0 { gie "x" + "y" }
    gie bridge(xs, n - 1)
}
dae bridge(xs, n) { gie second(xs, n) }
dae second(xs, n) {
    gin n == 0 {
        xs[0] = "c" + "d"
        gie "x" + "y"
    }
    gie first(xs, n - 1)
}
dae run() {
    ken xs = ["a" + "b"]
    blether xs[0] + first(xs, 3)
    blether xs[0]
}
run()
"#;
    assert_exact_run(src, "abxy\ncd\n");
}

#[test]
fn rc7_a6_a_method_preserves_the_callers_borrowed_field() {
    let src = r#"
kin Box {
    dae init() { masel.text = "a" + "b" }
    dae overwrite() {
        masel.text = "c" + "d"
        gie "x" + "y"
    }
}
dae run() {
    ken box = Box()
    blether box.text + box.overwrite()
    blether box.text
}
run()
"#;
    assert_exact_run(src, "abxy\ncd\n");
}

#[test]
fn rc7_a7_a_late_bound_mutator_preserves_a_borrowed_element() {
    let src = r#"
dae wrapper(xs) { gie later(xs) }
dae later(xs) {
    xs[0] = "c" + "d"
    gie "x" + "y"
}
dae run() {
    ken xs = ["a" + "b"]
    blether xs[0] + wrapper(xs)
    blether xs[0]
}
run()
"#;
    assert_exact_run(src, "abxy\ncd\n");
}

/// A-8 substitutes a logging callback for sort's unavailable comparator argument.
/// Native code has no log_set_callback builtin: log_init's callback sink installs the
/// same C callback as __mdh_log_set_callback. The log statement calls __mdh_log, whose
/// __mdh_call_with_list runs user code (runtime/mdh_runtime.c), rather than just printing.
#[test]
fn rc7_a8_a_runtime_callback_preserves_the_callers_old_element() {
    let src = r#"
dae logged() {
    log_blether "mutate"
    gie "x" + "y"
}
dae run() {
    ken xs = ["a" + "b"]
    ken overwrite = |record| {
        xs[0] = "c" + "d"
    }
    log_init({"sinks": [{"kind": "callback", "fn": overwrite}]})
    blether xs[0] + logged()
    blether xs[0]
    log_init()
}
run()
"#;
    assert_exact_run(src, "abxy\ncd\n");
}

#[test]
fn rc7_a9_a_captured_cell_write_preserves_its_old_value() {
    let src = r#"
dae run() {
    ken text = "a" + "b"
    ken overwrite = || {
        text = "c" + "d"
        gie "x" + "y"
    }
    blether text + overwrite()
    blether text
}
run()
"#;
    assert_exact_run(src, "abxy\ncd\n");
}

const RC7_CLOSURE_FN: &str = r#"
dae adder(n) { gie |x| x + n }
dae run() {
    ken fs = []
    ken i = 0
    whiles i < 10 {
        shove(fs, adder(i))
        i = i + 1
    }
    ken s = 0
    ken r = 0
    whiles r < 100 {
        ken f = fs[r % 10]
        s = s + f(r)
        r = r + 1
    }
    gie s
}
blether run()
"#;

const RC7_A_CLEAN_CALL: &str = r#"
dae callee(xs) { gie "xy" }
dae run() {
    ken xs = ["a" + "b"]
    blether xs[0] + callee(xs)
    blether xs[0]
}
run()
"#;

/// The first element is used after callee returns. Its own retain and temp park
/// must agree: without the park its statement-end release sees only nil and folds.
fn assert_rc7_element_pair(src: &str, retained: bool) {
    let ir = LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_ir(&parse(src).expect("parse"))
        .expect("IR");
    let body = ir
        .split("\ndefine ")
        .find(|body| {
            body.lines()
                .next()
                .is_some_and(|line| line.contains("@__mdh_fn_run("))
        })
        .expect("run body");
    assert!(
        body.contains("%elem_val_fast = load"),
        "the borrowed element must exist"
    );
    assert!(
        body.contains("call { i8, i64 } @__mdh_fn_callee("),
        "the direct call must exist"
    );
    assert_eq!(
        body.contains("call void @__mdh_retain_fast({ i8, i64 } %elem_val_fast)"),
        retained,
        "the element's retain must reflect the callee's heap-drop summary"
    );
    assert_eq!(
        body.contains("store { i8, i64 } %elem_val_fast,"),
        retained,
        "only a retained element may be parked for statement-end release"
    );
}

#[test]
fn rc7_a_clean_direct_call_elides_the_element_pair() {
    // Baseline: retain(elem_val_fast), park it, release the park after concat.
    // A: the clean direct callee cannot drop xs[0]'s holder; retain and park go.
    assert_rc7_element_pair(RC7_A_CLEAN_CALL, false);
    assert_exact_run(RC7_A_CLEAN_CALL, "abxy\nab\n");
}

#[test]
fn rc7_a_destructive_direct_call_keeps_the_element_pair() {
    let src = RC7_A_CLEAN_CALL.replace("gie \"xy\"", "xs[0] = \"c\" + \"d\"\n gie \"xy\"");
    assert_rc7_element_pair(&src, true);
    assert_exact_run(&src, "abxy\ncd\n");
}

#[test]
fn rc7_a10_a_spawn_reached_through_a_helper_keeps_borrows_owned() {
    let threaded_clean = format!(
        "{}\ndae worker() {{ gie 7 }}\ndae start() {{ gie thread_join(thread_spawn(worker, [])) }}\nblether start()\n",
        RC7_A_CLEAN_CALL
    );
    assert_rc7_element_pair(&threaded_clean, true);
    assert_exact_run(&threaded_clean, "abxy\nab\n7\n");
    let src = format!(
        "{}\ndae worker() {{ gie 7 }}\ndae start() {{ gie thread_join(thread_spawn(worker, [])) }}\nblether start()\n",
        RC7_CLOSURE_FN
    );
    assert!(
        retains_in(&src, "run") > 0,
        "a helper's spawn must disable borrow elision"
    );
    assert_exact_run(&src, "5400\n7\n");
}
