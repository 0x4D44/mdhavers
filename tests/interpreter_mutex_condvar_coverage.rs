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
fn interpreter_mutex_and_condvar_builtins_cover_registry_and_branches_for_coverage() {
    let interp = Interpreter::new();

    let mutex_new = native(&interp, "mutex_new");
    let mutex_lock = native(&interp, "mutex_lock");
    let mutex_unlock = native(&interp, "mutex_unlock");
    let mutex_try_lock = native(&interp, "mutex_try_lock");

    let condvar_new = native(&interp, "condvar_new");
    let condvar_wait = native(&interp, "condvar_wait");
    let condvar_timed_wait = native(&interp, "condvar_timed_wait");
    let condvar_signal = native(&interp, "condvar_signal");
    let condvar_broadcast = native(&interp, "condvar_broadcast");

    let Value::Integer(mutex_id) = (mutex_new.func)(vec![]).unwrap() else {
        panic!("expected mutex_new to return integer id");
    };

    // mutex_try_lock: success then failure when already locked.
    assert_eq!(
        (mutex_try_lock.func)(vec![Value::Integer(mutex_id)]).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        (mutex_try_lock.func)(vec![Value::Integer(mutex_id)]).unwrap(),
        Value::Bool(false)
    );

    // mutex_unlock then lock/unlock via explicit calls.
    assert_eq!(
        (mutex_unlock.func)(vec![Value::Integer(mutex_id)]).unwrap(),
        Value::Nil
    );
    assert_eq!(
        (mutex_lock.func)(vec![Value::Integer(mutex_id)]).unwrap(),
        Value::Nil
    );
    assert_eq!(
        (mutex_unlock.func)(vec![Value::Integer(mutex_id)]).unwrap(),
        Value::Nil
    );

    // Unknown mutex handle error path.
    assert!(
        (mutex_unlock.func)(vec![Value::Integer(999_999)]).is_err(),
        "expected unknown mutex handle error"
    );

    let Value::Integer(condvar_id) = (condvar_new.func)(vec![]).unwrap() else {
        panic!("expected condvar_new to return integer id");
    };

    // condvar_wait/timed_wait/signal/broadcast: success paths.
    assert_eq!(
        (condvar_wait.func)(vec![Value::Integer(condvar_id), Value::Integer(mutex_id)]).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        (condvar_timed_wait.func)(vec![
            Value::Integer(condvar_id),
            Value::Integer(mutex_id),
            Value::Float(5.0),
        ])
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        (condvar_timed_wait.func)(vec![
            Value::Integer(condvar_id),
            Value::Integer(mutex_id),
            Value::Integer(5),
        ])
        .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        (condvar_signal.func)(vec![Value::Integer(condvar_id)]).unwrap(),
        Value::Nil
    );
    assert_eq!(
        (condvar_broadcast.func)(vec![Value::Integer(condvar_id)]).unwrap(),
        Value::Nil
    );

    // condvar_timed_wait: argument validation error.
    assert!(
        (condvar_timed_wait.func)(vec![
            Value::Integer(condvar_id),
            Value::Integer(mutex_id),
            Value::String("nope".to_string()),
        ])
        .is_err(),
        "expected timeout validation error"
    );

    // Unknown condvar handle error path.
    assert!(
        (condvar_wait.func)(vec![Value::Integer(999_999), Value::Integer(mutex_id)]).is_err(),
        "expected unknown condvar handle error"
    );
}
