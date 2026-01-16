#![cfg(all(feature = "llvm", coverage))]

use mdhavers::ast::{Expr, Literal, Program, Span, Stmt};
use mdhavers::{HaversError, LLVMCompiler};

fn call_assert_with_args(args: Vec<Expr>, span: Span) -> Program {
    let call = Expr::Call {
        callee: Box::new(Expr::Variable {
            name: "assert".to_string(),
            span,
        }),
        arguments: args,
        span,
    };

    Program::new(vec![Stmt::Expression { expr: call, span }])
}

#[test]
fn llvm_codegen_covers_assert_arg_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = call_assert_with_args(
        vec![
            Expr::Spread {
                expr: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                span,
            },
            Expr::Literal {
                value: Literal::String("msg".to_string()),
                span,
            },
        ],
        span,
    );

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");

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
fn llvm_codegen_covers_assert_msg_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = call_assert_with_args(
        vec![
            Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            Expr::Spread {
                expr: Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }),
                span,
            },
        ],
        span,
    );

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");

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
