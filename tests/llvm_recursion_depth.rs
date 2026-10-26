#![cfg(feature = "llvm")]
//! Native backend: unbounded recursion raises the interpreter's catchable "Stack's fair
//! puggled!" error instead of crashing, on the main thread and on spawned threads
//! (MDH-BUG-FLU-00164).

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_native(source: &str, refcounting: bool) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("recursion_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

const PUGGLED: &str =
    "Stack's fair puggled! Too many nested calls at line 1 - yer recursion's gone radge!";

#[test]
fn unbounded_recursion_raises_the_scots_error_and_exits_1() {
    for refcounting in [false, true] {
        let run = run_native(
            "dae r(n) { gie 1 + r(n + 1) }\nblether \"start\"\nblether r(0)\n",
            refcounting,
        );
        assert_eq!(run.code, Some(1), "rc={refcounting}: {}", run.stderr);
        assert_eq!(run.stdout.trim(), "start");
        assert!(
            run.stderr.contains(PUGGLED),
            "rc={refcounting}: {}",
            run.stderr
        );
    }
}

#[test]
fn the_error_is_catchable_again_and_again() {
    let src = r#"dae r(n) { gie 1 + r(n + 1) }
dae s(n) {
    ken label = "deep " + tae_string(n)
    gie len(label) + s(n + 1)
}
fer k in 0..3 {
    hae_a_bash {
        blether r(0)
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        blether s(0)
    } gin_it_gangs_wrang e {
        blether "caught s"
    }
}
blether "after"
"#;
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        let mut expected = Vec::new();
        for _ in 0..3 {
            expected.push(PUGGLED);
            expected.push("caught s");
        }
        expected.push("after");
        assert_eq!(lines, expected, "rc={refcounting}");
    }
}

#[test]
fn a_spawned_thread_has_its_own_limit() {
    let src = r#"dae r(n) { gie 1 + r(n + 1) }
ken t = thread_spawn(|| {
    hae_a_bash {
        r(0)
    } gin_it_gangs_wrang e {
        gie "thread caught"
    }
}, [])
blether thread_join(t)
ken u = thread_spawn(|| r(0), [])
hae_a_bash {
    thread_join(u)
} gin_it_gangs_wrang e {
    blether "join raised"
}
"#;
    let run = run_native(src, false);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout.lines().collect::<Vec<_>>(),
        vec!["thread caught", "join raised"]
    );
}

#[test]
fn deep_but_finite_recursion_still_works() {
    let run = run_native(
        "dae d(n) {\n    gin n == 0 { gie 0 }\n    gie 1 + d(n - 1)\n}\nblether d(5000)\n",
        false,
    );
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout.trim(), "5000");
}

#[test]
fn formatting_a_too_deeply_nested_list_raises_the_error() {
    // The runtime formats nested values recursively: it stops with the stack error at a
    // checked headroom, from normal code, rather than running into the tripwire.
    let src = r#"ken l = []
fer i in 0..300000 {
    l = [l]
}
hae_a_bash {
    blether l
} gin_it_gangs_wrang e {
    blether "caught: " + e
}
hae_a_bash {
    blether json_stringify(l)
} gin_it_gangs_wrang e {
    blether "json caught: " + e
}
blether "after"
blether l
"#;
    let run = run_native(src, false);
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    let puggled = |line: usize| {
        format!(
            "Stack's fair puggled! Too many nested calls at line {line} - yer recursion's gone radge!"
        )
    };
    assert_eq!(
        run.stdout.lines().collect::<Vec<_>>(),
        vec![
            format!("caught: {}", puggled(6)),
            format!("json caught: {}", puggled(11)),
            "after".to_string()
        ]
    );
    assert!(run.stderr.contains(&puggled(16)), "{}", run.stderr);
}

#[test]
fn json_of_a_too_deeply_nested_key_or_fallback_value_raises_the_error() {
    // json_stringify (Rust) formats a non-string dict key, or a value JSON has no form
    // for, with the C formatter: running too deep there comes back to Rust as an error,
    // not a raise across its frames.
    let src = r#"ken l = []
fer i in 0..300000 {
    l = [l]
}
ken d = {}
d["a"] = "x"
d[l] = 1
ken s = empty_creel()
toss_in(s, l)
ken small = {}
small[[1, [2]]] = [empty_creel()]
blether json_stringify(small)
fer i in 0..3 {
    hae_a_bash {
        blether json_stringify(d)
    } gin_it_gangs_wrang e {
        blether "key caught: " + e
    }
    hae_a_bash {
        blether json_stringify({"s": s})
    } gin_it_gangs_wrang e {
        blether "value caught: " + e
    }
}
blether json_stringify(small)
"#;
    let puggled = |line: usize| {
        format!(
            "Stack's fair puggled! Too many nested calls at line {line} - yer recursion's gone radge!"
        )
    };
    let small = r#"{"[1, [2]]": ["creel{}"]}"#.to_string();
    let mut expected = vec![small.clone()];
    for _ in 0..3 {
        expected.push(format!("key caught: {}", puggled(15)));
        expected.push(format!("value caught: {}", puggled(20)));
    }
    expected.push(small);
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        assert_eq!(
            run.stdout.lines().collect::<Vec<_>>(),
            expected,
            "rc={refcounting}"
        );
    }
}

