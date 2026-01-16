#![cfg(all(feature = "llvm", coverage))]

use mdhavers::ast::{Expr, Literal, Program, Span, Stmt};
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

fn slice_program(
    object: Expr,
    start: Option<Expr>,
    end: Option<Expr>,
    step: Option<Expr>,
    span: Span,
) -> Program {
    Program::new(vec![Stmt::Expression {
        expr: Expr::Slice {
            object: Box::new(object),
            start: start.map(Box::new),
            end: end.map(Box::new),
            step: step.map(Box::new),
            span,
        },
        span,
    }])
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
fn llvm_codegen_covers_slice_object_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = slice_program(spread_outside_list(span), None, None, None, span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_slice_step_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let list_obj = Expr::List {
        elements: vec![Expr::Literal {
            value: Literal::Integer(1),
            span,
        }],
        span,
    };
    let program = slice_program(list_obj, None, None, Some(spread_outside_list(span)), span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_slice_string_start_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let string_obj = Expr::Literal {
        value: Literal::String("abc".to_string()),
        span,
    };
    let program = slice_program(
        string_obj,
        Some(spread_outside_list(span)),
        None,
        None,
        span,
    );

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_slice_string_end_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let string_obj = Expr::Literal {
        value: Literal::String("abc".to_string()),
        span,
    };
    let program = slice_program(
        string_obj,
        None,
        Some(spread_outside_list(span)),
        None,
        span,
    );

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_slice_list_start_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let list_obj = Expr::List {
        elements: vec![Expr::Literal {
            value: Literal::Integer(1),
            span,
        }],
        span,
    };
    let program = slice_program(list_obj, Some(spread_outside_list(span)), None, None, span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_slice_list_end_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let list_obj = Expr::List {
        elements: vec![Expr::Literal {
            value: Literal::Integer(1),
            span,
        }],
        span,
    };
    let program = slice_program(list_obj, None, Some(spread_outside_list(span)), None, span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}
