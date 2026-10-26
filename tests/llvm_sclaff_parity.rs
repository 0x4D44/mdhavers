#![cfg(feature = "llvm")]
//! Native vs interpreter parity for `sclaff` (full flatten, MDH-BUG-FLU-00176) and the
//! error texts of `stoater`, `dicht` and `is_a`.

use std::process::{Command, Stdio};

use mdhavers::{parse, Interpreter, LLVMCompiler};
use tempfile::tempdir;

fn run_interpreter(source: &str) -> String {
    let program = parse(source).expect("parse");
    let mut interp = Interpreter::new();
    interp.interpret(&program).expect("interpret");
    interp.get_output().join("\n")
}

fn run_native(source: &str) -> String {
    let program = parse(source).expect("parse");
    let dir = tempdir().expect("temp dir");
    let exe = dir.path().join("parity_exe");
    LLVMCompiler::new()
        .compile_to_native(&program, &exe, 1)
        .expect("compile to native");
    let output = Command::new(&exe)
        .stdin(Stdio::null())
        .output()
        .expect("run executable");
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_string()
}

/// Both backends print the same thing; returns it.
fn same_on_both(source: &str) -> String {
    let interpreted = run_interpreter(source);
    let native = run_native(source);
    assert_eq!(native, interpreted, "native differs for:\n{source}");
    native
}

fn caught(call: &str) -> String {
    format!("hae_a_bash {{\n    blether {call}\n}} gin_it_gangs_wrang e {{\n    blether e\n}}\n")
}

#[test]
fn sclaff_flattens_every_level() {
    let out = same_on_both("blether sclaff([1, [2, [3, [4]]], 5])\n");
    assert_eq!(out, "[1, 2, 3, 4, 5]");
}

#[test]
fn sclaff_handles_empty_and_flat_and_string_lists() {
    let out = same_on_both(
        "blether sclaff([])\nblether sclaff([[], [[]]])\nblether sclaff([\"a\", [\"b\", [\"c\"]]])\nblether sclaff([1, 2])\n",
    );
    assert_eq!(out, "[]\n[]\n[a, b, c]\n[1, 2]");
}

#[test]
fn sclaff_flattens_more_levels_than_a_stack_would_like() {
    let out = same_on_both(
        "ken v = [7]\nken i = 0\nwhiles i < 300 {\n    v = [v, i]\n    i = i + 1\n}\nken flat = sclaff(v)\nblether len(flat)\nblether flat[0]\nblether flat[299]\n",
    );
    assert_eq!(out, "301\n7\n298");
}

#[test]
fn flatten_still_goes_one_level() {
    let out = same_on_both("blether flatten([1, [2, [3]], 4])\n");
    assert_eq!(out, "[1, 2, [3], 4]");
}

#[test]
fn sclaff_errors_match() {
    same_on_both(&caught("sclaff(5)"));
    let cyclic = "ken a = [1]\nshove(a, a)\n".to_string() + &caught("sclaff(a)");
    same_on_both(&cyclic);
}

#[test]
fn stoater_on_an_empty_list_has_the_interpreters_text() {
    let out = same_on_both(&caught("stoater([])"));
    assert!(
        out.ends_with("Cannae find a stoater in an empty list!"),
        "{out}"
    );
}

#[test]
fn dicht_out_of_range_has_the_interpreters_text() {
    same_on_both(&caught("dicht([1, 2, 3], 5)"));
    let out = same_on_both(&caught("dicht([1, 2, 3], -5)"));
    assert!(
        out.contains("Index 18446744073709551614 oot o' bounds fer list o' length 3"),
        "{out}"
    );
}

#[test]
fn is_a_with_a_non_string_type_has_the_interpreters_text() {
    let out = same_on_both(&caught("is_a(1, 2)"));
    assert!(out.ends_with("is_a() needs a type name string"), "{out}");
}

/// Wrong-type arguments to `stoater`, `dicht` and `chynge`, and an out-of-range `chynge`,
/// raise the interpreter's text (MDH-BUG-FLU-00185).
#[test]
fn wrong_type_arguments_have_the_interpreters_text() {
    let cases = [
        ("stoater(5)", "stoater() needs a list"),
        ("dicht(5, 0)", "dicht() needs a list"),
        ("dicht([1, 2], \"a\")", "dicht() needs an integer index"),
        ("chynge(5, 0, 1)", "chynge() needs a list"),
        (
            "chynge([1, 2], \"a\", 1)",
            "chynge() needs an integer index",
        ),
    ];
    for (call, text) in cases {
        let out = same_on_both(&caught(call));
        assert!(
            out.ends_with(&format!("Jings! Something went awfy wrang: {text}")),
            "{call}: {out}"
        );
    }
}

#[test]
fn chynge_out_of_range_has_the_interpreters_text() {
    let out = same_on_both(&caught("chynge([1, 2, 3], 5, 9)"));
    assert!(
        out.ends_with(
            "Jings! Something went awfy wrang: Index 5 oot o' bounds fer list o' length 3"
        ),
        "{out}"
    );
    let out = same_on_both(&caught("chynge([1, 2, 3], -9, 9)"));
    assert!(
        out.contains("Index 18446744073709551610 oot o' bounds fer list o' length 3"),
        "{out}"
    );
    assert_eq!(
        same_on_both("blether chynge([1, 2, 3], 3, 9)\nblether chynge([1, 2, 3], -1, 9)\n"),
        "[1, 2, 3, 9]\n[1, 2, 9, 3]"
    );
}
