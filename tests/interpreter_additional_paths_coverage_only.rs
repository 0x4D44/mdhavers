#![cfg(coverage)]

use mdhavers::{parse, Interpreter};

fn interpret_ok(source: &str) -> Vec<String> {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let mut interp = Interpreter::new();
    interp
        .interpret(&program)
        .unwrap_or_else(|e| panic!("interpret failed for:\n{source}\nerr={e:?}"));
    interp.get_output().to_vec()
}

fn interpret_err(source: &str) -> String {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let mut interp = Interpreter::new();
    match interp.interpret(&program) {
        Ok(value) => panic!("expected interpreter error for:\n{source}\n\nbut got Ok: {value:?}"),
        Err(err) => format!("{err:?}"),
    }
}

#[test]
fn interpreter_exercises_additional_error_and_edge_paths_for_coverage() {
    // Native builtin edge/error branches.
    for src in [
        // clype on less-common value kinds
        r#"
dae f() { gie 1 }
blether clype(f)
blether clype(len)
kin C { dae init() { } }
blether clype(C)
blether clype(C())
"#,
        // JSON number exponent (+/-) and string escaping
        r#"blether json_parse("1e+2")"#,
        r#"blether json_parse("-1.2E-3")"#,
        r#"blether json_stringify("a\\\"b")"#,
        r#"blether json_stringify("\\\\")"#,
        r#"blether json_stringify("a\\nb\\t")"#,
        r#"blether json_stringify("a\\rb")"#,
        // BlockExpr catching break/continue (shouldn't happen, but it is handled)
        r#"blether { brak }"#,
        r#"blether { haud }"#,
        // Operator overloading via instance binary-op dispatch (call_method_on_instance)
        r#"
kin AddBreak {
    dae __pit_thegither__(other) { brak }
}
ken a = AddBreak()
blether a + 1
"#,
        r#"
kin AddContinue {
    dae __pit_thegither__(other) { haud }
}
ken a = AddContinue()
blether a + 1
"#,
        r#"
kin AddReturn {
    dae __pit_thegither__(other) { gie 123 }
}
ken a = AddReturn()
blether a + 1
"#,
        r#"
kin AddOk {
    dae __pit_thegither__(other) { }
}
ken a = AddOk()
blether a + 1
"#,
        // call_function_with_env break/continue catch paths (shouldn't happen, but handled)
        r#"
dae f() { brak }
blether f()
"#,
        // Float-path in maxaw
        r#"blether maxaw([1.0, 2.5, 0.1])"#,
        // JSON empty container fast-paths
        r#"blether len(json_parse("{}"))"#,
        r#"blether len(json_parse("[]"))"#,
        // JSON literal parsing
        r#"blether json_parse("true")"#,
        r#"blether json_parse("false")"#,
        r#"blether json_parse("null")"#,
        // JSON stringify branches
        r#"blether json_stringify(nae)"#,
        r#"blether json_stringify(sqrt(-1))"#,
        r#"blether json_stringify_pretty(sqrt(-1))"#,
        r#"blether json_stringify_pretty([])"#,
        r#"blether json_stringify_pretty({})"#,
        r#"blether json_stringify(chr(1))"#,
        r#"blether json_stringify(chr(10) + chr(9) + chr(13))"#,
        // Unknown escape in JSON strings falls back to literal escaped char
        r#"blether json_parse(chr(34) + chr(92) + "q" + chr(34))"#,
    ] {
        let out = interpret_ok(src);
        assert!(!out.is_empty(), "expected some output for:\n{src}");
    }

    // Interpreter error paths (evaluate/execute).
    for src in [
        // chynge/dicht out-of-bounds branches
        r#"blether chynge([1, 2, 3], 99, 0)"#,
        r#"blether dicht([1, 2, 3], 99)"#,
        // Undefined superclass resolution in class statement
        r#"kin Child fae Missing { dae init() { } }"#,
        // Fixed-arity function wrong-arity branch
        r#"
dae add(a, b) { gie a + b }
blether add(1)
"#,
        // Spread operator errors
        r#"ken xs = [...1]"#,
        // Range bounds must be integers
        r#"ken r = ("a")..3"#,
        // masel outside class/method
        r#"blether masel"#,
        // speir/input is disabled under coverage runs
        r#"speir "prompt> ""#,
        // Not-callable error branch
        r#"1()"#,
        // Higher-order builtins type errors (correct arity, wrong type)
        r#"gaun(1, |x| x)"#,
        r#"sieve(1, |x| aye)"#,
        r#"tumble(1, 0, |a, b| a)"#,
        r#"ilk(1, |x| x)"#,
        r#"hunt(1, |x| aye)"#,
        r#"ony(1, |x| aye)"#,
        r#"aw(1, |x| aye)"#,
        r#"grup_up(1, |x| x)"#,
        r#"pairt_by(1, |x| x)"#,
        // Unknown builtin marker should be rejected
        r#"("__builtin_nope__")(1)"#,
        // JSON parser error branches
        r#"json_parse("")"#,
        r#"json_parse("{")"#,
        r#"json_parse("[1,")"#,
        r#"json_parse("tru")"#,
        r#"json_parse("fal")"#,
        r#"json_parse("nul")"#,
        r#"json_parse('{"a" 1}')"#,
        r#"json_parse('{"a":1')"#,
        r#"json_parse('{"a":1;}')"#,
        r#"json_parse("[1")"#,
        r#"json_parse("[1;]")"#,
        // JSON string escape error branches
        r#"json_parse(chr(34) + chr(92))"#,
        r#"json_parse(chr(34) + chr(92) + "u12" + chr(34))"#,
        // Operator-overload method error propagation (call_method_on_instance Err(e))
        r#"
kin AddErr {
    dae __pit_thegither__(other) { 1 / 0 }
}
ken a = AddErr()
a + 1
"#,
        // lerp second-argument type error path
        r#"lerp(1, "x", 0.5)"#,
    ] {
        let err = interpret_err(src);
        assert!(!err.is_empty(), "expected error string for:\n{src}");
    }
}

