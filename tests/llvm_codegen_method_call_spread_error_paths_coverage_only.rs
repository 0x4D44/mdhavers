#![cfg(all(feature = "llvm", coverage))]

use mdhavers::ast::{Expr, Literal, Program, Span, Stmt};
use mdhavers::{HaversError, LLVMCompiler};

const SPREAD_ERR: &str = "Spread operator can only be used inside list literals";

fn compile_to_ir_err(program: Program) -> HaversError {
    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_ir(&program)
        .expect_err("expected compile error")
}

fn assert_compile_error_contains(err: HaversError, needle: &str) {
    match err {
        HaversError::CompileError(message) => assert!(
            message.contains(needle),
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

fn method_call(receiver: Expr, args: Vec<Expr>, span: Span) -> Expr {
    Expr::Call {
        callee: Box::new(Expr::Get {
            object: Box::new(receiver),
            property: "m".to_string(),
            span,
        }),
        arguments: args,
        span,
    }
}

#[test]
fn llvm_codegen_method_call_receiver_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Expression {
        expr: method_call(spread_expr(span), Vec::new(), span),
        span,
    }]);

    let err = compile_to_ir_err(program);
    assert_compile_error_contains(err, SPREAD_ERR);
}

#[test]
fn llvm_codegen_method_call_arg_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Expression {
        expr: method_call(
            Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            vec![spread_expr(span)],
            span,
        ),
        span,
    }]);

    let err = compile_to_ir_err(program);
    assert_compile_error_contains(err, SPREAD_ERR);
}

#[test]
fn llvm_codegen_class_instantiation_arg_compile_expr_error_is_covered_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        Stmt::Class {
            name: "C".to_string(),
            superclass: None,
            methods: Vec::new(),
            span,
        },
        Stmt::Expression {
            expr: Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "C".to_string(),
                    span,
                }),
                arguments: vec![spread_expr(span)],
                span,
            },
            span,
        },
    ]);

    let err = compile_to_ir_err(program);
    assert_compile_error_contains(err, SPREAD_ERR);
}
