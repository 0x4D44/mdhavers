use mdhavers::{run, Value};

#[test]
fn interpreter_json_parse_invalid_numbers_and_pretty_list_are_covered() {
    // parse_json_number: cover the float parse error mapping ("1e").
    assert!(run(r#"json_parse("1e")"#).is_err());

    // parse_json_number: cover the integer parse error mapping ("-").
    assert!(run(r#"json_parse("-")"#).is_err());

    // value_to_json_pretty: cover non-empty list formatting.
    assert_eq!(
        run(r#"json_pretty([1, 2])"#).unwrap(),
        Value::String("[\n  1,\n  2\n]".to_string())
    );
}

