use std::cell::Cell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use regex::Regex;
use serde_json::Value as JsonValue;

#[cfg(feature = "audio")]
mod audio;
#[cfg(not(feature = "audio"))]
mod audio_stub;
#[cfg(not(target_os = "emscripten"))]
mod net;
#[cfg(target_os = "emscripten")]
mod net_stub;
#[cfg(feature = "graphics3d")]
mod tri_engine;
#[cfg(feature = "graphics3d")]
mod tri_runtime;

// Re-export the same C ABI from this staticlib. Immediate values no longer call
// back into C; the dependency-free core can also be tested without this crate's
// networking/media dependencies.
pub use mdh_runtime_core::{
    __mdh_get_tag, __mdh_make_bool, __mdh_make_float, __mdh_make_int, __mdh_make_nil, MdhRsResult,
    MdhValue,
};

#[repr(C)]
struct MdhList {
    items: *mut MdhValue,
    length: i64,
    capacity: i64,
}

#[cfg(feature = "graphics3d")]
pub(crate) use mdh_runtime_core::MDH_TAG_NATIVE;
pub(crate) use mdh_runtime_core::{
    MDH_TAG_BOOL, MDH_TAG_BYTES, MDH_TAG_CLOSURE, MDH_TAG_DICT, MDH_TAG_FLOAT, MDH_TAG_FUNCTION,
    MDH_TAG_INT, MDH_TAG_LIST, MDH_TAG_NIL, MDH_TAG_SET, MDH_TAG_STRING,
};

extern "C" {
    fn __mdh_stack_low() -> i32;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_stack_overflow_message__rsabi")]
    fn __mdh_stack_overflow_message() -> MdhValue;
    // These never raise, which would longjmp across our frames: out of memory, they
    // return 0 (MDH-BUG-FLU-00188). Every value this file stores is one it has just made,
    // so it hands its reference over: push/set_owned take the caller's +1 (reference
    // counting, mdh_runtime.c), and release it if they fail.
    fn __mdh_make_string_try(value: *const c_char, out: *mut MdhValue) -> i32;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_list_push_owned_try__rsabi")]
    fn __mdh_list_push_owned_try(list: MdhValue, value: MdhValue) -> i32;
    fn __mdh_bytes_new_try(size: i64, out: *mut MdhValue) -> i32;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_dict_set_owned_try__rsabi")]
    fn __mdh_dict_set_owned_try(dict: MdhValue, key: MdhValue, value: MdhValue) -> i32;
    fn __mdh_make_list_try(capacity: i32, out: *mut MdhValue) -> i32;
    fn __mdh_empty_dict_try(out: *mut MdhValue) -> i32;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_out_of_memory_message__rsabi")]
    fn __mdh_out_of_memory_message() -> MdhValue;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_dict_get_default__rsabi")]
    fn __mdh_dict_get_default(dict: MdhValue, key: MdhValue, default_val: MdhValue) -> MdhValue;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_to_string_checked__rsabi")]
    fn __mdh_to_string_checked(value: MdhValue, out: *mut MdhValue) -> i32;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_kv_densify__rsabi")]
    fn __mdh_kv_densify(value: MdhValue) -> i32;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_release__rsabi")]
    fn __mdh_release(value: MdhValue);
}

#[cfg(all(debug_assertions, any(feature = "audio", feature = "graphics3d")))]
extern "C" {
    fn __mdh_rust_enter() -> i64;
    fn __mdh_rust_leave(outer: i64);
}

/// While alive, debug builds' C runtime aborts on any raise that would longjmp across
/// this frame (MDH-BUG-FLU-00189, `__mdh_rust_enter` in mdh_runtime.c). For the entries
/// compiled code calls directly (audio, tri); the C runtime marks its own calls into
/// Rust. Drop it before the entry raises its error. Nothing in release builds.
#[cfg(any(feature = "audio", feature = "graphics3d"))]
pub(crate) struct RustFrames {
    #[cfg(debug_assertions)]
    outer: i64,
}

#[cfg(any(feature = "audio", feature = "graphics3d"))]
impl RustFrames {
    pub(crate) fn enter() -> Self {
        RustFrames {
            #[cfg(debug_assertions)]
            outer: unsafe { __mdh_rust_enter() },
        }
    }
}

#[cfg(any(feature = "audio", feature = "graphics3d"))]
impl Drop for RustFrames {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        unsafe {
            __mdh_rust_leave(self.outer)
        }
    }
}

fn cstring_lossy(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("Runtime error").expect("CString literal"))
}

