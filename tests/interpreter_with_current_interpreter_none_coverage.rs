use mdhavers::{Interpreter, Value};

#[test]
fn interpreter_log_event_outside_interpreter_hits_missing_current_interpreter_branch_for_coverage() {
    let interp = Interpreter::new();
    let exports = interp.globals.borrow().get_exports();
    let log_event = exports
        .into_iter()
        .find_map(|(name, value)| (name == "log_event").then_some(value))
        .expect("log_event not found");

    let Value::NativeFunction(log_event) = log_event else {
        panic!("expected log_event native function");
    };

    let err = (log_event.func)(vec![
        Value::Integer(3),
        Value::String("msg".to_string()),
    ])
    .expect_err("expected log_event to fail outside the interpreter");
    assert!(err.contains("unavailable outside the interpreter"));
}

#[test]
fn interpreter_log_init_outside_interpreter_hits_missing_current_interpreter_branch_for_coverage() {
    let interp = Interpreter::new();
    let exports = interp.globals.borrow().get_exports();
    let log_init = exports
        .into_iter()
        .find_map(|(name, value)| (name == "log_init").then_some(value))
        .expect("log_init not found");

    let Value::NativeFunction(log_init) = log_init else {
        panic!("expected log_init native function");
    };

    let err = (log_init.func)(vec![]).expect_err("expected log_init to fail outside the interpreter");
    assert!(err.contains("unavailable outside the interpreter"));
}

#[test]
fn interpreter_log_span_outside_interpreter_uses_default_target_for_coverage() {
    let interp = Interpreter::new();
    let exports = interp.globals.borrow().get_exports();
    let log_span = exports
        .into_iter()
        .find_map(|(name, value)| (name == "log_span").then_some(value))
        .expect("log_span not found");

    let Value::NativeFunction(log_span) = log_span else {
        panic!("expected log_span native function");
    };

    let span = (log_span.func)(vec![Value::String("cov".to_string())])
        .expect("expected log_span to succeed outside the interpreter");
    assert!(matches!(span, Value::NativeObject(_)));
}

#[test]
fn interpreter_log_span_in_outside_interpreter_hits_missing_current_interpreter_branch_for_coverage() {
    let interp = Interpreter::new();
    let exports = interp.globals.borrow().get_exports();
    let log_span = exports
        .iter()
        .find_map(|(name, value)| (name == "log_span").then_some(value.clone()))
        .expect("log_span not found");
    let log_span_in = exports
        .into_iter()
        .find_map(|(name, value)| (name == "log_span_in").then_some(value))
        .expect("log_span_in not found");

    let Value::NativeFunction(log_span) = log_span else {
        panic!("expected log_span native function");
    };
    let Value::NativeFunction(log_span_in) = log_span_in else {
        panic!("expected log_span_in native function");
    };

    let span = (log_span.func)(vec![Value::String("cov-in".to_string())])
        .expect("expected log_span to succeed outside the interpreter");
    let err = (log_span_in.func)(vec![span, Value::Nil])
        .expect_err("expected log_span_in to fail outside the interpreter");
    assert!(err.contains("unavailable outside the interpreter"));
}
