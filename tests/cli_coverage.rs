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

    // This helper owns and closes the child's stdin, including after piped input.
    #[cfg(coverage)]
    cmd.env("MDHAVERS_COVERAGE_ALLOW_STDIN", "1");

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
fn cli_repl_quits_cleanly_with_piped_input() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    // Exercise a few REPL command branches, then exit.
    let (code, _out, err) = run_mdhavers(
        &["repl"],
        Some("help\n:trace\n:trace v\n:vars\nquit\n"),
        home,
    );
    assert_eq!(code, 0, "stderr: {err}");
}

#[test]
#[cfg(coverage)]
fn cli_repl_history_falls_back_when_home_dir_is_unavailable_for_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path().join("home");
    fs::create_dir_all(&home).expect("create home");
    let cwd = dir.path().join("cwd");
    fs::create_dir_all(&cwd).expect("create cwd");

    let mut cmd = Command::new(mdhavers_bin());
    cmd.args(["repl"])
        .env("HOME", &home)
        .env("NO_COLOR", "1")
        .env("MDHAVERS_COVERAGE_HOME_DIR_NONE", "1")
        .current_dir(&cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("spawn mdhavers");
    {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(b"quit\n")
            .expect("write stdin");
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let code = output.status.code().unwrap_or(-1);
    let err = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(code, 0, "stderr: {err}");
}

#[test]
#[cfg(coverage)]
fn cli_repl_reports_editor_init_failure_for_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path().join("home");
    fs::create_dir_all(&home).expect("create home");

    let mut cmd = Command::new(mdhavers_bin());
    cmd.args(["repl"])
        .env("HOME", &home)
        .env("NO_COLOR", "1")
        .env("MDHAVERS_COVERAGE_EDITOR_NEW_ERR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().expect("run mdhavers");
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(err.contains("coverage DefaultEditor::new error"));
}

#[test]
#[cfg(coverage)]
fn cli_repl_reports_defaulteditor_new_io_err_arm_for_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path().join("home");
    fs::create_dir_all(&home).expect("create home");

    let mut cmd = Command::new(mdhavers_bin());
    cmd.args(["repl"])
        .env("HOME", &home)
        .env("NO_COLOR", "1")
        .env("MDHAVERS_COVERAGE_DEFAULTEDITOR_NEW_IO_ERR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd.output().expect("run mdhavers");
    let code = output.status.code().unwrap_or(-1);
    assert_eq!(code, 1);
    let err = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(err.contains("coverage forced DefaultEditor::new IO error"));
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

    let llvm_builtins_braw = dir.path().join("llvm_builtins.braw");
    write_file(
        &llvm_builtins_braw,
        r#"
blether abs(-42)
blether floor(3.2)
blether ceil(3.2)
blether round(3.7)
blether sqrt(16)

blether radians(180)
blether degrees(3.14159)

blether sumaw([1, 2, 3])
blether sumaw([1.0, 2.0, 3.5])

blether sin(0)
blether cos(0)
blether tan(0)
blether log(1)
blether log10(100)
blether exp(0)

blether pow(2, 3)
blether pooer(2, 3)
blether atan2(0, 1)

blether "done"
"#,
    );

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

    // build --emit-llvm output for more builtins (to exercise LLVM codegen instantiations in the CLI binary)
    let ll_builtins_out = dir.path().join("builtins.ll");
    let (code, _out, err) = run_mdhavers(
        &[
            "build",
            llvm_builtins_braw.to_str().unwrap(),
            "--emit-llvm",
            "-O",
            "0",
            "--output",
            ll_builtins_out.to_str().unwrap(),
        ],
        None,
        home,
    );
    if cfg!(feature = "llvm") {
        assert_eq!(code, 0, "stderr: {err}");
        assert!(ll_builtins_out.exists());
    } else {
        assert_ne!(code, 0);
        assert!(!ll_builtins_out.exists());
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
fn cli_run_exercises_core_language_features_for_interpreter_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let braw = dir.path().join("core_features.braw");
    write_file(
        &braw,
        r#"
ken sum = 0
fer i in [1, 2, 3, 4] {
    sum = sum + i
}
blether sum

dae add(a, b = 2) {
    gie a + b
}
blether add(3)
blether add(3, 4)

kin Animal {
    dae init(name, sound) {
        masel.name = name
        masel.sound = sound
    }

    dae speak() {
        gie masel.name + " says: " + masel.sound
    }
}

ken dug = Animal("Dug", "Woof!")
blether dug.speak()

hae_a_bash {
    mak_siccar nae, "forced failure"
} gin_it_gangs_wrang e {
    blether "caught"
}

ken x = 2
keek x {
    whan 1 -> { blether "one" }
    whan 2 -> { blether "two" }
    whan _ -> { blether "other" }
}

ken [a, b, ...rest] = [1, 2, 3, 4]
blether a
blether b
blether len(rest)
"#,
    );

    let (code, out, err) = run_mdhavers(&["run", braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(
        out.trim(),
        "10\n5\n7\nDug says: Woof!\ncaught\ntwo\n1\n2\n2"
    );
}

#[test]
fn cli_run_exercises_builtin_type_errors_for_interpreter_binary_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let braw = dir.path().join("builtin_type_errors.braw");
    write_file(
        &braw,
        r#"
hae_a_bash {
    env_get(1)
} gin_it_gangs_wrang e {
    blether "caught env_get"
}

hae_a_bash {
    env_set(1, "x")
} gin_it_gangs_wrang e {
    blether "caught env_set"
}

hae_a_bash {
    shell(1)
} gin_it_gangs_wrang e {
    blether "caught shell"
}

hae_a_bash {
    shell_status(1)
} gin_it_gangs_wrang e {
    blether "caught shell_status"
}

hae_a_bash {
    chdir(1)
} gin_it_gangs_wrang e {
    blether "caught chdir"
}

blether "done"
"#,
    );

    let (code, out, err) = run_mdhavers(&["run", braw.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(
        out.trim(),
        "caught env_get\ncaught env_set\ncaught shell\ncaught shell_status\ncaught chdir\ndone"
    );
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
    let missing = missing.to_str().unwrap();

    // Ensure each subcommand surfaces the read_file() error path.
    for args in [
        vec!["run", missing],
        vec![missing],
        vec!["check", missing],
        vec!["compile", missing],
        vec!["wasm", missing],
        vec!["trace", missing],
        vec!["tokens", missing],
        vec!["ast", missing],
        vec!["fmt", missing],
        vec!["build", missing, "-O", "0"],
    ] {
        let (code, _out, err) = run_mdhavers(&args, None, home);
        assert_ne!(code, 0, "expected failure for args={args:?}");
        assert!(err.contains("Cannae read"), "unexpected stderr: {err}");
    }
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
    assert!(
        out.contains("1.5"),
        "expected float value, got stdout:\n{out}"
    );
}

#[test]
fn cli_repl_vars_function_list_can_have_no_hidden_suffix_when_prelude_fails_to_load() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    write_file(&stdlib_dir.join("prelude.braw"), "ken =\n");

    // Ensure we cover the "no hidden functions" branch by keeping the number of
    // functions small (no prelude + one user-defined function).
    let script = "dae f0() { gie 0 }\n:vars\nquit\n";
    let (code, out, err) = run_mdhavers_in_dir(&["repl"], Some(script), home, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("Functions:"), "stdout:\n{out}\nstderr:\n{err}");
    assert!(out.contains("f0"), "stdout:\n{out}\nstderr:\n{err}");
    assert!(
        !out.contains("... and"),
        "expected no hidden suffix in stdout:\n{out}\nstderr:\n{err}"
    );
}

#[test]
fn cli_repl_vars_values_only_path_is_reachable_when_prelude_fails_and_no_functions_defined() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let stdlib_dir = dir.path().join("stdlib");
    fs::create_dir_all(&stdlib_dir).unwrap();
    write_file(&stdlib_dir.join("prelude.braw"), "ken =\n");

    let script = "ken x = 1\n:vars\nquit\n";
    let (code, out, err) = run_mdhavers_in_dir(&["repl"], Some(script), home, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert!(out.contains("Values:"), "stdout:\n{out}\nstderr:\n{err}");
    assert!(out.contains("x : int"), "stdout:\n{out}\nstderr:\n{err}");
    assert!(
        !out.contains("Functions:"),
        "expected functions section to be absent in stdout:\n{out}\nstderr:\n{err}"
    );
}

#[test]
fn cli_trace_runtime_error_path_is_covered() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let runtime_error_braw = dir.path().join("runtime_error.braw");
    write_file(&runtime_error_braw, "blether 1 / 0\n");

    let (code, _out, _err) =
        run_mdhavers(&["trace", runtime_error_braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
}

#[test]
fn cli_run_can_surface_module_not_found_error_without_line_context() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let braw = dir.path().join("missing_module.braw");
    write_file(
        &braw,
        "fetch \"definitely_missing_mdhavers_module_for_coverage\" tae missing\n",
    );

    let (code, _out, err) = run_mdhavers(&["run", braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
    assert!(err.contains("Cannae find module"), "stderr: {err}");
}

#[test]
fn cli_wasm_reports_internal_errors_without_line_context_for_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    // Trigger the WASM backend's "max 8" method call arity internal error, which does not carry a
    // line number. This should exercise the `format_parse_error` no-line/no-suggestion branches in
    // the CLI binary instantiation.
    let braw = dir.path().join("too_many_wasm_args.braw");
    write_file(&braw, "ken xs = [1]\nxs.push(1, 2, 3, 4, 5, 6, 7, 8, 9)\n");

    let (code, _out, err) = run_mdhavers(&["wasm", braw.to_str().unwrap()], None, home);
    assert_ne!(code, 0);
    assert!(
        err.contains("Method call arity too large for WASM backend"),
        "stderr: {err}"
    );
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
        ")",
        "]",
        "}",
        "ken xs = [",
        "1]",
        "ken y = (",
        "1)",
        "blether \"unterminated",
        "\"",
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
fn cli_repl_readline_utf8_error_path_is_covered() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let mut cmd = Command::new(mdhavers_bin());
    cmd.args(["repl"])
        .env("HOME", home)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("spawn mdhavers repl");
    {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(&[0xff, 0xfe, b'\n'])
            .expect("write invalid UTF-8");
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    let code = output.status.code().unwrap_or(-1);
    let err = String::from_utf8_lossy(&output.stderr).to_string();
    assert_eq!(code, 0, "stderr: {err}");
    assert!(
        err.contains("Error:"),
        "expected non-EOF readline error in stderr, got: {err}"
    );
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

    let (code, _out, _err) = run_mdhavers(
        &["fmt", "--check", bad_syntax.to_str().unwrap()],
        None,
        home,
    );
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

#[cfg(feature = "llvm")]
#[test]
fn cli_build_emit_llvm_resolves_sibling_import_from_source_directory() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    write_file(&sub.join("helper.braw"), "dae twice(x) { gie x * 2 }\n");
    write_file(
        &sub.join("main.braw"),
        "fetch \"helper\"\nblether twice(21)\n",
    );

    let (code, out, err) = run_mdhavers_in_dir(
        &["build", "sub/main.braw", "--emit-llvm", "-o", "m.ll"],
        None,
        dir.path(),
        dir.path(),
    );
    assert_eq!(
        code, 0,
        "sibling import failed: stdout: {out}\nstderr: {err}"
    );
    let ir = fs::read_to_string(dir.path().join("m.ll")).unwrap();
    assert!(
        ir.lines()
            .any(|line| line.starts_with("define ") && line.contains("@__mdh_fn_twice(")),
        "IR should define the imported twice function"
    );
}

#[cfg(feature = "llvm")]
#[test]
fn cli_build_emit_llvm_resolves_nested_relative_imports() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("sub");
    let modules = sub.join("modules");
    let shared = sub.join("shared");
    fs::create_dir_all(&modules).unwrap();
    fs::create_dir(&shared).unwrap();
    write_file(&shared.join("double.braw"), "dae twice(x) { gie x * 2 }\n");
    write_file(
        &modules.join("helper.braw"),
        "fetch \"../shared/double\"\ndae answer(x) { gie twice(x) }\n",
    );
    write_file(
        &sub.join("main.braw"),
        "fetch \"modules/helper\"\nblether answer(21)\n",
    );

    let (code, out, err) = run_mdhavers_in_dir(
        &["build", "sub/main.braw", "--emit-llvm", "-o", "m.ll"],
        None,
        dir.path(),
        dir.path(),
    );
    assert_eq!(
        code, 0,
        "nested import failed: stdout: {out}\nstderr: {err}"
    );
    let ir = fs::read_to_string(dir.path().join("m.ll")).unwrap();
    for name in ["twice", "answer"] {
        assert!(
            ir.lines()
                .any(|line| line.starts_with("define ")
                    && line.contains(&format!("@__mdh_fn_{name}("))),
            "IR should define the imported {name} function"
        );
    }
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
fn cli_compile_bool_literals_for_branch_coverage() {
    let dir = tempdir().unwrap();
    let home = dir.path();

    let bools = dir.path().join("bools.braw");
    write_file(&bools, "ken t = aye\nken f = naw\n");

    let (code, _out, err) = run_mdhavers(&["compile", bools.to_str().unwrap()], None, home);
    assert_eq!(code, 0, "stderr: {err}");
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
    // util-linux `script` takes `-c CMD FILE`; BSD/macOS `script` takes `FILE CMD ARGS...`
    // (the old Linux-style args made macOS `script` exit at once, so the first write got BrokenPipe).
    let mut cmd = Command::new("script");
    #[cfg(target_os = "macos")]
    cmd.arg("-q")
        .arg("/dev/null")
        .arg(mdhavers_bin())
        .arg("repl");
    #[cfg(not(target_os = "macos"))]
    {
        let repl_cmd = format!("{} repl", mdhavers_bin().display());
        cmd.args(["-q", "-c", &repl_cmd, "/dev/null"]);
    }
    cmd.env("HOME", home)
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

#[test]
fn cli_prelude_does_not_shadow_native_pad_builtins() {
    // MDH-BUG-FLU-00115: the prelude used to define 3-arg-only pad_left/pad_right.
    let dir = tempdir().unwrap();
    let prog = dir.path().join("pad.braw");
    write_file(
        &prog,
        "blether \"[\" + pad_right(\"ab\", 5) + \"]\"\n\
         blether \"[\" + pad_right(\"ab\", 5, \"*\") + \"]\"\n\
         blether \"[\" + pad_left(\"ab\", 5) + \"]\"\n\
         blether \"[\" + pad_left(\"ab\", 5, \"0\") + \"]\"\n",
    );
    // Run from the repo root so the real stdlib/prelude.braw is the one loaded.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (code, out, err) =
        run_mdhavers_in_dir(&["run", prog.to_str().unwrap()], None, dir.path(), &root);
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out, "[ab   ]\n[ab***]\n[   ab]\n[000ab]\n");
}

#[test]
fn cli_prelude_does_not_shadow_native_builtins() {
    // MDH-BUG-FLU-00147: the prelude used to redefine average, median, flatten,
    // unique, ony, the_noo and backside_forrit, so `mdhavers run` disagreed with
    // the library interpreter. Native builtins win.
    let dir = tempdir().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let run_src = |name: &str, src: &str| {
        let prog = dir.path().join(name);
        write_file(&prog, src);
        run_mdhavers_in_dir(&["run", prog.to_str().unwrap()], None, dir.path(), &root)
    };

    let (code, out, err) = run_src(
        "ok.braw",
        "blether backside_forrit(\"abc\")\n\
         blether backside_forrit([1, 2, 3])\n\
         blether whit_kind(the_noo())\n\
         blether flatten([[1, 2], [3], 4])\n\
         blether unique([1, 1, 2])\n\
         blether ony([1, 2, 3], |x| x > 2)\n\
         blether average([2, 4])\n\
         blether median([3, 1, 2])\n",
    );
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(
        out,
        "cba\n[3, 2, 1]\ninteger\n[1, 2, 3, 4]\n[1, 2]\naye\n3\n2\n"
    );

    // Native average/median of an empty list raise (the prelude gave 0 / naething).
    let (code, _out, err) = run_src("avg.braw", "blether average([])\n");
    assert_ne!(code, 0, "average([]) should raise; stderr: {err}");
    let (code, _out, err) = run_src("med.braw", "blether median([])\n");
    assert_ne!(code, 0, "median([]) should raise; stderr: {err}");
}

#[test]
fn cli_speir_at_end_of_input_is_naething_but_blank_line_is_empty() {
    // MDH-BUG-FLU-00136: EOF must be distinguishable from an empty line.
    let dir = tempdir().unwrap();
    let prog = dir.path().join("eof.braw");
    write_file(
        &prog,
        "ken a = speir \"\"\n\
         ken b = speir \"\"\n\
         blether a == \"\"\n\
         blether b == naething\n",
    );
    let (code, out, err) = run_mdhavers(&["run", prog.to_str().unwrap()], Some("\n"), dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out, "aye\naye\n");

    // Closed (empty) stdin: the first read is already naething.
    let prog2 = dir.path().join("eof2.braw");
    write_file(&prog2, "ken v = speir \"\"\nblether v == naething\n");
    let (code, out, err) = run_mdhavers(&["run", prog2.to_str().unwrap()], None, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out, "aye\n");
}

#[test]
fn cli_compiled_js_speir_at_end_of_input_is_null() {
    // MDH-BUG-FLU-00136 on the JS backend (skipped when node is missing).
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }
    let dir = tempdir().unwrap();
    let prog = dir.path().join("eof.braw");
    let js = dir.path().join("eof.js");
    write_file(
        &prog,
        "ken a = speir \"\"\nken b = speir \"\"\nblether a == \"\"\nblether b == naething\n",
    );
    let (code, _o, err) = run_mdhavers(
        &[
            "compile",
            prog.to_str().unwrap(),
            "-o",
            js.to_str().unwrap(),
        ],
        None,
        dir.path(),
    );
    assert_eq!(code, 0, "stderr: {err}");
    let mut child = Command::new("node")
        .arg(&js)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        child.stdin.as_mut().unwrap().write_all(b"\n").unwrap();
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout, "aye\naye\n");
}

// MDH-BUG-FLU-00165: the CLI must give scripts the big stack in every feature
// configuration (a default macOS build runs on the main thread for raylib).
#[test]
fn cli_run_recursion_5000_deep_succeeds() {
    let dir = tempdir().unwrap();
    let prog = dir.path().join("deep.braw");
    write_file(
        &prog,
        "dae r(n) { gin n == 0 { gie 0 } gie 1 + r(n - 1) }\nblether r(5000)\n",
    );
    let (code, out, err) = run_mdhavers(&["run", prog.to_str().unwrap()], None, dir.path());
    assert_eq!(code, 0, "stderr: {err}");
    assert_eq!(out.trim(), "5000");
}

#[test]
fn cli_run_unbounded_recursion_gives_scots_stack_error() {
    let dir = tempdir().unwrap();
    let prog = dir.path().join("endless.braw");
    write_file(&prog, "dae r(n) { gie 1 + r(n + 1) }\nblether r(0)\n");
    let (code, _out, err) = run_mdhavers(&["run", prog.to_str().unwrap()], None, dir.path());
    assert_eq!(code, 1, "stderr: {err}");
    assert!(err.contains("Stack's fair puggled"), "stderr: {err}");
}
