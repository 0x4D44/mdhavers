#![cfg(feature = "llvm")]
//! Native backend: a builtin used as a value (MDH-BUG-FLU-00158). `ken f = len` makes a
//! function value that prints `<native dae len>`, can be called, and can be passed to
//! gaun, sieve and ilk, as in the interpreter. A user function of the same name wins.

use std::process::{Command, Stdio};

use mdhavers::{parse, Interpreter, LLVMCompiler};
use tempfile::tempdir;

fn run_interpreter(source: &str) -> String {
    let program = parse(source).expect("parse");
    let mut interp = Interpreter::new();
    interp.interpret(&program).expect("interpret");
    interp.get_output().join("\n")
}

fn run_native(source: &str, refcounting: bool) -> (String, String) {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("builtin_values_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .env("MDH_RC_REPORT", "1")
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    (
        String::from_utf8_lossy(&output.stdout)
            .trim_end_matches('\n')
            .to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

const SOURCE: &str = r#"
ken f = len
blether f
blether f([1, 2, 3])
blether f("hullo")
ken g = tae_string
blether g(42) + "!"
blether gaun([1, 22, 333], tae_string)
blether gaun(["a", "bcd", ""], len)
blether sieve(["a", "", "bc"], len)
ilk([1, 2], tae_string)
ken h = gaun
blether h
blether h([1, 2], tae_string)
blether [len, tae_string]
blether tae_string(len)
blether f"v={len}"
"#;

const SHADOWED: &str = r#"
dae len(x) { gie 99 }
ken f = len
blether f
blether f([1, 2, 3])
blether gaun([[1], [2, 3]], len)
ken tae_string = |x| "mine"
blether gaun([1, 2], tae_string)
"#;

#[test]
fn builtins_work_as_values_like_the_interpreter() {
    let expected = run_interpreter(SOURCE);
    assert!(
        expected.starts_with("<native dae len>\n3\n5\n"),
        "{expected}"
    );
    let (native, _) = run_native(SOURCE, false);
    assert_eq!(native, expected);
}

#[test]
fn a_user_binding_shadows_the_builtin() {
    let expected = run_interpreter(SHADOWED);
    assert!(expected.starts_with("<dae len>\n99\n"), "{expected}");
    let (native, _) = run_native(SHADOWED, false);
    assert_eq!(native, expected);
}

#[test]
fn builtin_values_are_constants_under_reference_counting() {
    let expected = run_interpreter(SOURCE);
    let (native, stderr) = run_native(SOURCE, true);
    assert_eq!(native, expected);
    assert!(stderr.contains("total=0"), "{stderr}");
}

// MDH-BUG-FLU-00162: one function value per builtin, and clype names it a native function.
const IDENTITY: &str = r#"
ken f = len
ken g = len
blether f == g
blether len == len
blether len == tae_string
blether f == tae_string
dae foo() { gie 1 }
blether foo == foo
ken h = foo
blether h == foo
blether foo == len
blether [len] == [len]
"#;

const CLYPE: &str = r#"
blether clype(len)
ken f = tae_string
blether clype(f)
dae foo() { gie 1 }
blether clype(foo)
blether tae_string(len)
blether len
blether f"{len}"
"#;

#[test]
fn two_uses_of_a_builtin_are_the_same_value() {
    let expected = run_interpreter(IDENTITY);
    assert!(expected.starts_with("aye\naye\nnae\nnae\n"), "{expected}");
    // MDH-BUG-FLU-00190: the last line is `[len] == [len]`, aye in both.
    assert!(expected.ends_with("nae\naye"), "{expected}");
    for rc in [false, true] {
        let (native, stderr) = run_native(IDENTITY, rc);
        assert!(native.ends_with("nae\naye"), "refcounting={rc}: {native}");
        assert_eq!(native, expected, "refcounting={rc}");
        if rc {
            assert!(stderr.contains("total=0"), "{stderr}");
        }
    }
}

// MDH-BUG-FLU-00190: function values inside lists and dicts compare by identity, as in the
// interpreter.
const CONTAINER_EQUALITY: &str = r#"
blether [len] == [len]
blether [[len]] == [[len]]
blether {"k": [tae_string, len]} == {"k": [tae_string, len]}
dae mk() { gie [len, tae_string] }
blether mk() == mk()
ken a = [len]
ken b = [len]
blether a == b
blether a != b
blether [len] == [tae_string]
blether [len] == [len, len]
dae foo() { gie 1 }
blether [foo, len] == [foo, len]
ken c = |x| x + 1
blether [c] == [c]
blether [|x| x] == [|x| x]
"#;

#[test]
fn lists_and_dicts_of_function_values_compare_like_the_interpreter() {
    let expected = run_interpreter(CONTAINER_EQUALITY);
    assert!(
        expected.starts_with("aye\naye\naye\naye\naye\nnae\nnae\nnae\naye\naye\n"),
        "{expected}"
    );
    for rc in [false, true] {
        let (native, _) = run_native(CONTAINER_EQUALITY, rc);
        assert_eq!(native, expected, "refcounting={rc}");
    }
}

#[test]
fn clype_of_a_builtin_value_matches_the_interpreter() {
    let expected = run_interpreter(CLYPE);
    assert!(
        expected.starts_with("[native function] native function 'len'\n"),
        "{expected}"
    );
    assert!(expected.contains("<native dae len>"), "{expected}");
    let (native, _) = run_native(CLYPE, false);
    assert_eq!(native, expected);
}
