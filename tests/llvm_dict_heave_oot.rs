#![cfg(feature = "llvm")]
//! Native backend: `heave_oot(dict, key)` deletes in place (MDH-BUG-FLU-00091 rest), so a
//! remove-heavy loop is linear, and with reference counting on it releases the key and value.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    elapsed: Duration,
}

fn run_native(source: &str, refcounting: bool, envs: &[(&str, &str)]) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("heave_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let mut cmd = Command::new(&exe_path);
    cmd.stdin(Stdio::null());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let start = Instant::now();
    let output = cmd.output().expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        elapsed: start.elapsed(),
    }
}

#[test]
fn heave_oot_on_a_dict_removes_a_hundred_thousand_keys_in_linear_time() {
    // dict_remove copies the dict each time (quadratic: 100k x 100k entries); heave_oot
    // leaves a tombstone, so this finishes in a fraction of a second.
    let src = r#"
ken d = {}
ken i = 0
whiles i < 100000 {
    d["k" + tae_string(i)] = i
    i = i + 1
}
i = 0
whiles i < 100000 {
    heave_oot(d, "k" + tae_string(i))
    i = i + 1
}
blether len(d)
d["back"] = 1
blether keys(d)
"#;
    let run = run_native(src, false, &[]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "0\n[back]\n");
    assert!(
        run.elapsed < Duration::from_secs(10),
        "100k in-place deletes took {:?}",
        run.elapsed
    );
}

#[test]
fn heave_oot_on_a_dict_releases_the_removed_key_and_value() {
    let src = r#"
ken d = {}
ken i = 0
whiles i < 200 {
    d["key" + tae_string(i)] = ["val" + tae_string(i), {"in": "x" + tae_string(i)}]
    i = i + 1
}
i = 0
whiles i < 150 {
    heave_oot(d, "key" + tae_string(i))
    i = i + 1
}
heave_oot(d, "missing" + "")
blether len(d)
d = naething
"#;
    let run = run_native(src, true, &[("MDH_RC_REPORT", "1")]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, "50\n");
    assert!(run.stderr.contains("total=0"), "{}", run.stderr);
}
