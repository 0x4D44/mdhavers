use mdhavers::{run, Value};

#[test]
fn interpreter_is_a_covers_remaining_type_name_arms_for_coverage() {
    assert_eq!(run(r#"is_a(1, "int")"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"is_a(1, "integer")"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"is_a(1.0, "float")"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"is_a("hi", "string")"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"is_a("hi", "str")"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"is_a(aye, "bool")"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"is_a([1, 2], "list")"#).unwrap(), Value::Bool(true));
    assert_eq!(
        run(r#"is_a(bytes_from_string("hi"), "bytes")"#).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        run(r#"is_a(bytes_from_string("hi"), "byte")"#).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(run(r#"is_a({"a": 1}, "dict")"#).unwrap(), Value::Bool(true));
    assert_eq!(
        run(r#"is_a(naething, "naething")"#).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(run(r#"is_a(naething, "nil")"#).unwrap(), Value::Bool(true));

    assert_eq!(
        run(
            r#"
dae foo() { gie 1 }
is_a(foo, "dae")
"#,
        )
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(run(r#"is_a(len, "dae")"#).unwrap(), Value::Bool(true));

    assert_eq!(run(r#"is_a(1, "nope")"#).unwrap(), Value::Bool(false));
    assert!(run(r#"is_a(1, 2)"#).is_err());

    // bytes(): cover negative-size clamp branch.
    assert_eq!(run(r#"len(bytes(-1))"#).unwrap(), Value::Integer(0));
}

#[test]
fn interpreter_split_by_covers_all_predicate_branches_for_coverage() {
    let value = run(
        r#"
ken a = split_by([-2, -1, 0, 1, 2, 3], "odd")
ken b = split_by([-2, -1, 0, 1, 2, 3], "negative")
ken c = split_by([-1.0, 0.0, 1.0], "positive")
ken d = split_by([-1.0, 0.0, 1.0], "negative")
ken e = split_by([0, 1, naething, "x"], "truthy")
ken f = split_by([naething, 1], "nil")
ken g = split_by(["a", 1], "string")
ken h = split_by(["a", 1, 2.0], "number")
len(a[0]) + len(b[0]) + len(c[0]) + len(d[0]) + len(e[0]) + len(f[0]) + len(g[0]) + len(h[0])
"#,
    )
    .unwrap();

    assert_eq!(value, Value::Integer(13));
    assert!(run(r#"split_by(1, "even")"#).is_err());
    assert!(run(r#"split_by([1], 1)"#).is_err());
}

#[test]
fn interpreter_glaikit_covers_additional_empty_variants_for_coverage() {
    assert_eq!(run(r#"glaikit(0.0)"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"glaikit("   ")"#).unwrap(), Value::Bool(true));
    assert_eq!(run(r#"glaikit([])"#).unwrap(), Value::Bool(true));

    // Produce an empty dict via dict_remove ({} is ambiguous with blocks).
    assert_eq!(
        run(r#"glaikit(dict_remove({"a": 1}, "a"))"#).unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn interpreter_drookit_rejects_non_list_for_coverage() {
    assert!(run(r#"drookit("abc")"#).is_err());
}

#[test]
fn interpreter_log_init_covers_text_and_compact_formats_for_coverage() {
    assert_eq!(
        run(r#"log_init({"format": "text"})"#).unwrap(),
        Value::Nil
    );
    assert_eq!(
        run(r#"log_init({"format": "compact"})"#).unwrap(),
        Value::Nil
    );
}