thread_local! {
    /// Set when the C runtime had no memory for a value this crate asked it to make
    /// (MDH-BUG-FLU-00188). The helpers below then hand back nil (or skip the store) and
    /// the builtin carries on with that; the extern entry, as its last act, reports the
    /// "ower muckle" error instead of its result (`mdh_finish`, `tri_finish`), so the
    /// error is raised with no Rust value alive, as for MDH-BUG-FLU-00183.
    static NO_MEMORY: Cell<bool> = const { Cell::new(false) };
}

fn note_no_memory() {
    NO_MEMORY.with(|flag| flag.set(true));
}

/// Whether memory ran out since the last call, clearing the record.
pub(crate) fn take_no_memory() -> bool {
    NO_MEMORY.with(|flag| flag.replace(false))
}

/// The builtin's result, or the "ower muckle" error when memory ran out making it: the
/// last thing each `MdhRsResult` entry does.
unsafe fn mdh_finish(result: MdhRsResult) -> MdhRsResult {
    if !take_no_memory() {
        return result;
    }
    __mdh_release(result.value);
    __mdh_release(result.error);
    MdhRsResult {
        ok: 0,
        value: __mdh_make_nil(),
        error: __mdh_out_of_memory_message(),
    }
}

unsafe fn mdh_ok(value: MdhValue) -> MdhRsResult {
    MdhRsResult {
        ok: 1,
        value,
        error: __mdh_make_nil(),
    }
}

unsafe fn mdh_err(msg: &str) -> MdhRsResult {
    MdhRsResult {
        ok: 0,
        value: __mdh_make_nil(),
        error: mdh_error_string(msg),
    }
}

pub(crate) unsafe fn mdh_string_to_rust(value: MdhValue) -> String {
    if value.tag != MDH_TAG_STRING || value.data == 0 {
        return String::new();
    }
    let ptr = value.data as *const c_char;
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

/// A new string value, or nil with the failure noted (see NO_MEMORY).
pub(crate) unsafe fn mdh_make_string_from_rust(s: &str) -> MdhValue {
    let cstr = cstring_lossy(s);
    let mut out = __mdh_make_nil();
    if __mdh_make_string_try(cstr.as_ptr(), &mut out) == 0 {
        note_no_memory();
    }
    out
}

/// An error message to raise: when there is no memory for it, the "ower muckle" error
/// itself (a static string, so this always succeeds).
pub(crate) unsafe fn mdh_error_string(msg: &str) -> MdhValue {
    let cstr = cstring_lossy(msg);
    let mut out = __mdh_make_nil();
    if __mdh_make_string_try(cstr.as_ptr(), &mut out) == 0 {
        return __mdh_out_of_memory_message();
    }
    out
}

/// A new empty list with room for `capacity` items, or nil with the failure noted (see
/// NO_MEMORY). Pushing onto that nil fails quietly too (MDH-BUG-FLU-00189).
pub(crate) unsafe fn mdh_new_list(capacity: i32) -> MdhValue {
    let mut out = __mdh_make_nil();
    if __mdh_make_list_try(capacity, &mut out) == 0 {
        note_no_memory();
    }
    out
}

/// A new empty dict, or nil with the failure noted (see NO_MEMORY). Setting a key on that
/// nil fails quietly too (MDH-BUG-FLU-00189).
pub(crate) unsafe fn mdh_new_dict() -> MdhValue {
    let mut out = __mdh_make_nil();
    if __mdh_empty_dict_try(&mut out) == 0 {
        note_no_memory();
    }
    out
}

/// Push a value this crate made onto a list (taking its reference); noted on failure.
pub(crate) unsafe fn mdh_list_push(list: MdhValue, value: MdhValue) {
    if __mdh_list_push_owned_try(list, value) == 0 {
        note_no_memory();
    }
}

/// dict[key] = value, taking both references; noted on failure. The dict stays the same
/// object. A nil key is a key string that could not be made (already noted): skipped.
pub(crate) unsafe fn mdh_dict_set(dict: MdhValue, key: MdhValue, value: MdhValue) {
    if key.tag == MDH_TAG_NIL {
        __mdh_release(value);
        return;
    }
    if __mdh_dict_set_owned_try(dict, key, value) == 0 {
        note_no_memory();
    }
}

/// The runtime's text for `value`, or why it could not be made (too deep to format, or
/// out of memory): the C formatter reports those rather than raising, which would
/// longjmp across our frames (MDH-BUG-FLU-00164, MDH-BUG-FLU-00184).
unsafe fn mdh_value_to_string(value: MdhValue) -> Result<String, Abandon> {
    let mut s_val = __mdh_make_nil();
    match __mdh_to_string_checked(value, &mut s_val) {
        1 => {}
        0 => return Err(Abandon::StackLow),
        _ => return Err(Abandon::NoMemory),
    }
    let s = mdh_string_to_rust(s_val);
    __mdh_release(s_val);
    Ok(s)
}

/// `dict[key]` or nil, borrowed (as __mdh_dict_get_default); the key made for the lookup
/// is dropped.
// The browser build has neither user (net.rs, tri_runtime.rs).
#[cfg_attr(target_os = "emscripten", allow(dead_code))]
pub(crate) unsafe fn mdh_dict_get_named(dict: MdhValue, key: &str) -> MdhValue {
    let key_val = mdh_make_string_from_rust(key);
    let val = __mdh_dict_get_default(dict, key_val, __mdh_make_nil());
    __mdh_release(key_val);
    val
}

pub(crate) unsafe fn mdh_float_value(value: MdhValue) -> f64 {
    f64::from_bits(value.data as u64)
}

fn lenient_json_for_serde_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if !in_string {
            if ch == '"' {
                in_string = true;
            }
            out.push(ch);
            continue;
        }

        if escape {
            match ch {
                '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u' => {
                    out.push('\\');
                    out.push(ch);
                }
                other => {
                    // Interpreter compatibility: unknown escapes become the literal char.
                    out.push(other);
                }
            }
            escape = false;
            continue;
        }

        match ch {
            '\\' => {
                escape = true;
            }
            '"' => {
                in_string = false;
                out.push('"');
            }
            other => out.push(other),
        }
    }

    if escape {
        // Trailing backslash - keep it so serde_json reports an error.
        out.push('\\');
    }

    out
}

