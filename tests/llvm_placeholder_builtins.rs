#![cfg(feature = "llvm")]
//! Native vs interpreter parity for prelude builtins that native once placeholdered
//! (MDH-BUG-FLU-00155), the screen/keyboard builtins native cannot support, and `clype`
//! on a class instance (MDH-BUG-FLU-00159).

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
    let exe_path = dir.path().join("placeholder_exe");
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

/// Interpreter output for `src`, with the prelude loaded as the CLI does.
fn interpreter_lines(src: &str) -> Vec<String> {
    let src = src.to_string();
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            let program = parse(&src).expect("parse");
            let mut interp = mdhavers::Interpreter::new();
            interp.load_prelude().expect("prelude");
            interp.interpret(&program).expect("interpret");
            interp.get_output().to_vec()
        })
        .unwrap()
        .join()
        .unwrap()
}

fn assert_native_matches_interpreter(src: &str, must_contain: &[&str]) {
    let expected = interpreter_lines(src);
    let native = run_ok(src);
    assert_eq!(native, expected);
    for needle in must_contain {
        assert!(
            expected.iter().any(|l| l.contains(needle)),
            "interpreter output lacks {needle:?}: {expected:?}"
        );
    }
}

#[test]
fn native_dae_times_and_efter_match_the_interpreter() {
    assert_native_matches_interpreter(
        r#"dae inc(x) { gie x + 1 }
dae dbl(x) { gie x * 2 }
ken k = 10
dae addk(x) { gie x + k }
blether dae_times(3, 0, inc)
blether dae_times(2, 5, |x| x * 2)
blether dae_times(3, 1, addk)
blether dae_times(0, 7, inc)
blether efter(inc, dbl)(5)
blether efter(|x| x + k, |x| x * k)(2)
"#,
        &["3", "20", "31", "7", "12", "120"],
    );
}

#[test]
fn native_compose_apply_n_constant_same_nae_that_match_the_interpreter() {
    assert_native_matches_interpreter(
        r#"dae inc(x) { gie x + 1 }
ken k = 4
dae big(x) { gie x > 3 }
blether compose(inc, |x| x * k)(2)
blether compose(|x| x - 1, inc)(2)
blether apply_n(inc, 4, 0)
blether apply_n(|x| x * k, 3, 1)
blether apply_n(inc, 0, 9)
ken c = constant("hi")
blether c(1)
blether c(naething)
blether same([1, 2])
blether nae_that(big)(5)
blether nae_that(|x| x > k)(9)
"#,
        &["9", "64", "hi", "[1, 2]", "nae"],
    );
}

#[test]
fn native_group_by_swatch_assert_that_match_the_interpreter() {
    assert_native_matches_interpreter(
        r#"dae len1(s) { gie len(s) }
ken m = 3
blether group_by([1, 2, 3, 4, 5], |x| x % 2)
blether group_by(["a", "bb", "cc", "d"], len1)
blether group_by([1, 2, 3, 4, 5, 6], |x| x % m)
ken r = swatch("lbl", 42)
blether r
assert_that(aye, "fine")
hae_a_bash {
    assert_that(1 == 2, "boom")
} gin_it_gangs_wrang e {
    blether "caught: " + e
}
"#,
        &["[swatch] lbl: 42", "caught: Mak siccar failed at line"],
    );
}

#[test]
fn native_mak_siccar_failure_carries_the_interpreters_wording() {
    assert_native_matches_interpreter(
        "hae_a_bash {\n    mak_siccar 1 == 2, \"sums\"\n} gin_it_gangs_wrang e {\n    blether e\n}\nhae_a_bash {\n    mak_siccar nae\n} gin_it_gangs_wrang e {\n    blether e\n}\n",
        &["Mak siccar failed at line 2! sums", "Mak siccar failed at line 7! Assertion failed"],
    );
}

#[test]
fn native_scots_unit_converters_match_the_interpreter() {
    assert_native_matches_interpreter(
        r#"blether scots_miles_tae_km(10)
blether scots_miles_tae_km(2.5)
blether scots_pints_tae_litres(4)
blether scots_pints_tae_litres(1.5)
blether stanes_tae_kg(2)
blether stanes_tae_kg(0)
hae_a_bash {
    blether scots_miles_tae_km("a")
} gin_it_gangs_wrang e {
    blether "caught: " + e
}
"#,
        &["18", "6.8", "12.7"],
    );
}

#[test]
fn native_screen_and_key_builtins_hurl_instead_of_returning_made_up_values() {
    for call in [
        "screen_width()",
        "screen_height()",
        "key_down(\"space\")",
        "key_pressed(\"space\")",
    ] {
        let run = compile_and_run(&format!(
            "blether \"before\"\nblether {call}\nblether \"after\"\n"
        ));
        assert_ne!(run.code, Some(0), "{call} must not succeed natively");
        assert_eq!(run.stdout.trim(), "before", "{call}");
        assert!(run.stderr.contains("Nae windae"), "{call}: {}", run.stderr);
    }
    // screen_end is a no-op in the interpreter too, so native keeps it a no-op.
    assert_eq!(run_ok("screen_end()\nblether \"x\"\n"), vec!["x"]);
    // A caught error still reaches the handler.
    let out = run_ok(
        "hae_a_bash {\n    blether screen_width()\n} gin_it_gangs_wrang e {\n    blether e\n}\n",
    );
    assert!(out[0].starts_with("Nae windae!"), "{out:?}");
}

#[test]
fn native_clype_names_the_class_of_an_instance_mdh_bug_flu_00159() {
    assert_native_matches_interpreter(
        r#"kin Dog {
    dae init() { masel.n = 1 }
}
blether clype(Dog())
blether clype(1)
blether clype([1])
"#,
        &["[instance] instance o' 'Dog'"],
    );
}
