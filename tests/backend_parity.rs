//! Backend parity smoke tests.
//!
//! Goal: catch basic semantic drift between the interpreter and other backends.

use mdhavers::Interpreter;
use std::process::Command;

fn run_interpreter(source: &str) -> Result<String, String> {
    let program = mdhavers::parse(source).map_err(|e| format!("{e}"))?;
    let mut interp = Interpreter::new();
    interp.interpret(&program).map_err(|e| format!("{e}"))?;
    Ok(interp.get_output().join("\n"))
}

fn run_js(source: &str) -> Result<String, String> {
    let js = mdhavers::compile_to_js(source).map_err(|e| format!("{e}"))?;
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let js_path = dir.path().join("parity.js");
    std::fs::write(&js_path, js).map_err(|e| e.to_string())?;

    let output = Command::new("node")
        .arg(&js_path)
        .output()
        .map_err(|e| format!("Failed to run node: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "node exited with {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string())
}

#[test]
fn parity_interpreter_vs_js_smoke() {
    if Command::new("node").arg("--version").output().is_err() {
        eprintln!("Skipping JS parity tests: node not found");
        return;
    }

    let cases: &[(&str, &str)] = &[
        (r#"blether "hullo""#, "hullo"),
        ("blether 1 + 2 * 3", "7"),
        (
            r#"
ken x = 10
gin x > 5 { blether "big" } ither { blether "wee" }
"#,
            "big",
        ),
        (
            r#"
dae add(a, b = 1) { gie a + b }
blether add(41)
"#,
            "42",
        ),
        (
            r#"
ken i = 0
whiles i < 3 {
  blether i
  i = i + 1
}
"#,
            "0\n1\n2",
        ),
        (
            r#"
ken s = "abc"
blether s[0]
"#,
            "a",
        ),
    ];

    for (source, expected) in cases {
        let interp_out = run_interpreter(source).unwrap_or_else(|e| panic!("interpreter: {e}"));
        assert_eq!(interp_out.trim(), *expected);

        let js_out = run_js(source).unwrap_or_else(|e| panic!("js: {e}"));
        assert_eq!(js_out.trim(), *expected);
    }
}