fn json_escape_string(s: &str) -> String {
    let mut result = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

unsafe fn json_fallback_string(value: MdhValue) -> Result<String, Abandon> {
    let s = mdh_value_to_string(value)?;
    let escaped = s.replace('"', "\\\"");
    Ok(format!("\"{}\"", escaped))
}

/// Why JSON output was given up, reported as the runtime's error.
enum Abandon {
    /// `mdh_value_to_json` ran too deep (nested values recurse there, so it stops while
    /// there is room to raise the runtime's stack error), or the C formatter did
    /// (MDH-BUG-FLU-00164).
    StackLow,
    /// The C runtime ran out of memory formatting a value or compacting a dict
    /// (MDH-BUG-FLU-00184).
    NoMemory,
}

/// The C runtime's text for running out of memory building a string
/// (MDH_STRING_TOO_BIG_MSG in runtime/mdh_runtime.c).
const STRING_TOO_BIG_MSG: &str = "Och! That string would be ower muckle tae fit in memory";

unsafe fn json_result(result: Result<String, Abandon>) -> MdhRsResult {
    match result {
        Ok(s) => mdh_ok(mdh_make_string_from_rust(&s)),
        Err(Abandon::StackLow) => MdhRsResult {
            ok: 0,
            value: __mdh_make_nil(),
            error: __mdh_stack_overflow_message(),
        },
        Err(Abandon::NoMemory) => mdh_err(STRING_TOO_BIG_MSG),
    }
}

unsafe fn mdh_value_to_json(
    value: MdhValue,
    pretty: bool,
    indent: usize,
) -> Result<String, Abandon> {
    if __mdh_stack_low() != 0 {
        return Err(Abandon::StackLow);
    }
    mdh_value_to_json_level(value, pretty, indent)
}

unsafe fn mdh_value_to_json_level(
    value: MdhValue,
    pretty: bool,
    indent: usize,
) -> Result<String, Abandon> {
    let ws = "  ".repeat(indent);
    let ws_inner = "  ".repeat(indent + 1);

    Ok(match value.tag {
        MDH_TAG_NIL => "null".to_string(),
        MDH_TAG_BOOL => {
            if value.data != 0 {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        MDH_TAG_INT => value.data.to_string(),
        MDH_TAG_FLOAT => {
            let f = mdh_float_value(value);
            if f.is_nan() || f.is_infinite() {
                "null".to_string()
            } else {
                f.to_string()
            }
        }
        MDH_TAG_STRING => {
            let s = mdh_string_to_rust(value);
            json_escape_string(&s)
        }
        MDH_TAG_LIST => {
            if value.data == 0 {
                return Ok("[]".to_string());
            }
            let list_ptr = value.data as *const MdhList;
            if list_ptr.is_null() {
                return Ok("[]".to_string());
            }
            let list = &*list_ptr;
            let len = list.length.max(0) as usize;
            if len == 0 {
                return Ok("[]".to_string());
            }
            let items = if list.items.is_null() {
                &[] as &[MdhValue]
            } else {
                std::slice::from_raw_parts(list.items, len)
            };
            if pretty {
                let parts = items
                    .iter()
                    .map(|v| {
                        Ok(format!(
                            "{}{}",
                            ws_inner,
                            mdh_value_to_json(*v, true, indent + 1)?
                        ))
                    })
                    .collect::<Result<Vec<String>, Abandon>>()?;
                format!("[\n{}\n{}]", parts.join(",\n"), ws)
            } else {
                let parts = items
                    .iter()
                    .map(|v| mdh_value_to_json(*v, false, indent))
                    .collect::<Result<Vec<String>, Abandon>>()?;
                format!("[{}]", parts.join(", "))
            }
        }
        MDH_TAG_SET | MDH_TAG_FUNCTION | MDH_TAG_CLOSURE | MDH_TAG_BYTES => {
            json_fallback_string(value)?
        }
        MDH_TAG_DICT => {
            if value.data == 0 {
                return Ok("{}".to_string());
            }
            // Dict values store a stable handle pointer to their payload pointer (see
            // runtime/mdh_runtime.c: __mdh_kv_payload_ptr).
            let handle_ptr = value.data as *const i64;
            if handle_ptr.is_null() {
                return Ok("{}".to_string());
            }
            // Drop any removed (tombstoned) entries first, so the payload holds exactly
            // `count` live entries (MDH-BUG-FLU-00091).
            if __mdh_kv_densify(value) == 0 {
                return Err(Abandon::NoMemory);
            }
            let payload_ptr = *handle_ptr as *const i64;
            if payload_ptr.is_null() {
                return Ok("{}".to_string());
            }
            let count = *payload_ptr;
            if count <= 0 {
                return Ok("{}".to_string());
            }
            let entries_ptr = payload_ptr.add(1) as *const MdhValue;
            let entries = std::slice::from_raw_parts(entries_ptr, (count * 2) as usize);
            let mut parts = Vec::with_capacity(count as usize);
            for i in 0..count {
                let key_val = entries[(i * 2) as usize];
                let val = entries[(i * 2 + 1) as usize];
                let key_str = if key_val.tag == MDH_TAG_STRING {
                    mdh_string_to_rust(key_val)
                } else {
                    mdh_value_to_string(key_val)?
                };
                let key_json = json_escape_string(&key_str);
                let val_json = if pretty {
                    mdh_value_to_json(val, true, indent + 1)?
                } else {
                    mdh_value_to_json(val, false, indent)?
                };
                if pretty {
                    parts.push(format!("{}{}: {}", ws_inner, key_json, val_json));
                } else {
                    parts.push(format!("{}: {}", key_json, val_json));
                }
            }
            if pretty {
                format!("{{\n{}\n{}}}", parts.join(",\n"), ws)
            } else {
                format!("{{{}}}", parts.join(", "))
            }
        }
        _ => json_fallback_string(value)?,
    })
}

fn json_to_mdh(value: &JsonValue) -> Result<MdhValue, String> {
    unsafe {
        match value {
            JsonValue::Null => Ok(__mdh_make_nil()),
            JsonValue::Bool(b) => Ok(__mdh_make_bool(*b)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    return Ok(__mdh_make_int(i));
                }
                if let Some(u) = n.as_u64() {
                    if u <= i64::MAX as u64 {
                        return Ok(__mdh_make_int(u as i64));
                    }
                    return Err("Integer out of range".to_string());
                }
                if let Some(f) = n.as_f64() {
                    return Ok(__mdh_make_float(f));
                }
                Err("Invalid number".to_string())
            }
            JsonValue::String(s) => Ok(mdh_make_string_from_rust(s)),
            JsonValue::Array(items) => {
                let list = mdh_new_list(items.len() as i32);
                for item in items {
                    let v = json_to_mdh(item)?;
                    mdh_list_push(list, v);
                }
                Ok(list)
            }
            JsonValue::Object(map) => {
                let dict = mdh_new_dict();
                for (k, v) in map.iter() {
                    let key = mdh_make_string_from_rust(k);
                    let val = json_to_mdh(v)?;
                    mdh_dict_set(dict, key, val);
                }
                Ok(dict)
            }
        }
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_json_parse__rsabi")]
pub extern "C" fn __mdh_rs_json_parse(json_str: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if json_str.tag != MDH_TAG_STRING {
            return mdh_err("json_parse expects a string");
        }
        let text = mdh_string_to_rust(json_str);
        let text = lenient_json_for_serde_json(&text);
        let parsed: JsonValue = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => return mdh_err(&format!("Invalid JSON: {}", e)),
        };
        match json_to_mdh(&parsed) {
            Ok(v) => mdh_ok(v),
            Err(e) => mdh_err(&format!("Invalid JSON: {}", e)),
        }
    }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in json_parse")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_json_stringify__rsabi")]
pub extern "C" fn __mdh_rs_json_stringify(value: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe { json_result(mdh_value_to_json(value, false, 0)) }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in json_stringify")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_json_pretty__rsabi")]
pub extern "C" fn __mdh_rs_json_pretty(value: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe { json_result(mdh_value_to_json(value, true, 0)) }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in json_pretty")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_regex_test__rsabi")]
pub extern "C" fn __mdh_rs_regex_test(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_test expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        mdh_ok(__mdh_make_bool(re.is_match(&text_s)))
    }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in regex_test")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_regex_match__rsabi")]
