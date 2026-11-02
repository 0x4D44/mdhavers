//! The allocation-free part of the native runtime's C ABI.
//!
//! These leaf operations have no dependencies on the C runtime, allocator, reference
//! counts, or error machinery. They cannot raise or call back into C. Keep fallible
//! and heap-touching operations out until they have an explicit ownership and error
//! contract; in particular, C `longjmp` must never cross a live Rust frame.
//!
//! See `docs/runtime-rust-migration.md` and `runtime/mdh_runtime.h` for the shared ABI.

#![no_std]

/// A native value, with C layout (not packed). This is not a wire format.
///
/// Copying this carrier does **not** retain a heap object. A heap-tagged value's
/// ownership is determined by the runtime function contract, not by Rust `Copy`.
/// Padding is unspecified and must never be compared, hashed, or serialized.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MdhValue {
    pub tag: u8,
    pub data: i64,
}

/// Fallible Rust helpers return to C before C raises a language error.
/// `ok` is 0 or 1; the inactive value/error field is nil. Active heap results carry
/// the ownership specified by the helper (normally +1).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MdhRsResult {
    pub ok: u8,
    pub value: MdhValue,
    pub error: MdhValue,
}

// Must match MdhTag in runtime/mdh_runtime.h and ValueTag in src/llvm/types.rs.
pub const MDH_TAG_NIL: u8 = 0;
pub const MDH_TAG_BOOL: u8 = 1;
pub const MDH_TAG_INT: u8 = 2;
pub const MDH_TAG_FLOAT: u8 = 3;
pub const MDH_TAG_STRING: u8 = 4;
pub const MDH_TAG_LIST: u8 = 5;
pub const MDH_TAG_DICT: u8 = 6;
pub const MDH_TAG_FUNCTION: u8 = 7;
pub const MDH_TAG_CLASS: u8 = 8;
pub const MDH_TAG_INSTANCE: u8 = 9;
pub const MDH_TAG_RANGE: u8 = 10;
pub const MDH_TAG_SET: u8 = 11;
pub const MDH_TAG_CLOSURE: u8 = 12;
pub const MDH_TAG_BYTES: u8 = 13;
pub const MDH_TAG_NATIVE: u8 = 14;

// The existing 64-bit native ABI. Other targets need their own C/Rust/LLVM calling
// convention checks; repr(C) alone does not establish LLVM aggregate ABI lowering.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<MdhValue>() == 16);
    assert!(core::mem::align_of::<MdhValue>() == 8);
    assert!(core::mem::offset_of!(MdhValue, tag) == 0);
    assert!(core::mem::offset_of!(MdhValue, data) == 8);
    assert!(core::mem::size_of::<MdhRsResult>() == 40);
    assert!(core::mem::align_of::<MdhRsResult>() == 8);
    assert!(core::mem::offset_of!(MdhRsResult, ok) == 0);
    assert!(core::mem::offset_of!(MdhRsResult, value) == 8);
    assert!(core::mem::offset_of!(MdhRsResult, error) == 24);
};

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_make_nil__rsabi")]
pub extern "C" fn __mdh_make_nil() -> MdhValue {
    MdhValue {
        tag: MDH_TAG_NIL,
        data: 0,
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_make_bool__rsabi")]
pub extern "C" fn __mdh_make_bool(value: bool) -> MdhValue {
    MdhValue {
        tag: MDH_TAG_BOOL,
        data: i64::from(value),
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_make_int__rsabi")]
pub extern "C" fn __mdh_make_int(value: i64) -> MdhValue {
    MdhValue {
        tag: MDH_TAG_INT,
        data: value,
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_make_float__rsabi")]
pub extern "C" fn __mdh_make_float(value: f64) -> MdhValue {
    // Preserve the entire IEEE-754 representation, including -0 and NaN payloads.
    // A numeric cast or a byte-order-dependent reinterpretation would change the ABI.
    MdhValue {
        tag: MDH_TAG_FLOAT,
        data: value.to_bits() as i64,
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_get_tag__rsabi")]
pub extern "C" fn __mdh_get_tag(value: MdhValue) -> u8 {
    // A borrowed read of the carrier, even for heap or unknown tags; never dereference.
    value.tag
}
