#![cfg(feature = "llvm")]
//! Native backend: a `ken` in a block binds a name seen only inside that block, as in the
//! interpreter (MDH-BUG-FLU-00163). It must not overwrite an outer binding of the same
//! name: a local, a parameter, or a global (from top-level code or from a function).

use std::process::{Command, Stdio};

use mdhavers::{parse, Interpreter, LLVMCompiler};
use tempfile::tempdir;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_native(source: &str, refcounting: bool) -> Run {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("block_scope_exe");
    LLVMCompiler::new()
        .with_refcounting(refcounting)
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .env("MDH_RC_REPORT", "1")
        .output()
        .expect("run executable");
    Run {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// The interpreter's output lines, on a thread with room for its deep frames.
fn run_interpreter(source: &str) -> Vec<String> {
    let source = source.to_string();
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || {
            let program = parse(&source).expect("parse");
            let mut interp = Interpreter::new();
            interp.interpret(&program).expect("interpreter run");
            interp.get_output().to_vec()
        })
        .expect("spawn interpreter thread")
        .join()
        .expect("interpreter thread")
}

/// Native output (plain and with reference counting) matches the interpreter's, and the
/// counted build frees everything it made.
fn assert_parity(source: &str) {
    let expected: Vec<String> = run_interpreter(source)
        .join("\n")
        .lines()
        .map(str::to_string)
        .collect();
    for refcounting in [false, true] {
        let run = run_native(source, refcounting);
        assert_eq!(
            run.code,
            Some(0),
            "native (rc={refcounting}) failed: stdout={:?} stderr={:?}",
            run.stdout,
            run.stderr
        );
        let got: Vec<String> = run.stdout.lines().map(str::to_string).collect();
        assert_eq!(got, expected, "native (rc={refcounting}) differs");
        if refcounting {
            assert!(
                run.stderr.contains(" total=0 "),
                "counted build leaked: {}",
                run.stderr
            );
        }
    }
}

#[test]
fn top_level_blocks_do_not_overwrite_outer_variables() {
    assert_parity(
        r#"
ken x = 10
ken s = "out"
ken l = [1, 2]
{
    ken x = 30
    blether x
}
blether x
gin aye {
    ken x = 40
    ken s = s + "!"
    blether s
}
blether x
blether s
gin nae {
    ken x = 1
} ither {
    ken x = 2
    ken l = l + [3]
    blether len(l)
}
blether x
blether len(l)
fer i in 0..3 {
    ken x = 50 + i
    ken s = "it" + tae_string(i)
    s = s + "?"
    blether s
    blether x
}
blether x
blether s
ken n = 0
whiles n < 2 {
    ken x = 60 + n
    blether x
    n = n + 1
}
blether x
{
    ken x = x + 1
    {
        ken x = x * 2
        blether x
    }
    blether x
}
blether x
{
    ken [x, y] = [7, 8]
    blether x + y
}
blether x
"#,
    );
}

#[test]
fn function_blocks_do_not_overwrite_locals_parameters_or_globals() {
    assert_parity(
        r#"
ken v = "global v"
ken g = 1
dae f(p) {
    ken y = 1
    {
        ken y = 2
        ken p = "inner p"
        blether y
        blether p
    }
    ken v = "local v"
    ken g = 2
    gie [y, p, v, g]
}
blether f("p")
blether v
blether g
dae loopy() {
    ken total = 0
    fer i in 0..4 {
        ken total = i
        total = total + 100
    }
    ken j = 0
    whiles j < 3 {
        ken total = "s"
        j = j + 1
    }
    gie total
}
blether loopy()
kin Thing {
    dae show() {
        ken v = "method v"
        gie v
    }
}
blether Thing().show()
ken lam = |a| {
    ken v = a + 1
    v
}
blether lam(4)
blether v
"#,
    );
}

#[test]
fn closures_capture_the_inner_binding() {
    assert_parity(
        r#"
ken x = 1
ken f = naething
{
    ken x = 99
    f = || x
    dae g() { gie x + 1 }
    blether g()
}
blether f()
blether x
dae outer() {
    ken c = 1
    {
        ken c = 2
        dae inner() { gie c * 100 }
        blether inner()
    }
    gie c
}
blether outer()
dae counter() {
    ken n = 0
    ken bump = naething
    {
        ken n = 10
        bump = || {
            n = n + 1
            n
        }
    }
    bump()
    bump()
    gie [n, bump()]
}
blether counter()
dae h(k) {
    ken acc = 0
    ken fs = []
    fer i in 0..k {
        ken acc = i * 10
        ken m = "m" + tae_string(i)
        shove(fs, || m + tae_string(acc))
        whiles acc > 5 {
            ken acc = 0
            acc = acc - 1
            brak
        }
    }
    fer q in fs { blether q() }
    gie acc
}
blether h(4)
"#,
    );
}

#[test]
fn a_block_binding_is_gone_after_its_block() {
    let program = parse("{\n    ken z = 7\n}\nblether z\n").expect("parse");
    let dir = tempdir().expect("temp dir");
    let err = LLVMCompiler::new()
        .compile_to_native(&program, &dir.path().join("gone_exe"), 2)
        .expect_err("z is not defined outside its block");
    assert!(
        format!("{err}").contains("Undefined variable: z"),
        "unexpected error: {err}"
    );
}
