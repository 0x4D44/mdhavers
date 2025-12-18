use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use regex::Regex;
use serde_json::Value as JsonValue;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MdhValue {
    pub tag: u8,
    pub data: i64,
}

#[repr(C)]
struct MdhList {
    items: *mut MdhValue,
    length: i64,
    capacity: i64,
}

const MDH_TAG_NIL: u8 = 0;
const MDH_TAG_BOOL: u8 = 1;
const MDH_TAG_INT: u8 = 2;
const MDH_TAG_FLOAT: u8 = 3;
const MDH_TAG_STRING: u8 = 4;
const MDH_TAG_LIST: u8 = 5;
const MDH_TAG_DICT: u8 = 6;

const MDH_CREEL_SENTINEL: i64 = 0x4d4448435245454c; // "MDHCREEL"

extern "C" {
    fn __mdh_make_nil() -> MdhValue;
    fn __mdh_make_bool(value: bool) -> MdhValue;
    fn __mdh_make_int(value: i64) -> MdhValue;
    fn __mdh_make_float(value: f64) -> MdhValue;
    fn __mdh_make_string(value: *const c_char) -> MdhValue;
    fn __mdh_make_list(capacity: i32) -> MdhValue;
    fn __mdh_list_push(list: MdhValue, value: MdhValue);
    fn __mdh_empty_dict() -> MdhValue;
    fn __mdh_dict_set(dict: MdhValue, key: MdhValue, value: MdhValue) -> MdhValue;
    fn __mdh_to_string(value: MdhValue) -> MdhValue;
    fn __mdh_type_error(op: *const c_char, got1: u8, got2: u8);
    fn __mdh_hurl(value: MdhValue);
}

const OP_JSON_PARSE: &[u8] = b"json_parse\0";
const OP_REGEX_TEST: &[u8] = b"regex_test\0";
const OP_REGEX_MATCH: &[u8] = b"regex_match\0";
const OP_REGEX_MATCH_ALL: &[u8] = b"regex_match_all\0";
const OP_REGEX_REPLACE: &[u8] = b"regex_replace\0";
const OP_REGEX_REPLACE_FIRST: &[u8] = b"regex_replace_first\0";
const OP_REGEX_SPLIT: &[u8] = b"regex_split\0";

unsafe fn mdh_type_error(op: &[u8], got1: u8, got2: u8) {
    __mdh_type_error(op.as_ptr() as *const c_char, got1, got2);
}

unsafe fn mdh_hurl_msg(msg: &str) {
    let cmsg = CString::new(msg).unwrap_or_else(|_| CString::new("Runtime error").unwrap());
    let v = __mdh_make_string(cmsg.as_ptr());
    __mdh_hurl(v);
}

