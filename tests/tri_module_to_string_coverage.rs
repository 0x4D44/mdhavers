use mdhavers::tri::tri_module_value;
use mdhavers::value::Value;

fn native_object_to_string(value: Value) -> Option<String> {
    match value {
        Value::NativeObject(obj) => Some(obj.to_string()),
        _ => None,
    }
}

#[test]
fn tri_module_to_string_is_covered_in_dependency_crate_instance() {
    assert_eq!(
        native_object_to_string(tri_module_value()),
        Some("<native tri.module>".to_string())
    );
    assert_eq!(native_object_to_string(Value::Nil), None);
}
