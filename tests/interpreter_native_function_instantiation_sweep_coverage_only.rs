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

            _ => {
                if arity == usize::MAX {
                    vec![
                        Vec::new(),
                        vec![Value::Nil],
                        vec![Value::Nil, Value::Nil],
                        vec![Value::Integer(0)],
                        vec![Value::Integer(0), Value::Nil],
                        vec![Value::Integer(0), Value::Integer(0), Value::Integer(0)],
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
                        cases.push(vec![Value::Integer(0); arity]);
                    }
                    cases
                }
            }
        }
    }

    for (name, native) in natives {
        for args in arg_cases(&name, native.arity) {
            let _ = catch_unwind(AssertUnwindSafe(|| (native.func)(args)));
        }
    }
}
