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

impl BuiltinInfo {
    const fn new(
        name: &'static str,
        runtime_name: &'static str,
        min_arity: usize,
        max_arity: Option<usize>,
    ) -> Self {
        BuiltinInfo {
            name,
            runtime_name,
            min_arity,
            max_arity,
        }
    }

    const fn fixed(name: &'static str, runtime_name: &'static str, arity: usize) -> Self {
        BuiltinInfo::new(name, runtime_name, arity, Some(arity))
    }
}

/// All built-in functions
pub static BUILTINS: &[BuiltinInfo] = &[
    // I/O
    BuiltinInfo::fixed("blether", "__mdh_blether", 1),
    BuiltinInfo::fixed("speir", "__mdh_speir", 1),
    // Type conversion
    BuiltinInfo::fixed("tae_string", "__mdh_to_string", 1),
    BuiltinInfo::fixed("tae_int", "__mdh_to_int", 1),
    BuiltinInfo::fixed("tae_float", "__mdh_to_float", 1),
    // Type checking
    BuiltinInfo::fixed("whit_kind", "__mdh_type_of", 1),
    // List operations
    BuiltinInfo::fixed("len", "__mdh_len", 1),
    BuiltinInfo::fixed("shove", "__mdh_list_push", 2),
    BuiltinInfo::fixed("yank", "__mdh_list_pop", 1),
    // Math
    BuiltinInfo::fixed("abs", "__mdh_abs", 1),
    BuiltinInfo::new("jammy", "__mdh_random", 0, Some(2)),
    BuiltinInfo::fixed("floor", "__mdh_floor", 1),
    BuiltinInfo::fixed("ceil", "__mdh_ceil", 1),
    BuiltinInfo::fixed("round", "__mdh_round", 1),
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
