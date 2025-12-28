use mdhavers::ast::{Expr, Literal, Program, Span, Stmt};
use mdhavers::interpreter::TraceMode;
use mdhavers::value::RangeValue;
use mdhavers::{parse, HaversError, Interpreter, Value};

fn run(source: &str) -> Result<Value, HaversError> {
    let program = parse(source).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program)
}

#[test]
fn interpreter_for_loop_over_injected_range_value_is_covered() {
    let program = parse(
        r#"
ken sum = 0
fer i in r {
    sum = sum + i
}
sum
"#,
    )
    .unwrap();

    let mut interp = Interpreter::new();
    interp.globals.borrow_mut().define(
        "r".to_string(),
        Value::Range(RangeValue::new(1, 4, false)),
    );

    let value = interp.interpret(&program).unwrap();
    assert_eq!(value, Value::Integer(6));
}

#[test]
fn interpreter_trace_break_and_continue_paths_are_covered() {
    let program = parse(
        r#"
ken n = 0
whiles aye {
    n = n + 1
    gin n == 1 { haud }
    brak
}
n
"#,
    )
    .unwrap();

    let mut interp = Interpreter::new();
    interp.set_trace_mode(TraceMode::Verbose);
    let value = interp.interpret(&program).unwrap();
    assert_eq!(value, Value::Integer(2));
}

#[test]
fn interpreter_try_catch_success_path_is_covered() {
    let value = run(
        r#"
hae_a_bash { ken x = 1 } gin_it_gangs_wrang e { blether e }
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Nil);
}

#[test]
fn interpreter_assert_default_message_branch_is_covered() {
    let err = run("mak_siccar nae").unwrap_err();
    match err {
        HaversError::AssertionFailed { message, .. } => {
            assert!(message.contains("Assertion failed"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn interpreter_destructure_too_few_elements_error_is_covered() {
    let err = run("ken [a, b] = [1]").unwrap_err();
    match err {
        HaversError::TypeError { message, .. } => {
            assert!(message.contains("need at least"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn interpreter_dict_property_set_and_invalid_target_are_covered() {
    assert_eq!(
        run(
            r#"
ken d = {"a": 1}
d.a = 2
d.a
"#,
        )
        .unwrap(),
        Value::Integer(2)
    );

    let err = run(
        r#"
ken x = 1
x.a = 2
"#,
    )
    .unwrap_err();
    match err {
        HaversError::TypeError { message, .. } => {
            assert!(message.contains("Cannae set property"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn interpreter_negative_list_index_get_set_and_string_oob_are_covered() {
    assert_eq!(
        run(
            r#"
ken xs = [1, 2, 3]
xs[-1] = 9
xs[-1]
"#,
        )
        .unwrap(),
        Value::Integer(9)
    );

    let err = run(r#""hi"[5]"#).unwrap_err();
    match err {
        HaversError::IndexOutOfBounds { .. } => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn interpreter_class_superclass_not_class_error_is_covered() {
    let err = run(
        r#"
ken A = 1
kin C fae A { dae m() { gie 1 } }
"#,
    )
    .unwrap_err();

    match err {
        HaversError::TypeError { message, .. } => {
            assert!(message.contains("isnae a class"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn interpreter_class_method_non_function_path_is_covered() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Class {
        name: "Weird".to_string(),
        superclass: None,
        methods: vec![Stmt::VarDecl {
            name: "nope".to_string(),
            initializer: Some(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            span,
        }],
        span,
    }]);

    let mut interp = Interpreter::new();
    interp
        .interpret(&program)
        .expect("class with non-function methods should still interpret");

    let weird = interp.globals.borrow().get("Weird");
    match weird {
        Some(Value::Class(_)) => {}
        other => panic!("expected Weird to be defined as class, got {other:?}"),
    }
}

#[test]
fn interpreter_division_by_zero_error_is_covered() {
    let err = run("1 / 0").unwrap_err();
    match err {
        HaversError::DivisionByZero { .. } => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
