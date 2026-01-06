#![cfg(all(feature = "llvm", coverage))]

use std::fs;

use mdhavers::ast::{Expr, Literal, Param, Program, Span, Stmt};
use mdhavers::{HaversError, LLVMCompiler};

const SPREAD_ERR: &str = "Spread operator can only be used inside list literals";
const MASEL_ERR: &str = "'masel' used outside of a method";

fn spread_outside_list(span: Span) -> Expr {
    Expr::Spread {
        expr: Box::new(Expr::Literal {
            value: Literal::Integer(1),
            span,
        }),
        span,
    }
}

fn assert_compile_error_contains(err: HaversError, expected_substring: &str) {
    match err {
        HaversError::CompileError(message) => assert!(
            message.contains(expected_substring),
            "unexpected error message: {message}"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}

fn assert_program_compile_error_contains(program: Program, expected_substring: &str) {
    let compiler = LLVMCompiler::new();
    let err = compiler
        .compile_to_ir(&program)
        .expect_err("expected compile error");
    assert_compile_error_contains(err, expected_substring);
}

#[test]
fn llvm_codegen_covers_user_function_default_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        Stmt::Function {
            name: "f".to_string(),
            params: vec![
                Param {
                    name: "a".to_string(),
                    default: None,
                },
                Param {
                    name: "b".to_string(),
                    default: Some(spread_outside_list(span)),
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
                arguments: vec![Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }],
                span,
            },
            span,
        },
    ]);

    assert_program_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_covers_imported_function_default_compile_expr_error_path_for_coverage() {
    let dir = tempfile::Builder::new()
        .prefix("import_default_expr_err")
        .tempdir_in("target")
        .expect("tempdir");

    let module_path = dir.path().join("m.braw");
    fs::write(
        &module_path,
        r#"
dae f(a, b = masel) { }
"#,
    )
    .expect("write module");

    let span = Span::new(1, 1);
    let program = Program::new(vec![
        Stmt::Import {
            path: module_path.to_string_lossy().to_string(),
            alias: None,
            span,
        },
        Stmt::Expression {
            expr: Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "f".to_string(),
                    span,
                }),
                arguments: vec![Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                }],
                span,
            },
            span,
        },
    ]);

    assert_program_compile_error_contains(program, MASEL_ERR);
}

#[test]
fn llvm_codegen_covers_assert_condition_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Assert {
        condition: spread_outside_list(span),
        message: None,
        span,
    }]);

    assert_program_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_covers_assert_message_compile_expr_error_path_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Assert {
        condition: Expr::Literal {
            value: Literal::Bool(true),
            span,
        },
        message: Some(spread_outside_list(span)),
        span,
    }]);

    assert_program_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_covers_method_default_compile_expr_error_in_dyn_dispatch_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        Stmt::Class {
            name: "C".to_string(),
            superclass: None,
            methods: vec![Stmt::Function {
                name: "m".to_string(),
                params: vec![Param {
                    name: "x".to_string(),
                    default: Some(spread_outside_list(span)),
                }],
                body: Vec::new(),
                span,
            }],
            span,
        },
        Stmt::Expression {
            expr: Expr::Call {
                callee: Box::new(Expr::Get {
                    object: Box::new(Expr::Call {
                        callee: Box::new(Expr::Variable {
                            name: "C".to_string(),
                            span,
                        }),
                        arguments: Vec::new(),
                        span,
                    }),
                    property: "m".to_string(),
                    span,
                }),
                arguments: Vec::new(),
                span,
            },
            span,
        },
    ]);

    assert_program_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_covers_method_default_compile_expr_error_in_direct_call_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![
        Stmt::Class {
            name: "C".to_string(),
            superclass: None,
            methods: vec![Stmt::Function {
                name: "m".to_string(),
                params: vec![Param {
                    name: "x".to_string(),
                    default: Some(spread_outside_list(span)),
                }],
                body: Vec::new(),
                span,
            }],
            span,
        },
        Stmt::VarDecl {
            name: "x".to_string(),
            initializer: Some(Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "C".to_string(),
                    span,
                }),
                arguments: Vec::new(),
                span,
            }),
            span,
        },
        Stmt::Expression {
            expr: Expr::Call {
                callee: Box::new(Expr::Get {
                    object: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span,
                    }),
                    property: "m".to_string(),
                    span,
                }),
                arguments: Vec::new(),
                span,
            },
            span,
        },
    ]);

    assert_program_compile_error_contains(program, SPREAD_ERR);
}

#[test]
fn llvm_codegen_covers_compile_class_skips_non_function_methods_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Class {
        name: "Skip".to_string(),
        superclass: None,
        methods: vec![
            Stmt::Expression {
                expr: Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                },
                span,
            },
            Stmt::Function {
                name: "m".to_string(),
                params: Vec::new(),
                body: Vec::new(),
                span,
            },
        ],
        span,
    }]);

    let compiler = LLVMCompiler::new();
    compiler
        .compile_to_ir(&program)
        .expect("expected compile to succeed");
}
