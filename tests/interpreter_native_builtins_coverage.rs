use std::cell::RefCell;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use mdhavers::value::{DictValue, SetValue};
use mdhavers::{Interpreter, Value};

fn sample_list() -> Value {
    Value::List(Rc::new(RefCell::new(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ])))
}

fn sample_dict() -> Value {
    let mut dict = DictValue::new();
    dict.set(Value::String("a".to_string()), Value::Integer(1));
    Value::Dict(Rc::new(RefCell::new(dict)))
}

fn sample_set() -> Value {
    let mut set = SetValue::new();
    set.insert(Value::String("a".to_string()));
    Value::Set(Rc::new(RefCell::new(set)))
}

#[test]
fn interpreter_calls_all_native_builtins_for_coverage() {
    let interp = Interpreter::new();
    let exports = interp.globals.borrow().get_exports();

    let temp = tempfile::tempdir().unwrap();
    let temp_file = temp.path().join("mdh_native_cov.txt");
    std::fs::write(&temp_file, "hello\nworld\n").unwrap();
    let temp_path = temp_file.to_string_lossy().to_string();

    let arity1_args: &[Value] = &[
        Value::Nil,
        Value::Bool(true),
        Value::Integer(1),
        Value::Float(1.5),
        Value::String("hello".to_string()),
        sample_list(),
        sample_dict(),
        sample_set(),
    ];

    // Keep this conservative: the goal is to execute native builtin bodies without hanging the test
    // suite (e.g., `get_key`), or terminating the process (`exit`).
    let skip = ["exit", "get_key"];

    for (name, value) in exports {
        let Value::NativeFunction(native) = value else {
            continue;
        };
        if skip.contains(&name.as_str()) {
            continue;
        }

        let candidates: Vec<Vec<Value>> = match (name.as_str(), native.arity) {
            ("bide", 1) => vec![vec![Value::Integer(0)]],
            ("shell", 1) => vec![vec![Value::String("echo hello".to_string())]],
            ("shell_status", 1) => vec![vec![Value::String("exit 0".to_string())]],
            ("env_get", 1) => vec![vec![Value::String("PATH".to_string())]],
            ("scrieve", 2) => vec![vec![
                Value::String(temp_path.clone()),
                Value::String("hello\nworld\n".to_string()),
            ]],
            ("append_file", 2) => vec![vec![
                Value::String(temp_path.clone()),
                Value::String("!".to_string()),
            ]],
            ("read_file", 1) => vec![vec![Value::String(temp_path.clone())]],
            ("read_lines", 1) => vec![vec![Value::String(temp_path.clone())]],
            ("file_exists", 1) => vec![vec![Value::String(temp_path.clone())]],
            (_, 0) => vec![vec![]],
            (_, 1) => arity1_args.iter().cloned().map(|v| vec![v]).collect(),
            (_, 2) => vec![
                vec![
                    Value::String("hello".to_string()),
                    Value::String("he".to_string()),
                ],
                vec![sample_list(), Value::Integer(1)],
                vec![sample_dict(), Value::String("a".to_string())],
                vec![Value::Integer(1), Value::Integer(2)],
                vec![Value::Float(1.0), Value::Float(2.0)],
            ],
            (_, 3) => vec![
                vec![
                    Value::String("hello".to_string()),
                    Value::Integer(5),
                    Value::String(" ".to_string()),
                ],
                vec![
                    Value::Integer(0),
                    Value::Integer(1),
                    Value::String("seconds".to_string()),
                ],
                vec![
                    Value::String("a1b2".to_string()),
                    Value::String("[0-9]".to_string()),
                    Value::String("".to_string()),
                ],
            ],
            // Unusual arities are uncommon; call with nils to at least execute the arity checks/type
            // errors inside the native function.
            (_, n) => vec![vec![Value::Nil; n]],
        };

        for args in candidates {
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| (native.func)(args)));
        }
    }
}
