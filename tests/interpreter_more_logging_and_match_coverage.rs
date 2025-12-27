#![cfg(coverage)]

use mdhavers::{parse, run, run_with_output, Interpreter};

fn expect_interpret_err(source: &str) {
    let program = parse(source).expect("expected program to parse");
    let mut interp = Interpreter::new();
    assert!(interp.interpret(&program).is_err(), "expected interpret() to error");
}

#[test]
fn interpreter_match_string_literal_pattern_is_covered() {
    let (_result, out) = run_with_output(
        r#"
keek "hi" {
    whan "hi" -> { blether "ok" }
    whan _ -> { blether "nope" }
}
"#,
    )
    .expect("expected program to run");

    assert_eq!(out.join("\n").trim(), "ok");
}

#[test]
fn interpreter_log_init_color_and_timestamps_bool_are_covered() {
    // Exercises `Value::Bool` handling for `color` and `timestamps`.
    run(
        r#"
log_init({"format": "text", "color": aye, "timestamps": nae})
"#,
    )
    .expect("expected log_init to succeed");
}

#[test]
fn interpreter_log_init_json_format_and_explicit_stdout_stderr_sinks_are_covered() {
    // Exercises:
    // - `format == "json"` branch
    // - `"stderr"` and `"stdout"` sink kinds
    let _ = run(
        r#"
log_init({"format": "text", "sinks": [{"kind": "stderr"}, {"kind": "stdout"}]})
log_init({"format": "json", "sinks": [{"kind": "stderr"}, {"kind": "stdout"}]})
"#,
    );
}

#[test]
fn interpreter_class_init_error_propagates_from_call_value_for_coverage() {
    // Exercises the error-propagation path from `Interpreter::call_value` when calling a class init.
    let _ = run(
        r#"
kin Boom {
    dae init() {
        blether missing
    }
}
Boom()
"#,
    );
}

