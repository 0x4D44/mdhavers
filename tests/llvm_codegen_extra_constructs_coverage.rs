#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{parse, llvm::LLVMCompiler};

fn compile_to_ir_ok(source: &str) {
    let program = parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let ir = LLVMCompiler::new()
        .compile_to_ir(&program)
        .unwrap_or_else(|e| panic!("compile failed for:\n{source}\nerr={e:?}"));
    assert!(!ir.is_empty());
}

fn compile_to_ir_err(source: &str) {
    let program = parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error");
    let s = format!("{err:?}");
    assert!(!s.is_empty());
}

#[test]
fn llvm_codegen_exercises_more_constructs_and_error_paths_for_coverage() {
    let ok_cases: &[&str] = &[
        // Boxed capture: nested function mutates captured local.
        r#"
dae outer() {
    ken x = 0
    dae inc() {
        x = x + 1
        gie x
    }
    gie inc()
}
blether outer()
"#,
        // Return a closure capturing mutable state.
        r#"
dae counter() {
    ken x = 0
    dae inc() {
        x = x + 1
        gie x
    }
    gie inc
}
ken c = counter()
blether c()
blether c()
"#,
        // Struct declaration + instantiation + dict-style access.
        r#"
thing Pair { a, b }
ken p = Pair(1, 2)
blether p["a"]
blether p["b"]
"#,
        // Try/catch + hurl.
        r#"
hae_a_bash {
    hurl "boom"
} gin_it_gangs_wrang e {
    blether e
}
"#,
        // Spread operator in call args.
        r#"
dae add3(a, b, c) { gie a + b + c }
ken xs = [1, 2, 3]
blether add3(...xs)
"#,
        // Pipe temp collision: restore old var (Variable-case + Call-case) + general pipe case.
        r#"
ken __pipe_tmp_0 = 99
dae inc(x) { gie x + 1 }
blether 5 |> inc
blether 5 |> tae_string()
ken d = {"f": |x| x + 1}
blether 5 |> d["f"]
"#,
        // List index assignment on non-variable object (exercise list_index_set_fast non-var path).
        r#"blether [0, 0][0] = 5"#,
        // Dict index assignment for Expr::Get and non-lvalue object.
        r#"
kin Holder {
    dae init() { masel.d = {"a": 1} }
}
ken h = Holder()
h.d["a"] = 2
blether h.d["a"]
blether {"a": 1}["a"] = 3
"#,
        // Empty match (no arms) should compile.
        r#"
keek 1 {
}
blether 1
"#,
        // Duplicate struct declaration hits "already compiled" short-circuit.
        r#"
thing S { a }
thing S { a }
ken s = S(1)
blether s["a"]
"#,
        // Default-arg filling with nil for missing non-default parameters (init + method).
        r#"
kin Foo {
    dae init(a, b, c = 2) {
        masel.c = c
    }
    dae m(a, b, c = 2) {
        gie c
    }
}
ken f = Foo(1)
blether f.m(1)
"#,
    ];

    let err_cases: &[&str] = &[
        // break/continue outside loop
        "brak",
        "haud",
        // masel outside class/method
        "blether masel",
        // import missing module
        r#"fetch "definitely_no_such_module""#,
        // pipe lambda must be unary
        r#"blether 5 |> |a, b| a + b"#,
        // builtin assert arity errors
        r#"assert()"#,
        r#"assert(1, 2, 3)"#,
        // assignment to undefined variable
        r#"x = 1"#,
    ];

    for src in ok_cases {
        compile_to_ir_ok(src);
    }
    for src in err_cases {
        compile_to_ir_err(src);
    }
}
