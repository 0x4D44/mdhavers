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
