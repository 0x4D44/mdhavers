#![cfg(coverage)]

use mdhavers::ast::{LogLevel, LogicalOp, UnaryOp};
use mdhavers::compiler::Compiler;
use mdhavers::parse;

#[test]
fn public_wrappers_formatter_and_wasm_instantiations_are_covered_in_dependency_instance() {
    let formatter_src = r#"
dae add(a, b = 2) {
    gie a + b
}

ken xs = [1, 2, 3, 4]
ken [first, _, ...rest] = xs
ken slice = xs[1:3:2]
ken d = {"a": 1, "b": 2}
ken f = add
blether f(1, 2)
"#;

    let formatted = mdhavers::format_source(formatter_src).expect("format_source");
    assert!(formatted.contains("dae add"));
    assert!(formatted.contains("ken [first, _, ...rest]"));

    let wasm_src = r#"
dae add(a, b) {
    gie a + b
}

dae run(dummy) {
    ken f = add
    blether f(1, 2)
    blether "a b"
    blether "he\\\"llo\\n"
}

run(0)
"#;

    let wat = mdhavers::compile_to_wat(wasm_src).expect("compile_to_wat");
    assert!(wat.contains("(module"));

    let audio_wat = mdhavers::compile_to_wat("soond_stairt()").expect("audio wat");
    assert!(audio_wat.contains("(import \"env\" \"soond_stairt\""));

    let err = mdhavers::compile_to_wat("fetch \"math\" tae m").unwrap_err();
    assert!(err.to_string().contains("Only the tri module"));

    let wasm_param_call = r#"
dae apply(f) {
    f(1)
}

dae id(x) {
    gie x
}

apply(id)
"#;
    let wat_param = mdhavers::compile_to_wat(wasm_param_call).expect("param call wat");
    assert!(wat_param.contains("(module"));
    assert!(wat_param.contains("(call $mdh_method_call1)"));

    {
        use mdhavers::ast::{Expr, Literal, Param, Program, Span, Stmt};
        let span = Span::new(1, 1);
        let apply_fn = Stmt::Function {
            name: "apply".to_string(),
            params: vec![Param {
                name: "f".to_string(),
                default: None,
            }],
            body: vec![Stmt::Expression {
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
            }],
            span,
        };
        let program = Program::new(vec![apply_fn]);
        let mut wasm_compiler = mdhavers::wasm_compiler::WasmCompiler::new();
        let wat_manual = wasm_compiler.compile(&program).expect("manual wasm compile");
        assert!(wat_manual.contains("(call $mdh_method_call1)"));
    }

    // Cover WasmCompiler::default() in the dependency crate instance.
    let _ = mdhavers::wasm_compiler::WasmCompiler::default();

    // Cover compile_to_llvm_ir wrapper in the dependency crate instance.
    #[cfg(feature = "llvm")]
    {
        let ir = mdhavers::compile_to_llvm_ir("ken x = 1").expect("compile_to_llvm_ir");
        assert!(!ir.is_empty());
    }
}

#[test]
fn compiler_ast_and_llvm_type_instantiations_are_covered_in_dependency_instance() {
    let js_src = r#"
fetch "tri" tae t
fetch "foo/bar.braw"
"#;

    let program = parse(js_src).expect("parse");
    let mut compiler = Compiler::default();
    let js = compiler.compile(&program).expect("compile");
    assert!(js.contains("__havers_tri"));
    assert!(js.contains("require('foo/bar.braw')"));

    // Cover ast Display impl instantiations in the dependency crate instance.
    let _ = format!("{}", LogLevel::Blether);
    let _ = format!("{}", UnaryOp::Not);
    let _ = format!("{}", LogicalOp::And);

    #[cfg(feature = "llvm")]
    {
        use std::hint::black_box;
        use inkwell::context::Context;
        use mdhavers::llvm::codegen::CodeGen;
        use mdhavers::llvm::{InferredType, MdhTypes, ValueTag};
        use mdhavers::llvm::runtime::RuntimeFunctions;

        let context = Context::create();
        let types = MdhTypes::new(&context);
        black_box(types.value_basic_type());

        // Cover CodeGen's coverage-only helpers in the dependency-crate instance to keep
        // instantiation coverage at 100% under cargo-llvm-cov.
        let mut codegen = CodeGen::new(&context, "coverage_codegen_dep_instance");
        codegen.coverage_compile_condition_direct_error_branches();

        let module = context.create_module("mdhavers_runtime_declare");
        let runtime = RuntimeFunctions::declare(&module, &types);
        black_box(runtime.make_nil);

        let inferred = black_box(InferredType::Int);
        assert!(black_box(inferred.is_known()));

        let inferred = black_box(InferredType::Nil);
        assert_eq!(black_box(inferred.tag()), Some(ValueTag::Nil));
    }
}
