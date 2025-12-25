use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use mdhavers::value::NativeObject;
use mdhavers::{parse, HaversError, Interpreter, Value};

#[derive(Debug)]
struct TestNative {
    fields: RefCell<HashMap<String, Value>>,
}

impl TestNative {
    fn new() -> Self {
        Self {
            fields: RefCell::new(HashMap::new()),
        }
    }
}

impl NativeObject for TestNative {
    fn type_name(&self) -> &str {
        "test_native"
    }

    fn get(&self, prop: &str) -> Result<Value, HaversError> {
        self.fields
            .borrow()
            .get(prop)
            .cloned()
            .ok_or_else(|| HaversError::UndefinedVariable {
                name: prop.to_string(),
                line: 0,
            })
    }

    fn set(&self, prop: &str, value: Value) -> Result<Value, HaversError> {
        self.fields
            .borrow_mut()
            .insert(prop.to_string(), value.clone());
        Ok(value)
    }

    fn call(&self, method: &str, _args: Vec<Value>) -> Result<Value, HaversError> {
        Err(HaversError::UndefinedVariable {
            name: method.to_string(),
            line: 0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct SetFailNative;

impl NativeObject for SetFailNative {
    fn type_name(&self) -> &str {
        "set_fail_native"
    }

    fn get(&self, prop: &str) -> Result<Value, HaversError> {
        Err(HaversError::UndefinedVariable {
            name: prop.to_string(),
            line: 0,
        })
    }

    fn set(&self, prop: &str, _value: Value) -> Result<Value, HaversError> {
        Err(HaversError::UndefinedVariable {
            name: prop.to_string(),
            line: 0,
        })
    }

    fn call(&self, method: &str, _args: Vec<Value>) -> Result<Value, HaversError> {
        Err(HaversError::UndefinedVariable {
            name: method.to_string(),
            line: 0,
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
struct CallOkNative;

impl NativeObject for CallOkNative {
    fn type_name(&self) -> &str {
        "call_ok_native"
    }

    fn get(&self, prop: &str) -> Result<Value, HaversError> {
        Err(HaversError::UndefinedVariable {
            name: prop.to_string(),
            line: 0,
        })
    }

    fn set(&self, prop: &str, _value: Value) -> Result<Value, HaversError> {
        Err(HaversError::UndefinedVariable {
            name: prop.to_string(),
            line: 0,
        })
    }

    fn call(&self, method: &str, args: Vec<Value>) -> Result<Value, HaversError> {
        if method == "ping" {
            Ok(Value::Integer(args.len() as i64))
        } else {
            Err(HaversError::UndefinedVariable {
                name: method.to_string(),
                line: 0,
            })
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn interpreter_native_object_get_and_call_errors_map_line_numbers_for_coverage() {
    let mut interp = Interpreter::new();
    interp
        .globals
        .borrow_mut()
        .define("obj".to_string(), Value::NativeObject(Rc::new(TestNative::new())));

    // NativeObject property get error -> map_err(|err| err.with_line_if_zero(span.line))
    let program = parse("obj.missing").unwrap();
    let err = interp.interpret(&program).unwrap_err();
    match err {
        HaversError::UndefinedVariable { line, .. } => assert_ne!(line, 0),
        other => panic!("expected UndefinedVariable, got {other:?}"),
    }

    // NativeObject method call error -> map_err(|err| err.with_line_if_zero(span.line))
    let program = parse("obj.nope()").unwrap();
    let err = interp.interpret(&program).unwrap_err();
    match err {
        HaversError::UndefinedVariable { line, .. } => assert_ne!(line, 0),
        other => panic!("expected UndefinedVariable, got {other:?}"),
    }
}

#[test]
fn interpreter_native_object_method_call_success_path_is_covered() {
    let mut interp = Interpreter::new();
    interp.globals.borrow_mut().define(
        "obj".to_string(),
        Value::NativeObject(Rc::new(CallOkNative)),
    );

    let program = parse("obj.ping(1, 2)").unwrap();
    let value = interp.interpret(&program).unwrap();
    assert_eq!(value, Value::Integer(2));
}

#[test]
fn interpreter_native_object_set_errors_map_line_numbers_for_coverage() {
    let mut interp = Interpreter::new();
    interp.globals.borrow_mut().define(
        "obj".to_string(),
        Value::NativeObject(Rc::new(SetFailNative)),
    );

    // NativeObject property set error -> map_err(|err| err.with_line_if_zero(span.line))
    let program = parse("obj.foo = 1").unwrap();
    let err = interp.interpret(&program).unwrap_err();
    match err {
        HaversError::UndefinedVariable { line, .. } => assert_ne!(line, 0),
        other => panic!("expected UndefinedVariable, got {other:?}"),
    }
}

#[test]
fn interpreter_log_span_in_rejects_wrong_native_object_type_for_coverage() {
    let mut interp = Interpreter::new();
    interp.globals.borrow_mut().define(
        "bogus".to_string(),
        Value::NativeObject(Rc::new(TestNative::new())),
    );

    let program = parse("log_span_in(bogus, 123)").unwrap();
    let err = interp.interpret(&program).unwrap_err();
    let s = format!("{err:?}");
    assert!(
        s.contains("log_span_in() expects a log span handle"),
        "unexpected error: {s}"
    );
}