unsafe fn mdh_string_to_rust(value: MdhValue) -> String {
    if value.tag != MDH_TAG_STRING || value.data == 0 {
        return String::new();
    }
    let ptr = value.data as *const c_char;
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

unsafe fn mdh_make_string_from_rust(s: &str) -> MdhValue {
    let cstr = CString::new(s).unwrap_or_else(|_| CString::new("").unwrap());
    __mdh_make_string(cstr.as_ptr())
}

unsafe fn mdh_value_to_string(value: MdhValue) -> String {
    let s_val = __mdh_to_string(value);
    mdh_string_to_rust(s_val)
}

unsafe fn mdh_float_value(value: MdhValue) -> f64 {
    f64::from_bits(value.data as u64)
}

unsafe fn mdh_dict_is_creel(dict: MdhValue) -> bool {
    if dict.tag != MDH_TAG_DICT || dict.data == 0 {
        return false;
    }
    let dict_ptr = dict.data as *const i64;
    if dict_ptr.is_null() {
        return false;
    }
    let count = *dict_ptr;
    if count == 0 {
        return *dict_ptr.add(1) == MDH_CREEL_SENTINEL;
    }
    let entries_ptr = dict_ptr.add(1) as *const MdhValue;
    for i in 0..count {
        let key = *entries_ptr.add((i * 2) as usize);
        let val = *entries_ptr.add((i * 2 + 1) as usize);
        if key.tag != val.tag || key.data != val.data {
            return false;
        }
    }
    true
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

unsafe fn json_fallback_string(value: MdhValue) -> String {
    let s = mdh_value_to_string(value);
    let escaped = s.replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

unsafe fn mdh_value_to_json(value: MdhValue, pretty: bool, indent: usize) -> String {
    let ws = "  ".repeat(indent);
    let ws_inner = "  ".repeat(indent + 1);

    match value.tag {
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
                return "[]".to_string();
            }
            let list_ptr = value.data as *const MdhList;
            if list_ptr.is_null() {
                return "[]".to_string();
            }
            let list = &*list_ptr;
            let len = list.length.max(0) as usize;
            if len == 0 {
                return "[]".to_string();
            }
            let items = if list.items.is_null() {
                &[] as &[MdhValue]
            } else {
                std::slice::from_raw_parts(list.items, len)
            };
            if pretty {
                let parts: Vec<String> = items
                    .iter()
                    .map(|v| format!("{}{}", ws_inner, mdh_value_to_json(*v, true, indent + 1)))
                    .collect();
                format!("[\n{}\n{}]", parts.join(",\n"), ws)
            } else {
                let parts: Vec<String> = items
                    .iter()
                    .map(|v| mdh_value_to_json(*v, false, indent))
                    .collect();
                format!("[{}]", parts.join(", "))
            }
        }
        MDH_TAG_DICT => {
            if mdh_dict_is_creel(value) {
                return json_fallback_string(value);
            }
            if value.data == 0 {
                return "{}".to_string();
            }
            let dict_ptr = value.data as *const i64;
            if dict_ptr.is_null() {
                return "{}".to_string();
            }
            let count = *dict_ptr;
            if count <= 0 {
                return "{}".to_string();
            }
            let entries_ptr = dict_ptr.add(1) as *const MdhValue;
            let entries = std::slice::from_raw_parts(entries_ptr, (count * 2) as usize);
            let mut parts = Vec::with_capacity(count as usize);
            for i in 0..count {
                let key_val = entries[(i * 2) as usize];
                let val = entries[(i * 2 + 1) as usize];
                let key_str = if key_val.tag == MDH_TAG_STRING {
                    mdh_string_to_rust(key_val)
                } else {
                    mdh_value_to_string(key_val)
                };
                let key_json = json_escape_string(&key_str);
                let val_json = if pretty {
                    mdh_value_to_json(val, true, indent + 1)
                } else {
                    mdh_value_to_json(val, false, indent)
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
        _ => json_fallback_string(value),
    }
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
                let list = __mdh_make_list(items.len() as i32);
                for item in items {
                    let v = json_to_mdh(item)?;
                    __mdh_list_push(list, v);
                }
                Ok(list)
            }
            JsonValue::Object(map) => {
                let mut dict = __mdh_empty_dict();
                for (k, v) in map.iter() {
                    let key = mdh_make_string_from_rust(k);
                    let val = json_to_mdh(v)?;
                    dict = __mdh_dict_set(dict, key, val);
                }
                Ok(dict)
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_json_parse(json_str: MdhValue) -> MdhValue {
    unsafe {
        if json_str.tag != MDH_TAG_STRING {
            mdh_type_error(OP_JSON_PARSE, json_str.tag, 0);
            return __mdh_make_nil();
        }
        let text = mdh_string_to_rust(json_str);
        let parsed: JsonValue = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid JSON: {}", e));
                return __mdh_make_nil();
            }
        };
        match json_to_mdh(&parsed) {
            Ok(v) => v,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid JSON: {}", e));
                __mdh_make_nil()
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_json_stringify(value: MdhValue) -> MdhValue {
    unsafe {
        let s = mdh_value_to_json(value, false, 0);
        mdh_make_string_from_rust(&s)
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_json_pretty(value: MdhValue) -> MdhValue {
    unsafe {
        let s = mdh_value_to_json(value, true, 0);
        mdh_make_string_from_rust(&s)
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_test(text: MdhValue, pattern: MdhValue) -> MdhValue {
    unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            mdh_type_error(OP_REGEX_TEST, text.tag, pattern.tag);
            return __mdh_make_bool(false);
        }
        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid regex '{}': {}", pat_s, e));
                return __mdh_make_bool(false);
            }
        };
        __mdh_make_bool(re.is_match(&text_s))
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_match(text: MdhValue, pattern: MdhValue) -> MdhValue {
    unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            mdh_type_error(OP_REGEX_MATCH, text.tag, pattern.tag);
            return __mdh_make_nil();
        }
        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid regex '{}': {}", pat_s, e));
                return __mdh_make_nil();
            }
        };
        if let Some(m) = re.find(&text_s) {
            let mut dict = __mdh_empty_dict();
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("match"),
                mdh_make_string_from_rust(m.as_str()),
            );
            dict = __mdh_dict_set(dict, mdh_make_string_from_rust("start"), __mdh_make_int(m.start() as i64));
            dict = __mdh_dict_set(dict, mdh_make_string_from_rust("end"), __mdh_make_int(m.end() as i64));
            dict
        } else {
            __mdh_make_nil()
        }
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_match_all(text: MdhValue, pattern: MdhValue) -> MdhValue {
    unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            mdh_type_error(OP_REGEX_MATCH_ALL, text.tag, pattern.tag);
            return __mdh_make_list(0);
        }
        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid regex '{}': {}", pat_s, e));
                return __mdh_make_list(0);
            }
        };
        let result = __mdh_make_list(8);
        for m in re.find_iter(&text_s) {
            let mut dict = __mdh_empty_dict();
            dict = __mdh_dict_set(
                dict,
                mdh_make_string_from_rust("match"),
                mdh_make_string_from_rust(m.as_str()),
            );
            dict = __mdh_dict_set(dict, mdh_make_string_from_rust("start"), __mdh_make_int(m.start() as i64));
            dict = __mdh_dict_set(dict, mdh_make_string_from_rust("end"), __mdh_make_int(m.end() as i64));
            __mdh_list_push(result, dict);
        }
        result
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_replace(
    text: MdhValue,
    pattern: MdhValue,
    replacement: MdhValue,
) -> MdhValue {
    unsafe {
        if text.tag != MDH_TAG_STRING
            || pattern.tag != MDH_TAG_STRING
            || replacement.tag != MDH_TAG_STRING
        {
            mdh_type_error(OP_REGEX_REPLACE, text.tag, pattern.tag);
            return if text.tag == MDH_TAG_STRING {
                text
            } else {
                mdh_make_string_from_rust("")
            };
        }
        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let repl_s = mdh_string_to_rust(replacement);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid regex '{}': {}", pat_s, e));
                return mdh_make_string_from_rust("");
            }
        };
        let replaced = re.replace_all(&text_s, repl_s.as_str()).to_string();
        mdh_make_string_from_rust(&replaced)
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_replace_first(
    text: MdhValue,
    pattern: MdhValue,
    replacement: MdhValue,
) -> MdhValue {
    unsafe {
        if text.tag != MDH_TAG_STRING
            || pattern.tag != MDH_TAG_STRING
            || replacement.tag != MDH_TAG_STRING
        {
            mdh_type_error(OP_REGEX_REPLACE_FIRST, text.tag, pattern.tag);
            return if text.tag == MDH_TAG_STRING {
                text
            } else {
                mdh_make_string_from_rust("")
            };
        }
        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let repl_s = mdh_string_to_rust(replacement);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid regex '{}': {}", pat_s, e));
                return mdh_make_string_from_rust("");
            }
        };
        let replaced = re.replacen(&text_s, 1, repl_s.as_str()).to_string();
        mdh_make_string_from_rust(&replaced)
    }
}

#[no_mangle]
pub extern "C" fn __mdh_rs_regex_split(text: MdhValue, pattern: MdhValue) -> MdhValue {
    unsafe {
        if text.tag != MDH_TAG_STRING || pattern.tag != MDH_TAG_STRING {
            mdh_type_error(OP_REGEX_SPLIT, text.tag, pattern.tag);
            return __mdh_make_list(0);
        }
        let text_s = mdh_string_to_rust(text);
        let pat_s = mdh_string_to_rust(pattern);
        let re = match Regex::new(&pat_s) {
            Ok(r) => r,
            Err(e) => {
                mdh_hurl_msg(&format!("Invalid regex '{}': {}", pat_s, e));
                return __mdh_make_list(0);
            }
        };
        let result = __mdh_make_list(8);
        for part in re.split(&text_s) {
            let v = mdh_make_string_from_rust(part);
            __mdh_list_push(result, v);
        }
        result
    }
}
