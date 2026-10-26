#![cfg(feature = "llvm")]
//! Native backend: function values show their names as the interpreter does
//! (MDH-BUG-FLU-00146): `<dae name>` for a function, a bound method or a closure over a
//! named function, `<dae <lambda>>` for a lambda, `<kin Name>` for a class.

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
    let exe_path = dir.path().join("fn_names_exe");
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
dae greet(n) { gie "hi " + n }
dae make_adder(k) {
    dae add(x) { gie x + k }
    gie add
}
kin Dog {
    dae init(nm) { masel.nm = nm }
    dae bark() { gie masel.nm + " woof" }
}
ken lam = |x| x + 1
ken base = 10
ken clo = |x| x + base
ken adder = make_adder(5)
ken d = Dog("rex")
ken m = d.bark
blether greet
blether lam
blether clo
blether adder
blether m
blether Dog
blether [greet, lam, clo, adder, m, Dog]
blether tae_string(greet)
blether f"v={lam} {clo} {greet}"
blether tae_string({"f": greet, "l": lam})
blether clype(greet)
blether clype(clo)
blether clype(adder)
blether clype(Dog)
blether clype(m)
blether json_stringify({"f": greet, "c": clo})
blether adder(1)
blether m()
"#;

#[test]
fn function_values_print_like_the_interpreter() {
    let expected = run_interpreter(SOURCE);
    assert!(
        expected.starts_with("<dae greet>\n<dae <lambda>>\n"),
        "{expected}"
    );
    let (native, _) = run_native(SOURCE, false);
    assert_eq!(native, expected);
}

#[test]
fn function_names_are_constants_under_reference_counting() {
    let expected = run_interpreter(SOURCE);
    let (native, stderr) = run_native(SOURCE, true);
    assert_eq!(native, expected);
    assert!(stderr.contains("total=0"), "{stderr}");
}