pub extern "C" fn __mdh_rs_regex_match(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_match expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        if let Some(m) = re.find(&text_s) {
            let dict = mdh_new_dict();
            mdh_dict_set(
                dict,
                mdh_make_string_from_rust("match"),
                mdh_make_string_from_rust(m.as_str()),
            );
            mdh_dict_set(
                dict,
                mdh_make_string_from_rust("start"),
                __mdh_make_int(m.start() as i64),
            );
            mdh_dict_set(
                dict,
                mdh_make_string_from_rust("end"),
                __mdh_make_int(m.end() as i64),
            );
            mdh_ok(dict)
        } else {
            mdh_ok(__mdh_make_nil())
        }
    }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in regex_match")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_regex_match_all__rsabi")]
pub extern "C" fn __mdh_rs_regex_match_all(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_match_all expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let result = mdh_new_list(8);
        for m in re.find_iter(&text_s) {
            let dict = mdh_new_dict();
            mdh_dict_set(
                dict,
                mdh_make_string_from_rust("match"),
                mdh_make_string_from_rust(m.as_str()),
            );
            mdh_dict_set(
                dict,
                mdh_make_string_from_rust("start"),
                __mdh_make_int(m.start() as i64),
            );
            mdh_dict_set(
                dict,
                mdh_make_string_from_rust("end"),
                __mdh_make_int(m.end() as i64),
            );
            mdh_list_push(result, dict);
        }
        mdh_ok(result)
    }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in regex_match_all")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_regex_replace__rsabi")]