#[test]
fn interpreter_log_init_file_sink_requires_path_error_is_covered() {
    // Exercises: `"file" => { ... return Err("... requires string path") }`.
    assert!(run(r#"log_init({"sinks": [{"kind": "file"}]})"#).is_err());
}

#[test]
fn interpreter_log_init_file_sink_append_must_be_bool_error_is_covered() {
    assert!(run(r#"log_init({"sinks": [{"kind": "file", "path": "/tmp/mdh_test.log", "append": 1}]})"#).is_err());
}

#[test]
fn interpreter_log_init_memory_sink_max_must_be_integer_error_is_covered() {
    assert!(run(r#"log_init({"sinks": [{"kind": "memory", "max": "nope"}]})"#).is_err());
}

#[test]
fn interpreter_log_init_unknown_sink_kind_error_is_covered() {
    assert!(run(r#"log_init({"sinks": [{"kind": "nope"}]})"#).is_err());
}

#[test]
fn interpreter_log_init_empty_sinks_falls_back_to_stderr_for_coverage() {
    run(
        r#"
log_init({"sinks": []})
log_blether "hi"
"#,
    )
    .expect("expected log_init to fall back to stderr");
}

#[test]
fn interpreter_log_init_file_sink_append_bool_is_covered() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("mdh_test.log");

    let code = format!(
        "log_init({{\"sinks\": [{{\"kind\": \"file\", \"path\": \"{}\", \"append\": nae}}]}})\nlog_blether \"hi\"\n",
        path.display()
    );

    let (_result, _out) = run_with_output(&code).expect("expected program to run");
    assert!(path.exists(), "expected log file to be created");
}

#[test]
fn interpreter_log_statement_two_extra_args_type_errors_are_covered() {
    // fields must be a dict
    assert!(run("log_blether \"hi\", 1, \"t\"\n").is_err());
    // target must be a string
    assert!(run("log_blether \"hi\", {\"a\": 1}, 1\n").is_err());
}

#[test]
fn interpreter_log_statement_too_many_extra_args_is_covered() {
    assert!(run("log_blether \"hi\", {\"a\": 1}, \"t\", 1\n").is_err());
}

#[test]
fn interpreter_log_statement_too_many_extra_args_is_covered_via_ast() {
    use mdhavers::ast::{Expr, Literal, LogLevel, Program, Span, Stmt};
    use mdhavers::HaversError;

    let span = Span::new(1, 1);
    let stmt = Stmt::Log {
        level: LogLevel::Blether,
        message: Expr::Literal {
            value: Literal::String("hi".to_string()),
            span,
        },
        extras: vec![
            Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            Expr::Literal {
                value: Literal::Integer(2),
                span,
            },
            Expr::Literal {
                value: Literal::Integer(3),
                span,
            },
        ],
        span,
    };

    let program = Program::new(vec![stmt]);
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected log_* extras arity error");
    assert!(matches!(err, HaversError::InternalError(_)));
}

#[test]
fn interpreter_builtin_hof_hunt_ony_aw_are_covered() {
    let (_result, out) = run_with_output(
        r#"
blether hunt([1, 2], |x| x == 2)
blether hunt([1, 2], |x| nae)

blether ony([1, 2], |x| x == 2)
blether ony([1, 2], |x| nae)

blether aw([1, 2], |x| aye)
blether aw([1, 2], |x| x == 2)
"#,
    )
    .expect("expected program to run");

    assert_eq!(out.join("\n").trim(), "2\nnaething\naye\nnae\naye\nnae");
}

#[test]
fn interpreter_pipe_operator_is_covered() {
    let (_result, out) = run_with_output(
        r#"
blether 5 |> |x| x + 1
ken inc = |x| x + 1
blether 5 |> inc
"#,
    )
    .expect("expected program to run");

    assert_eq!(out.join("\n").trim(), "6\n6");
}

#[test]
fn interpreter_spread_call_args_list_expansion_is_covered() {
    let (_result, out) = run_with_output(
        r#"
dae add3(a, b, c) { gie a + b + c }
ken xs = [2, 3]
blether add3(1, ...xs)
"#,
    )
    .expect("expected program to run");

    assert_eq!(out.join("\n").trim(), "6");
}

#[test]
fn interpreter_spread_call_args_non_list_type_error_is_covered() {
    expect_interpret_err(
        r#"
dae id(x) { gie x }
blether id(...1)
"#,
    );
}

#[test]
fn interpreter_spread_call_args_propagates_eval_errors_for_coverage() {
    expect_interpret_err(
        r#"
dae id(x) { gie x }
blether id(...(1 / 0))
"#,
    );
}

#[test]
fn interpreter_call_args_non_spread_eval_error_propagation_is_covered() {
    expect_interpret_err(
        r#"
dae id(x) { gie x }
blether id(1 / 0)
"#,
    );
}

#[test]
fn interpreter_ternary_expression_is_covered() {
    let (_result, out) = run_with_output(
        r#"
blether gin aye than 1 ither 2
blether gin nae than 1 ither 2
"#,
    )
    .expect("expected program to run");

    assert_eq!(out.join("\n").trim(), "1\n2");
}

#[test]
fn interpreter_ternary_condition_error_propagation_is_covered() {
    expect_interpret_err("blether gin (1 / 0) than 1 ither 2\n");
}

#[test]
fn interpreter_pipe_operator_error_propagation_is_covered() {
    // left evaluation error
    expect_interpret_err("blether (1 / 0) |> |x| x\n");
    // right evaluation error
    expect_interpret_err("blether 1 |> missing\n");
}

#[test]
fn interpreter_float_and_list_binary_ops_are_covered() {
    // Covers float arithmetic, mixed int+float, list concatenation, and modulo-on-float.
    assert!(run(
        r#"
blether 1.5 + 2.5
blether 1 + 2.5
blether 5.0 - 2.0
blether 2.0 * 3.0
blether 5.0 / 2.0
blether 5.0 % 2.0
blether [1, 2] + [3]
"#,
    )
    .is_ok());
}

#[test]
fn interpreter_division_by_zero_error_is_covered() {
    assert!(run("1 / 0\n").is_err());
}

#[test]
fn interpreter_negative_repeat_error_is_covered() {
    assert!(run("\"a\" * -1\n").is_err());
}
