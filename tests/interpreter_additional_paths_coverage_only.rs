#![cfg(coverage)]

use mdhavers::{parse, Interpreter};

fn interpret_ok(source: &str) -> Vec<String> {
    let program = parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let mut interp = Interpreter::new();
    interp
        .interpret(&program)
        .unwrap_or_else(|e| panic!("interpret failed for:\n{source}\nerr={e:?}"));
    interp.get_output().to_vec()
}

fn interpret_err(source: &str) -> String {
    let program = parse(source).unwrap_or_else(|e| panic!("parse failed for:\n{source}\nerr={e:?}"));
    let mut interp = Interpreter::new();
    match interp.interpret(&program) {
        Ok(value) => panic!(
            "expected interpreter error for:\n{source}\n\nbut got Ok: {value:?}"
        ),
        Err(err) => format!("{err:?}"),
    }
}

#[test]
fn interpreter_exercises_additional_error_and_edge_paths_for_coverage() {
    // Native builtin edge/error branches.
    for src in [
        // clype on less-common value kinds
        r#"
dae f() { gie 1 }
blether clype(f)
blether clype(len)
kin C { dae init() { } }
blether clype(C)
blether clype(C())
"#,
        // JSON number exponent (+/-) and string escaping
        r#"blether json_parse("1e+2")"#,
        r#"blether json_parse("-1.2E-3")"#,
        r#"blether json_stringify("a\\\"b")"#,
        r#"blether json_stringify("\\\\")"#,
        r#"blether json_stringify("a\\nb\\t")"#,
        r#"blether json_stringify("a\\rb")"#,
        // BlockExpr catching break/continue (shouldn't happen, but it is handled)
        r#"blether { brak }"#,
        r#"blether { haud }"#,
    ] {
        let out = interpret_ok(src);
        assert!(!out.is_empty(), "expected some output for:\n{src}");
    }

    // Interpreter error paths (evaluate/execute).
    for src in [
        // chynge/dicht out-of-bounds branches
        r#"blether chynge([1, 2, 3], 99, 0)"#,
        r#"blether dicht([1, 2, 3], 99)"#,
        // Undefined superclass resolution in class statement
        r#"kin Child fae Missing { dae init() { } }"#,
        // Fixed-arity function wrong-arity branch
        r#"
dae add(a, b) { gie a + b }
blether add(1)
"#,
        // Spread operator errors
        r#"ken xs = [...1]"#,
        // Range bounds must be integers
        r#"ken r = ("a")..3"#,
        // masel outside class/method
        r#"blether masel"#,
        // speir/input is disabled under coverage runs
        r#"speir "prompt> ""#,
    ] {
        let err = interpret_err(src);
        assert!(!err.is_empty(), "expected error string for:\n{src}");
    }
}
