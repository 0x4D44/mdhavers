#![cfg(feature = "cli")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::tempdir;

fn mdhavers_bin() -> PathBuf {
    let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // `cargo llvm-cov` builds into `target/llvm-cov-target` and sets LLVM_PROFILE_FILE.
    // Use the instrumented binary in that mode so spawned subprocesses contribute to coverage.
    if std::env::var_os("LLVM_PROFILE_FILE").is_some() {
        let p = manifest_root.join("target/llvm-cov-target/debug/mdhavers");
        if p.exists() {
            return p;
        }
    }

    if let Some(p) = std::env::var_os("CARGO_BIN_EXE_mdhavers") {
        return PathBuf::from(p);
    }

    // Normal `cargo test` path.
    manifest_root.join("target/debug/mdhavers")
}

fn run_mdhavers_impl(
    args: &[&str],
    stdin: Option<&str>,
    home: &Path,
    cwd: Option<&Path>,
) -> (i32, String, String) {
    let mut cmd = Command::new(mdhavers_bin());
    cmd.args(args)
        .env("HOME", home)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

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

fn run_mdhavers(args: &[&str], stdin: Option<&str>, home: &Path) -> (i32, String, String) {
    run_mdhavers_impl(args, stdin, home, None)
}

fn run_mdhavers_in_dir(
    args: &[&str],
    stdin: Option<&str>,
    home: &Path,
    cwd: &Path,
) -> (i32, String, String) {
    run_mdhavers_impl(args, stdin, home, Some(cwd))
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
    if cfg!(feature = "llvm") {
        assert_eq!(code, 0, "stderr: {err}");
        assert!(ll_out.exists());
    } else {
        assert_ne!(code, 0);
        assert!(!ll_out.exists());
        assert!(err.contains("LLVM"), "stderr: {err}");
    }

    // build --emit-llvm (default output path)
    let default_ll = dir.path().join("ok.ll");
    let (code, _out, err) = run_mdhavers(
        &["build", ok_braw.to_str().unwrap(), "--emit-llvm", "-O", "0"],
        None,
        home,
    );
    if cfg!(feature = "llvm") {
        assert_eq!(code, 0, "stderr: {err}");
        assert!(default_ll.exists());
    } else {
        assert_ne!(code, 0);
        assert!(!default_ll.exists());
        assert!(err.contains("LLVM"), "stderr: {err}");
    }

    // build native executable (default output path)
    let native_out = dir.path().join("ok");
    let (code, _out, err) =
        run_mdhavers(&["build", ok_braw.to_str().unwrap(), "-O", "0"], None, home);
    if cfg!(feature = "llvm") {
        assert_eq!(code, 0, "stderr: {err}");
        assert!(native_out.exists());
    } else {
        assert_ne!(code, 0);
        assert!(!native_out.exists());
        assert!(err.contains("LLVM"), "stderr: {err}");
    }

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
    if cfg!(feature = "llvm") {
        assert!(err.contains("Cannae write"), "stderr: {err}");
    } else {
        assert!(err.contains("LLVM"), "stderr: {err}");
    }
}

#[test]
fn cli_parse_errors_for_each_subcommand_trigger_their_specific_formatting_paths() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let bad = dir.path().join("bad.braw");
    write_file(&bad, "ken =\n");

    for args in [
        vec!["run", bad.to_str().unwrap()],
        vec!["compile", bad.to_str().unwrap()],
        vec!["wasm", bad.to_str().unwrap()],
        vec!["trace", bad.to_str().unwrap()],
        vec!["build", bad.to_str().unwrap(), "-O", "0"],
    ] {
        let (code, _out, _err) = run_mdhavers(&args, None, home);
        assert_ne!(code, 0, "expected parse error for args={args:?}");
    }
}

#[test]
fn cli_can_surface_prelude_parse_errors_as_a_user_facing_error() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    write_file(&stdlib_dir.join("prelude.braw"), "ken =\n");

    let ok = dir.path().join("ok.braw");
    write_file(&ok, "blether 1\n");

    for args in [
        vec!["run", "ok.braw"],
        vec!["trace", "ok.braw"],
        vec!["trace", "--verbose", "ok.braw"],
    ] {
        let (code, _out, err) = run_mdhavers_in_dir(&args, None, home, dir.path());
        assert_ne!(code, 0, "expected prelude load failure for args={args:?}");
        assert!(
            err.contains("Error loading prelude"),
            "stderr should mention prelude error, got: {err}"
        );
    }
}

