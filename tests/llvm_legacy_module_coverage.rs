#![cfg(feature = "llvm")]

use mdhavers::llvm::{builtins, runtime::RuntimeFunctions, types::MdhTypes};

#[test]
fn llvm_builtins_lookup_helpers_are_consistent() {
    // Basic sanity: known builtins exist and lookup helpers agree.
    assert!(builtins::is_builtin("blether"));
    assert!(builtins::is_builtin("len"));
    assert!(!builtins::is_builtin("definitely_not_a_builtin"));

    let map = builtins::get_builtin_map();
    let b1 = map.get("blether").copied().expect("blether in map");
    let b2 = builtins::get_builtin("blether").expect("blether via get_builtin");
    assert_eq!(b1.name, b2.name);
    assert_eq!(b1.runtime_name, b2.runtime_name);
    assert_eq!(b1.min_arity, b2.min_arity);
    assert_eq!(b1.max_arity, b2.max_arity);

    // Ensure at least one variadic/optional-arity builtin is represented.
    let jammy = builtins::get_builtin("jammy").expect("jammy builtin exists");
    assert_eq!(jammy.min_arity, 0);
    assert_eq!(jammy.max_arity, Some(2));
}

#[test]
fn llvm_runtime_functions_declare_registers_expected_symbols() {
    use inkwell::context::Context;

    let context = Context::create();
    let module = context.create_module("mdhavers_runtime_decl_test");
    let types = MdhTypes::new(&context);

    let _runtime = RuntimeFunctions::declare(&module, &types);

    // Spot-check a few critical runtime symbols that codegen relies on.
    for sym in [
        "__mdh_make_int",
        "__mdh_make_float",
        "__mdh_make_string",
        "__mdh_add",
        "__mdh_eq",
        "__mdh_truthy",
        "__mdh_len",
        "__mdh_to_string",
    ] {
        assert!(
            module.get_function(sym).is_some(),
            "expected runtime symbol to be declared: {sym}"
        );
    }
}

#[test]
fn llvm_types_helpers_cover_basic_invariants() {
    use inkwell::context::Context;
    use mdhavers::llvm::types::{InferredType, ValueTag};

    let context = Context::create();
    let types = MdhTypes::new(&context);

    // Ensure the value type is the expected { i8, i64 } layout (2 fields).
    assert_eq!(types.value_type.count_fields(), 2);

    // InferredType helpers.
    assert!(InferredType::Int.is_known());
    assert!(!InferredType::Unknown.is_known());
    assert_eq!(InferredType::Int.tag(), Some(ValueTag::Int));
    assert_eq!(InferredType::Numeric.tag(), None);
}

