use mdhavers::{parse, Interpreter};

#[test]
fn interpreter_bytes_basics() {
    let code = r#"
ken b = bytes(4)
bytes_set(b, 0, 1)
bytes_set(b, 1, 2)
bytes_set(b, 2, 3)
bytes_set(b, 3, 4)

blether len(b)
blether bytes_get(b, 2)
blether bytes_read_u16be(b, 1)

ken s = bytes_slice(b, 1, 3)
blether len(s)
blether bytes_get(s, 0)
"#;

    let program = parse(code).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program).unwrap();
    let out = interp.get_output().join("\n");
    assert_eq!(out.trim(), "4\n3\n515\n2\n2");
}

#[test]
fn interpreter_bytes_slice_rejects_non_integer_start_for_coverage() {
    let program = parse(
        r#"
ken b = bytes(1)
bytes_slice(b, "nope", 1)
"#,
    )
    .unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected bytes_slice start type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("bytes_slice() expects integer start"),
        "unexpected error: {s}"
    );
}

#[test]
fn interpreter_bytes_slice_rejects_non_integer_end_for_coverage() {
    let program = parse(
        r#"
ken b = bytes(1)
bytes_slice(b, 0, "nope")
"#,
    )
    .unwrap();
    let mut interp = Interpreter::new();
    let err = interp
        .interpret(&program)
        .expect_err("expected bytes_slice end type error");
    let s = format!("{err:?}");
    assert!(
        s.contains("bytes_slice() expects integer end"),
        "unexpected error: {s}"
    );
}
