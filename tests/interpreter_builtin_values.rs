//! MDH-BUG-FLU-00013: the higher-order builtins (gaun, sieve, tumble, ...)
//! must be real function values, not callable magic strings.

use mdhavers::{parse, HaversError, Interpreter};

const HOFS: [&str; 9] = [
    "gaun", "sieve", "tumble", "ilk", "hunt", "ony", "aw", "grup_up", "pairt_by",
];

fn run_ok(code: &str) -> String {
    let program = parse(code).expect("parse");
    let mut interp = Interpreter::new();
    if let Err(e) = interp.interpret(&program) {
        panic!("program failed: {e}");
    }
    interp.get_output().join("\n")
}

fn run_err(code: &str) -> HaversError {
    let program = parse(code).expect("parse");
    let mut interp = Interpreter::new();
    match interp.interpret(&program) {
        Ok(_) => panic!("expected an error, got output: {:?}", interp.get_output()),
        Err(e) => e,
    }
}

#[test]
fn higher_order_builtins_have_the_same_kind_as_other_builtins() {
    let len_kind = run_ok("blether whit_kind(len)");
    for name in HOFS {
        assert_eq!(
            run_ok(&format!("blether whit_kind({name})")),
            len_kind,
            "whit_kind({name})"
        );
    }
}

#[test]
fn higher_order_builtins_print_like_other_builtins() {
    assert_eq!(run_ok("blether len"), "<native dae len>");
    for name in HOFS {
        assert_eq!(
            run_ok(&format!("blether {name}")),
            format!("<native dae {name}>")
        );
    }
}

#[test]
fn forged_builtin_string_is_not_callable() {
    for name in HOFS {
        let err = run_err(&format!(
            r#"ken f = "__builtin_{name}__"
blether f([1, 2], |x| x)"#
        ));
        assert!(
            matches!(err, HaversError::NotCallable { .. }),
            "{name}: expected NotCallable, got {err:?}"
        );
    }
    // A forged string is still just a string.
    assert_eq!(
        run_ok(r#"blether whit_kind("__builtin_gaun__")"#),
        run_ok(r#"blether whit_kind("hello")"#)
    );
}

#[test]
fn higher_order_builtins_still_work_as_values() {
    // Called directly with lambdas.
    assert_eq!(run_ok("blether gaun([1, 2, 3], |x| x * 2)"), "[2, 4, 6]");
    assert_eq!(
        run_ok("blether sieve([1, 2, 3, 4], |x| x % 2 == 0)"),
        "[2, 4]"
    );
    assert_eq!(run_ok("blether tumble([1, 2, 3], 0, |a, b| a + b)"), "6");
    // A native builtin passed as the function argument.
    assert_eq!(
        run_ok(r#"blether gaun(["a", "bb", "ccc"], len)"#),
        "[1, 2, 3]"
    );
    // Stored in a variable and passed to a user function.
    assert_eq!(
        run_ok(
            r#"
dae apply(f, xs, g) {
    gie f(xs, g)
}
ken m = gaun
blether apply(m, [1, 2], |x| x + 1)
blether apply(sieve, [1, 2, 3], |x| x > 1)
"#
        ),
        "[2, 3]\n[2, 3]"
    );
    // One higher-order builtin passed to another.
    assert_eq!(
        run_ok(
            r#"
ken fs = [gaun, sieve]
ken outs = gaun(fs, |f| f([1, 2, 3], |x| x > 1))
blether outs
"#
        ),
        "[[nae, aye, aye], [2, 3]]"
    );
    // Equality and arity errors behave like other function values.
    assert_eq!(
        run_ok("blether gaun == gaun\nblether gaun == sieve"),
        "aye\nnae"
    );
    assert!(matches!(
        run_err("gaun([1])"),
        HaversError::WrongArity { .. }
    ));
}
