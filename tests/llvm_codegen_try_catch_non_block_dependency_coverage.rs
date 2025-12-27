#![cfg(all(feature = "llvm", coverage))]

use mdhavers::ast::{Expr, Literal, Program, Span, Stmt};
use mdhavers::llvm::LLVMCompiler;

#[test]
fn llvm_codegen_try_catch_non_block_bodies_are_covered_in_dependency_instance() {
    let span = Span::new(1, 1);
    let try_stmt = Stmt::Expression {
        expr: Expr::Literal {
            value: Literal::Integer(1),
            span,
        },
        span,
    };
    let catch_stmt = Stmt::Expression {
        expr: Expr::Literal {
            value: Literal::Integer(2),
            span,
        },
        span,
    };
    let stmt = Stmt::TryCatch {
        try_block: Box::new(try_stmt),
        error_name: "e".to_string(),
        catch_block: Box::new(catch_stmt),
        span,
    };
    let program = Program::new(vec![stmt]);

    let ir = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect("expected compilation to succeed");
    assert!(!ir.is_empty());
}

