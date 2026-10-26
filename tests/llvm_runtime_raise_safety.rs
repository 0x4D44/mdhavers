#![cfg(feature = "llvm")]
//! Native backend: runtime errors raised from the Rust runtime leave it in a usable state
//! (MDH-BUG-FLU-00183).

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
    let exe_path = dir.path().join("raise_safety_exe");
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
