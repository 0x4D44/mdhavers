#![cfg(coverage)]

use mdhavers::value::{DictValue, Environment, NativeFunction, RangeValue, SetValue};
use mdhavers::Value;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn value_helpers_are_exercised_for_instantiation_coverage() {
    let bytes = Value::Bytes(Rc::new(RefCell::new(vec![1u8, 2u8])));
    assert!(bytes.as_bytes().is_some());
    assert!(bytes.is_truthy());

    let empty_bytes = Value::Bytes(Rc::new(RefCell::new(Vec::new())));
    assert!(!empty_bytes.is_truthy());

    let list = Value::List(Rc::new(RefCell::new(vec![Value::Nil])));
    assert!(list.as_list().is_some());

    let dict = Value::Dict(Rc::new(RefCell::new(DictValue::new())));
    assert!(dict.as_dict().is_some());

    let set = Value::Set(Rc::new(RefCell::new(SetValue::new())));
    assert!(set.as_set().is_some());

    let n = Value::Integer(1);
    assert!(n.as_float().is_some());

    let native = NativeFunction::new("n", 0, |_args| Ok(Value::Nil));
    let _ = format!("{native:?}");
    let native_val = Value::NativeFunction(Rc::new(native));
    assert!(native_val.as_native_function().is_some());

    let _ = DictValue::default();
    let _ = SetValue::default();
    let _ = Environment::default();

    let range = RangeValue {
        start: 0,
        end: 1,
        inclusive: true,
    };
    let mut iter = range.iter();
    assert_eq!(iter.next(), Some(0));

    let mut a = SetValue::new();
    a.insert(Value::Integer(1));
    let mut b = SetValue::new();
    b.insert(Value::Integer(1));
    b.insert(Value::Integer(2));

    assert!(a.is_subset(&b));
    assert!(b.is_superset(&a));
    assert!(!a.is_disjoint(&b));

    let mut c = SetValue::new();
    c.insert(Value::Integer(3));
    assert!(a.is_disjoint(&c));
}
