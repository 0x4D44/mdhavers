//! Interpreter-level checks for the parser follow-ups
//! (MDH-BUG-FLU-00066, 00075, 00078, 00080): the programs parse *and* mean
//! what they say.

use mdhavers::{parse, Interpreter};

fn run(code: &str) -> String {
    let program = parse(code).unwrap_or_else(|e| panic!("should parse: {e}\n{code}"));
    let mut interp = Interpreter::new();
    interp
        .interpret(&program)
        .unwrap_or_else(|e| panic!("should run: {e}\n{code}"));
    interp.get_output().join("\n").trim().to_string()
}

// --- MDH-BUG-FLU-00066: a lambda can return a dict literal ---

#[test]
fn lambda_body_can_be_a_dict_literal() {
    let code = r#"
ken pair = |x| {"item": x, "key": x * 2}
ken d = pair(3)
blether d["item"]
blether d["key"]
"#;
    assert_eq!(run(code), "3\n6");
}

#[test]
fn lambda_dict_body_may_span_lines_and_be_indexed() {
    let code = r#"
ken f = |x| {
    "v": x,
    "w": x + 1
}
blether f(1)["w"]
ken g = |k| {"a": 1, "b": 2}[k]
blether g("b")
"#;
    assert_eq!(run(code), "2\n2");
}

#[test]
fn lambda_block_bodies_still_mean_blocks() {
    let code = r#"
ken total = 0
ken add = |n| { total = total + n }
add(4)
add(5)
blether total
ken inc = |x| {
    ken y = x + 1
    gie y
}
blether inc(41)
ken toom = |x| {}
blether toom(1)
"#;
    assert_eq!(run(code), "9\n42\nnaething");
}

// --- MDH-BUG-FLU-00075: newlines in call arguments, leading-dot chains ---

#[test]
fn call_arguments_may_span_lines() {
    let code = r#"
dae add3(a, b, c) {
    gie a + b + c
}
blether add3(
    1,
    2,
    3
)
blether add3(1,
             20,
             300)
blether add3(
    4, 5, 6)
dae nane() {
    gie 7
}
blether nane(
)
"#;
    assert_eq!(run(code), "6\n321\n15\n7");
}

#[test]
fn leading_dot_continues_a_method_chain() {
    let code = r#"
kin Coonter {
    dae init() {
        masel.n = 0
    }
    dae bump() {
        masel.n = masel.n + 1
        gie masel
    }
}
ken c = Coonter()
ken r = c
    .bump()

    .bump()
    .bump()
blether r.n
ken d = {"a": {"b": 5}}
blether d
    .a
    .b
"#;
    assert_eq!(run(code), "3\n5");
}

#[test]
fn statement_after_an_expression_is_still_a_new_statement() {
    // Only a `.` on the next line continues the expression; onything else
    // still starts a new statement.
    let code = "ken x = 1\nken y = x\n-1\nblether y\n";
    assert_eq!(run(code), "1");
}

#[test]
fn line_numbers_survive_multiline_calls() {
    let code =
        "dae f(a, b) {\n    gie a / b\n}\nken z = f(\n    1,\n    0\n)\nblether \"unreached\"\n";
    let program = parse(code).expect("should parse");
    let mut interp = Interpreter::new();
    let err = interp.interpret(&program).expect_err("divide by zero");
    assert!(
        err.to_string().contains("line 2"),
        "error should point at the division on line 2: {err}"
    );
}

// --- MDH-BUG-FLU-00078: inclusive range patterns ---

#[test]
fn inclusive_range_pattern_includes_its_end() {
    let code = r#"
dae size(n) {
    keek n {
        whan 1..=3 -> gie "wee"
        whan 4..6 -> gie "middlin"
        whan -2..=0 -> gie "nae much"
        whan _ -> gie "muckle"
    }
}
fer i in -3..8 {
    blether f"{i} {size(i)}"
}
"#;
    assert_eq!(
        run(code),
        "-3 muckle\n-2 nae much\n-1 nae much\n0 nae much\n1 wee\n2 wee\n3 wee\n4 middlin\n5 middlin\n6 muckle\n7 muckle"
    );
}

#[test]
fn inclusive_range_pattern_end_can_be_an_expression() {
    let code = r#"
ken top = 4
keek 5 {
    whan 0..=top + 1 -> blether "in"
    whan _ -> blether "oot"
}
keek 6 {
    whan 0..=top + 1 -> blether "in"
    whan _ -> blether "oot"
}
"#;
    assert_eq!(run(code), "in\noot");
}

#[test]
fn formatter_keeps_inclusive_range_patterns() {
    let code = "keek 3 {\n    whan 1..=3 -> blether \"aye\"\n    whan 4..6 -> blether \"nae\"\n}\n";
    // format_source_checked also re-parses the output an' checks the AST matches.
    let formatted = mdhavers::formatter::format_source_checked(code)
        .unwrap_or_else(|e| panic!("should format: {e:?}"));
    assert!(formatted.contains("1..=3"), "formatted: {formatted}");
    assert!(formatted.contains("4..6"), "formatted: {formatted}");
    assert!(!formatted.contains("4..=6"), "formatted: {formatted}");
}

#[test]
fn formatter_round_trips_a_lambda_returning_a_dict() {
    let code = "ken f = |x| {\"k\": x}\n";
    let formatted = mdhavers::formatter::format_source_checked(code)
        .unwrap_or_else(|e| panic!("should format: {e:?}"));
    assert!(formatted.contains("|x| {"), "formatted: {formatted}");
}

// --- MDH-BUG-FLU-00080: the shipped tri stub and stability benchmark parse ---

#[test]
fn tri_stub_and_stability_benchmark_parse() {
    let root = env!("CARGO_MANIFEST_DIR");
    for rel in ["stdlib/tri.braw", "benchmarks/stress/stability_test.braw"] {
        let path = format!("{root}/{rel}");
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        if let Err(e) = parse(&src) {
            panic!("{rel} should parse: {e}");
        }
    }
}
