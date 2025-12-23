use mdhavers::HaversError;

#[test]
fn havers_error_with_line_if_zero_sets_line_on_all_supported_variants() {
    let line = 123;

    let cases: Vec<HaversError> = vec![
        HaversError::UnkentToken {
            lexeme: "?".to_string(),
            line: 0,
            column: 9,
        },
        HaversError::UnexpectedToken {
            expected: "ken".to_string(),
            found: "nae".to_string(),
            line: 0,
        },
        HaversError::UndefinedVariable {
            name: "x".to_string(),
            line: 0,
        },
        HaversError::DivisionByZero { line: 0 },
        HaversError::TypeError {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::NotCallable {
            name: "x".to_string(),
            line: 0,
        },
        HaversError::WrongArity {
            name: "f".to_string(),
            expected: 2,
            got: 1,
            line: 0,
        },
        HaversError::IndexOutOfBounds {
            index: 9,
            size: 1,
            line: 0,
        },
        HaversError::ParseError {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::BreakOutsideLoop { line: 0 },
        HaversError::ContinueOutsideLoop { line: 0 },
        HaversError::StackOverflow { line: 0 },
        HaversError::UnterminatedString { line: 0 },
        HaversError::InvalidNumber {
            value: "NaN".to_string(),
            line: 0,
        },
        HaversError::AlreadyDefined {
            name: "x".to_string(),
            line: 0,
        },
        HaversError::NotAnObject {
            name: "x".to_string(),
            line: 0,
        },
        HaversError::UndefinedProperty {
            property: "x".to_string(),
            line: 0,
        },
        HaversError::InfiniteLoop { line: 0 },
        HaversError::NotAList { line: 0 },
        HaversError::NotADict { line: 0 },
        HaversError::KeyNotFound {
            key: "x".to_string(),
            line: 0,
        },
        HaversError::InvalidOperation {
            operation: "nope".to_string(),
            line: 0,
        },
        HaversError::AssertionFailed {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::ReturnOutsideFunction { line: 0 },
        HaversError::NotIterable {
            type_name: "naething".to_string(),
            line: 0,
        },
        HaversError::PatternError {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::IntegerOverflow { line: 0 },
        HaversError::NegativeIndexOutOfBounds { index: -1, line: 0 },
        HaversError::EmptyCollection {
            operation: "maxaw".to_string(),
            line: 0,
        },
        HaversError::InvalidRegex {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::FormatError {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::JsonError {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::IncomparableTypes {
            left_type: "int".to_string(),
            right_type: "string".to_string(),
            line: 0,
        },
        HaversError::InvalidNumberOperation {
            message: "nope".to_string(),
            line: 0,
        },
        HaversError::NonExhaustiveMatch { line: 0 },
        HaversError::DuplicateKey {
            key: "x".to_string(),
            line: 0,
        },
        HaversError::ExecutionTimeout { line: 0 },
        HaversError::OutOfMemory { line: 0 },
        HaversError::PrivateMemberAccess {
            member: "x".to_string(),
            line: 0,
        },
        HaversError::ImmutableVariable {
            name: "x".to_string(),
            line: 0,
        },
        HaversError::UserError {
            message: "nope".to_string(),
            line: 0,
        },
    ];

    for err in cases {
        let updated = err.clone().with_line_if_zero(line);
        assert_eq!(
            updated.line(),
            Some(line),
            "expected {:?} to adopt line {line}, got {:?}",
            err,
            updated
        );
    }
}

#[test]
fn havers_error_with_line_if_zero_does_not_override_existing_line() {
    let err = HaversError::DivisionByZero { line: 7 };
    let updated = err.clone().with_line_if_zero(123);
    assert_eq!(updated, err);
    assert_eq!(updated.line(), Some(7));
}

#[test]
fn havers_error_with_line_if_zero_is_noop_for_non_line_variants() {
    let err = HaversError::ModuleNotFound {
        name: "missing".to_string(),
    };
    let updated = err.clone().with_line_if_zero(123);
    assert_eq!(updated, err);
    assert_eq!(updated.line(), None);
}

