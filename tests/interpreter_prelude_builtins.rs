//! MDH-BUG-FLU-00192: the prelude no longer redefines is_even, is_odd and assert_equal, so
//! the builtins (which match native) win. The prelude is loaded, as the CLI does.
use mdhavers::{parse, Interpreter};

fn run(code: &str) -> Result<String, String> {
    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.load_prelude().unwrap();
    interp.interpret(&program).map_err(|e| e.to_string())?;
    Ok(interp.get_output().join("\n"))
}

#[test]
fn is_even_and_is_odd_handle_zero_and_negatives() {
    let out = run(
        "blether is_even(0)\nblether is_even(-2)\nblether is_even(-3)\n\
         blether is_odd(0)\nblether is_odd(-3)\nblether is_odd(-2)\nblether is_odd(7)",
    )
    .unwrap();
    assert_eq!(out, "aye\naye\nnae\nnae\naye\nnae\naye");
}

#[test]
fn is_even_and_is_odd_reject_floats_and_non_numbers() {
    for (code, name) in [
        ("is_even(2.0)", "is_even"),
        ("is_odd(3.5)", "is_odd"),
        ("is_even(\"a\")", "is_even"),
        ("is_odd(naething)", "is_odd"),
        ("is_even([2])", "is_even"),
    ] {
        let err = run(code).expect_err(code);
        assert!(
            err.contains(&format!("{name}() needs an integer")),
            "{code}: {err}"
        );
    }
}

#[test]
fn assert_equal_returns_aye_or_raises_the_builtin_message() {
    let out = run("blether assert_equal(1, 1)\nblether assert_equal([1, 2], [1, 2])").unwrap();
    assert_eq!(out, "aye\naye");
    let err = run("assert_equal(1, 2)").expect_err("1 != 2");
    assert!(
        err.contains("Assertion failed: expected 1 but got 2"),
        "{err}"
    );
    let err = run("assert_equal(\"a\", \"b\")").expect_err("a != b");
    assert!(
        err.contains("Assertion failed: expected a but got b"),
        "{err}"
    );
}
