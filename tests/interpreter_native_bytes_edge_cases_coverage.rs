use std::cell::RefCell;
use std::rc::Rc;

use mdhavers::value::NativeFunction;
use mdhavers::{Interpreter, Value};

fn bytes(data: &[u8]) -> Value {
    Value::Bytes(Rc::new(RefCell::new(data.to_vec())))
}

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
fn interpreter_bytes_builtins_cover_edge_branches_for_coverage() {
    let interp = Interpreter::new();

    let bytes_slice = native(&interp, "bytes_slice");
    let bytes_get = native(&interp, "bytes_get");
    let bytes_set = native(&interp, "bytes_set");
    let bytes_append = native(&interp, "bytes_append");
    let bytes_read_u16be = native(&interp, "bytes_read_u16be");
    let bytes_read_u32be = native(&interp, "bytes_read_u32be");
    let bytes_write_u16be = native(&interp, "bytes_write_u16be");
    let bytes_write_u32be = native(&interp, "bytes_write_u32be");

    // bytes_slice: type errors + negative/clamped indices.
    assert!((bytes_slice.func)(vec![bytes(&[1, 2, 3]), Value::Nil, Value::Integer(1)]).is_err());
    assert!((bytes_slice.func)(vec![bytes(&[1, 2, 3]), Value::Integer(0), Value::Nil]).is_err());
    let _ = (bytes_slice.func)(vec![bytes(&[1, 2, 3]), Value::Integer(-1), Value::Integer(99)])
        .unwrap();
    let _ = (bytes_slice.func)(vec![bytes(&[1, 2, 3]), Value::Integer(-99), Value::Integer(2)])
        .unwrap();
    let _ = (bytes_slice.func)(vec![bytes(&[1, 2, 3]), Value::Integer(2), Value::Integer(-99)])
        .unwrap();

    // bytes_get: type error + negative index + oob.
    assert!((bytes_get.func)(vec![bytes(&[1, 2, 3]), Value::Nil]).is_err());
    let _ = (bytes_get.func)(vec![bytes(&[1, 2, 3]), Value::Integer(-1)]).unwrap();
    assert!((bytes_get.func)(vec![bytes(&[1, 2, 3]), Value::Integer(99)]).is_err());

    // bytes_set: type errors, float conversion, range checks, negative idx handling.
    assert!((bytes_set.func)(vec![bytes(&[1, 2, 3]), Value::Nil, Value::Integer(1)]).is_err());
    let _ = (bytes_set.func)(vec![bytes(&[0, 0, 0]), Value::Integer(0), Value::Float(42.0)])
        .unwrap();
    assert!((bytes_set.func)(vec![bytes(&[0, 0, 0]), Value::Integer(0), Value::Nil]).is_err());
    assert!(
        (bytes_set.func)(vec![bytes(&[0, 0, 0]), Value::Integer(0), Value::Integer(256)]).is_err()
    );
    let _ = (bytes_set.func)(vec![bytes(&[0, 0, 0]), Value::Integer(-1), Value::Integer(7)])
        .unwrap();
    assert!(
        (bytes_set.func)(vec![bytes(&[0, 0, 0]), Value::Integer(99), Value::Integer(7)]).is_err()
    );

    // bytes_append: second arg type error.
    assert!((bytes_append.func)(vec![bytes(&[1, 2]), Value::Integer(1)]).is_err());

    // bytes_read_u16be/u32be: type errors + oob.
    assert!((bytes_read_u16be.func)(vec![bytes(&[1, 2]), Value::Nil]).is_err());
    assert!((bytes_read_u16be.func)(vec![bytes(&[1]), Value::Integer(0)]).is_err());
    assert!((bytes_read_u32be.func)(vec![bytes(&[1, 2, 3, 4]), Value::Nil]).is_err());
    assert!((bytes_read_u32be.func)(vec![bytes(&[1, 2, 3]), Value::Integer(0)]).is_err());

    // bytes_write_u16be/u32be: float conversion + range checks + oob.
    let _ = (bytes_write_u16be.func)(vec![bytes(&[0, 0]), Value::Integer(0), Value::Float(7.0)])
        .unwrap();
    assert!(
        (bytes_write_u16be.func)(vec![bytes(&[0, 0]), Value::Integer(0), Value::Integer(-1)])
            .is_err()
    );
    assert!(
        (bytes_write_u16be.func)(vec![bytes(&[0]), Value::Integer(0), Value::Integer(1)]).is_err()
    );

    let _ = (bytes_write_u32be.func)(vec![
            bytes(&[0, 0, 0, 0]),
            Value::Integer(0),
            Value::Float(7.0),
        ])
        .unwrap();
    assert!(
        (bytes_write_u32be.func)(vec![
            bytes(&[0, 0, 0, 0]),
            Value::Integer(0),
            Value::Integer(-1)
        ])
        .is_err()
    );
    assert!(
        (bytes_write_u32be.func)(vec![bytes(&[0]), Value::Integer(0), Value::Integer(1)]).is_err()
    );
}
