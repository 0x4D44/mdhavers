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
        // String repetition supports both `"s" * n` and `n * "s"`; cover both match-arm variants.
        r#"
blether 2 * "ab"
blether "ab" * 2
"#,
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
        // sort float NaN compare fallback (partial_cmp returns None)
        r#"blether sort([sqrt(-1), 1.0])"#,
        // call_function_with_env nil-binding branch (class init is called without arity checks).
        r#"
kin C {
    dae init(a, b) { masel.a = a }
}
ken c = C(1)
blether c.a
"#,
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
        // Match statement: pattern eval error propagation (range end).
        r#"
keek 1 {
    whan 1..missing -> blether 1
}
"#,
        // Destructure: RHS eval error propagation.
        r#"
ken [a] = missing
"#,
        // Get expression: object eval error propagation.
        r#"blether missing.foo"#,
        // BlockExpr: statement error propagation through execute_stmt_with_control(...)?.
        r#"blether { missing }"#,
        // HOF builtins: callback error propagation through call_value(...)?.
        r#"blether gaun([1], |x| missing)"#,
        r#"blether sieve([1], |x| missing)"#,
        r#"blether tumble([1], 0, |a, b| missing)"#,
        r#"ilk([1], |x| missing)"#,
        r#"blether hunt([1], |x| missing)"#,
        r#"blether ony([1], |x| missing)"#,
        r#"blether aw([1], |x| missing)"#,
        r#"blether grup_up([1], |x| missing)"#,
        r#"blether pairt_by([1], |x| missing)"#,
        // Function default value eval error propagation.
        r#"
dae f(x = missing) { gie x }
f()
"#,
        // JSON: object key string escape error (parse_json_string called via parse_json_object).
        r#"json_parse("{\"\\u")"#,
        // Log statement: emit_log fields_from_dict error propagation (non-string dict key).
        r#"log_blether "hi", {1: 2}"#,
        // Native logging helpers: argument parsing error propagation.
        r#"log_enabled("nae_a_level")"#,
        r#"log_event("nae_a_level", "hi")"#,
        r#"log_span("span", "nae_a_level")"#,
        r#"log_span("span", "blether", 1)"#,
        r#"log_span("span", "blether", {"a": 1}, 1)"#,
        // thread_spawn propagates errors from native calls.
        r#"thread_spawn(log_set_filter, ["net=nae-a-level"])"#,
    ] {
        let err = interpret_err(src);
        assert!(!err.is_empty(), "expected error string for:\n{src}");
    }
}

#[test]
fn interpreter_match_range_start_eval_error_branch_is_covered_via_ast_for_coverage() {
    use mdhavers::ast::{Expr, Literal, MatchArm, Pattern, Program, Span, Stmt};

    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Match {
        value: Expr::Literal {
            value: Literal::Integer(1),
            span,
        },
        arms: vec![MatchArm {
            pattern: Pattern::Range {
                start: Box::new(Expr::Variable {
                    name: "missing".to_string(),
                    span,
                }),
                end: Box::new(Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                }),
            },
            body: Stmt::Expression {
                expr: Expr::Literal {
                    value: Literal::Integer(0),
                    span,
                },
                span,
            },
            span,
        }],
        span,
    }]);

    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected error from range-start eval");
    let msg = format!("{err:?}");
    assert!(!msg.is_empty(), "expected non-empty error debug");
}

#[cfg(all(feature = "native", unix))]
#[test]
fn interpreter_event_watch_reports_unknown_loop_handle_errors_for_coverage() {
    let err = interpret_err(
        r#"
ken s = socket_udp()
gin s["ok"] {
    ken sock = s["value"]
    # invalid loop id exercises the with_loop_mut(...)? error propagation path
    event_watch_read(999999, sock, nae)
}
"#,
    );
    assert!(
        err.contains("Unknown event loop handle"),
        "unexpected error: {err}"
    );

    let err = interpret_err(
        r#"
ken s = socket_udp()
gin s["ok"] {
    ken sock = s["value"]
    event_watch_write(999999, sock, nae)
}
"#,
    );
    assert!(
        err.contains("Unknown event loop handle"),
        "unexpected error: {err}"
    );
}

#[test]
fn interpreter_shell_and_shell_status_spawn_failure_paths_are_testable_via_mdh_shell_override() {
    // Use the coverage-only MDH_SHELL override to force the spawn failure branches in
    // `shell`/`shell_status` without mutating process-wide environment.
    let _guard = mdhavers::interpreter::set_mdh_shell_override_for_coverage(Some(
        "/definitely/no/such/shell".to_string(),
    ));

    for src in [r#"shell("echo hi")"#, r#"shell_status("echo hi")"#] {
        let err = interpret_err(src);
        assert!(
            err.contains("Shell command failed"),
            "expected spawn failure error, got: {err}"
        );
    }
}

#[test]
fn interpreter_cwd_error_path_can_be_triggered_without_process_cwd_mutation() {
    let _guard = mdhavers::interpreter::set_force_current_dir_error_for_coverage(true);
    let err = interpret_err("cwd()");
    assert!(
        err.contains("Couldnae get current directory"),
        "expected cwd() failure, got: {err}"
    );
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

#[test]
fn interpreter_import_path_with_extension_exercises_resolve_module_path_noop_branch_for_coverage() {
    use std::fs;

    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(
        dir.path().join("mymod.braw"),
        r#"
ken x = 1
"#,
    )
    .expect("write module");

    let program = parse(r#"fetch "mymod.braw" tae m"#).expect("parse import");
    let mut interp = Interpreter::new();
    interp.set_current_dir(dir.path());
    interp
        .interpret(&program)
        .expect("expected import to succeed");
}

#[test]
fn interpreter_method_call_env_falls_back_to_globals_when_closure_missing_for_coverage() {
    use std::rc::Rc;

    use mdhavers::ast::{Expr, Literal, Span, Stmt};
    use mdhavers::value::{FunctionParam, HaversClass, HaversFunction};

    let span = Span::new(1, 1);
    let method = Rc::new(HaversFunction::new(
        "m".to_string(),
        Vec::<FunctionParam>::new(),
        vec![Stmt::Return {
            value: Some(Expr::Literal {
                value: Literal::Integer(123),
                span,
            }),
            span,
        }],
        None,
    ));

    let mut class = HaversClass::new("C".to_string(), None);
    class.methods.insert("m".to_string(), method);

    let program = parse(
        r#"
ken c = C()
c.m()
"#,
    )
    .expect("parse");
    let mut interp = Interpreter::new();
    interp
        .globals
        .borrow_mut()
        .define("C".to_string(), mdhavers::Value::Class(Rc::new(class)));
    let value = interp.interpret(&program).expect("interpret");
    assert_eq!(value, mdhavers::Value::Integer(123));
}