#[test]
fn recursion_that_calls_the_runtime_only_now_and_then_raises_the_error() {
    // Most levels call nothing out of compiled code; every k-th calls a runtime
    // function from wherever the stack has got to.
    let src = r#"dae r(n, k) {
    gin n % k == 0 {
        ken s = tae_string(n) + "!"
    }
    gie 1 + r(n + 1, k)
}
fer k in [1000, 997, 991, 983, 977, 971, 967, 953, 947, 941, 7, 3] {
    hae_a_bash {
        blether r(0, k)
    } gin_it_gangs_wrang e {
        blether "caught " + tae_string(k)
    }
}
blether "after"
"#;
    let ks = [1000, 997, 991, 983, 977, 971, 967, 953, 947, 941, 7, 3];
    let mut expected: Vec<String> = ks.iter().map(|k| format!("caught {k}")).collect();
    expected.push("after".to_string());
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        assert_eq!(
            run.stdout.lines().collect::<Vec<_>>(),
            expected,
            "rc={refcounting}"
        );
    }
}

#[test]
fn a_function_with_a_frame_bigger_than_the_tripwire_raises_the_error() {
    // 400 hae_a_bash sites reserve about 200 KiB of handler buffers in one frame, more
    // than the tripwire: the frame is probed page by page, so it cannot step over it.
    let body: String = (0..400)
        .map(|i| format!("    hae_a_bash {{ x = x + {i} }} gin_it_gangs_wrang e{i} {{ x = 0 }}\n"))
        .collect();
    let src = format!(
        "dae big(n) {{\n    ken x = n\n{body}    gie 1 + big(n + 1)\n}}\n\
         hae_a_bash {{\n    blether big(0)\n}} gin_it_gangs_wrang e {{\n    blether \"big caught\"\n}}\n\
         blether \"after\"\n"
    );
    let run = run_native(&src, false);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout.lines().collect::<Vec<_>>(),
        vec!["big caught", "after"]
    );
}

/// Preloaded, so it runs before main: a SIGSEGV/SIGBUS action chosen by PRIOR_MODE (f: an
/// SA_SIGINFO handler, i: SIG_IGN, d: SIG_DFL; m and n: an SA_NODEFER handler whose mask
/// does (m) or does not (n) hold its own signals, which reports whether its signal is
/// blocked; r: a one-shot SA_RESETHAND handler that returns, so the fault is taken again),
/// and an exit hook that, once the runtime has put its own handler in front of it, faults
/// (f, m, n), faults on four threads at once (r) or raises SIGSEGV.
#[cfg(unix)]
const PRIOR_HANDLER_C: &str = r#"
#include <pthread.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
static char mode = 'f';
static volatile int calls = 0;
static void prior(int sig, siginfo_t *info, void *ctx) {
    (void)sig; (void)ctx;
    calls++;
    char msg[] = "prior N\n";
    msg[6] = (char)('0' + calls % 10);
    write(2, msg, sizeof msg - 1);
    _exit(info->si_code > 0 ? 42 : 43);
}
static void prior_masked(int sig, siginfo_t *info, void *ctx) {
    (void)info; (void)ctx;
    sigset_t now;
    pthread_sigmask(SIG_BLOCK, NULL, &now);
    if (sigismember(&now, sig)) {
        write(2, "blocked\n", 8);
    } else {
        write(2, "unblocked\n", 10);
    }
    _exit(42);
}
static int one_shot_calls = 0;
static void prior_one_shot(int sig, siginfo_t *info, void *ctx) {
    (void)sig; (void)info; (void)ctx;
    int n = __atomic_add_fetch(&one_shot_calls, 1, __ATOMIC_SEQ_CST);
    write(2, "one-shot\n", 9);
    if (n >= 4) {
        _exit(44);
    }
    /* Long enough for every other thread to fault meanwhile; then the fault repeats. */
    struct timespec ts = {0, 200 * 1000 * 1000};
    nanosleep(&ts, NULL);
}
/* macOS has no pthread barriers. */
static int ready = 0;
static void *fault_thread(void *arg) {
    (void)arg;
    __atomic_add_fetch(&ready, 1, __ATOMIC_SEQ_CST);
    while (__atomic_load_n(&ready, __ATOMIC_SEQ_CST) < 4) {
    }
    *(volatile int *)(uintptr_t)8 = 1;
    return NULL;
}
static void fault_at_exit(void) {
    fflush(stdout);
    if (mode == 'r') {
        pthread_t t[4];
        for (int i = 0; i < 4; i++) {
            pthread_create(&t[i], NULL, fault_thread, NULL);
        }
        for (int i = 0; i < 4; i++) {
            pthread_join(t[i], NULL);
        }
        write(2, "survived\n", 9);
        _exit(7);
    }
    if (mode == 'f' || mode == 'm' || mode == 'n') {
        *(volatile int *)(uintptr_t)8 = 1;
    }
    raise(SIGSEGV);
    write(2, "survived\n", 9);
    _exit(7);
}
__attribute__((constructor)) static void setup(void) {
    const char *m = getenv("PRIOR_MODE");
    if (m) mode = m[0];
    struct sigaction sa;
    memset(&sa, 0, sizeof sa);
    sigemptyset(&sa.sa_mask);
    if (mode == 'i') {
        sa.sa_handler = SIG_IGN;
    } else if (mode == 'd') {
        sa.sa_handler = SIG_DFL;
    } else if (mode == 'm' || mode == 'n') {
        sa.sa_sigaction = prior_masked;
        sa.sa_flags = SA_SIGINFO | SA_NODEFER;
        if (mode == 'm') {
            sigaddset(&sa.sa_mask, SIGSEGV);
            sigaddset(&sa.sa_mask, SIGBUS);
        }
    } else if (mode == 'r') {
        sa.sa_sigaction = prior_one_shot;
        sa.sa_flags = SA_SIGINFO | SA_RESETHAND;
    } else {
        sa.sa_sigaction = prior;
        sa.sa_flags = SA_SIGINFO;
    }
    sigaction(SIGSEGV, &sa, NULL);
    sigaction(SIGBUS, &sa, NULL);
    atexit(fault_at_exit);
}
"#;

