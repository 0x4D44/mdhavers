#![cfg(feature = "llvm")]
//! Native reference counting: builtins and statements that used to leak
//! (MDH-BUG-FLU-00174). Each program is built with counting on and must print the right
//! output and leave nothing live at exit (`total=0`).

use std::fs;
use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

/// Builds `main` (with `lib/strings.braw` beside it when `module` is given), runs it with
/// `MDH_RC_REPORT=1` and returns `(stdout, stderr)`.
fn run_counted(main: &str, module: Option<&str>) -> (String, String) {
    let dir = tempdir().expect("temp dir");
    let main_path = dir.path().join("main.braw");
    fs::write(&main_path, main).expect("write main");
    if let Some(module) = module {
        fs::create_dir_all(dir.path().join("lib")).expect("lib dir");
        fs::write(dir.path().join("lib/strings.braw"), module).expect("write module");
    }
    let program = parse(main).expect("parse");
    let exe = dir.path().join("rc_leak_exe");
    LLVMCompiler::new()
        .with_refcounting(true)
        .compile_to_native_with_source(&program, &exe, 2, Some(&main_path))
        .expect("compile to native");
    let output = Command::new(&exe)
        .env("MDH_RC_REPORT", "1")
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn assert_clean(main: &str, module: Option<&str>, expected: &str) {
    let (stdout, stderr) = run_counted(main, module);
    assert_eq!(stdout, expected, "{stderr}");
    assert!(
        stderr.contains(" total=0 "),
        "leaked objects for {main:?}:\n{stderr}"
    );
}

#[test]
fn string_destructuring_releases_its_characters() {
    assert_clean(
        "ken [c1, c2] = \"AB\"\nblether c1\nblether c2\n",
        None,
        "A\nB\n",
    );
}

#[test]
fn destructuring_a_list_with_a_rest_name_stays_clean() {
    assert_clean(
        "ken [a, ...rest] = [\"x\", \"y\", \"z\"]\nblether a\nblether rest\n",
        None,
        "x\n[y, z]\n",
    );
}

#[test]
fn items_releases_its_key_and_value_lists() {
    assert_clean("ken p = items({\"a\": 1})\nblether p\n", None, "[[a, 1]]\n");
}

#[test]
fn roar_and_shout_release_the_uppercased_temp() {
    assert_clean("blether roar(\"hello\")\n", None, "HELLO!\n");
    assert_clean("blether shout(\"hi\")\n", None, "HI!\n");
}

#[test]
fn log_span_is_counted_and_releases_its_fields() {
    assert_clean("ken sp = log_span(\"s\")\nblether \"ok\"\n", None, "ok\n");
}

#[test]
fn an_entered_span_stays_alive_until_it_exits() {
    assert_clean(
        "ken sp = log_span(\"s\")\nlog_span_enter(sp)\nsp = naething\nblether \"in\"\nlog_span_exit(log_span_current())\nblether \"out\"\n",
        None,
        "in\nout\n",
    );
}

#[test]
fn a_module_alias_releases_its_dict_at_exit() {
    assert_clean(
        "fetch \"lib/strings\" tae str\nblether \"ok\"\n",
        Some("dae shout_it(s) {\n    gie s + \"!\"\n}\n"),
        "ok\n",
    );
}

#[test]
fn grup_up_releases_keys_and_groups() {
    assert_clean(
        "blether grup_up([1, 2, 3], |x| x % 2)\n",
        None,
        "{\"1\": [1, 3], \"0\": [2]}\n",
    );
}

#[test]
fn pairt_by_releases_its_partition_lists() {
    assert_clean(
        "blether pairt_by([1, 2, 3], |x| x > 1)\n",
        None,
        "[[2, 3], [1]]\n",
    );
}

#[test]
fn a_return_in_a_loop_whose_condition_reads_a_reassigned_variable_stays_clean() {
    // The loop condition reads `e`, and `e = e.outer` later in the body writes it, so the
    // condition's read takes its own reference after the `gie` was compiled: the `gie`
    // must release it too (the blether_lisp / highland_adventure leak).
    assert_clean(
        r#"kin Env {
    dae init(outer) {
        masel.vars = {}
        masel.outer = outer
    }
}
dae look(env, name) {
    ken e = env
    whiles e != naething {
        gin name == "z" {
            gie e
        }
        e = e.outer
    }
    gie naething
}
ken g = Env(naething)
ken e = Env(g)
ken r = look(e, "z")
blether r == e
blether look(e, "q")
"#,
        None,
        "aye\nnaething\n",
    );
}

#[test]
fn sclaff_leaves_nothing_live() {
    assert_clean(
        "blether sclaff([1, [2, [3, [4]]], 5])\nblether sclaff([\"a\", [\"b\"]])\n",
        None,
        "[1, 2, 3, 4, 5]\n[a, b]\n",
    );
}
