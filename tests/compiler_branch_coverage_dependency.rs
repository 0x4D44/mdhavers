#![cfg(coverage)]

use mdhavers::ast::*;
use mdhavers::compiler::Compiler;

#[test]
fn compiler_dependency_branch_matrix_for_coverage() {
    let span = Span::new(1, 1);

    let statements = vec![
        Stmt::Import {
            path: "tri".to_string(),
            alias: Some("tri".to_string()),
            span,
        },
        Stmt::Import {
            path: "tri.js".to_string(),
            alias: Some("tri_js".to_string()),
            span,
        },
        Stmt::Import {
            path: "tri.braw".to_string(),
            alias: Some("tri_braw".to_string()),
            span,
        },
        Stmt::Import {
            path: "lib/util.braw".to_string(),
            alias: None,
            span,
        },
        Stmt::VarDecl {
            name: "x".to_string(),
            initializer: None,
            span,
        },
        Stmt::VarDecl {
            name: "y".to_string(),
            initializer: Some(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            span,
        },
        Stmt::If {
            condition: Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            then_branch: Box::new(Stmt::Break { span }),
            else_branch: None,
            span,
        },
        Stmt::If {
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            then_branch: Box::new(Stmt::Break { span }),
            else_branch: Some(Box::new(Stmt::Break { span })),
            span,
        },
        Stmt::Return { value: None, span },
        Stmt::Return {
            value: Some(Expr::Literal {
                value: Literal::Integer(2),
                span,
            }),
            span,
        },
        Stmt::Class {
            name: "C".to_string(),
            superclass: Some("Base".to_string()),
            methods: vec![
                Stmt::Function {
                    name: "init".to_string(),
                    params: Vec::new(),
                    body: vec![Stmt::Return { value: None, span }],
                    span,
                },
                Stmt::Function {
                    name: "ping".to_string(),
                    params: Vec::new(),
                    body: vec![Stmt::Return {
                        value: Some(Expr::Literal {
                            value: Literal::Integer(3),
                            span,
                        }),
                        span,
                    }],
                    span,
                },
            ],
            span,
        },
        Stmt::Match {
            value: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            arms: vec![
                MatchArm {
                    pattern: Pattern::Identifier("val".to_string()),
                    body: Stmt::Break { span },
                    span,
                },
                MatchArm {
                    pattern: Pattern::Literal(Literal::Integer(2)),
                    body: Stmt::Break { span },
                    span,
                },
            ],
            span,
        },
        Stmt::Match {
            value: Expr::Literal {
                value: Literal::Integer(1),
                span,
            },
            arms: Vec::new(),
            span,
        },
        Stmt::Assert {
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            message: None,
            span,
        },
        Stmt::Assert {
            condition: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            message: Some(Expr::Literal {
                value: Literal::String("nope".to_string()),
                span,
            }),
            span,
        },
        Stmt::Destructure {
            patterns: vec![
                DestructPattern::Variable("a".to_string()),
                DestructPattern::Ignore,
            ],
            value: Expr::List {
                elements: vec![
                    Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    Expr::Literal {
                        value: Literal::Integer(2),
                        span,
                    },
                ],
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Literal {
                value: Literal::Bool(true),
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: "foo".to_string(),
                    span,
                }),
                arguments: vec![
                    Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    Expr::Literal {
                        value: Literal::Integer(2),
                        span,
                    },
                ],
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Slice {
                object: Box::new(Expr::Variable {
                    name: "xs".to_string(),
                    span,
                }),
                start: None,
                end: None,
                step: Some(Box::new(Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                })),
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Slice {
                object: Box::new(Expr::Variable {
                    name: "xs".to_string(),
                    span,
                }),
                start: Some(Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                })),
                end: Some(Box::new(Expr::Literal {
                    value: Literal::Integer(3),
                    span,
                })),
                step: Some(Box::new(Expr::Literal {
                    value: Literal::Integer(1),
                    span,
                })),
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Slice {
                object: Box::new(Expr::Variable {
                    name: "xs".to_string(),
                    span,
                }),
                start: Some(Box::new(Expr::Literal {
                    value: Literal::Integer(0),
                    span,
                })),
                end: None,
                step: None,
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Slice {
                object: Box::new(Expr::Variable {
                    name: "xs".to_string(),
                    span,
                }),
                start: None,
                end: Some(Box::new(Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                })),
                step: None,
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Range {
                start: Box::new(Expr::Literal {
                    value: Literal::Integer(0),
                    span,
                }),
                end: Box::new(Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                }),
                inclusive: true,
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::Range {
                start: Box::new(Expr::Literal {
                    value: Literal::Integer(0),
                    span,
                }),
                end: Box::new(Expr::Literal {
                    value: Literal::Integer(2),
                    span,
                }),
                inclusive: false,
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::List {
                elements: vec![
                    Expr::Literal {
                        value: Literal::Integer(1),
                        span,
                    },
                    Expr::Literal {
                        value: Literal::Integer(2),
                        span,
                    },
                ],
                span,
            },
            span,
        },
        Stmt::Expression {
            expr: Expr::FString {
                parts: vec![FStringPart::Text("price $".to_string())],
                span,
            },
            span,
        },
    ];

    let program = Program { statements };
    let mut compiler = Compiler::new();
    let _ = compiler.compile(&program).expect("compile");
}
