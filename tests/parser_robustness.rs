//! Interpreter-level checks for the parser robustness fixes
//! (MDH-BUG-FLU-00014..00024): the programs parse *and* mean what they say.

use mdhavers::{parse, HaversError, Interpreter};

fn run(code: &str) -> String {
    let program = parse(code).unwrap_or_else(|e| panic!("should parse: {e}\n{code}"));
    let mut interp = Interpreter::new();
    interp
        .interpret(&program)
        .unwrap_or_else(|e| panic!("should run: {e}\n{code}"));
    interp.get_output().join("\n").trim().to_string()
}

fn assert_return_outside_function(code: &str, want_line: usize) {
    match parse(code) {
        Err(HaversError::ReturnOutsideFunction { line }) => assert_eq!(line, want_line, "{code}"),
        other => panic!("expected ReturnOutsideFunction for {code:?}, got {other:?}"),
    }
    // Every backend parses first, so none o' them gets as far as codegen.
    assert!(
        matches!(
            mdhavers::compile_to_js(code),
            Err(HaversError::ReturnOutsideFunction { .. })
        ),
        "JS backend accepted {code:?}"
    );
    assert!(
        matches!(
            mdhavers::wasm_compiler::compile_to_wat(code),
            Err(HaversError::ReturnOutsideFunction { .. })
        ),
        "WASM backend accepted {code:?}"
    );
}

#[test]
fn top_level_gie_is_return_outside_function_decision_f4() {
    assert_return_outside_function("blether 1\ngie 5\nblether 2", 2);
    assert_return_outside_function("gie", 1);
    assert_return_outside_function("gin aye {\n    gie 1\n}", 2);
    assert_return_outside_function("fer i in 0..3 {\n    gie i\n}", 2);
    assert_return_outside_function("keek 1 {\n    whan 1 -> gie 2\n}", 2);
    // A gie efter a function ends is back at the top level.
    assert_return_outside_function("dae f() { gie 1 }\ngie 2", 2);
    // A block expression is no' a function: its `gie` returns frae the
    // enclosing one, so it needs one (DEC row 64, MDH-BUG-FLU-00094).
    assert_return_outside_function("ken b = { gie 4 }", 1);
    assert_return_outside_function("blether { gie 4 }", 1);
    assert_return_outside_function("ken b = {\n    ken a = { gie 1 }\n    a\n}", 2);
    assert_return_outside_function("dae f() { gie 1 }\nken b = { gie 2 }", 2);
    let msg = parse("gie 5").unwrap_err().to_string();
    assert!(msg.contains("ootside a function"), "{msg}");
}

#[test]
fn gie_inside_functions_lambdas_and_blocks_still_parses_decision_f4() {
    assert_eq!(run("dae f() { gie 1 }\nblether f()"), "1");
    assert_eq!(
        run("kin K {\n    dae m() { gie 2 }\n}\nblether K().m()"),
        "2"
    );
    assert_eq!(run("ken g = |x| { gie x + 1 }\nblether g(2)"), "3");
    assert_eq!(
        run("dae f() {\n    ken b = { gie 4 }\n    gie 9\n}\nblether f()"),
        "4"
    );
    assert_eq!(
        run("ken g = |x| {\n    ken b = { gie x }\n    gie 0\n}\nblether g(6)"),
        "6"
    );
    assert_eq!(
        run("dae h() {\n    ken k = |y| { gie y }\n    gie k(5)\n}\nblether h()"),
        "5"
    );
}

#[test]
fn fer_over_negative_range_counts_from_the_negative_start() {
    // MDH-BUG-FLU-00014 (used tae be -(1..2): "Cannae negate a list")
    assert_eq!(run("fer i in -1..2 {\n    blether i\n}"), "-1\n0\n1");
}

#[test]
fn game_of_life_neighbour_loops_visit_all_nine_cells() {
    // The shape o' examples/showcases/game_of_life/game_of_life.braw:89
    let code = r#"
ken count = 0
fer dy in -1..2 {
    fer dx in -1..2 {
        gin dx != 0 or dy != 0 {
            count = count + 1
        }
    }
}
blether count
"#;
    assert_eq!(run(code), "8");
}

#[test]
fn addition_binds_tighter_than_range() {
    // MDH-BUG-FLU-00015: x + 1..5 is (x + 1)..5
    let code = "ken x = 2\nken r = x + 1..5\nblether len(r)\nblether r[0]\nblether len(0..x * 2)";
    assert_eq!(run(code), "2\n3\n4");
}

#[test]
fn match_range_pattern_with_negative_start() {
    let code = r#"
dae sign(n) {
    keek n {
        whan -100..0 -> gie "negative"
        whan 0 -> gie "zero"
        whan _ -> gie "positive"
    }
}
blether sign(-3)
blether sign(0)
blether sign(7)
"#;
    assert_eq!(run(code), "negative\nzero\npositive");
}

