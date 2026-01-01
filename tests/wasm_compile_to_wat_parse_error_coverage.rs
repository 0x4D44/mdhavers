use mdhavers::HaversError;

#[test]
fn compile_to_wat_propagates_parse_errors_for_coverage() {
    let err = mdhavers::compile_to_wat("ken = 1").unwrap_err();
    assert!(matches!(
        err,
        HaversError::ParseError { .. } | HaversError::UnexpectedToken { .. } | HaversError::UnkentToken { .. }
    ));
}

