//! Threading smoke test for LLVM runtime.

#![cfg(feature = "llvm")]

use std::process::Command;

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn compile_and_run(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;

    let dir = tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let exe_path = dir.path().join("thread_test_exe");

    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_native(&program, &exe_path, 2)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    let output = Command::new(&exe_path)
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Executable failed with exit code: {:?}, stderr: {}",
            output.status.code(),
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[test]
fn llvm_thread_spawn_and_atomic() {
    let out = compile_and_run(
        r#"
dae worker(a) {
    atomic_add(a, 1)
    gie naething
}

ken a = atomic_new(0)
ken t = thread_spawn(worker, [a])
thread_join(t)
blether atomic_load(a)
"#,
    )
    .expect("compile/run failed");
    assert_eq!(out.trim(), "1");
}

/// Compile `source` with reference counting on (MDH_RC=1) and run it `runs` times with
/// `envs`, returning each run's stdout.
fn compile_rc_and_run(source: &str, runs: usize, envs: &[(&str, &str)]) -> Vec<String> {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("thread_rc_test_exe");
    LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile");
    (0..runs)
        .map(|_| {
            let output = Command::new(&exe_path)
                .envs(envs.iter().copied())
                .stdin(std::process::Stdio::null())
                .output()
                .expect("run executable");
            assert!(
                output.status.success(),
                "exit {:?}, stderr: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8_lossy(&output.stdout).to_string()
        })
        .collect()
}

/// Stage R3: counts go atomic at the first thread_spawn. Six threads take and drop
/// references to the same strings, list, dict and closure, and push to one list under a
/// mutex. With plain counts, lost updates kill shared objects early, which
/// MDH_RC_NOFREE=1 turns into an abort on the next touch.
#[test]
fn llvm_threads_share_counted_values_under_refcounting() {
    let source = r#"
# Several threads share strings, a list, a dict, a closure and a channel under MDH_RC=1.
# Shared containers are only mutated under the mutex.
ken base = 5
ken bump = |x| x + base
ken greet = f"hullo-{base}"

dae worker(id, ctx, n) {
    ken shared = ctx[0]
    ken table = ctx[1]
    ken f = ctx[2]
    ken m = ctx[3]
    ken out = ctx[4]
    ken ch = ctx[5]
    ken total = 0
    fer i in 0..n {
        ken s = shared[i % 4]
        ken w = greet + s
        total = total + len(w)
        ken v = table["b"]
        total = total + len(v) + len(table)
        total = total + f(i % 3)
        ken pair = [s, v, w]
        total = total + len(pair)
        gin i % 500 == 0 {
            mutex_lock(m)
            shove(out, w)
            mutex_unlock(m)
        }
    }
    chan_send(ch, greet + f"-{id}")
    gie total
}

ken shared = []
fer i in 0..4 {
    shove(shared, f"item{i}")
}
ken table = {"a": [1, 2], "b": [shared[0], shared[1], 3]}
ken m = mutex_new()
ken out = []
ken ch = chan_new(16)
ken n = 20000
ken handles = []
fer t in 0..6 {
    shove(handles, thread_spawn(worker, [t, [shared, table, bump, m, out, ch], n]))
}
ken total = 0
fer h in handles {
    total = total + thread_join(h)
}
ken got = 0
whiles aye {
    ken v = chan_try_recv(ch)
    gin v == naething { brak }
    got = got + len(v)
}
blether f"total={total} out={len(out)} got={got} shared={shared[3]}"
"#;
    for out in compile_rc_and_run(source, 3, &[("MDH_RC_NOFREE", "1")]) {
        assert_eq!(out.trim(), "total=3119994 out=240 got=54 shared=item3");
    }
}

/// DEC round 3, R3 (striped lock): two threads reassign one global string while two
/// others read it, with no mutex. Each write releases the string it replaces, so a
/// reader that loaded the old value just before must already hold its own reference.
/// Without the slot lock this frees strings under readers (an abort or a crash).
#[test]
fn llvm_threads_reassign_shared_global_without_mutex_under_refcounting() {
    let source = r#"
ken g = "start"

dae writer(id, n) {
    fer i in 0..n {
        g = f"w{id}-{i}"
    }
    gie 0
}

dae reader(n) {
    ken seen = 0
    fer i in 0..n {
        ken s = g
        gin len(s) > 0 {
            seen = seen + 1
        }
    }
    gie seen
}

ken n = 20000
ken handles = [
    thread_spawn(writer, [1, n]),
    thread_spawn(reader, [n]),
    thread_spawn(writer, [2, n]),
    thread_spawn(reader, [n])
]
ken total = 0
fer h in handles {
    total = total + thread_join(h)
}
blether f"seen={total} last={len(g) > 0}"
"#;
    for envs in [&[][..], &[("MDH_RC_NOFREE", "1")][..]] {
        for out in compile_rc_and_run(source, 5, envs) {
            assert_eq!(out.trim(), "seen=40000 last=aye");
        }
    }
}

/// The same race through a captured variable: nested functions share a cell, and the
/// threads running them reassign and read it with no mutex.
#[test]
fn llvm_threads_reassign_captured_variable_without_mutex_under_refcounting() {
    let source = r#"
dae run(n) {
    ken c = "start"
    dae writer(id) {
        fer i in 0..n {
            c = f"w{id}-{i}"
        }
        gie 0
    }
    dae reader() {
        ken seen = 0
        fer i in 0..n {
            ken s = c
            gin len(s) > 0 {
                seen = seen + 1
            }
        }
        gie seen
    }
    ken handles = [
        thread_spawn(writer, [1]),
        thread_spawn(reader, []),
        thread_spawn(writer, [2]),
        thread_spawn(reader, [])
    ]
    ken total = 0
    fer h in handles {
        total = total + thread_join(h)
    }
    gie f"seen={total} last={len(c) > 0}"
}

blether run(20000)
"#;
    for envs in [&[][..], &[("MDH_RC_NOFREE", "1")][..]] {
        for out in compile_rc_and_run(source, 5, envs) {
            assert_eq!(out.trim(), "seen=40000 last=aye");
        }
    }
}

/// Build once with RC explicitly disabled, then collect every run before asserting:
/// the ARM publication race can produce a wrong checksum without crashing.
#[cfg(feature = "cli")]
fn assert_rc_off_thread_checksum(source: &str) {
    let scratch = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".scratch");
    std::fs::create_dir_all(&scratch).expect("create scratch directory");
    let dir = tempfile::tempdir_in(&scratch).expect("temp dir");
    let source_path = dir.path().join("shared_slot.braw");
    let exe_path = dir.path().join("shared_slot_exe");
    std::fs::write(&source_path, source).expect("write source");
    let build = Command::new("deltic")
        .args(["timeout", "30"])
        .arg(env!("CARGO_BIN_EXE_mdhavers"))
        .arg("build")
        .arg(&source_path)
        .args(["-O", "2", "-o"])
        .arg(&exe_path)
        .env("MDH_RC", "0")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("build native executable");
    assert!(
        build.status.success(),
        "build failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let runs = 10;
    let mut failures = Vec::new();
    for run in 1..=runs {
        let output = Command::new("deltic")
            .args(["timeout", "30"])
            .arg(&exe_path)
            .env("MDH_RC", "0")
            .stdin(std::process::Stdio::null())
            .output()
            .expect("run executable");
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() || stdout.trim() != "2800000" {
            failures.push(format!(
                "run {run}: status={}, stdout={stdout:?}, stderr={:?}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    eprintln!(
        "RC-off shared-slot checksum: {}/{runs} failed",
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{}/{runs} RC-off runs failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// MDH-BUG-FLU-00216: the original captured-string reproducer, without the
/// benchmark's interpreter compatibility wrapper.
#[cfg(feature = "cli")]
#[test]
fn llvm_threads_original_captured_string_repro_without_refcounting() {
    assert_rc_off_thread_checksum(
        r#"
dae run(n) {
    ken shared = "shared0"
    dae writer(count) {
        ken i = 0
        whiles i < count {
            shared = "shared" + tae_string(i % 10)
            i = i + 1
        }
        gie 0
    }
    dae reader(count) {
        ken total = 0
        ken i = 0
        whiles i < count {
            ken snapshot = shared
            total = total + len(snapshot)
            i = i + 1
        }
        gie total
    }
    ken a = thread_spawn(writer, [n])
    ken b = thread_spawn(reader, [n])
    ken c = thread_spawn(reader, [n])
    gie thread_join(a) + thread_join(b) + thread_join(c)
}
blether run(200000)
"#,
    );
}

#[cfg(feature = "cli")]
#[test]
fn llvm_threads_global_benchmark_without_refcounting() {
    assert_rc_off_thread_checksum(include_str!("../benchmarks/rc/thr_globals.braw"));
}

#[cfg(feature = "cli")]
#[test]
fn llvm_threads_captured_cell_benchmark_without_refcounting() {
    assert_rc_off_thread_checksum(include_str!("../benchmarks/rc/thr_cell.braw"));
}

/// Synchronizing RC-off shared slots must keep the no-spawn fast path free of
/// shared-slot helper calls, for both global bindings and captured cells.
#[test]
fn llvm_no_spawn_without_refcounting_has_no_shared_slot_calls() {
    let source = r#"
ken global = "start"
dae run() {
    ken captured = "start"
    dae update() {
        global = captured
        captured = global
        global = global + "g"
        captured = captured + "c"
        gie len(global) + len(captured)
    }
    gie update()
}
blether run()
"#;
    let ir = LLVMCompiler::new()
        .with_refcounting(false)
        .compile_to_ir(&parse(source).expect("parse"))
        .expect("compile to IR");
    for line in ir.lines().filter(|line| line.contains("call ")) {
        assert!(!line.contains("@__mdh_shared_"), "{line}");
    }
    let threaded_source =
        format!("{source}\ndae worker() {{ gie run() }}\nthread_join(thread_spawn(worker, []))\n");
    let threaded_ir = LLVMCompiler::new()
        .with_refcounting(false)
        .compile_to_ir(&parse(&threaded_source).expect("parse threaded variant"))
        .expect("compile threaded variant to IR");
    assert!(
        threaded_ir
            .lines()
            .any(|line| line.contains("call ") && line.contains("@__mdh_shared_")),
        "RC-off threaded code must synchronize shared slots"
    );
}
