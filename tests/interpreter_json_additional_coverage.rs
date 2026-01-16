use std::rc::Rc;

use mdhavers::value::NativeFunction;
use mdhavers::{run, Interpreter, Value};

fn native(interp: &Interpreter, name: &str) -> Rc<NativeFunction> {
    let exports = interp.globals.borrow().get_exports();
    exports
        .into_iter()
        .find_map(|(n, v)| {
            if n == name {
                match v {
                    Value::NativeFunction(native) => Some(native),
                    other => panic!("expected native function {name}, got {other:?}"),
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("native function not found: {name}"))
}

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

#[test]
fn interpreter_json_parse_additional_branches_are_covered() {
    assert!(run(r#"json_parse("")"#).is_err());
    assert!(run(r#"json_parse("tru")"#).is_err());
    assert!(run(r#"json_parse("fals")"#).is_err());
    assert!(run(r#"json_parse("nul")"#).is_err());

    assert!(run(r#"json_parse("[")"#).is_err());
    assert!(run(r#"json_parse("{")"#).is_err());
    assert!(run(r#"json_parse("[1 2]")"#).is_err());
    assert!(run(r#"json_parse("{1: 2}")"#).is_err());
    assert!(run(r#"json_parse("{\"a\": 1 x}")"#).is_err());

    let value = run(r#"json_parse("1e+2")"#).unwrap();
    assert!(matches!(value, Value::Float(_)));
    let value = run(r#"json_parse("1E-2")"#).unwrap();
    assert!(matches!(value, Value::Float(_)));

    assert_eq!(
        run(r#"json_pretty([])"#).unwrap(),
        Value::String("[]".to_string())
    );
    assert_eq!(
        run(r#"json_pretty({})"#).unwrap(),
        Value::String("{}".to_string())
    );
}

#[test]
fn interpreter_json_stringify_control_chars_are_covered() {
    let interp = Interpreter::new();
    let json_stringify = native(&interp, "json_stringify");
    let result = (json_stringify.func)(vec![Value::String("\u{0001}".to_string())]).unwrap();
    let Value::String(s) = result else {
        panic!("expected json_stringify to return string, got {result:?}");
    };
    assert!(s.contains("\\u0001"), "unexpected json escape: {s}");
}
