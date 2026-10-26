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
