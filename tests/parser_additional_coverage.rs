use mdhavers::ast::{Expr, Literal, Pattern, Stmt};
use mdhavers::parser::Parser;
use mdhavers::token::{Token, TokenKind};
use mdhavers::{parse, HaversError};

#[test]
fn parser_treats_identifier_underscore_token_as_wildcard_pattern_for_coverage() {
    let tokens = vec![
        Token::new(TokenKind::Keek, "keek".to_string(), 1, 1),
        Token::new(TokenKind::Integer(1), "1".to_string(), 1, 6),
        Token::new(TokenKind::LeftBrace, "{".to_string(), 1, 8),
        Token::new(TokenKind::Whan, "whan".to_string(), 1, 10),
        Token::new(TokenKind::Identifier("_".to_string()), "_".to_string(), 1, 15),
        Token::new(TokenKind::Arrow, "->".to_string(), 1, 17),
        Token::new(TokenKind::Integer(0), "0".to_string(), 1, 20),
        Token::new(TokenKind::RightBrace, "}".to_string(), 1, 21),
        Token::eof(1),
    ];

    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("program should parse");
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            assert!(matches!(arms[0].pattern, Pattern::Wildcard));
        }
        other => panic!("expected match statement, got {other:?}"),
    }
}

#[test]
fn parser_reports_expected_pattern_error_for_coverage() {
    let err = parse("keek 1 { whan (1) -> 1 }").unwrap_err();
    match err {
        HaversError::ParseError { message, .. } => {
            assert!(message.starts_with("Expected pattern"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn parser_reports_invalid_assignment_target_for_coverage() {
    let err = parse("1 = 2").unwrap_err();
    match err {
        HaversError::ParseError { message, .. } => {
            assert_eq!(message, "Invalid assignment target");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn parser_accepts_bang_unary_and_slice_forms_for_coverage() {
    let program = parse(
        r#"
ken a = !aye
xs[::]
xs[0::]
xs[:3:]
xs[1:3:]
"#,
    )
    .expect("program should parse");
    assert!(program.statements.len() >= 5);
}

#[test]
fn parser_accepts_trailing_commas_and_statement_separators_for_coverage() {
    let program = parse(
        r#"
ken xs = [1, 2,]
ken d = {"a": 1,}
ken x = 1; ken y = 2
ken m = 1 ken n = 2
"#,
    )
    .expect("program should parse");
    assert_eq!(program.statements.len(), 6);
}

#[test]
fn parser_accepts_fstrings_with_escaped_and_unmatched_braces_for_coverage() {
    let program = parse(
        r#"
ken a = f"{{}}"
ken b = f"}}"
ken c = f"}"
ken d = f"{ {1: 2} }"
ken e = f"{\"a\\\\b\nc\"}"
"#,
    )
    .expect("program should parse");
    assert_eq!(program.statements.len(), 5);
}

#[test]
fn parser_process_escapes_handles_apostrophe_null_and_unknown_for_coverage() {
    let program = parse(r#"ken s = "hi\'\0\q""#).expect("string escapes should parse");
    assert_eq!(program.statements.len(), 1);

    let s = match &program.statements[0] {
        Stmt::VarDecl {
            initializer: Some(Expr::Literal { value, .. }),
            ..
        } => match value {
            Literal::String(s) => s.clone(),
            other => panic!("expected string literal, got {other:?}"),
        },
        other => panic!("expected var decl with string initializer, got {other:?}"),
    };

    assert_eq!(s, "hi'\0\\q".to_string());
}

#[test]
#[cfg(coverage)]
fn parser_coverage_only_wrappers_exercise_internal_branches() {
    assert!(mdhavers::parser::previous_is_none_at_start_for_coverage());
    assert!(!mdhavers::parser::is_nae_followed_by_operand_end_of_stream_for_coverage());
    assert_eq!(
        mdhavers::parser::process_escapes_for_coverage("hello\\"),
        "hello\\"
    );

    let err = mdhavers::parser::parse_fstring_for_coverage("{\\").unwrap_err();
    assert!(matches!(err, HaversError::UnkentToken { .. }));
}
