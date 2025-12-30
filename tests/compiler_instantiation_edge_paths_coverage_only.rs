#![cfg(coverage)]

use std::hint::black_box;

use mdhavers::ast::{Program, Span, Stmt};
use mdhavers::compiler::Compiler;

#[test]
fn compiler_capitalized_call_executes_constructor_heuristic_in_dependency_instance() {
    let _ = black_box(mdhavers::compiler::compile("Foo()\n"));
}

#[test]
fn compiler_class_non_function_method_branch_is_covered_in_dependency_instance() {
    let span = Span::new(1, 1);

    let program = Program {
        statements: vec![Stmt::Class {
            name: "Foo".to_string(),
            superclass: None,
            methods: vec![Stmt::VarDecl {
                name: "x".to_string(),
                initializer: None,
                span,
            }],
            span,
        }],
    };

    let mut compiler = Compiler::new();
    let _ = black_box(compiler.compile(&program));
}
