use mdhavers::wasm_compiler;

#[test]
fn wasm_import_scanner_exercises_more_stmt_and_expr_kinds_for_coverage() {
    // The WASM compiler intentionally does not support all statement/expression kinds yet.
    // This test is written to *parse successfully* and then drive the import-requirements
    // scanner through additional AST variants before compilation fails on unsupported nodes.
    let src = r#"
fetch "tri"
fetch "tri.braw"

dae add(x, y = 2) { gie x + y }

thing Point { x, y }
kin Foo { dae bar() { gie 1 } }

ken xs = [1, 2, 3, 4]
ken ys = [...xs, 5]
ken [a, b, ...rest] = xs
ken add1 = |x| x + 1
ken p = a |> add1
ken d = {"a": 1, "b": 2}
ken grp = (a + b)
ken r = 1..10
ken neg = -a
ken t = gin aye than 1 ither 2
ken blk = {
    ken z = 1
    gie z
}
ken who = speir "who?"
ken msg = f"hi {a}"

gin aye { blether 1 } ither { blether 2 }
whiles nae { brak }
fer i in 0..3 { haud }

log_mutter "hi", a, b
mak_siccar a == 1, "a should be 1"

a = a + 1
add(3)

hae_a_bash {
    keek a {
        whan 1 -> { xs[0] = 9 }
        whan _ -> { hurl "boom" }
    }
} gin_it_gangs_wrang e {
    blether e
}

ken slice = xs[0:4:2]
blether add(3)
blether len(rest)
"#;

    mdhavers::parser::parse(src).expect("expected source to parse successfully");
    let result = wasm_compiler::compile_to_wat(src);
    assert!(result.is_err(), "expected unsupported nodes to fail WASM compilation");
}
