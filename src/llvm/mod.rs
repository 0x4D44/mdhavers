//! LLVM Backend for mdhavers
//!
//! Compiles mdhavers AST to LLVM IR for native code generation.
//! Uses inkwell as a safe Rust wrapper around the LLVM C API.

pub mod builtins;
pub mod codegen;
pub mod compiler;
pub mod runtime;
pub mod types;

// Re-export main types
pub use compiler::LLVMCompiler;
pub use types::{InferredType, MdhTypes, ValueTag};
