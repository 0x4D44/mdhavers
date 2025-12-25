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
fn interpreter_atomic_and_channel_unknown_handles_cover_ok_or_for_coverage() {
    let interp = Interpreter::new();

    // atomic_* unknown handle should hit the internal ok_or("Unknown atomic handle") path.
    let atomic_load = native(&interp, "atomic_load");
    let atomic_store = native(&interp, "atomic_store");
    let atomic_add = native(&interp, "atomic_add");
    let atomic_cas = native(&interp, "atomic_cas");

    assert!((atomic_load.func)(vec![Value::Integer(999_999)]).is_err());
    assert!(
        (atomic_store.func)(vec![Value::Integer(999_999), Value::Integer(0)]).is_err(),
        "atomic_store should error on unknown handle"
    );
    assert!(
        (atomic_add.func)(vec![Value::Integer(999_999), Value::Integer(1)]).is_err(),
        "atomic_add should error on unknown handle"
    );
    assert!(
        (atomic_cas.func)(vec![
            Value::Integer(999_999),
            Value::Integer(0),
            Value::Integer(1)
        ])
        .is_err(),
        "atomic_cas should error on unknown handle"
    );

    // chan_* unknown handle should hit the internal ok_or("Unknown channel handle") path.
    let chan_send = native(&interp, "chan_send");
    let chan_recv = native(&interp, "chan_recv");
    let chan_close = native(&interp, "chan_close");

    assert!(
        (chan_send.func)(vec![Value::Integer(999_999), Value::Integer(1)]).is_err(),
        "chan_send should error on unknown handle"
    );
    assert!(
        (chan_recv.func)(vec![Value::Integer(999_999)]).is_err(),
        "chan_recv should error on unknown handle"
    );
    assert!(
        (chan_close.func)(vec![Value::Integer(999_999)]).is_err(),
        "chan_close should error on unknown handle"
    );
}

