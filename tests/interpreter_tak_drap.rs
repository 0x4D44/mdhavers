//! MDH-BUG-FLU-00171: tak/drap clamp negative counts and reject non-lists.
//! The prelude is loaded, as the CLI does, so a prelude redefinition cannae
//! shadow the builtins again.
use mdhavers::{parse, Interpreter};

fn run(code: &str) -> Result<String, String> {
    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.load_prelude().unwrap();
    interp.interpret(&program).map_err(|e| e.to_string())?;
    Ok(interp.get_output().join("\n"))
}

#[test]
fn drap_negative_count_returns_input_unchanged() {
    let out = run("blether drap([3, 1, 2], -1)\nblether drap(\"Hello\", -1)").unwrap();
    assert_eq!(out, "[3, 1, 2]\n[H, e, l, l, o]");
}

#[test]
fn tak_negative_count_is_empty_and_strings_work() {
    let out =
        run("blether tak([3, 1, 2], -1)\nblether tak(\"Hello\", 2)\nblether drap(\"Hello\", 2)")
            .unwrap();
    assert_eq!(out, "[]\n[H, e]\n[l, l, o]");
}

#[test]
fn tak_and_drap_reject_non_list_and_non_integer() {
    for code in [
        "tak(7, 0)",
        "drap(7, 0)",
        "tak([1], \"a\")",
        "drap([1], \"a\")",
    ] {
        let err = run(code).expect_err(code);
        assert!(err.contains("needs a list and an integer"), "{code}: {err}");
    }
}
