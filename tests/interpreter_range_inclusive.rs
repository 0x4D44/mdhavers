use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_inclusive_range_includes_end_value() {
    let code = r#"
blether len(1..=3)

ken sum = 0
fer i in 1..=3 {
    sum = sum + i
}
blether sum
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "3\n6");
}

