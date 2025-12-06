//! LLVM Backend for mdhavers
//!
//! Compiles mdhavers AST to LLVM IR for native code generation.
//! Uses inkwell as a safe Rust wrapper around the LLVM C API.
//!
//! Note: This module is a work-in-progress. Some types and functions are
//! defined but not yet used.

#[allow(dead_code)]
pub mod builtins;
pub mod codegen;
pub mod compiler;
#[allow(dead_code)]
pub mod runtime;
#[allow(dead_code)]
pub mod types;

// Re-export main types
pub use compiler::LLVMCompiler;
#[allow(unused_imports)]
pub use types::{InferredType, MdhTypes, ValueTag};
