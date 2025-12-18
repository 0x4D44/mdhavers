#![cfg(feature = "llvm")]

use mdhavers::llvm::{builtins, types};

#[test]
fn llvm_builtins_helpers_are_exercised() {
    assert!(builtins::is_builtin("len"));
    assert!(builtins::is_builtin("tae_string"));
    assert!(!builtins::is_builtin("definitely_not_a_builtin"));

    let map = builtins::get_builtin_map();
    assert!(map.contains_key("len"));

    let len = builtins::get_builtin("len").expect("len builtin");
    assert_eq!(len.name, "len");
    assert_eq!(len.min_arity, 1);
    assert_eq!(len.max_arity, Some(1));
}

#[test]
fn llvm_types_helpers_are_exercised() {
    use inkwell::context::Context;

    // ValueTag conversions
    assert_eq!(types::ValueTag::Nil.as_u8(), 0);
    assert_eq!(types::ValueTag::Bool.as_u8(), 1);
    assert_eq!(types::ValueTag::Int.as_u8(), 2);
    assert_eq!(types::ValueTag::Float.as_u8(), 3);
    assert_eq!(types::ValueTag::String.as_u8(), 4);
    assert_eq!(types::ValueTag::List.as_u8(), 5);
    assert_eq!(types::ValueTag::Dict.as_u8(), 6);
    assert_eq!(types::ValueTag::Function.as_u8(), 7);
    assert_eq!(types::ValueTag::Class.as_u8(), 8);
    assert_eq!(types::ValueTag::Instance.as_u8(), 9);
    assert_eq!(types::ValueTag::Range.as_u8(), 10);

    // MdhTypes construction + basic type helper
    let ctx = Context::create();
    let t = types::MdhTypes::new(&ctx);
    let _ = t.value_basic_type();

    // InferredType helpers
    assert!(!types::InferredType::Unknown.is_known());
    for ty in [
        types::InferredType::Nil,
        types::InferredType::Bool,
        types::InferredType::Int,
        types::InferredType::Float,
        types::InferredType::String,
        types::InferredType::List,
        types::InferredType::Dict,
        types::InferredType::Function,
        types::InferredType::Numeric,
    ] {
        let _ = ty.is_known();
        let _ = ty.tag();
    }
}