pub extern "C" fn __mdh_rs_regex_replace(
    text: MdhValue,
    pattern: MdhValue,
    replacement: MdhValue,
) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING
            || pattern.tag != MDH_TAG_STRING
            || replacement.tag != MDH_TAG_STRING
        {
            return mdh_err("regex_replace expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let repl_s = mdh_string_to_rust(replacement);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let replaced = re.replace_all(&text_s, repl_s.as_str()).to_string();
        mdh_ok(mdh_make_string_from_rust(&replaced))
    }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in regex_replace")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_regex_replace_first__rsabi")]
pub extern "C" fn __mdh_rs_regex_replace_first(
    text: MdhValue,
    pattern: MdhValue,
    replacement: MdhValue,
) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING
            || pattern.tag != MDH_TAG_STRING
            || replacement.tag != MDH_TAG_STRING
        {
            return mdh_err("regex_replace_first expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let repl_s = mdh_string_to_rust(replacement);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let replaced = re.replacen(&text_s, 1, repl_s.as_str()).to_string();
        mdh_ok(mdh_make_string_from_rust(&replaced))
    }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in regex_replace_first")) },
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_rs_regex_split__rsabi")]
pub extern "C" fn __mdh_rs_regex_split(text: MdhValue, pattern: MdhValue) -> MdhRsResult {
    match std::panic::catch_unwind(|| unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            return mdh_err("regex_split expects strings");
        }

        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => return mdh_err(&format!("Invalid regex '{}': {}", pat_s, e)),
        };

        let result = mdh_new_list(8);
        for part in re.split(&text_s) {
            mdh_list_push(result, mdh_make_string_from_rust(part));
        }
        mdh_ok(result)
    }) {
        Ok(result) => unsafe { mdh_finish(result) },
        Err(_) => unsafe { mdh_finish(mdh_err("Rust panic in regex_split")) },
    }
}
