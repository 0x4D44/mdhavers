#![cfg(feature = "llvm")]
//! MDH-BUG-FLU-00145: the native backend no longer has placeholder builtins (bodies that
//! returned nil, an empty string or their first argument). A call to a removed name is the
//! normal "undefined" compile error; the names that really work natively are kept.
//! The full list is in `wrk_docs/2026.10.25 - INV - Native-only builtins kept and removed.md`.

use std::process::{Command, Stdio};

use mdhavers::{parse, LLVMCompiler};
use tempfile::tempdir;

fn compile_error(source: &str) -> String {
    let program = parse(source).expect("parse");
    LLVMCompiler::new()
        .compile_to_ir(&program)
        .expect_err("a removed placeholder builtin must not compile")
        .to_string()
}

fn run_lines(source: &str) -> Vec<String> {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe_path = dir.path().join("kept_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe_path, 2)
        .expect("compile to native");
    let output = Command::new(&exe_path)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn removed_placeholder_builtins_are_undefined() {
    let calls = [
        ("curry", "curry(|a, b| a + b)"),
        ("partial", "partial(|a, b| a + b, 1)"),
        ("memoize", "memoize(|a| a)"),
        ("throttle", "throttle(|a| a, 10)"),
        ("debounce", "debounce(|a| a, 10)"),
        ("pipe", "pipe(|a| a, |b| b)"),
        ("matrix_new", "matrix_new(2, 2)"),
        ("matrix_mul", "matrix_mul([[1]], [[1]])"),
        ("event_on", "event_on(\"x\", |a| a)"),
        ("http_get", "http_get(\"http://example.invalid\")"),
        ("uuid", "uuid()"),
        ("md5", "md5(\"x\")"),
        ("sha256", "sha256(\"x\")"),
        ("base64_encode", "base64_encode(\"x\")"),
        ("url_encode", "url_encode(\"a b\")"),
        ("slugify", "slugify(\"Hello World\")"),
        ("camel_case", "camel_case(\"hello world\")"),
        ("insert_at", "insert_at([1, 2], 1, 9)"),
        ("deep_copy", "deep_copy([1, 2])"),
        ("dict_pop", "dict_pop({\"a\": 1}, \"a\")"),
        ("char_code_at", "char_code_at(\"a\")"),
        ("times", "times(3, |i| i)"),
        ("unless", "unless(aye, 1)"),
        ("log_info", "log_info(\"hello\")"),
        ("runtime_platform", "runtime_platform()"),
    ];
    for (name, call) in calls {
        let err = compile_error(&format!("ken r = {call}\nblether r\n"));
        assert!(
            err.contains(&format!("Undefined variable: {name}")),
            "{name}: unexpected error: {err}"
        );
    }
}

#[test]
fn user_functions_named_like_removed_builtins_still_work() {
    let out = run_lines(
        "dae curry(f) { gie f }\ndae slugify(s) { gie s + \"!\" }\nblether slugify(\"a\")\nblether curry(7)\n",
    );
    assert_eq!(out, vec!["a!", "7"]);
}

#[test]
fn kept_native_only_builtins_still_work() {
    let out = run_lines(
        r#"
blether bit_shl(1, 4)
blether popcount(7)
blether trim_left("  hi")
blether str(5) + "x"
blether is_list([1, 2])
blether sum([1, 2, 3])
blether repeat_string("ab", 3)
blether avg([2, 4])
blether identity(7)
blether clamp_value(15, 0, 10)
blether to_int("42") + 1
"#,
    );
    assert_eq!(
        out,
        vec!["16", "3", "hi", "5x", "aye", "6", "ababab", "3", "7", "10", "43"]
    );
}
