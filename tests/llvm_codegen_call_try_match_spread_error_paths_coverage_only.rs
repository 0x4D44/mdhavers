#![cfg(all(feature = "llvm", coverage))]

use mdhavers::ast::{BinaryOp, Expr, Literal, MatchArm, Param, Pattern, Program, Span, Stmt};
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

fn binary_with_nested_spread(span: Span) -> Expr {
    Expr::Binary {
        left: Box::new(spread_outside_list(span)),
        operator: BinaryOp::Add,
        right: Box::new(Expr::Literal {
            value: Literal::Integer(2),
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

fn program_with_user_function_call(arguments: Vec<Expr>, span: Span) -> Program {
    Program::new(vec![
        Stmt::Function {
            name: "f".to_string(),
            params: vec![
                Param {
                    name: "a".to_string(),
                    default: None,
                },
                Param {
                    name: "b".to_string(),
                    default: None,
                },
            ],
            body: Vec::new(),
            span,
        },
        Stmt::Expression {
            expr: Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "f".to_string(),
                    span,
                }),
                arguments,
                span,
            },
            span,
        },
    ])
}

fn program_with_try_catch(try_block: Stmt, catch_block: Stmt, span: Span) -> Program {
    Program::new(vec![Stmt::TryCatch {
        try_block: Box::new(try_block),
        error_name: "e".to_string(),
        catch_block: Box::new(catch_block),
        span,
    }])
}

fn program_with_match(value: Expr, span: Span) -> Program {
    Program::new(vec![Stmt::Match {
        value,
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            body: Stmt::Expression {
                expr: Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                },
                span,
            },
            span,
        }],
        span,
    }])
}

#[test]
fn llvm_codegen_covers_user_function_call_arg_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = program_with_user_function_call(vec![binary_with_nested_spread(span)], span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_user_function_call_non_spread_arg_error_path_when_any_spread_present_for_coverage(
) {
    let span = Span::new(1, 1);
    let spread_list = Expr::Spread {
        expr: Box::new(Expr::List {
            elements: vec![Expr::Literal {
                value: Literal::Integer(1),
                span,
            }],
            span,
        }),
        span,
    };
    let program = program_with_user_function_call(
        vec![binary_with_nested_spread(span), spread_list],
        span,
    );

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_user_function_call_spread_inner_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let nested_spread = Expr::Spread {
        expr: Box::new(spread_outside_list(span)),
        span,
    };
    let program = program_with_user_function_call(vec![nested_spread], span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_try_block_stmt_compile_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let try_block = Stmt::Block {
        statements: vec![Stmt::Expression {
            expr: spread_outside_list(span),
            span,
        }],
        span,
    };
    let catch_block = Stmt::Block {
        statements: Vec::new(),
        span,
    };
    let program = program_with_try_catch(try_block, catch_block, span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_try_non_block_stmt_compile_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let try_block = Stmt::Expression {
        expr: spread_outside_list(span),
        span,
    };
    let catch_block = Stmt::Block {
        statements: Vec::new(),
        span,
    };
    let program = program_with_try_catch(try_block, catch_block, span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_catch_block_stmt_compile_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let try_block = Stmt::Block {
        statements: Vec::new(),
        span,
    };
    let catch_block = Stmt::Block {
        statements: vec![Stmt::Expression {
            expr: spread_outside_list(span),
            span,
        }],
        span,
    };
    let program = program_with_try_catch(try_block, catch_block, span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_catch_non_block_stmt_compile_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let try_block = Stmt::Block {
        statements: Vec::new(),
        span,
    };
    let catch_block = Stmt::Expression {
        expr: spread_outside_list(span),
        span,
    };
    let program = program_with_try_catch(try_block, catch_block, span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

#[test]
fn llvm_codegen_covers_match_value_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = program_with_match(spread_outside_list(span), span);

    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected spread outside list to be a compile error");
    assert_spread_outside_list_compile_error(err);
}

