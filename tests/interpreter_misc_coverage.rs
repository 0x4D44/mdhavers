use std::rc::Rc;

use mdhavers::value::NativeFunction;
use mdhavers::interpreter::{get_stack_trace, pop_stack_frame, push_stack_frame};
use mdhavers::{parse, Interpreter, Value};

fn run(source: &str) -> Result<Value, mdhavers::HaversError> {
    let program = parse(source).unwrap();
    let mut interp = Interpreter::new();
    interp.interpret(&program)
}

fn native(interp: &Interpreter, name: &str) -> Rc<NativeFunction> {
    let exports = interp.globals.borrow().get_exports();
    exports
        .into_iter()
        .find_map(|(n, v)| match (n == name, v) {
            (true, Value::NativeFunction(native)) => Some(native),
            _ => None,
        })
        .unwrap_or_else(|| panic!("native function not found: {name}"))
}

#[test]
fn interpreter_for_loop_over_range_branch_is_covered() {
    let value = run(
        r#"
ken sum = 0
fer i in 1..4 {
    sum = sum + i
}
sum
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(6));
}

#[test]
fn interpreter_operator_overload_method_path_is_used() {
    let value = run(
        r#"
kin Box {
    dae __pit_thegither__(other) { gie 123 }
}
ken a = Box()
ken b = Box()
a + b
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(123));
}

#[test]
fn interpreter_call_get_on_dict_falls_back_to_normal_call_path() {
    let value = run(
        r#"
dae inc(x) { gie x + 1 }
ken d = {"f": inc}
d.f(1)
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(2));
}

#[test]
fn interpreter_json_string_escapes_quote_and_backslash_are_covered() {
    let value = run(r#"json_parse("\"a\\\"b\\\\c\"")"#).unwrap();
    assert_eq!(value, Value::String("a\"b\\c".to_string()));
}

#[test]
fn interpreter_json_stringify_handles_non_string_dict_keys() {
    let value = run(r#"json_stringify({1: 2})"#).unwrap();
    let Value::String(s) = value else {
        panic!("expected json_stringify to return string, got {value:?}");
    };
    assert!(s.contains("\"1\""), "unexpected json: {s}");

    let value = run(r#"json_pretty({1: 2})"#).unwrap();
    let Value::String(s) = value else {
        panic!("expected json_pretty to return string, got {value:?}");
    };
    assert!(s.contains("\"1\""), "unexpected json: {s}");

    // Cover bool-false pretty branch too.
    let value = run(r#"json_pretty(nae)"#).unwrap();
    assert_eq!(value, Value::String("false".to_string()));
}

#[test]
fn interpreter_compare_float_and_string_paths_are_covered() {
    assert_eq!(run("1.0 <= 2.0").unwrap(), Value::Bool(true));
    assert_eq!(run(r#""a" <= "b""#).unwrap(), Value::Bool(true));
    assert_eq!(run("2.0 >= 1.0").unwrap(), Value::Bool(true));
    assert_eq!(run(r#""b" >= "a""#).unwrap(), Value::Bool(true));
}

#[test]
fn interpreter_stack_trace_helpers_are_covered() {
    push_stack_frame("<test>", 1);
    let trace = get_stack_trace();
    assert!(!trace.is_empty());
    pop_stack_frame();
}

#[cfg(all(feature = "native", unix))]
#[test]
fn interpreter_native_ipv4_resolution_and_nonblocking_false_path_are_covered() {
    let interp = Interpreter::new();
    let socket_udp = native(&interp, "socket_udp");
    let socket_bind = native(&interp, "socket_bind");
    let socket_set_nonblocking = native(&interp, "socket_set_nonblocking");
    let socket_close = native(&interp, "socket_close");

    let created = (socket_udp.func)(Vec::new()).expect("socket_udp ok");
    let Value::Dict(created) = created else {
        panic!("expected result dict, got {created:?}");
    };
    let created = created.borrow();
    assert_eq!(
        created.get(&Value::String("ok".to_string())),
        Some(&Value::Bool(true))
    );
    let sock_id = match created.get(&Value::String("value".to_string())) {
        Some(Value::Integer(id)) => *id,
        other => panic!("unexpected socket id: {other:?}"),
    };
    drop(created);

    // Bind to an IPv4 literal so the resolver loop breaks on a v4 address.
    let bound = (socket_bind.func)(vec![
        Value::Integer(sock_id),
        Value::String("127.0.0.1".to_string()),
        Value::Integer(0),
    ])
    .expect("socket_bind ok");
    let Value::Dict(bound) = bound else {
        panic!("expected result dict, got {bound:?}");
    };
    assert_eq!(
        bound.borrow().get(&Value::String("ok".to_string())),
        Some(&Value::Bool(true))
    );

    // Exercise the enable=false branch in socket_set_nonblocking.
    let updated = (socket_set_nonblocking.func)(vec![Value::Integer(sock_id), Value::Bool(false)])
        .expect("socket_set_nonblocking ok");
    let Value::Dict(updated) = updated else {
        panic!("expected result dict, got {updated:?}");
    };
    assert_eq!(
        updated.borrow().get(&Value::String("ok".to_string())),
        Some(&Value::Bool(true))
    );

    // Clean up FD.
    let closed = (socket_close.func)(vec![Value::Integer(sock_id)]).expect("socket_close ok");
    let Value::Dict(closed) = closed else {
        panic!("expected result dict, got {closed:?}");
    };
    assert_eq!(
        closed.borrow().get(&Value::String("ok".to_string())),
        Some(&Value::Bool(true))
    );
}
