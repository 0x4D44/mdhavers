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
    let (code, out, _err) =
        run_mdhavers(&["fmt", "--check", ok_braw.to_str().unwrap()], None, home);
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

    // build --emit-llvm (default output path)
    let default_ll = dir.path().join("ok.ll");
    let (code, _out, err) = run_mdhavers(
        &["build", ok_braw.to_str().unwrap(), "--emit-llvm", "-O", "0"],
        None,
        home,
    );
    assert_eq!(code, 0, "stderr: {err}");
    assert!(default_ll.exists());

    // build native executable (default output path)
    let native_out = dir.path().join("ok");
    let (code, _out, err) =
        run_mdhavers(&["build", ok_braw.to_str().unwrap(), "-O", "0"], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(native_out.exists());

    // compile to JS (default output path)
    let default_js = dir.path().join("ok.js");
    let (code, _out, err) = run_mdhavers(&["compile", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(default_js.exists());

    // wasm (default output path)
    let default_wat = dir.path().join("ok.wat");
    let (code, _out, err) = run_mdhavers(&["wasm", ok_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(default_wat.exists());

    // trace (verbose)
    let (code, _out, err) = run_mdhavers(
        &["trace", "--verbose", ok_braw.to_str().unwrap()],
        None,
        home,
    );
    assert_eq!(code, 0, "stderr: {err}");

    // fmt without --check (writes file)
    let (code, _out, err) = run_mdhavers(&["fmt", unformatted_braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    let formatted = fs::read_to_string(&unformatted_braw).unwrap();
    assert!(formatted.contains("ken x = 1"));

    // parse error path
    let (code, _out, _err) =
        run_mdhavers(&["check", bad_syntax_braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);

    // runtime error path
    let (code, _out, _err) =
        run_mdhavers(&["run", runtime_error_braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);

    // extension warning path (non-.braw)
    let txt = dir.path().join("warn.txt");
    write_file(&txt, "blether 1\n");
    let (code, _out, err) = run_mdhavers(&["check", txt.to_str().unwrap()], None, home);
    // still should parse as source text (extension warning only)
    assert_eq!(code, 0, "stderr: {err}");
}

#[test]
fn cli_argument_errors_and_missing_inputs() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let (code, _out, _err) = run_mdhavers(&["run"], None, home);
    assert_ne!(code, 0);

    let (code, _out, _err) = run_mdhavers(&["compile"], None, home);
    assert_ne!(code, 0);

    let (code, _out, _err) = run_mdhavers(&["not-a-command"], None, home);
    assert_ne!(code, 0);

    let missing = dir.path().join("missing.braw");
    let (code, _out, err) = run_mdhavers(&["run", missing.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
    assert!(err.contains("Cannae read"));
}

#[test]
fn cli_write_failures_surface_errors() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let ok_braw = dir.path().join("ok.braw");
    write_file(&ok_braw, "ken x = 1\n");

    let out_dir = dir.path().join("out_dir");
    fs::create_dir(&out_dir).unwrap();
    let out_dir = out_dir.to_str().unwrap();

    let (code, _out, err) = run_mdhavers(
        &["compile", ok_braw.to_str().unwrap(), "--output", out_dir],
        None,
        home,
    );
    assert_ne!(code, 0);
    assert!(err.contains("Cannae write"));

    let (code, _out, err) = run_mdhavers(
        &["wasm", ok_braw.to_str().unwrap(), "--output", out_dir],
        None,
        home,
    );
    assert_ne!(code, 0);
    assert!(err.contains("Cannae write"));

    let (code, _out, err) = run_mdhavers(
        &[
            "build",
            ok_braw.to_str().unwrap(),
            "--emit-llvm",
            "-O",
            "0",
            "--output",
            out_dir,
        ],
        None,
        home,
    );
    assert_ne!(code, 0);
    assert!(err.contains("Cannae write"));
}

#[test]
fn cli_repl_scripted_session_exits_cleanly() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    // Seed a history file to hit load_history path.
    write_file(&home.join(".mdhavers_history"), "ken x = 1\n");

    let script = [
        "",
        "help",
        "clear",
        ":wisdom",
        ":codewisdom",
        ":examples",
        ":vars",
        ":trace",
        ":trace",
        ":trace verbose",
        "ken x = 41",
        "x + 1",
        "ken big = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"",
        ":vars",
        "ken =",
        "1 / 0",
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

#[test]
fn cli_no_args_starts_repl_and_accepts_quit() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let (code, out, err) = run_mdhavers(&[], Some("quit\n"), home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("mdhavers REPL"));
}

#[test]
fn cli_repl_history_save_error_path_is_non_fatal() {
    let dir = tempdir().unwrap();
    let home_as_file = dir.path().join("home_is_a_file");
    write_file(&home_as_file, "not a directory");
    let (code, out, err) = run_mdhavers(&["repl"], Some("quit\n"), &home_as_file);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("mdhavers REPL"));
}
