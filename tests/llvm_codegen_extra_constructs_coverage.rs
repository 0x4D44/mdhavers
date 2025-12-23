#![cfg(all(feature = "llvm", coverage))]

use mdhavers::{ast::*, llvm::LLVMCompiler, parse};

fn compile_to_ir_ok(source: &str) {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let ir = LLVMCompiler::new()
        .compile_to_ir(&program)
        .unwrap_or_else(|e| panic!("compile failed for:\n{source}\nerr={e:?}"));
    assert!(!ir.is_empty());
}

fn compile_to_ir_err(source: &str) {
    let program =
        parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error");
    let s = format!("{err:?}");
    assert!(!s.is_empty());
}

#[test]
fn llvm_codegen_exercises_more_constructs_and_error_paths_for_coverage() {
    let ok_cases: &[&str] = &[
        // Built-in math constants.
        r#"
blether PI
blether E
blether TAU
"#,
        // Var declaration without initializer (defaults to nil).
        r#"
ken u
blether u
"#,
        // Empty f-string should compile (exercises empty-parts fast path).
        "blether f\"\"",
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
ken xs = [1, 2]
blether add3(0, ...xs)
"#,
        // Pipe lambda parameter collision should restore old binding.
        r#"
ken x = 99
blether 5 |> |x| x + 1
blether x
"#,
        // Pipe Call-case should remove temp var when not previously bound.
        r#"blether 5 |> tae_string()"#,
        // Pipe temp collision: restore old var (Variable-case + Call-case) + general pipe case.
        r#"
ken __pipe_tmp_0 = 99
ken __pipe_tmp_1 = 100
dae inc(x) { gie x + 1 }
blether 5 |> inc
blether 5 |> tae_string()
ken d = {"f": |x| x + 1}
blether 5 |> d["f"]
"#,
        // Pipe unary lambda - should compile and inline.
        r#"blether 5 |> |x| x + 1"#,
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
        // Duplicate class declaration hits preregister short-circuit.
        r#"
kin D {
    dae init() { }
}
kin D {
    dae init() { }
}
ken d = D()
blether "ok"
"#,
        // Duplicate method declaration hits per-method "already declared" continue.
        r#"
kin M {
    dae m() { gie 1 }
    dae m() { gie 2 }
}
ken m = M()
blether m.m()
"#,
        // Boxed capture used in direct condition compilation paths (int shadow absent -> extract).
        r#"
dae outer() {
    ken x = 0
    dae inc() { x = x + 1 }
    gin x == 0 { blether 1 }
    inc()
    gin x == 1 { blether 2 }
}
outer()
"#,
        // List index in condition, exercising list_ptr shadow and non-variable index objects.
        r#"
dae f() {
    ken xs = [1, 2, 3]
    gin xs[0] == 1 { blether 1 }
    gin [1, 2, 3][0] == 1 { blether 1 }
}
f()
"#,
        // Condition-direct index path for list variables (and non-variable list objects).
        r#"
dae f() {
    ken xs = [1, 0]
    gin xs[0] { blether 1 }
    gin xs[-1] { blether 1 }
    gin [1, 0][0] { blether 1 }
}
f()
"#,
        // Condition-direct int comparisons where compile_int_expr() falls back from unary ints.
        r#"
dae f() {
    gin -1 == 0 { blether 1 }
    gin -1 != 0 { blether 1 }
    gin -1 < 0 { blether 1 }
    gin -1 <= 0 { blether 1 }
    gin -1 > -2 { blether 1 }
    gin 1 + 2 { blether 1 }
}
f()
"#,
        // List var shadowing a predefined global should take the list-ptr-shadow alloca fallback.
        r#"
dae f() {
    ken __current_suite = [1, 2]
    gin __current_suite[0] { blether 1 }
}
f()
"#,
        // Int var shadowing a predefined global inside a loop should allocate boxed storage in-loop.
        r#"
dae f() {
    whiles aye {
        ken _tick_counter = 1
        brak
    }
}
f()
"#,
        // Int var with unary initializer should bypass compile_int_expr fast init path.
        r#"
dae f() {
    ken n = -1
    blether n
}
f()
"#,
        // Return without value should compile to nil.
        r#"
dae f() {
    gie
}
f()
"#,
        // In-loop int shadow -> boxed MdhValue fast path for variable reads.
        r#"
dae g(x) { gie x }
dae f() {
    ken i = 0
    whiles i < 3 {
        g(i)
        i = i + 1
    }
}
f()
"#,
        // Assignment to an import alias should drop alias-tracking metadata.
        r#"
fetch "bytes" tae b
b = 1
blether b
"#,
        // Import aliasing (including tri special-case) + nested imports for coverage.
        r#"
fetch "tui" tae t

ken bytes_mod = nil
fetch "bytes" tae bytes_mod

fetch "tri" tae __current_suite

dae inner() {
    fetch "tri" tae local_tri
    gie local_tri
}
inner()

blether t["blank"]()
blether bytes_mod["make_bytes"](1)
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
        // tri import requires an alias
        r#"fetch "tri""#,
        // referencing an undefined variable should be a compile error
        r#"blether __definitely_undefined_var__"#,
        // Capturing a name that's not in scope should error during capture/boxing.
        r#"
dae outer() {
    dae inner() { x = x + 1 }
    inner()
}
outer()
"#,
        // Capturing a missing binding when closing over should also error.
        r#"
dae outer() {
    dae inner() { gie x }
    gie inner
}
outer()
"#,
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

#[test]
fn llvm_codegen_rejects_spread_outside_list_literals_for_coverage() {
    let span = Span::new(1, 1);
    let program = Program::new(vec![Stmt::Print {
        value: Expr::Spread {
            expr: Box::new(Expr::Literal {
                value: Literal::Integer(1),
                span,
            }),
            span,
        },
        span,
    }]);
    let err = LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("expected compile error");
    let s = format!("{err:?}");
    assert!(!s.is_empty());
}
