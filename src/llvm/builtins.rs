//! Built-in functions for LLVM codegen
//!
//! Maps mdhavers built-in functions to runtime calls.

use std::collections::HashMap;

/// Information about a built-in function
#[derive(Debug, Clone)]
pub struct BuiltinInfo {
    /// Name in mdhavers
    pub name: &'static str,
    /// Corresponding runtime function name
    pub runtime_name: &'static str,
    /// Minimum number of arguments
    pub min_arity: usize,
    /// Maximum number of arguments (None = variadic)
    pub max_arity: Option<usize>,
}

/// All built-in functions
pub static BUILTINS: &[BuiltinInfo] = &[
    // I/O
    BuiltinInfo {
        name: "blether",
        runtime_name: "__mdh_blether",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "speir",
        runtime_name: "__mdh_speir",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "get_key",
        runtime_name: "__mdh_get_key",
        min_arity: 0,
        max_arity: Some(0),
    },
    // Type conversion
    BuiltinInfo {
        name: "tae_string",
        runtime_name: "__mdh_to_string",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "tae_int",
        runtime_name: "__mdh_to_int",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "tae_float",
        runtime_name: "__mdh_to_float",
        min_arity: 1,
        max_arity: Some(1),
    },
    // Type checking
    BuiltinInfo {
        name: "whit_kind",
        runtime_name: "__mdh_type_of",
        min_arity: 1,
        max_arity: Some(1),
    },
    // List operations
    BuiltinInfo {
        name: "len",
        runtime_name: "__mdh_len",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "shove",
        runtime_name: "__mdh_list_push",
        min_arity: 2,
        max_arity: Some(2),
    },
    BuiltinInfo {
        name: "yank",
        runtime_name: "__mdh_list_pop",
        min_arity: 1,
        max_arity: Some(1),
    },
    // Math
    BuiltinInfo {
        name: "abs",
        runtime_name: "__mdh_abs",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "jammy",
        runtime_name: "__mdh_random",
        min_arity: 0,
        max_arity: Some(2),
    },
    BuiltinInfo {
        name: "floor",
        runtime_name: "__mdh_floor",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "ceil",
        runtime_name: "__mdh_ceil",
        min_arity: 1,
        max_arity: Some(1),
    },
    BuiltinInfo {
        name: "round",
        runtime_name: "__mdh_round",
        min_arity: 1,
        max_arity: Some(1),
    },
];

/// Lookup table for quick builtin resolution
pub fn get_builtin_map() -> HashMap<&'static str, &'static BuiltinInfo> {
    BUILTINS.iter().map(|b| (b.name, b)).collect()
}

/// Check if a name is a built-in function
pub fn is_builtin(name: &str) -> bool {
    BUILTINS.iter().any(|b| b.name == name)
}

/// Get builtin info by name
pub fn get_builtin(name: &str) -> Option<&'static BuiltinInfo> {
    BUILTINS.iter().find(|b| b.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_info_struct_is_constructible() {
        let info = BuiltinInfo {
            name: "x",
            runtime_name: "y",
            min_arity: 1,
            max_arity: Some(2),
        };
        assert_eq!(info.name, "x");
        assert_eq!(info.runtime_name, "y");
        assert_eq!(info.min_arity, 1);
        assert_eq!(info.max_arity, Some(2));

        let fixed = BuiltinInfo {
            name: "z",
            runtime_name: "w",
            min_arity: 0,
            max_arity: Some(0),
        };
        assert_eq!(fixed.name, "z");
        assert_eq!(fixed.runtime_name, "w");
        assert_eq!(fixed.min_arity, 0);
        assert_eq!(fixed.max_arity, Some(0));
    }

    #[cfg(coverage)]
    #[test]
    fn builtin_resolution_helpers_are_exercised_for_instantiation_coverage() {
        use std::hint::black_box;

        let map = black_box(get_builtin_map());
        assert!(map.contains_key("len"));
        assert!(black_box(is_builtin("len")));
        assert!(!black_box(is_builtin("definitely_not_a_builtin_7b8c8236")));
        assert!(black_box(get_builtin("len")).is_some());
        assert!(black_box(get_builtin("nope")).is_none());
    }
}
