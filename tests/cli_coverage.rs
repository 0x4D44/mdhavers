#![cfg(feature = "cli")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::tempdir;

fn mdhavers_bin() -> PathBuf {
    // `cargo llvm-cov` builds into `target/llvm-cov-target` and sets LLVM_PROFILE_FILE.
    // Use the instrumented binary in that mode so spawned subprocesses contribute to coverage.
    if std::env::var_os("LLVM_PROFILE_FILE").is_some() {
        let p = PathBuf::from("target/llvm-cov-target/debug/mdhavers");
        if p.exists() {
            return p;
        }
    }

    // Normal `cargo test` path.
    PathBuf::from("target/debug/mdhavers")
}

fn run_mdhavers(args: &[&str], stdin: Option<&str>, home: &Path) -> (i32, String, String) {
    let mut cmd = Command::new(mdhavers_bin());
    cmd.args(args)
        .env("HOME", home)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("spawn mdhavers");
    if let Some(input) = stdin {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("write stdin");
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let code = output.status.code().unwrap_or(-1);
    (
        code,
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write file");
}

#[test]
fn cli_help_and_version_work() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let (code, out, _err) = run_mdhavers(&["--help"], None, home);
    assert_eq!(code, 0);
    assert!(out.contains("mdhavers"));

    let (code, out, _err) = run_mdhavers(&["--version"], None, home);
    assert_eq!(code, 0);
    assert!(out.trim().starts_with("mdhavers"));
}

#[test]
fn cli_subcommands_cover_success_and_error_paths() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let ok_braw = dir.path().join("ok.braw");
    write_file(
        &ok_braw,
        r#"
ken x = 41
blether x + 1
"#,
    );

    let unformatted_braw = dir.path().join("unformatted.braw");
    write_file(&unformatted_braw, "ken x=1\nblether x\n");

    let bad_syntax_braw = dir.path().join("bad_syntax.braw");
    write_file(&bad_syntax_braw, "ken =\n");

    let runtime_error_braw = dir.path().join("runtime_error.braw");
    write_file(&runtime_error_braw, "blether 1 / 0\n");

    // run (explicit subcommand)
    let (code, out, err) = run_mdhavers(&["run", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out.trim(), "42");

    // run (file arg without subcommand)
    let (code, out, err) = run_mdhavers(&[ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out.trim(), "42");

    // check
    let (code, out, err) = run_mdhavers(&["check", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("Lexing passed"));
    assert!(out.contains("Parsing passed"));

    // fmt --check success (already formatted after we overwrite)
    write_file(&ok_braw, "ken x = 1\nblether x\n");
    let (code, out, _err) = run_mdhavers(&["fmt", "--check", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0);
    assert!(out.contains("already formatted"));

    // fmt --check error path
    let (code, _out, _err) = run_mdhavers(
        &["fmt", "--check", unformatted_braw.to_str().unwrap()],
        None,
        home,
    );
    assert_ne!(code, 0);

    // tokens
    let (code, out, err) = run_mdhavers(&["tokens", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("Tokens:"));

    // ast
    let (code, out, err) = run_mdhavers(&["ast", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("AST:"));

    // trace (non-verbose)
    let (code, _out, err) = run_mdhavers(&["trace", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");

    // wasm output
    let wasm_out = dir.path().join("out.wat");
    let (code, _out, err) = run_mdhavers(
        &[
            "wasm",
            ok_braw.to_str().unwrap(),
            "--output",
            wasm_out.to_str().unwrap(),
        ],
        None,
        home,
    );
    assert_eq!(code, 0, "stderr: {err}");
    assert!(wasm_out.exists());

    // build --emit-llvm output
    let ll_out = dir.path().join("out.ll");
    let (code, _out, err) = run_mdhavers(
        &[
            "build",
            ok_braw.to_str().unwrap(),
            "--emit-llvm",
            "-O",
            "0",
            "--output",
            ll_out.to_str().unwrap(),
        ],
        None,
        home,
    );
    assert_eq!(code, 0, "stderr: {err}");
    assert!(ll_out.exists());

    // parse error path
    let (code, _out, _err) = run_mdhavers(&["check", bad_syntax_braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);

    // runtime error path
    let (code, _out, _err) = run_mdhavers(&["run", runtime_error_braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);

    // extension warning path (non-.braw)
    let txt = dir.path().join("warn.txt");
    write_file(&txt, "blether 1\n");
    let (code, _out, err) = run_mdhavers(&["check", txt.to_str().unwrap()], None, home);
    // still should parse as source text (extension warning only)
    assert_eq!(code, 0, "stderr: {err}");
}

#[test]
fn cli_repl_scripted_session_exits_cleanly() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let script = [
        "help",
        ":wisdom",
        ":codewisdom",
        ":examples",
        ":trace",
        ":trace verbose",
        "ken x = 41",
        "x + 1",
        ":vars",
        ":reset",
        "quit",
    ]
    .join("\n")
        + "\n";

    let (code, out, err) = run_mdhavers(&["repl"], Some(&script), home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("mdhavers REPL"));
    assert!(out.contains("mdhavers Help"));
}

#[test]
fn cli_repl_handles_eof() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    // No input: causes EOF on first readline -> clean exit path.
    let (code, out, err) = run_mdhavers(&["repl"], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("mdhavers REPL"));
}
