#![cfg(feature = "llvm")]
//! Native backend: runtime errors raised from the Rust runtime leave it in a usable state
//! (MDH-BUG-FLU-00183), and running out of memory while building a string raises the
//! "ower muckle" error instead of crashing (MDH-BUG-FLU-00184).

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_native(source: &str, refcounting: bool) -> Run {
    run_native_with_env(source, refcounting, &[])
}

fn run_native_with_env(source: &str, refcounting: bool, env: &[(&str, &str)]) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("raise_safety_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .envs(env.iter().copied())
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// An audio error used to be raised while the audio state was borrowed, which the
/// longjmp then left borrowed: the next audio call panicked and aborted the program.
#[test]
fn a_caught_audio_error_leaves_audio_usable() {
    let src = r#"fer k in 0..3 {
    hae_a_bash {
        soond_luid("lood")
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether "efter"
"#;
    for refcounting in [false, true] {
        let run = run_native(src, refcounting);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(lines.len(), 4, "rc={refcounting}: {lines:?}");
        assert_eq!(lines[3], "efter");
        if cfg!(feature = "audio") {
            assert_eq!(lines[0], "soond_luid needs a nummer");
        }
        assert!(
            lines[..3].iter().all(|l| *l == lines[0]),
            "rc={refcounting}: {lines:?}"
        );
    }
}

const TOO_BIG: &str = "Och! That string would be ower muckle tae fit in memory";

/// With the test-only knob of debug builds (runtime/gc_stub.c), every string allocation
/// of 1 MiB or more fails. Formatting a big list (C formatter, raising) and the JSON of a
/// big creel (formatted for the Rust runtime in quiet mode, reported as a status) both
/// used to write through the NULL and crash.
#[cfg(debug_assertions)]
#[test]
fn running_out_of_memory_formatting_raises_the_too_big_error() {
    let src = r#"ken big = []
fer i in 0..200000 {
    shove(big, i)
}
fer k in 0..2 {
    hae_a_bash {
        blether len(tae_string(big))
    } gin_it_gangs_wrang e {
        blether e
    }
    hae_a_bash {
        blether len(json_stringify([creel(big)]))
    } gin_it_gangs_wrang e {
        blether e
    }
}
blether tae_string([1, 2, 3])
"#;
    for refcounting in [false, true] {
        let run = run_native_with_env(src, refcounting, &[("MDH_TEST_ALLOC_FAIL_OVER", "1048576")]);
        assert_eq!(run.code, Some(0), "rc={refcounting}: {}", run.stderr);
        let lines: Vec<&str> = run.stdout.lines().collect();
        assert_eq!(
            lines,
            vec![TOO_BIG, TOO_BIG, TOO_BIG, TOO_BIG, "[1, 2, 3]"],
            "rc={refcounting}"
        );
    }
}
