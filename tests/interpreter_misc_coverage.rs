use std::cell::RefCell;
use std::rc::Rc;

use mdhavers::interpreter::{get_stack_trace, pop_stack_frame, push_stack_frame};
use mdhavers::value::{DictValue, NativeFunction};
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
fn interpreter_json_unicode_escape_paths_are_covered() {
    assert_eq!(
        run(r#"json_parse("\"\\u0041\"")"#).unwrap(),
        Value::String("A".to_string())
    );
    assert_eq!(
        run(r#"json_parse("\"\\uD800\"")"#).unwrap(),
        Value::String("".to_string())
    );
    assert_eq!(
        run(r#"json_parse("\"\\uZZZZ\"")"#).unwrap(),
        Value::String("".to_string())
    );
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
fn interpreter_destructure_ignore_before_rest_is_covered() {
    let value = run(
        r#"
ken [_, a, ...rest] = [1, 2, 3, 4]
a + len(rest)
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(4));
}

#[test]
fn interpreter_destructure_ignore_after_rest_is_covered() {
    let value = run(
        r#"
ken [a, ...rest, _] = [1, 2, 3, 4]
a + len(rest)
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(3));
}

#[test]
fn interpreter_slice_step_branches_for_list_and_string_are_covered() {
    let value = run(
        r#"
ken l = [1, 2, 3, 4, 5]
ken a = l[0:5:2]
ken b = l[5::-1]
ken s = "hello"
ken c = s[0:5:2]
ken d = s[5::-1]
len(a) + len(b) + len(c) + len(d)
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(16));
}

#[test]
fn interpreter_slice_negative_start_index_normalization_is_covered() {
    // Exercise the `start < 0` normalization branch in list/string slicing.
    let value = run(
        r#"
ken l = [1, 2, 3, 4, 5]
ken a = l[-1::-1]
ken s = "hello"
ken b = s[-1::-1]
len(a) + len(b)
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(10));
}

#[test]
fn interpreter_list_literal_spread_for_list_and_string_is_covered() {
    let value = run(
        r#"
ken xs = [0, ...[1, 2], 3]
ken ys = [..."ab"]
xs[1] + xs[2] + len(ys)
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(5));
}

#[test]
fn interpreter_sclaff_recursive_flatten_list_branch_is_covered() {
    let value = run(
        r#"
ken flat = sclaff([[1, [2]], 3])
flat[0] + flat[1] + flat[2]
"#,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(6));
}

#[test]
fn interpreter_log_level_integer_branches_are_covered() {
    for src in ["log_enabled(0)", "log_enabled(2)", "log_enabled(5)"] {
        let v = run(src).unwrap();
        assert!(matches!(v, Value::Bool(_)), "expected bool for {src}, got {v:?}");
    }

    assert!(run("log_enabled(6)").is_err());
    assert!(run("log_enabled([])").is_err());
}

#[cfg(all(coverage, not(target_arch = "wasm32")))]
#[test]
fn interpreter_import_covers_current_exe_error_branches_for_coverage() {
    for src in [
        r#"fetch "__mdhavers_coverage_current_exe_err__" tae m"#,
        r#"fetch "__mdhavers_coverage_current_exe_no_parent__" tae m"#,
    ] {
        let err = run(src).unwrap_err();
        assert!(
            matches!(err, mdhavers::HaversError::ModuleNotFound { .. }),
            "unexpected error for {src}: {err:?}"
        );
    }
}

#[cfg(coverage)]
#[test]
fn interpreter_format_braw_time_for_coverage_exercises_all_time_buckets_in_dependency() {
    let cases: &[(u64, u64, &str)] = &[
        (0, 0, "wee small hours"),
        (6, 0, "mornin'"),
        (12, 0, "high noon"),
        (13, 0, "efternoon"),
        (18, 0, "evenin'"),
        (22, 0, "gettin' late"),
    ];

    for (h, m, needle) in cases {
        let s = mdhavers::interpreter::format_braw_time_for_coverage(*h, *m);
        assert!(
            s.contains(needle),
            "unexpected bucket for {h:02}:{m:02}: {s}"
        );
    }
}

#[cfg(coverage)]
#[test]
fn interpreter_resolve_log_args_for_coverage_exercises_all_arms_in_dependency() {
    let mut dict = DictValue::new();
    dict.set(Value::String("a".to_string()), Value::Integer(1));
    let fields = Value::Dict(Rc::new(RefCell::new(dict)));

    assert_eq!(
        mdhavers::interpreter::resolve_log_args_for_coverage(&[]).unwrap(),
        (None, None)
    );
    assert!(mdhavers::interpreter::resolve_log_args_for_coverage(std::slice::from_ref(&fields))
        .unwrap()
        .0
        .is_some());
    assert_eq!(
        mdhavers::interpreter::resolve_log_args_for_coverage(&[Value::String("target".to_string())])
            .unwrap(),
        (None, Some("target".to_string()))
    );
    assert!(mdhavers::interpreter::resolve_log_args_for_coverage(&[Value::Integer(1)]).is_err());
    let (fields_val, target) = mdhavers::interpreter::resolve_log_args_for_coverage(&[
        fields.clone(),
        Value::String("t".to_string()),
    ])
    .unwrap();
    assert!(fields_val.is_some());
    assert_eq!(target, Some("t".to_string()));
    assert!(mdhavers::interpreter::resolve_log_args_for_coverage(&[
        Value::String("x".to_string()),
        Value::String("y".to_string())
    ])
    .is_err());
    assert!(mdhavers::interpreter::resolve_log_args_for_coverage(&[fields, Value::Integer(1)])
        .is_err());
    assert!(mdhavers::interpreter::resolve_log_args_for_coverage(&[
        Value::String("x".to_string()),
        Value::String("y".to_string()),
        Value::String("z".to_string())
    ])
    .is_err());
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
