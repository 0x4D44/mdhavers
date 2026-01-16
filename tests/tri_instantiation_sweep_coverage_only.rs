#![cfg(coverage)]

use mdhavers::tri;
use mdhavers::Value;

#[test]
fn tri_module_and_object_are_exercised_for_instantiation_coverage() {
    let module_val = tri::tri_module_value();
    let Value::NativeObject(module) = module_val else {
        panic!("expected tri module native object");
    };

    assert_eq!(module.type_name(), "tri.module");
    assert_eq!(module.to_string(), "<native tri.module>");
    let _ = module.as_any().type_id();

    let _ = module.get("DEG_TO_RAD").unwrap();
    let _ = module.get("RAD_TO_DEG").unwrap();

    let ctor_val = module.get("Thing3D").unwrap();
    let ctor = ctor_val
        .as_native_function()
        .expect("expected tri constructor");
    let obj_val = (ctor.func)(Vec::new()).unwrap();
    let Value::NativeObject(obj) = obj_val else {
        panic!("expected tri object");
    };

    assert_eq!(obj.type_name(), "Thing3D");
    assert_eq!(obj.to_string(), "<native Thing3D>");
    let _ = obj.as_any().type_id();

    let _ = obj.get("position").unwrap();
    let _ = obj.get("rotation").unwrap();
    let _ = obj.get("scale").unwrap();

    obj.set("custom", Value::Integer(1)).unwrap();
    assert_eq!(obj.get("custom").unwrap(), Value::Integer(1));
    assert!(obj.get("missing").is_err());

    obj.call("add", vec![Value::Integer(1), Value::Integer(2)])
        .unwrap();
    obj.call("remove", vec![Value::Integer(1)]).unwrap();
    let _ = obj.call("nope", vec![]).unwrap();

    // Exercise module.call constructor path (drives apply_constructor_args + set_arg).
    let mesch_val = module
        .call("Mesch", vec![Value::Integer(10), Value::Integer(11)])
        .unwrap();
    let Value::NativeObject(mesch) = mesch_val else {
        panic!("expected tri object from module.call");
    };
    assert_eq!(mesch.type_name(), "Mesch");

    // Error paths.
    let _ = module.get("NOPE").unwrap_err();
    let _ = module.set("x", Value::Nil).unwrap_err();
    let _ = module.call("NOPE", vec![]).unwrap_err();
}