#[test]
fn nested_brace_expressions_still_evaluate() {
    // MDH-BUG-FLU-00016: block expressions an' dicts nested inside each ither
    // still pick the same reading as afore the lookahead change.
    let code = r#"
ken d = {"k": {"j": 2}}
blether d["k"]["j"]
dae pick() {
    ken b = { gie {"j": 3} }
}
blether pick()["j"]
ken e = {
    "a": [1,
          2],
    "b": { 4 }
}
blether e["a"][1] + e["b"]
"#;
    assert_eq!(run(code), "2\n3\n6");
}

#[test]
fn deeply_nested_source_is_a_parse_error() {
    // MDH-BUG-FLU-00017: 200000 parens used tae overflow the stack
    let n = 200_000;
    let code = format!("ken a = {}1{}", "(".repeat(n), ")".repeat(n));
    match parse(&code) {
        Err(HaversError::ParseError { message, line }) => {
            assert!(message.contains("nested ower deep"), "{message}");
            assert_eq!(line, 1);
        }
        other => panic!("expected a nesting ParseError, got {other:?}"),
    }
}

#[test]
fn bare_gie_returns_naething() {
    // MDH-BUG-FLU-00018
    let code =
        "dae f() { gie }\ndae g() { gie; }\nblether f() == naething\nblether g() == naething";
    assert_eq!(run(code), "aye\naye");
}

#[test]
fn mak_siccar_message_is_attached_or_rejected() {
    // MDH-BUG-FLU-00019: wi' a comma the message is reported...
    let program = parse("mak_siccar 1 == 2, \"sums are aff\"").unwrap();
    let err = Interpreter::new().interpret(&program).unwrap_err();
    assert!(err.to_string().contains("sums are aff"), "{err}");
    // ...an' wi'oot ane it's a parse error, no' a silently drapped message.
    assert!(parse("mak_siccar 1 == 2 \"sums are aff\"").is_err());
}

#[test]
fn integer_literal_overflow_is_reported_as_a_bad_number() {
    // MDH-BUG-FLU-00020
    match parse("ken big = 9223372036854775808") {
        Err(HaversError::InvalidNumber { value, line }) => {
            assert_eq!(line, 1);
            assert!(value.contains("9223372036854775808"), "{value}");
        }
        other => panic!("expected InvalidNumber, got {other:?}"),
    }
}

#[test]
fn fstring_holes_reject_leftovers_and_report_real_lines() {
    // MDH-BUG-FLU-00021
    assert!(parse("ken a = 1\nken b = 2\nblether f\"{a b}\"").is_err());
    assert!(parse("ken x = 7\nblether f\"{x:>5}\"").is_err());
    let err = parse("# comment\nken a = 1\nken b = 2\nblether f\"{a +}\"").unwrap_err();
    assert_eq!(err.line(), Some(4), "{err}");
}

#[test]
fn fstring_holes_can_hold_string_literals() {
    // MDH-BUG-FLU-00022 (README.md example)
    let out = run(r#"blether f"Cities: {["Edinburgh", "Glasgow"]}""#);
    assert!(out.starts_with("Cities: ["), "{out}");
    assert!(
        out.contains("Edinburgh") && out.contains("Glasgow"),
        "{out}"
    );

    let code = r#"
ken d = {"k": 5}
blether f"{d["k"]} an' {"}"} an' {len("abc")}"
blether f"{{literal}} {1 + 1}"
blether f"it's {'single'}"
"#;
    assert_eq!(run(code), "5 an' } an' 3\n{literal} 2\nit's single");
}

#[test]
fn compound_assignment_on_fields_and_indexes() {
    // MDH-BUG-FLU-00023
    let code = r#"
kin C {
    dae init() { masel.n = 1 }
    dae bump() { masel.n += 1 }
}
ken c = C()
c.bump()
c.bump()
blether c.n
c.n *= 10
blether c.n
ken xs = [10, 20, 30]
xs[0] += 1
ken i = 1
xs[i + 1] -= 5
blether xs
ken d = {"k": 2}
d["k"] *= 3
blether d["k"]
ken grid = [[1, 2], [3, 4]]
grid[1][0] -= 1
blether grid[1][0]
"#;
    assert_eq!(run(code), "3\n30\n[11, 20, 25]\n6\n2");
}

#[test]
fn compound_assignment_evaluates_the_target_once_or_refuses() {
    // A target whose object or index would run code twice is refused.
    assert!(parse("ken xs = [1]\ndae f() { gie xs }\nf()[0] += 1").is_err());
}

#[test]
fn underscore_binders_work_in_loops_lambdas_and_functions() {
    // MDH-BUG-FLU-00024
    let code = r#"
ken n = 0
fer _ in 0..3 {
    n = n + 1
}
blether n
ken second = |_, y| y
blether second(1, 2)
dae middle(_, b, _) {
    gie b
}
blether middle(1, 2, 3)
"#;
    assert_eq!(run(code), "3\n2\n2");
}
