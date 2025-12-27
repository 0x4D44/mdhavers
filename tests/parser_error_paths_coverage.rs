use mdhavers::{parse, HaversError};

#[test]
fn parser_exercises_common_error_paths_for_coverage() {
    let cases = [
        ("var_decl_unexpected_token_after_expr", "ken x = 1)"),
        ("statement_end_requires_separator", "ken x = 1 2"),
        ("destructure_missing_equals", "ken [a, b] 1"),
        ("destructure_rest_missing_name", "ken [... ] = [1]"),
        ("function_missing_right_paren", "dae foo(a, b { gie a }"),
        ("class_missing_body", "kin C"),
        ("dict_literal_missing_colon", "ken d = {\"a\" 1}"),
        ("dict_literal_missing_comma", "ken d = {\"a\": 1 \"b\": 2}"),
        ("list_literal_missing_comma", "ken xs = [1 2]"),
        ("unclosed_grouping", "(1 + 2"),
        ("unclosed_list_literal", "[1, 2"),
        ("match_missing_arrow", "keek x { whan 1 { blether 1 } }"),
        ("compound_assign_invalid_target", "1 += 2"),
        ("import_requires_string_path", "fetch 1"),
        ("import_alias_requires_identifier", "fetch \"m\" tae 1"),
    ];

    for (name, source) in cases {
        let err = parse(source).expect_err(name);
        match err {
            HaversError::UnexpectedToken { .. } | HaversError::ParseError { .. } => {}
            other => panic!("unexpected error type for {name}: {other:?}"),
        }
    }
}
