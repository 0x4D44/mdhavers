use mdhavers::value::DictValue;
use mdhavers::Value;

#[test]
fn dict_remove_rebuilds_index_when_entries_shift_for_coverage() {
    let mut dict = DictValue::new();
    dict.set(Value::String("a".to_string()), Value::Integer(1));
    dict.set(Value::String("b".to_string()), Value::Integer(2));

    let removed = dict
        .remove(&Value::String("a".to_string()))
        .expect("remove should return the removed value");
    assert_eq!(removed.as_integer(), Some(1));

    assert_eq!(
        dict.get(&Value::String("b".to_string()))
            .and_then(Value::as_integer),
        Some(2)
    );
    assert_eq!(dict.len(), 1);
}