#[test]
fn cli_repl_vars_empty_message_is_reachable_when_prelude_fails_to_load() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    write_file(&stdlib_dir.join("prelude.braw"), "ken =\n");

    let script = ":vars\nquit\n";
    let (code, out, err) = run_mdhavers_in_dir(&["repl"], Some(script), home, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("Yer Variables"));
    assert!(out.contains("Nae variables defined yet"));
}

#[test]
fn cli_repl_vars_shows_user_float_values_for_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let script = "ken f = 1.5\n:vars\nquit\n";
    let (code, out, err) = run_mdhavers(&["repl"], Some(script), home);
    assert_eq!(code, 0, "stderr: {err}");
    assert!(
        out.contains("f : float"),
        "expected float variable in env, got stdout:\n{out}\nstderr:\n{err}"
    );
    assert!(out.contains("1.5"), "expected float value, got stdout:\n{out}");
}

#[test]
fn cli_trace_runtime_error_path_is_covered() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let runtime_error_braw = dir.path().join("runtime_error.braw");
    write_file(&runtime_error_braw, "blether 1 / 0\n");

    let (code, _out, _err) = run_mdhavers(&["trace", runtime_error_braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
}

#[test]
fn cli_build_native_reports_errors_when_output_is_a_directory() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let ok_braw = dir.path().join("ok.braw");
    write_file(&ok_braw, "ken x = 1\nblether x\n");

    let out_dir = dir.path().join("out_dir");
    fs::create_dir(&out_dir).unwrap();
    let out_dir = out_dir.to_str().unwrap();

    let (code, _out, err) = run_mdhavers(
        &[
            "build",
            ok_braw.to_str().unwrap(),
            "-O",
            "0",
            "--output",
            out_dir,
        ],
        None,
        home,
    );
    assert_ne!(code, 0);
    if cfg!(feature = "llvm") {
        assert!(!err.trim().is_empty());
    } else {
        assert!(err.contains("LLVM"), "stderr: {err}");
    }
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

#[test]
fn cli_run_and_trace_with_relative_paths_cover_empty_parent_branches() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    write_file(&dir.path().join("ok.braw"), "ken x = 41\nblether x + 1\n");

    let (code, out, err) = run_mdhavers_in_dir(&["run", "ok.braw"], None, home, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out.trim(), "42");

    let (code, _out, err) = run_mdhavers_in_dir(&["trace", "ok.braw"], None, home, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
}

#[test]
fn cli_check_and_tokens_surface_lexer_errors_and_parse_suggestions() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let bad_lex = dir.path().join("bad_lex.braw");
    write_file(&bad_lex, "@\n");

    let (code, _out, err) = run_mdhavers(&["check", bad_lex.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
    assert!(err.contains("Ah dinnae ken"), "stderr: {err}");

    let (code, _out, err) = run_mdhavers(&["tokens", bad_lex.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
    assert!(err.contains("Ah dinnae ken"), "stderr: {err}");

    // Trigger a parse error that has a helpful suggestion.
    let bad_parse = dir.path().join("bad_parse.braw");
    write_file(&bad_parse, "fetch )\n");

    let (code, _out, err) = run_mdhavers(&["check", bad_parse.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
    assert!(err.contains("Check yer brackets"), "stderr: {err}");
}

#[test]
fn cli_ast_and_fmt_parse_error_paths_are_covered() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let bad_syntax = dir.path().join("bad_syntax.braw");
    write_file(&bad_syntax, "ken =\n");

    let (code, _out, _err) = run_mdhavers(&["ast", bad_syntax.to_str().unwrap()], None, home);
    assert_ne!(code, 0);

    let (code, _out, _err) = run_mdhavers(&["fmt", "--check", bad_syntax.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
}

#[cfg(unix)]
#[test]
fn cli_fmt_write_error_path_is_covered() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let home = dir.path();

    let read_only = dir.path().join("read_only.braw");
    write_file(&read_only, "ken x=1\n");

    fs::set_permissions(&read_only, fs::Permissions::from_mode(0o444)).unwrap();

    let (code, _out, err) = run_mdhavers(&["fmt", read_only.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
    assert!(err.contains("Cannae write"), "stderr: {err}");
}

#[test]
fn cli_build_emit_llvm_error_path_is_covered() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let bad_llvm = dir.path().join("bad_llvm.braw");
    write_file(
        &bad_llvm,
        r#"
ken x = 1
x.foo(1,2,3,4,5,6,7,8,9)
"#,
    );

    let (code, _out, err) = run_mdhavers(
        &[
            "build",
            bad_llvm.to_str().unwrap(),
            "--emit-llvm",
            "-O",
            "0",
        ],
        None,
        home,
    );
    if cfg!(feature = "llvm") {
        assert_ne!(code, 0);
        assert!(
            err.contains("up to 8 arguments"),
            "stderr should mention arg limit, got: {err}"
        );
    } else {
        assert_ne!(code, 0);
        assert!(err.contains("LLVM"), "stderr: {err}");
    }
}

#[test]
fn cli_repl_multiline_cancel_and_prelude_warning_paths_are_covered() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    write_file(&stdlib_dir.join("prelude.braw"), "ken =\n");

    let script = [
        ":vars",
        "gin aye {",
        "",
        "help",
        ":trace",
        ":trace verbose",
        ":cancel",
        ":cancel",
        "# comment",
        "blether \"a\\\\b\"",
        "}",
        "ken x = [1, (2)]",
        ":reset",
        "quit",
    ]
    .join("\n")
        + "\n";

    let (code, out, err) = run_mdhavers_in_dir(&["repl"], Some(&script), home, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("mdhavers REPL"));
}

#[cfg(unix)]
#[test]
fn cli_repl_readline_interrupt_error_path_is_covered() {
    use std::io::{Read, Write};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    let dir = tempdir().unwrap();
    let home = dir.path();

    // Run the REPL under a pseudo-tty so sending ^C triggers ReadlineError::Interrupted rather than
    // being interpreted as a literal input character.
    let repl_cmd = format!("{} repl", mdhavers_bin().display());
    let mut cmd = Command::new("script");
    cmd.args(["-q", "-c", &repl_cmd, "/dev/null"])
        .env("HOME", home)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("spawn script");
    let mut child_stdin = child.stdin.take().expect("stdin");
    let child_stdout = child.stdout.take().expect("stdout");
    let child_stderr = child.stderr.take().expect("stderr");

    let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
    let stdout_handle = thread::spawn(move || {
        let mut r = child_stdout;
        let mut buf = [0u8; 4096];
        loop {
            match r.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = stdout_tx.send(buf[..n].to_vec());
                }
                Err(_) => break,
            }
        }
    });

    let (stderr_tx, stderr_rx) = mpsc::channel::<Vec<u8>>();
    let stderr_handle = thread::spawn(move || {
        let mut r = child_stderr;
        let mut buf = [0u8; 4096];
        loop {
            match r.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let _ = stderr_tx.send(buf[..n].to_vec());
                }
                Err(_) => break,
            }
        }
    });

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let start = Instant::now();
    loop {
        if String::from_utf8_lossy(&stdout).contains("mdhavers>") {
            break;
        }
        if start.elapsed() > Duration::from_secs(5) {
            while let Ok(chunk) = stderr_rx.try_recv() {
                stderr.extend_from_slice(&chunk);
            }
            panic!(
                "timed out waiting for REPL prompt; stdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
        }
        match stdout_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(chunk) => stdout.extend_from_slice(&chunk),
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // Send Ctrl-C to trigger ReadlineError::Interrupted and continue the loop.
    child_stdin.write_all(&[0x03]).expect("write ^C");
    child_stdin.flush().expect("flush ^C");

    let start = Instant::now();
    loop {
        if String::from_utf8_lossy(&stdout).contains("Interrupted! Use 'quit' tae leave.") {
            break;
        }
        if start.elapsed() > Duration::from_secs(5) {
            while let Ok(chunk) = stderr_rx.try_recv() {
                stderr.extend_from_slice(&chunk);
            }
            panic!(
                "timed out waiting for interrupt message; stdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
        }
        match stdout_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(chunk) => stdout.extend_from_slice(&chunk),
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    child_stdin.write_all(b"quit\n").expect("write quit");
    child_stdin.flush().expect("flush quit");
    drop(child_stdin);

    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            break status;
        }
        if start.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            panic!("timed out waiting for repl to exit");
        }
        thread::sleep(Duration::from_millis(10));
    };

    assert_eq!(status.code().unwrap_or(-1), 0);

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    while let Ok(chunk) = stdout_rx.try_recv() {
        stdout.extend_from_slice(&chunk);
    }
    while let Ok(chunk) = stderr_rx.try_recv() {
        stderr.extend_from_slice(&chunk);
    }

    let stdout = String::from_utf8_lossy(&stdout);
    assert!(
        stdout.contains("Interrupted! Use 'quit' tae leave."),
        "stdout missing interrupted message:\n{stdout}"
    );
}
