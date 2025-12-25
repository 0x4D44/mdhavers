use mdhavers::run;

#[test]
fn interpreter_log_statement_extras_evaluation_error_is_covered() {
    // Hit the `parse_log_extras` error-propagation path (`self.evaluate(expr)?`) by
    // making the extra expression fail at runtime.
    assert!(run("log_blether \"hi\", nope\n").is_err());
}

#[test]
fn interpreter_log_statement_message_evaluation_error_is_covered() {
    // Hit the `Stmt::Log` message evaluation error branch (`self.evaluate(message)?`).
    assert!(run("log_blether nope\n").is_err());
}

#[test]
fn interpreter_log_init_invalid_filter_is_covered() {
    // Hit `logging::set_filter(&filter_str)?` via an invalid filter spec.
    assert!(run("log_init({\"filter\": \"nope\"})\n").is_err());
}
