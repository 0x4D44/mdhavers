#![cfg(coverage)]

use mdhavers::error::{get_error_suggestion, HaversError};

#[test]
fn unexpected_token_closing_brace_suggestion_is_covered_in_dependency_instance() {
    let err = HaversError::UnexpectedToken {
        expected: "expression".to_string(),
        found: "}".to_string(),
        line: 1,
    };

    let suggestion = get_error_suggestion(&err).expect("expected suggestion");
    assert!(suggestion.contains("expression"));
}

#[test]
fn unexpected_token_equals_suggestion_is_covered_in_dependency_instance() {
    let err = HaversError::UnexpectedToken {
        expected: "expression".to_string(),
        found: "=".to_string(),
        line: 1,
    };

    let suggestion = get_error_suggestion(&err).expect("expected suggestion");
    assert!(suggestion.contains("=="));
}

#[test]
fn undefined_variable_string_suggestion_is_covered_in_dependency_instance() {
    let err = HaversError::UndefinedVariable {
        name: "String".to_string(),
        line: 1,
    };

    let suggestion = get_error_suggestion(&err).expect("expected suggestion");
    assert!(suggestion.contains("Strings are created"));
}

#[test]
fn unexpected_token_default_case_is_covered_in_dependency_instance() {
    use std::hint::black_box;

    let err = HaversError::UnexpectedToken {
        expected: "expression".to_string(),
        found: "?".to_string(),
        line: 1,
    };

    let _ = black_box(get_error_suggestion(&err));
}