#[cfg(unix)]
#[test]
fn signals_that_are_not_a_stack_overflow_get_the_prior_action() {
    use std::os::unix::process::ExitStatusExt;

    let dir = tempdir().expect("temp dir");
    let c_path = dir.path().join("prior.c");
    let lib_path = dir.path().join("libprior.so");
    std::fs::write(&c_path, PRIOR_HANDLER_C).expect("write C");
    let cc = Command::new("cc")
        .args(["-shared", "-fPIC", "-pthread", "-o"])
        .arg(&lib_path)
        .arg(&c_path)
        .status()
        .expect("run cc");
    assert!(cc.success());
    let program = parse("blether \"main done\"\n").expect("parse");
    let exe_path = dir.path().join("prior_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let preload = if cfg!(target_os = "macos") {
        "DYLD_INSERT_LIBRARIES"
    } else {
        "LD_PRELOAD"
    };
    let run = |mode: &str| {
        let out = Command::new(&exe_path)
            .env(preload, &lib_path)
            .env("PRIOR_MODE", mode)
            .stdin(Stdio::null())
            .output()
            .expect("run executable");
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        assert_eq!(String::from_utf8_lossy(&out.stdout), "main done\n");
        (out.status, stderr)
    };

    // A real fault goes to a prior SA_SIGINFO handler, with its siginfo.
    let (status, stderr) = run("f");
    assert_eq!(status.code(), Some(42), "{stderr}");
    assert_eq!(stderr, "prior 1\n");

    // A SIGSEGV sent under a prior SIG_IGN stays ignored. macOS gives a sent SIGSEGV a
    // fault's siginfo; on arm64 the runtime tells them apart by the thread's exception
    // syndrome (a system call, here raise). Intel Macs cannot: the default action.
    let (status, stderr) = run("i");
    if cfg!(all(target_os = "macos", not(target_arch = "aarch64"))) {
        assert_eq!(status.signal(), Some(11), "{status:?} {stderr}");
        assert_eq!(stderr, "");
    } else {
        assert_eq!(status.code(), Some(7), "{status:?} {stderr}");
        assert_eq!(stderr, "survived\n");
    }

    // Sent under a prior SIG_DFL, it takes the default action rather than vanishing.
    let (status, stderr) = run("d");
    assert_eq!(status.signal(), Some(11), "{status:?} {stderr}");
    assert_eq!(stderr, "");

    // A prior SA_NODEFER handler runs with its signal unblocked, unless its own mask
    // blocks it: the mask wins, as when the kernel runs the handler (MDH-BUG-FLU-00186).
    let (status, stderr) = run("m");
    assert_eq!(status.code(), Some(42), "{status:?} {stderr}");
    assert_eq!(stderr, "blocked\n");
    let (status, stderr) = run("n");
    assert_eq!(status.code(), Some(42), "{status:?} {stderr}");
    assert_eq!(stderr, "unblocked\n");

    // A one-shot (SA_RESETHAND) prior handler, with four threads faulting at once: one
    // delivery claims it and every other takes the default action, so the handler runs
    // once and the process dies of SIGSEGV. Whether two claims could race is not
    // observable from here; the claim is an atomic exchange, checked lock-free at
    // compile time. macOS never reports SA_RESETHAND back (a permanent limit, see
    // __mdh_stack_chain), so there the handler runs on every delivery: here until its
    // fourth call exits 44.
    let (status, stderr) = run("r");
    if cfg!(target_os = "macos") {
        assert_eq!(status.code(), Some(44), "{status:?} {stderr}");
        assert_eq!(stderr, "one-shot\n".repeat(4));
    } else {
        // The default action may kill the process before the claiming thread writes.
        assert_eq!(status.signal(), Some(11), "{status:?} {stderr}");
        assert!(stderr == "one-shot\n" || stderr.is_empty(), "{stderr}");
    }
}
