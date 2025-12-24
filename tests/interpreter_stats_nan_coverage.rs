use std::cell::RefCell;
use std::rc::Rc;

use mdhavers::value::NativeFunction;
use mdhavers::{Interpreter, Value};

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
fn interpreter_median_nan_error_branch_is_covered() {
    let interp = Interpreter::new();
    let median = native(&interp, "median");

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Float(f64::NAN)])));
    let err = (median.func)(vec![list]).expect_err("expected median(NaN) to error");
    assert!(err.contains("NaN"), "unexpected error: {err}");
}