#[test]
fn interpreter_shell_and_shell_status_spawn_failure_paths_are_testable_via_mdh_shell_override() {
    // Use the MDH_SHELL override to force the spawn failure branches in `shell`/`shell_status`.
    let prev = std::env::var("MDH_SHELL").ok();
    std::env::set_var("MDH_SHELL", "/definitely/no/such/shell");

    for src in [r#"shell("echo hi")"#, r#"shell_status("echo hi")"#] {
        let err = interpret_err(src);
        assert!(
            err.contains("Shell command failed"),
            "expected spawn failure error, got: {err}"
        );
    }

    match prev {
        Some(v) => std::env::set_var("MDH_SHELL", v),
        None => std::env::remove_var("MDH_SHELL"),
    }
}

#[test]
fn interpreter_cwd_error_path_can_be_triggered_by_deleted_working_directory() {
    use std::path::PathBuf;

    let old_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let dir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();
    let doomed = dir.path().to_path_buf();
    drop(dir); // deletes the directory

    let err = interpret_err("cwd()");
    assert!(
        err.contains("Couldnae get current directory"),
        "expected cwd() failure, got: {err}"
    );

    // Restore to avoid impacting other tests.
    let _ = std::env::set_current_dir(old_dir);
    // Best-effort cleanup; directory is already gone.
    let _ = std::fs::remove_dir_all(doomed);
}

#[test]
fn interpreter_global_log_level_fallback_branch_is_covered() {
    use mdhavers::ast::LogLevel;
    use mdhavers::interpreter::{
        get_global_log_level, set_global_log_level, set_global_log_level_raw,
    };

    // Force an invalid atomic value so the `_ => LogLevel::Blether` match arm runs.
    set_global_log_level_raw(200);
    assert_eq!(get_global_log_level(), LogLevel::Blether);

    // Restore a sane value for the rest of the suite.
    set_global_log_level(LogLevel::Blether);
}
