#![cfg(all(feature = "llvm", coverage))]

use mdhavers::ast::{Expr, Literal, Program, Span, Stmt};
use mdhavers::{HaversError, LLVMCompiler};

const SPREAD_ERR: &str = "Spread operator can only be used inside list literals";

fn assert_compile_error_contains(program: Program, expected_substring: &str) {
    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected compile error");

    match err {
        HaversError::CompileError(message) => assert!(
            message.contains(expected_substring),
            "unexpected error message: {message}"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}

fn spread_expr(span: Span) -> Expr {
    Expr::Spread {
        expr: Box::new(Expr::Literal {
            value: Literal::Integer(1),
            span,
        }),
        span,
    }
}

fn empty_dict(span: Span) -> Expr {
    Expr::Dict {
        pairs: Vec::new(),
        span,
    }
}

fn declare(name: &str, initializer: Expr, span: Span) -> Stmt {
    Stmt::VarDecl {
        name: name.to_string(),
        initializer: Some(initializer),
        span,
    }
}

fn index_set(object: Expr, index: Expr, value: Expr, span: Span) -> Expr {
    Expr::IndexSet {
        object: Box::new(object),
        index: Box::new(index),
        value: Box::new(value),
        span,
    }
}

#[test]
fn llvm_codegen_index_set_general_object_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Expression {
        expr: index_set(
            spread_expr(span),
            Expr::Literal {
                value: Literal::Integer(0),
                span,
            },
            Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            span,
        ),
        span,
    }]);

    assert_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_index_set_general_index_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        declare(
            "x",
            Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            span,
        ),
        Stmt::Expression {
            expr: index_set(
                Expr::Variable {
                    name: "x".to_string(),
                    span,
                },
                spread_expr(span),
                Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                },
                span,
            ),
            span,
        },
    ]);

    assert_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_index_set_general_value_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        declare(
            "x",
            Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            span,
        ),
        Stmt::Expression {
            expr: index_set(
                Expr::Variable {
                    name: "x".to_string(),
                    span,
                },
                Expr::Literal {
                    value: Literal::Integer(0),
                    span,
                },
                spread_expr(span),
                span,
            ),
            span,
        },
    ]);

    assert_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_index_set_dict_object_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let bad_dict = Expr::Dict {
        pairs: vec![(
            Expr::Literal {
                value: Literal::String("a".to_string()),
                span,
            },
            spread_expr(span),
        )],
        span,
    };
    let program = Program::new(vec![Stmt::Expression {
        expr: index_set(
            bad_dict,
            Expr::Literal {
                value: Literal::String("a".to_string()),
                span,
            },
            Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            span,
        ),
        span,
    }]);

    assert_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_index_set_dict_index_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        declare("d", empty_dict(span), span),
        Stmt::Expression {
            expr: index_set(
                Expr::Variable {
                    name: "d".to_string(),
                    span,
                },
                spread_expr(span),
                Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                },
                span,
            ),
            span,
        },
    ]);

    assert_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_index_set_dict_value_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        declare("d", empty_dict(span), span),
        Stmt::Expression {
            expr: index_set(
                Expr::Variable {
                    name: "d".to_string(),
                    span,
                },
                Expr::Literal {
                    value: Literal::String("a".to_string()),
                    span,
                },
                spread_expr(span),
                span,
            ),
            span,
        },
    ]);

    assert_compile_error_contains(program, SPREAD_ERR);
}
