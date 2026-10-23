#![cfg(feature = "llvm")]
//! Native-backend follow-ups: function values bound late (MDH-BUG-FLU-00070), a user
//! function called `main` (00071), caught hurl text (00073), float printing (00074),
//! the build run hint (00076), creel updates on boxed variables (00081) and checked
//! `replace` sizes (00082).

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    stdout: String,
    stderr: String,
    code: Option<i32>,
}

fn compile_and_run(source: &str) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("followup_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        code: output.status.code(),
    }
}

/// Runs a program that must exit cleanly; returns its trimmed stdout lines.
fn run_ok(source: &str) -> Vec<String> {
    let run = compile_and_run(source);
    assert_eq!(
        run.code,
        Some(0),
        "program failed: stdout={:?} stderr={:?}",
        run.stdout,
        run.stderr
    );
    run.stdout.trim().lines().map(str::to_string).collect()
}

// ---------- MDH-BUG-FLU-00070: calling a function value bound after the call site ----------

#[test]
fn lambda_passed_as_parameter_can_be_called() {
    let out = run_ok(
        r#"
dae c(op) { gie op(1) }
blether c(|a| a + 1)
"#,
    );
    assert_eq!(out, vec!["2"]);
}

#[test]
fn closures_and_builtins_reach_late_lambdas() {
    let out = run_ok(
        r#"
dae apply(f, x) { gie f(x) }
dae mapper(xs, f) { gie gaun(xs, f) }
dae make_adder(n) { gie |x| x + n }
blether apply(make_adder(10), 5)
blether mapper([1, 2, 3], |x| x * 2)
blether apply(|s| s + "!", "och")
"#,
    );
    assert_eq!(out, vec!["15", "[2, 4, 6]", "och!"]);
}

#[test]
fn calling_a_non_function_value_is_still_a_type_error() {
    let run = compile_and_run(
        r#"
dae c(op) { gie op(1) }
blether c(5)
"#,
    );
    assert_ne!(run.code, Some(0), "stdout={:?}", run.stdout);
    assert!(run.stderr.contains("call"), "stderr={:?}", run.stderr);
}

// ---------- MDH-BUG-FLU-00071: a user function called main ----------

#[test]
fn user_function_named_main_does_not_replace_the_entry_point() {
    let out = run_ok(
        r#"
dae helper() { gie 1 }
dae main() { blether helper() }
main()
blether "done"
"#,
    );
    assert_eq!(out, vec!["1", "done"]);
}

// ---------- MDH-BUG-FLU-00073: caught hurl text matches the interpreter ----------

#[test]
fn caught_hurl_carries_the_interpreter_prefix() {
    let out = run_ok(
        r#"
hae_a_bash {
    hurl "boom"
} gin_it_gangs_wrang err {
    blether "Caught: " + err
}
dae f() {
    hurl 42
}
hae_a_bash {
    f()
} gin_it_gangs_wrang e {
    blether e
}
"#,
    );
    assert_eq!(
        out,
        vec!["Caught: Hurled at line 3: boom", "Hurled at line 8: 42"]
    );
}

// ---------- MDH-BUG-FLU-00074: floats print as the interpreter prints them ----------

#[test]
fn floats_print_shortest_round_trip_like_the_interpreter() {
    let out = run_ok(
        r#"
blether 0.1 + 0.2
blether 1.0
blether 2.5
blether 1.0 / 3.0
blether 0.0 - 1.5
blether [0.5, 0.1 + 0.2]
blether "x=" + tae_string(0.1 + 0.2)
blether 1000000.0 * 1000000.0 * 1000000.0 * 1000.0
blether 0.0000001
blether 3.14159265358979
"#,
    );
    assert_eq!(
        out,
        vec![
            "0.30000000000000004",
            "1",
            "2.5",
            "0.3333333333333333",
            "-1.5",
            "[0.5, 0.30000000000000004]",
            "x=0.30000000000000004",
            "1000000000000000000000",
            "0.0000001",
            "3.14159265358979",
        ]
    );
}

// ---------- MDH-BUG-FLU-00081: toss_in/heave_oot on a boxed (captured) variable ----------

#[test]
fn creel_updates_keep_a_captured_variable_boxed() {
    let out = run_ok(
        r#"
dae f() {
    ken s = empty_creel()
    ken has = |x| is_in_creel(s, x)
    toss_in(s, "a")
    toss_in(s, "b")
    heave_oot(s, "a")
    blether has("b")
    blether has("a")
    blether is_in_creel(s, "b")
    blether len(creel_tae_list(s))
}
f()
"#,
    );
    assert_eq!(out, vec!["aye", "nae", "aye", "1"]);
}

// ---------- MDH-BUG-FLU-00082: replace sizes are checked ----------

#[test]
fn huge_replace_hurls_instead_of_writing_through_a_failed_allocation() {
    let out = run_ok(
        r#"
ken big = "a" * 16777216
hae_a_bash {
    blether len(replace(big, "a", big))
} gin_it_gangs_wrang e {
    blether "replace caught"
}
hae_a_bash {
    blether len(replace(big, "", big))
} gin_it_gangs_wrang e {
    blether "empty-pattern replace caught"
}
blether replace("banana", "a", "o")
blether replace("ab", "", "-")
"#,
    );
    assert_eq!(
        out,
        vec![
            "replace caught",
            "empty-pattern replace caught",
            "bonono",
            "-a-b-"
        ]
    );
}

// ---------- MDH-BUG-FLU-00076: the build run hint ----------

#[cfg(feature = "cli")]
fn mdhavers_bin() -> std::path::PathBuf {
    if let Some(p) = option_env!("CARGO_BIN_EXE_mdhavers") {
        return std::path::PathBuf::from(p);
    }
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/mdhavers")
}

#[cfg(feature = "cli")]
#[test]
fn build_run_hint_does_not_prefix_absolute_paths() {
    let dir = tempdir().expect("temp dir");
    let src = dir.path().join("hint.braw");
    std::fs::write(&src, "blether 1\n").expect("write source");
    let exe = dir.path().join("hint_exe");
    let output = Command::new(mdhavers_bin())
        .args(["build", src.to_str().unwrap(), "-O", "0", "-o"])
        .arg(&exe)
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .output()
        .expect("run mdhavers build");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout={stdout:?}");
    assert!(
        stdout.contains(&format!("$ {}", exe.display())),
        "stdout={stdout:?}"
    );
    assert!(!stdout.contains(".//"), "stdout={stdout:?}");
}
