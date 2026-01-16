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
fn llvm_codegen_covers_block_expr_stmt_compile_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Expression {
        expr: Expr::BlockExpr {
            statements: vec![Stmt::Expression {
                expr: spread_outside_list(span),
                span,
            }],
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
