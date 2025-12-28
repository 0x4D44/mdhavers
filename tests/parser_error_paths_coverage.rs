use mdhavers::{parse, HaversError};

#[test]
fn parser_exercises_common_error_paths_for_coverage() {
    let cases = [
        ("var_decl_unexpected_token_after_expr", "ken x = 1)"),
        ("statement_end_requires_separator", "ken x = 1 2"),
        ("import_missing_stmt_end", "fetch \"m\" 1"),
        ("destructure_missing_equals", "ken [a, b] 1"),
        ("destructure_invalid_variable_name", "ken [1] = [1]"),
        ("destructure_missing_right_bracket", "ken [a, b"),
        ("destructure_value_expr_error", "ken [a] = )"),
        ("destructure_missing_stmt_end", "ken [a] = [1] 2"),
        ("destructure_rest_missing_name", "ken [... ] = [1]"),
        ("function_missing_right_paren", "dae foo(a, b { gie a }"),
        ("function_missing_name", "dae () { gie 1 }"),
        ("function_missing_left_paren", "dae foo a) { gie a }"),
        ("function_param_name_must_be_identifier", "dae foo(1) { gie 1 }"),
        ("function_default_param_expr_error", "dae foo(a = ) { gie a }"),
        ("function_missing_left_brace", "dae foo() gie 1"),
        ("return_value_expr_error", "gie )"),
        ("return_requires_stmt_end", "gie 1 2"),
        ("class_missing_body", "kin C"),
        ("class_missing_name", "kin { }"),
        ("class_superclass_requires_identifier", "kin C fae 1 { }"),
        ("class_method_invalid_signature", "kin C { dae () { gie 1 } }"),
        ("class_method_must_be_function", "kin C { ken x = 1 }"),
        ("class_missing_right_brace", "kin C { dae m() { gie 1 }"),
        ("struct_missing_name", "thing { }"),
        ("struct_missing_left_brace", "thing S a"),
        ("struct_field_requires_identifier", "thing S { 1 }"),
        ("struct_missing_right_brace", "thing S { a, b"),
        ("dict_literal_missing_colon", "ken d = {\"a\" 1}"),
        ("dict_literal_missing_comma", "ken d = {\"a\": 1 \"b\": 2}"),
        ("list_literal_missing_comma", "ken xs = [1 2]"),
        ("unclosed_grouping", "(1 + 2"),
        ("unclosed_list_literal", "[1, 2"),
        ("pipe_rhs_expr_error", "1 |> )"),
        ("ternary_condition_expr_error", "ken x = gin ) than 1 ither 2"),
        ("ternary_then_expr_error", "ken x = gin aye than ) ither 2"),
        ("ternary_else_expr_error", "ken x = gin aye than 1 ither )"),
        ("logical_or_rhs_expr_error", "aye or )"),
        ("logical_and_rhs_expr_error", "aye an )"),
        ("equality_rhs_expr_error", "1 == )"),
        ("comparison_rhs_expr_error", "1 < )"),
        ("term_rhs_expr_error", "1 + )"),
        ("factor_rhs_expr_error", "1 * )"),
        ("unary_operand_expr_error", "-)"),
        ("unary_nae_operand_expr_error", "nae ()"),
        ("unary_bang_operand_expr_error", "!()"),
        ("index_first_expr_error", "xs[)]"),
        ("index_missing_right_bracket", "xs[0"),
        ("slice_start_none_end_expr_error", "xs[:)]"),
        ("slice_start_none_step_expr_error", "xs[::)]"),
        ("slice_start_none_missing_right_bracket", "xs[:1"),
        ("slice_end_expr_error", "xs[0:)]"),
        ("slice_step_expr_error", "xs[0::)]"),
        ("slice_missing_right_bracket", "xs[0:1"),
        ("call_spread_expr_error", "foo(...)"),
        ("call_missing_right_paren", "foo(1"),
        ("list_spread_expr_error", "ken xs = [... ]"),
        ("list_element_expr_error", "ken xs = [)]"),
        ("compound_assign_value_expr_error", "ken x = 1\nx += )"),
        ("match_missing_arrow", "keek x { whan 1 { blether 1 } }"),
        ("match_missing_open_brace", "keek 1 whan 1 -> 1 }"),
        ("match_missing_whan", "keek 1 { 1 -> 1 }"),
        ("match_value_expr_error", "keek ) { whan 1 -> 1 }"),
        ("match_arm_block_body_parse_error", "keek 1 { whan 1 -> { blether 1"),
        ("match_arm_range_end_expr_error", "keek 1 { whan 1.. ) -> 1 }"),
        ("match_arm_expr_body_parse_error", "keek 1 { whan 1 -> ) }"),
        ("match_arm_print_missing_expr", "keek 1 { whan 1 -> blether }"),
        ("print_requires_stmt_end", "blether 1 2"),
        ("match_arm_break_missing_stmt_end", "keek 1 { whan 1 -> brak 1 }"),
        ("match_arm_continue_missing_stmt_end", "keek 1 { whan 1 -> haud 1 }"),
        ("match_arm_return_invalid_expr", "keek 1 { whan 1 -> gie ) }"),
        ("compound_assign_invalid_target", "1 += 2"),
        ("break_requires_stmt_end", "brak 1"),
        ("continue_requires_stmt_end", "haud 1"),
        ("else_if_parse_error", "gin aye { blether 1 } ither gin ) { blether 1 }"),
        ("else_block_parse_error", "gin aye { blether 1 } ither brak"),
        ("if_condition_expr_error", "gin ) { blether 1 }"),
        ("while_condition_expr_error", "whiles ) { blether 1 }"),
        ("while_missing_block", "whiles aye blether 1"),
        ("for_variable_must_be_identifier", "fer 1 in 0..3 { blether 1 }"),
        ("for_missing_in_keyword", "fer i 0..3 { blether 1 }"),
        ("for_iterable_expr_error", "fer i in ) { blether 1 }"),
        ("for_missing_block", "fer i in 0..3 blether 1"),
        ("try_missing_block", "hae_a_bash brak"),
        ("try_missing_catch_keyword", "hae_a_bash { } nope e { }"),
        ("try_catch_error_name_must_be_identifier", "hae_a_bash { } gin_it_gangs_wrang 1 { }"),
        ("try_catch_missing_catch_block", "hae_a_bash { } gin_it_gangs_wrang e brak"),
        ("import_requires_string_path", "fetch 1"),
        ("import_alias_requires_identifier", "fetch \"m\" tae 1"),
        ("assert_condition_expr_error", "mak_siccar )"),
        ("assert_message_expr_error", "mak_siccar aye, )"),
        ("log_missing_stmt_end", "log_mutter \"x\" 1"),
        ("log_extras_expr_error", "log_mutter \"x\", )"),
        ("log_second_extra_expr_error", "log_mutter \"x\", 1, )"),
        ("hurl_message_expr_error", "hurl )"),
        ("hurl_missing_stmt_end", "hurl \"x\" 1"),
        ("ternary_missing_than", "ken x = gin aye 1 ither 2"),
        ("ternary_missing_ither", "ken x = gin aye than 1 2"),
        ("property_missing_name", "ken x = {}."),
        ("speir_prompt_expr_error", "speir )"),
        ("fstring_inner_expr_error", "ken s = f\"{)}\""),
        ("dict_first_value_expr_error", "ken d = {\"a\": )}"),
        ("dict_second_key_expr_error", "ken d = {\"a\": 1, )}"),
        ("dict_second_key_missing_colon", "ken d = {\"a\": 1, \"b\" 2}"),
        ("dict_second_value_expr_error", "ken d = {\"a\": 1, \"b\": )}"),
        ("block_expr_missing_right_brace", "ken x = { ken y = 1"),
        ("lambda_missing_right_pipe", "ken f = |x, y x"),
        ("lambda_block_missing_right_brace", "ken f = |x| { ken y = 1"),
        ("lambda_expr_body_parse_error", "ken f = |x| )"),
        ("range_inclusive_end_expr_error", "1..= )"),
        ("range_exclusive_end_expr_error", "1.. )"),
        ("lexer_unterminated_string", "ken s = \"unterminated"),
    ];

    for (name, source) in cases {
        let err = parse(source).expect_err(name);
        match err {
            HaversError::UnexpectedToken { .. }
            | HaversError::ParseError { .. }
            | HaversError::UnkentToken { .. } => {}
            other => panic!("unexpected error type for {name}: {other:?}"),
        }
    }
}

#[test]
fn parser_accepts_single_quote_string_literals_for_coverage() {
    let program = parse("ken s = 'hi'").expect("single-quote string should parse");
    assert!(!program.statements.is_empty());
}

#[test]
fn parser_accepts_single_quote_import_paths_and_string_match_patterns_for_coverage() {
    let import = parse("fetch 'tri'").expect("single-quote import path should parse");
    assert!(!import.statements.is_empty());

    let match_prog = parse(
        r#"
keek "a" {
    whan "a" -> 1
    whan 'b' -> 2
    whan _ -> 0
}
"#,
    )
    .expect("string patterns should parse");
    assert!(!match_prog.statements.is_empty());
}
