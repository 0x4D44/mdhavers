#![cfg(feature = "llvm")]

use mdhavers::{parse, LLVMCompiler};

fn compile_to_ir(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {e:?}"))?;
    LLVMCompiler::new()
        .compile_to_ir(&program)
        .map_err(|e| format!("Compile error: {e:?}"))
}

#[test]
fn llvm_codegen_hits_additional_paths_smoke() {
    let cases: &[&str] = &[
        // for-loop over list (exercise compile_for_list)
        r#"
ken sum = 0
fer x in [1, 2, 3] {
    sum = sum + x
}
blether sum
"#,
        // read_file builtin (alias for slurp)
        r#"blether read_file("no_such_file.txt")"#,
        // speir (input) - compilation only (keyword-style input expression)
        r#"blether speir("prompt")"#,
        // Nested function capturing `masel` inside a method (exercise captured-variable plumbing)
        r#"
kin C {
    dae init(v) { masel.v = v }
    dae get() {
        dae inner() { gie masel.v }
        gie inner()
    }
}
ken c = C(42)
blether c.get()
"#,
        // Additional builtin coverage: bitwise + math helpers
        r#"
blether bit_an(6, 3)
blether bit_or(6, 3)
blether bit_xor(6, 3)
blether bit_nae(1)
blether bit_shove_left(1, 2)
blether bit_shove_right(4, 1)
blether clamp(5, 0, 10)
blether product([2, 3, 4])
blether average([1, 2, 3, 4])
blether factorial(5)
"#,
        // Slice + range + pipe inside lambdas to exercise free-var analysis branches
        r#"
ken xs = [1, 2, 3, 4, 5]
ken slicer = |i| { gie xs[i:4:1] }
ken piper = |v| { gie v |> tae_string }
ken ranger = |x| { gie 1..4 }
blether len(slicer(1))
blether piper(123)
blether len(ranger(naething))
"#,
        // Nested class inside a function (exercise compile_class non-preregistered path)
        r#"
dae make_inner() {
    kin Inner {
        dae init() { masel.x = 1 }
        dae get() { gie masel.x }
    }
    ken i = Inner()
    gie i.get()
}
blether make_inner()
"#,
        // list shadow + shove with constant bool (exercise inline_shove_bool_fast)
        r#"
dae f() {
    ken xs = [1]
    shove(xs, aye)
    shove(xs, nae)
    gie len(xs)
}
blether f()
"#,
        // shuffle builtin
        r#"blether len(shuffle([1, 2, 3, 4]))"#,
        // sleep builtin (alias for bide)
        r#"sleep(0)"#,
        // destructure with rest and trailing variable (exercise compile_list_index_dynamic)
        r#"
ken [a, ...rest, last] = [1, 2, 3, 4]
blether a
blether last
blether len(rest)
"#,
        // block-lambda with match + destructure (exercise collect_pattern_bindings/add_destruct_pattern_bindings)
        r#"
ken outer = 10
ken f = |v| {
    ken [first, ...rest] = [1, 2, 3]
    keek v {
        whan x -> { blether x }
    }
    gie outer + first + len(rest)
}
blether f(5)
"#,
        // callable field pattern (exercise call_callable_value)
        r#"
kin C {
    dae init() {
        masel.cb = |x| x + 1
    }
}
ken c = C()
blether c.cb(41)
"#,
    ];

    for src in cases {
        compile_to_ir(src)
            .unwrap_or_else(|e| panic!("expected IR compile success for:\n{src}\n{e}"));
    }
}

#[test]
fn llvm_codegen_exercises_some_arity_errors() {
    let bad_cases: &[&str] = &[
        r#"shuffle([1, 2], 3)"#,
        r#"starts_wi("hello")"#,
        r#"gaun([1, 2])"#,
        r#"shove([1], 2, 3)"#,
        r#"read_file()"#,
    ];

    for src in bad_cases {
        let err = compile_to_ir(src).expect_err("expected compile error");
        assert!(!err.is_empty(), "error string should not be empty");
    }
}
