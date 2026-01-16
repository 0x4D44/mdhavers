#![cfg(coverage)]

use mdhavers::value::DictValue;
use mdhavers::{Interpreter, Value};
use std::cell::RefCell;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

#[test]
fn interpreter_native_functions_are_invoked_for_instantiation_coverage() {
    let interpreter = Interpreter::new();
    let exports = interpreter.globals.borrow().get_exports();

    let mut natives: Vec<_> = exports
        .into_iter()
        .filter_map(|(name, value)| match value {
            Value::NativeFunction(f) => Some((name, f)),
            _ => None,
        })
        .collect();
    natives.sort_by(|(a, _), (b, _)| a.cmp(b));

    fn dict_value(pairs: &[(&str, Value)]) -> Value {
        let mut dict = DictValue::new();
        for (k, v) in pairs {
            dict.set(Value::String((*k).to_string()), v.clone());
        }
        Value::Dict(Rc::new(RefCell::new(dict)))
    }

    fn arg_cases(name: &str, arity: usize) -> Vec<Vec<Value>> {
        fn bytes(data: &[u8]) -> Value {
            Value::Bytes(Rc::new(RefCell::new(data.to_vec())))
        }

        if matches!(
            name,
            "input"
                | "get_key"
                | "read_line"
                | "read_lines"
                | "socket_accept"
                | "udp_recv_from"
                | "tcp_recv"
                | "tls_recv"
                | "event_loop_poll"
        ) {
            return Vec::new();
        }

        match name {
            // These are variadic; provide shapes that reach argument parsing without needing an
            // active interpreter context (the call will error, but still exercises instantiations).
            "log_enabled" => vec![
                vec![Value::String("blether".to_string())],
                vec![
                    Value::Integer(3),
                    Value::String("".to_string()),
                ],
            ],
            "log_event" => vec![
                vec![
                    Value::String("blether".to_string()),
                    Value::String("msg".to_string()),
                ],
                vec![
                    Value::Integer(3),
                    Value::String("msg".to_string()),
                    Value::Dict(Rc::new(RefCell::new(DictValue::new()))),
                ],
            ],
            "log_init" => vec![vec![], vec![Value::Dict(Rc::new(RefCell::new(DictValue::new())))]],
            "log_span" => vec![vec![Value::String("span".to_string())]],

            // Exercise TLS config parsing + server-mode branch (expected to error due to missing
            // cert/key, but still hits the instantiations).
            "tls_client_new" => vec![vec![dict_value(&[
                ("mode", Value::String("server".to_string())),
                // Omit server_name to exercise fallback in tls_config_from_value.
            ])]],

            // JSON helpers (pure, no IO).
            "json_parse" => vec![
                vec![Value::String("123".to_string())],
                vec![Value::String("{\"a\": 1}".to_string())],
            ],
            "json_pretty" | "json_stringify_pretty" => vec![vec![
                Value::List(Rc::new(RefCell::new(vec![
                    Value::Integer(1),
                    Value::String("x".to_string()),
                ]))),
            ]],

            "bytes_get" => vec![
                vec![bytes(&[1, 2]), Value::Integer(-1)],
                vec![bytes(&[1, 2]), Value::Integer(99)],
            ],
            "bytes_set" => vec![
                vec![bytes(&[0, 1]), Value::Integer(-1), Value::Integer(1)],
                vec![bytes(&[0, 1]), Value::Integer(99), Value::Integer(1)],
                vec![bytes(&[0, 1]), Value::Integer(0), Value::Integer(-1)],
                vec![bytes(&[0, 1]), Value::Integer(0), Value::Integer(256)],
            ],
            "bytes_read_u16be" => vec![
                vec![bytes(&[1, 2]), Value::Integer(-1)],
                vec![bytes(&[1]), Value::Integer(0)],
            ],
            "bytes_read_u32be" => vec![
                vec![bytes(&[1, 2, 3, 4]), Value::Integer(-1)],
                vec![bytes(&[1, 2, 3]), Value::Integer(0)],
            ],
            "bytes_write_u16be" => vec![
                vec![bytes(&[0, 0]), Value::Integer(-1), Value::Integer(1)],
                vec![bytes(&[0, 0]), Value::Integer(0), Value::Integer(-1)],
                vec![bytes(&[0, 0]), Value::Integer(0), Value::Integer(70000)],
                vec![bytes(&[0]), Value::Integer(0), Value::Integer(1)],
            ],
            "bytes_write_u32be" => vec![
                vec![bytes(&[0, 0, 0, 0]), Value::Integer(-1), Value::Integer(1)],
                vec![bytes(&[0, 0, 0, 0]), Value::Integer(0), Value::Integer(-1)],
                vec![bytes(&[0, 0, 0, 0]), Value::Integer(0), Value::Integer(5_000_000_000_i64)],
                vec![bytes(&[0]), Value::Integer(0), Value::Integer(1)],
            ],
            "socket_bind" | "socket_connect" | "udp_send_to" => vec![
                vec![Value::Integer(0), Value::String("127.0.0.1".to_string()), Value::Integer(-1)],
                vec![Value::Integer(0), Value::String("127.0.0.1".to_string()), Value::Integer(70000)],
            ],
            "socket_set_ttl" => vec![
                vec![Value::Integer(0), Value::Integer(-1)],
                vec![Value::Integer(0), Value::Integer(999)],
            ],
            "socket_set_rcvbuf" | "socket_set_sndbuf" => vec![
                vec![Value::Integer(0), Value::Integer(-1)],
                vec![Value::Integer(0), Value::Integer(i64::from(i32::MAX) + 1)],
            ],
            "snooze" | "sleep" => vec![vec![Value::Integer(0)], vec![Value::Float(0.0)]],
            _ => {
                if arity == usize::MAX {
                    vec![
                        Vec::new(),
                        vec![Value::Nil],
                        vec![Value::Nil, Value::Nil],
                        vec![Value::Integer(0)],
                        vec![Value::Integer(-1)],
                        vec![Value::Integer(0), Value::Nil],
                        vec![Value::Integer(0), Value::Integer(0), Value::Integer(0)],
                        vec![Value::Integer(-1), Value::Integer(-1)],
                    ]
                } else {
                    let mut cases = Vec::new();
                    cases.push(vec![Value::Nil; arity]);
                    if arity > 0 {
                        cases.push(
                            std::iter::once(Value::Integer(0))
                                .chain(std::iter::repeat_with(|| Value::Nil).take(arity - 1))
                                .collect(),
                        );
                        cases.push(
                            std::iter::once(Value::Integer(-1))
                                .chain(std::iter::repeat_with(|| Value::Nil).take(arity - 1))
                                .collect(),
                        );
                        cases.push(
                            std::iter::once(Value::String(String::new()))
                                .chain(std::iter::repeat_with(|| Value::Nil).take(arity - 1))
                                .collect(),
                        );
                        cases.push(vec![Value::Integer(0); arity]);
                        if arity > 1 {
                            let mut combo = Vec::with_capacity(arity);
                            combo.push(Value::Integer(-1));
                            combo.push(Value::Integer(-1));
                            combo.extend(std::iter::repeat_with(|| Value::Nil).take(arity - 2));
                            cases.push(combo);
                        }
                    }
                    cases
                }
            }
        }
    }

    for (name, native) in natives {
        for args in arg_cases(&name, native.arity).into_iter().take(2) {
            let _ = catch_unwind(AssertUnwindSafe(|| (native.func)(args)));
        }
    }
}
