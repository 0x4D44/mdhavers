#![cfg(all(feature = "llvm", coverage))]

use mdhavers::ast::{DestructPattern, Expr, Literal, MatchArm, Pattern, Program, Span, Stmt};
use mdhavers::{HaversError, LLVMCompiler};

fn spread_outside_list(span: Span) -> Expr {
    Expr::Spread {
        expr: Box::new(Expr::Literal {
            value: Literal::Integer(1),
            span,
        }),
        span,
    }
}

fn assert_spread_outside_list_compile_error(err: HaversError) {
    match err {
        HaversError::CompileError(message) => {
            assert!(
                message.contains("Spread operator can only be used inside list literals"),
                "unexpected error message: {message}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn llvm_codegen_covers_set_object_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Expression {
        expr: Expr::Set {
            object: Box::new(spread_outside_list(span)),
            property: "prop".to_string(),
            value: Box::new(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            span,
        },
        span,
    }]);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_set_value_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Expression {
        expr: Expr::Set {
            object: Box::new(Expr::Dict {
                pairs: vec![],
                span,
            }),
            property: "prop".to_string(),
            value: Box::new(spread_outside_list(span)),
            span,
        },
        span,
    }]);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_destructure_value_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Destructure {
        patterns: vec![DestructPattern::Variable("a".to_string())],
        value: spread_outside_list(span),
        span,
    }]);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_match_range_start_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Match {
        value: Expr::Literal {
            value: Literal::Integer(0),
            span,
        },
        arms: vec![MatchArm {
            pattern: Pattern::Range {
                start: Box::new(spread_outside_list(span)),
                end: Box::new(Expr::Literal {
                    value: Literal::Integer(10),
                    span,
                }),
            },
            body: Stmt::Block {
                statements: vec![],
                span,
            },
            span,
        }],
        span,
    }]);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_match_range_end_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Match {
        value: Expr::Literal {
            value: Literal::Integer(0),
            span,
        },
        arms: vec![MatchArm {
            pattern: Pattern::Range {
                start: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                end: Box::new(spread_outside_list(span)),
            },
            body: Stmt::Block {
                statements: vec![],
                span,
            },
            span,
        }],
        span,
    }]);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_for_range_start_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::For {
        variable: "i".to_string(),
        iterable: Expr::Range {
            start: Box::new(spread_outside_list(span)),
            end: Box::new(Expr::Literal {
                value: Literal::Integer(10),
                span,
            }),
            inclusive: false,
            span,
        },
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    }]);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_for_range_end_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::For {
        variable: "i".to_string(),
        iterable: Expr::Range {
            start: Box::new(Expr::Literal {
                value: Literal::Integer(0),
                span,
            }),
            end: Box::new(spread_outside_list(span)),
            inclusive: false,
            span,
        },
        body: Box::new(Stmt::Block {
            statements: vec![],
            span,
        }),
        span,
    }]);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

